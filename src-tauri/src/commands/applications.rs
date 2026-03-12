use std::collections::{HashMap, HashSet};
use tauri::State;

use crate::commands::taxonomy::{
    delete_resource_terms, parse_tech_stack_terms, save_resource_terms, FIELD_TECH_STACK,
};
use crate::db::crud::{
    build_exists_like_clause, build_resource_where_clause, count_query, delete_with_audit,
    parse_filter_values, resolve_upsert_state, write_audit_log_entry, SqlParam,
};
use crate::db::transaction::with_transaction;
use crate::db::{get_conn, DbPool};
use crate::error::{AppError, AppResult};
use crate::models::application::Application;
use crate::models::common::{PagedResult, QueryParams};
use crate::validation::validate_application;

fn row_to_application(row: &rusqlite::Row) -> rusqlite::Result<Application> {
    Ok(Application {
        id: row.get(0)?,
        name: row.get(1)?,
        app_type: row.get(2)?,
        address: row.get(3)?,
        port: row.get(4)?,
        tech_stack: row.get(5)?,
        deploy_mode: row.get(6)?,
        env: row.get(7)?,
        git_repo: row.get(8)?,
        owner_contact_ids: Some(Vec::new()),
        owners: Some(Vec::new()),
        business_application_id: row.get(9)?,
        business_application_name: row.get(10)?,
        status: row.get(11)?,
        description: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

const SELECT_COLUMNS: &str = "id, name, type, address, port, tech_stack, deploy_mode, \
     env, git_repo, business_application_id, \
     (SELECT ba.name FROM business_applications ba WHERE ba.id = applications.business_application_id AND ba.is_deleted = 0) AS business_application_name, \
     status, description, created_at, updated_at";

#[cfg(test)]
fn parse_tech_stack_items(value: &str) -> Vec<String> {
    value
        .split(|ch| matches!(ch, ',' | ';' | '|' | '/' | '\u{FF0C}' | '\u{FF1B}'))
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

#[cfg(test)]
fn collect_top_tech_stacks<I>(rows: I, limit: usize) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    if limit == 0 {
        return Vec::new();
    }

    let mut counts: HashMap<String, usize> = HashMap::new();
    for row in rows {
        let mut unique_per_row: HashSet<String> = HashSet::new();
        for tech in parse_tech_stack_items(&row) {
            if unique_per_row.insert(tech.clone()) {
                *counts.entry(tech).or_insert(0) += 1;
            }
        }
    }

    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|(name_a, count_a), (name_b, count_b)| {
        count_b.cmp(count_a).then_with(|| name_a.cmp(name_b))
    });

    sorted
        .into_iter()
        .take(limit)
        .map(|(name, _)| name)
        .collect()
}

fn normalize_owner_contact_ids(owner_contact_ids: Option<Vec<String>>) -> Vec<String> {
    let mut normalized: Vec<String> = Vec::new();
    let mut dedupe: HashSet<String> = HashSet::new();

    if let Some(items) = owner_contact_ids {
        for item in items {
            let contact_id = item.trim().to_string();
            if contact_id.is_empty() {
                continue;
            }
            if dedupe.insert(contact_id.clone()) {
                normalized.push(contact_id);
            }
        }
    }

    normalized
}

fn load_owner_contacts_map(
    conn: &rusqlite::Connection,
    app_ids: &[String],
) -> Result<HashMap<String, Vec<(String, String)>>, rusqlite::Error> {
    if app_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = vec!["?"; app_ids.len()].join(", ");
    let sql = format!(
        "SELECT aoc.application_id, c.id, c.name
         FROM application_owner_contacts aoc
         JOIN contacts c ON c.id = aoc.contact_id
         WHERE aoc.is_deleted = 0
           AND c.is_deleted = 0
           AND aoc.application_id IN ({})
         ORDER BY c.name COLLATE NOCASE ASC, c.id ASC",
        placeholders
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = app_ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for row in rows {
        let (app_id, contact_id, contact_name) = row?;
        map.entry(app_id)
            .or_default()
            .push((contact_id, contact_name));
    }

    Ok(map)
}

fn merge_owner_fields(
    app: &mut Application,
    owner_map: &HashMap<String, Vec<(String, String)>>,
) {
    if let Some(items) = owner_map.get(&app.id) {
        app.owner_contact_ids = Some(items.iter().map(|(id, _)| id.clone()).collect());
        app.owners = Some(items.iter().map(|(_, name)| name.clone()).collect());
    } else {
        app.owner_contact_ids = Some(Vec::new());
        app.owners = Some(Vec::new());
    }
}

fn build_applications_where_clause(params: &QueryParams) -> (String, Vec<SqlParam>) {
    let owner_name_search_clause = build_exists_like_clause(
        "application_owner_contacts aoc JOIN contacts c ON c.id = aoc.contact_id",
        &[
            "aoc.application_id = applications.id",
            "aoc.is_deleted = 0",
            "c.is_deleted = 0",
        ],
        "c.name",
    );
    let owner_phone_search_clause = build_exists_like_clause(
        "application_owner_contacts aoc JOIN contacts c ON c.id = aoc.contact_id",
        &[
            "aoc.application_id = applications.id",
            "aoc.is_deleted = 0",
            "c.is_deleted = 0",
        ],
        "c.phone",
    );
    let owner_email_search_clause = build_exists_like_clause(
        "application_owner_contacts aoc JOIN contacts c ON c.id = aoc.contact_id",
        &[
            "aoc.application_id = applications.id",
            "aoc.is_deleted = 0",
            "c.is_deleted = 0",
        ],
        "c.email",
    );

    let (mut where_clause, mut sql_params) = build_resource_where_clause(
        &["applications.is_deleted = 0"],
        params,
        &[
            "applications.name",
            "applications.address",
            "applications.tech_stack",
            "applications.git_repo",
        ],
        &[
            owner_name_search_clause,
            owner_phone_search_clause,
            owner_email_search_clause,
        ],
        &[
            ("type", "applications.type"),
            ("env", "applications.env"),
            ("status", "applications.status"),
            ("deploy_mode", "applications.deploy_mode"),
        ],
        &[],
    );

    if let Some(filters) = params.filters.as_ref() {
        if let Some(owner_filter) = filters.get("owner") {
            let owner_contact_ids = parse_filter_values(owner_filter);
            if !owner_contact_ids.is_empty() {
                let placeholders = vec!["?"; owner_contact_ids.len()].join(", ");
                where_clause.push_str(&format!(
                    " AND EXISTS (
                        SELECT 1
                        FROM application_owner_contacts aoc
                        JOIN contacts c ON c.id = aoc.contact_id
                        WHERE aoc.application_id = applications.id
                          AND aoc.is_deleted = 0
                          AND c.is_deleted = 0
                          AND aoc.contact_id IN ({})
                    )",
                    placeholders
                ));
                for contact_id in owner_contact_ids {
                    sql_params.push(Box::new(contact_id));
                }
            }
        }
    }

    (where_clause, sql_params)
}

#[tauri::command]
pub fn list_applications(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Application>> {
    let command = "list_applications";
    let conn = get_conn(pool.inner(), command)?;

    let (where_clause, sql_params) = build_applications_where_clause(&params);
    let total = count_query(
        command,
        "查询应用数量",
        &conn,
        "applications",
        &where_clause,
        &sql_params,
    )?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM applications {} ORDER BY applications.created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );
    let mut query_params = sql_params;
    query_params.push(Box::new(page_size as i64));
    query_params.push(Box::new(offset as i64));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        query_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询应用列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_application)
        .map_err(|e| AppError::from_db_error(command, "读取应用列表", e))?;
    let mut data: Vec<Application> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::from_db_error(command, "读取应用列表", e))?;

    let app_ids: Vec<String> = data.iter().map(|app| app.id.clone()).collect();
    let owner_map = load_owner_contacts_map(&conn, &app_ids)
        .map_err(|e| AppError::from_db_error(command, "读取应用负责人", e))?;
    for app in &mut data {
        merge_owner_fields(app, &owner_map);
    }

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_application(pool: State<DbPool>, id: String) -> AppResult<Application> {
    let command = "get_application";
    let conn = get_conn(pool.inner(), command)?;

    let sql = format!(
        "SELECT {} FROM applications WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    let mut app = conn
        .query_row(&sql, rusqlite::params![id], row_to_application)
        .map_err(|e| AppError::not_found(command, "应用不存在或已删除。", Some(e.to_string())))?;

    let owner_map = load_owner_contacts_map(&conn, &[app.id.clone()])
        .map_err(|e| AppError::from_db_error(command, "读取应用负责人", e))?;
    merge_owner_fields(&mut app, &owner_map);
    Ok(app)
}

fn sync_application_owner_contacts(
    command: &str,
    conn: &rusqlite::Connection,
    application_id: &str,
    owner_contact_ids: &[String],
    now: &str,
) -> AppResult<()> {
    // 这里使用物理删除的方式同步绑定关系：实现简单、可重复执行，且与 host_ip_bindings 的处理方式一致。
    conn.execute(
        "DELETE FROM application_owner_contacts WHERE application_id = ?1",
        rusqlite::params![application_id],
    )
    .map_err(|e| AppError::from_db_error(command, "清理应用负责人绑定", e))?;

    if owner_contact_ids.is_empty() {
        return Ok(());
    }

    // 防御式校验：要求所有 contact_id 均存在且未删除，避免写入脏数据。
    let placeholders = vec!["?"; owner_contact_ids.len()].join(", ");
    let sql = format!(
        "SELECT id FROM contacts WHERE is_deleted = 0 AND id IN ({})",
        placeholders
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询联系人", e))?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(owner_contact_ids.iter()),
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| AppError::from_db_error(command, "读取联系人", e))?;

    let mut existing: HashSet<String> = HashSet::new();
    for row in rows {
        existing.insert(row.map_err(|e| AppError::from_db_error(command, "读取联系人", e))?);
    }

    let missing: Vec<String> = owner_contact_ids
        .iter()
        .filter(|id| !existing.contains(*id))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(AppError::validation(
            command,
            format!("负责人联系人不存在或已删除: {}", missing.join(", ")),
        ));
    }

    for contact_id in owner_contact_ids {
        conn.execute(
            "INSERT INTO application_owner_contacts (id, application_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, NULL, ?4, ?4)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), application_id, contact_id, now],
        )
        .map_err(|e| AppError::from_db_error(command, "创建应用负责人绑定", e))?;
    }

    Ok(())
}

fn save_application_inner(
    command: &str,
    conn: &rusqlite::Connection,
    data: Application,
) -> AppResult<String> {
    let mut normalized_data = data.clone();
    let normalized_owner_contact_ids = normalize_owner_contact_ids(data.owner_contact_ids.clone());
    normalized_data.owner_contact_ids = Some(normalized_owner_contact_ids.clone());
    let tech_stack_terms = parse_tech_stack_terms(normalized_data.tech_stack.as_deref());
    validate_application(&normalized_data).map_err(|e| AppError::validation(command, e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let (persisted_id, is_new) =
        resolve_upsert_state(command, conn, "applications", &normalized_data.id)?;

    with_transaction(conn, command, |conn| {
        if is_new {
            conn.execute(
                "INSERT INTO applications (id, name, type, address, port, tech_stack, deploy_mode,
                                           env, git_repo, business_application_id, status, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,0,NULL,?13,?13)",
                rusqlite::params![
                    &persisted_id,
                    normalized_data.name,
                    normalized_data.app_type,
                    normalized_data.address,
                    normalized_data.port,
                    normalized_data.tech_stack,
                    normalized_data.deploy_mode,
                    normalized_data.env,
                    normalized_data.git_repo,
                    normalized_data.business_application_id,
                    normalized_data.status,
                    normalized_data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建应用", e))?;
            sync_application_owner_contacts(
                command,
                conn,
                &persisted_id,
                &normalized_owner_contact_ids,
                &now,
            )?;
            save_resource_terms(
                conn,
                "application",
                &persisted_id,
                FIELD_TECH_STACK,
                &tech_stack_terms,
                &now,
            )
            .map_err(|e| AppError::from_db_error(command, "同步技术栈词条", e))?;
            write_audit_log_entry(
                command,
                conn,
                "create",
                "application",
                &persisted_id,
                Some(&normalized_data.name),
            )?;
            Ok(persisted_id)
        } else {
            conn.execute(
                "UPDATE applications SET name=?1, type=?2, address=?3, port=?4, tech_stack=?5, deploy_mode=?6,
                                         env=?7, git_repo=?8, business_application_id=?9, status=?10, description=?11, updated_at=?12
                 WHERE id=?13 AND is_deleted=0",
                rusqlite::params![
                    normalized_data.name,
                    normalized_data.app_type,
                    normalized_data.address,
                    normalized_data.port,
                    normalized_data.tech_stack,
                    normalized_data.deploy_mode,
                    normalized_data.env,
                    normalized_data.git_repo,
                    normalized_data.business_application_id,
                    normalized_data.status,
                    normalized_data.description,
                    now,
                    normalized_data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新应用", e))?;
            sync_application_owner_contacts(
                command,
                conn,
                &normalized_data.id,
                &normalized_owner_contact_ids,
                &now,
            )?;
            save_resource_terms(
                conn,
                "application",
                &normalized_data.id,
                FIELD_TECH_STACK,
                &tech_stack_terms,
                &now,
            )
            .map_err(|e| AppError::from_db_error(command, "同步技术栈词条", e))?;
            write_audit_log_entry(
                command,
                conn,
                "update",
                "application",
                &normalized_data.id,
                Some(&normalized_data.name),
            )?;
            Ok(normalized_data.id.clone())
        }
    })
}

#[tauri::command]
pub fn save_application(pool: State<DbPool>, data: Application) -> AppResult<String> {
    let command = "save_application";
    let conn = get_conn(pool.inner(), command)?;
    save_application_inner(command, &conn, data)
}

#[tauri::command]
pub fn delete_application(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "delete_application";
    let conn = get_conn(pool.inner(), command)?;

    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM applications WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();
    let now = chrono::Utc::now().to_rfc3339();

    with_transaction(&conn, command, |conn| {
        delete_with_audit(
            command,
            "删除应用",
            conn,
            "applications",
            "application",
            &id,
            name.as_deref(),
        )?;
        conn.execute(
            "DELETE FROM application_owner_contacts WHERE application_id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| AppError::from_db_error(command, "删除应用负责人绑定", e))?;
        delete_resource_terms(conn, "application", &id, &now)
            .map_err(|e| AppError::from_db_error(command, "删除应用词条绑定", e))?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_applications_where_clause, collect_top_tech_stacks, merge_owner_fields,
        normalize_owner_contact_ids, row_to_application, save_application_inner, SELECT_COLUMNS,
    };
    use crate::error::AppErrorCode;
    use crate::models::application::Application;
    use crate::models::common::QueryParams;
    use crate::test_helpers::setup_test_db;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn collect_top_tech_stacks_should_sort_by_count_desc_then_name_asc() {
        let rows = vec![
            "Vue, TypeScript, Pinia".to_string(),
            "TypeScript; Rust".to_string(),
            "Rust / Vue".to_string(),
            "Vue\u{FF0C}Vue".to_string(),
            "".to_string(),
        ];

        let result = collect_top_tech_stacks(rows, 3);
        assert_eq!(result, vec!["Vue", "Rust", "TypeScript"]);
    }

    #[test]
    fn collect_top_tech_stacks_should_respect_limit() {
        let rows = vec!["A,B,C,D,E,F,G,H,I,J,K".to_string(), "A,B,C,D,E".to_string()];

        let result = collect_top_tech_stacks(rows, 5);
        assert_eq!(result.len(), 5);
        assert_eq!(result, vec!["A", "B", "C", "D", "E"]);
    }

    #[test]
    fn normalize_owner_contact_ids_should_trim_deduplicate_and_drop_empty() {
        let owner_contact_ids = vec![
            " c-alice ".to_string(),
            "".to_string(),
            "c-bob".to_string(),
            "c-alice".to_string(),
            "   ".to_string(),
            "c-bob ".to_string(),
            "c-carol".to_string(),
        ];

        let result = normalize_owner_contact_ids(Some(owner_contact_ids));
        assert_eq!(result, vec!["c-alice", "c-bob", "c-carol"]);
    }

    #[test]
    fn build_applications_where_clause_should_support_owner_contact_filter_and_search() {
        let mut filters = HashMap::new();
        filters.insert("owner".to_string(), r#"["c-alice","c-bob"]"#.to_string());
        let params = QueryParams {
            search: Some("alice".to_string()),
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_applications_where_clause(&params);
        assert!(where_clause.contains("EXISTS"));
        assert!(where_clause.contains("application_owner_contacts"));
        assert!(where_clause.contains("contacts"));
        assert!(!where_clause.contains("taxonomy_bindings"));
        assert!(!where_clause.contains("applications.owner"));
        assert!(sql_params.len() >= 9);
    }

    #[test]
    fn merge_owner_fields_should_default_to_empty_owner_lists_when_binding_missing() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            "CREATE TABLE applications (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                address TEXT,
                port INTEGER,
                tech_stack TEXT,
                deploy_mode TEXT,
                env TEXT NOT NULL,
                git_repo TEXT,
                owner TEXT,
                business_application_id TEXT,
                status TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE business_applications (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0
            );",
        )
        .expect("create application tables with stale owner column");
        conn.execute(
            "INSERT INTO applications (
                id, name, type, address, port, tech_stack, deploy_mode,
                env, git_repo, owner, business_application_id, status, description, created_at, updated_at
             ) VALUES (
                'app-stale-owner', 'stale-owner-app', 'backend', NULL, NULL, NULL, NULL,
                'prod', NULL, 'ignored-owner', NULL, 'running', NULL, datetime('now'), datetime('now')
             )",
            [],
        )
        .expect("insert application row with stale owner column");

        let sql = format!(
            "SELECT {} FROM applications WHERE id = 'app-stale-owner'",
            SELECT_COLUMNS
        );
        let mut app = conn
            .query_row(&sql, [], row_to_application)
            .expect("query application row with stale owner column");

        merge_owner_fields(&mut app, &HashMap::new());
        assert_eq!(app.owner_contact_ids, Some(Vec::<String>::new()));
        assert_eq!(app.owners, Some(Vec::<String>::new()));
    }

    #[test]
    fn save_application_inner_should_sync_owner_contact_bindings_when_owner_contact_ids_provided()
    {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO contacts (id, name, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('c-alice', 'alice', NULL, NULL, NULL, 0, NULL, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("seed contact");

        let app: Application = serde_json::from_value(json!({
            "id": "",
            "name": "owners-missing",
            "type": "backend",
            "env": "prod",
            "status": "running",
            "owner_contact_ids": ["c-alice"]
        }))
        .expect("deserialize application");

        let created_id = save_application_inner("test", &conn, app).expect("create application");

        let owner_binding_count: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                 FROM application_owner_contacts
                 WHERE application_id = ?1 AND contact_id = 'c-alice' AND is_deleted = 0",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("count owner bindings");
        assert_eq!(owner_binding_count, 1);
    }

    fn make_new_application(name: &str) -> Application {
        Application {
            id: "".into(),
            name: name.into(),
            app_type: "backend".into(),
            address: Some("127.0.0.1".into()),
            port: Some(8080),
            tech_stack: Some("Rust".into()),
            deploy_mode: Some("docker".into()),
            env: "prod".into(),
            git_repo: None,
            owner_contact_ids: None,
            owners: None,
            business_application_id: None,
            business_application_name: None,
            status: "running".into(),
            description: None,
            created_at: "".into(),
            updated_at: "".into(),
        }
    }

    #[test]
    fn save_application_inner_should_return_generated_id_for_create() {
        let conn = setup_test_db();
        let app = make_new_application("app-create");

        let id =
            save_application_inner("test", &conn, app).expect("create application should pass");
        assert!(!id.is_empty());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .expect("query inserted app");
        assert_eq!(count, 1);
    }

    #[test]
    fn save_application_inner_should_return_original_id_for_update() {
        let conn = setup_test_db();
        let created_id = save_application_inner("test", &conn, make_new_application("app-update"))
            .expect("create");

        let mut updated = make_new_application("app-update-renamed");
        updated.id = created_id.clone();
        updated.owner_contact_ids = None;

        let returned_id = save_application_inner("test", &conn, updated).expect("update");
        assert_eq!(returned_id, created_id);
    }

    #[test]
    fn save_application_inner_should_replace_owner_contacts_on_update() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO contacts (id, name, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('c-alice', 'alice', NULL, NULL, NULL, 0, NULL, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("seed alice");
        conn.execute(
            "INSERT INTO contacts (id, name, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('c-bob', 'bob', NULL, NULL, NULL, 0, NULL, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("seed bob");

        let mut app = make_new_application("app-owner-source");
        app.owner_contact_ids = Some(vec!["c-alice".into()]);
        let created_id = save_application_inner("test", &conn, app).expect("create");
        assert!(!created_id.is_empty());

        let first_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM application_owner_contacts WHERE application_id = ?1 AND is_deleted = 0",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("count owner bindings");
        assert_eq!(first_count, 1);

        let mut updated = make_new_application("app-owner-source");
        updated.id = created_id.clone();
        updated.owner_contact_ids = Some(vec!["c-bob".into()]);
        save_application_inner("test", &conn, updated).expect("update");

        let second_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM application_owner_contacts WHERE application_id = ?1 AND is_deleted = 0",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("count owner bindings after update");
        assert_eq!(second_count, 1);

        let current_owner: String = conn
            .query_row(
                "SELECT contact_id FROM application_owner_contacts WHERE application_id = ?1 AND is_deleted = 0 LIMIT 1",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("query current owner contact id");
        assert_eq!(current_owner, "c-bob");
    }

    #[test]
    fn save_application_inner_should_allow_same_name_env_when_type_is_different() {
        let conn = setup_test_db();
        let first = make_new_application("shared-name");
        save_application_inner("test", &conn, first).expect("create first application");

        let mut second = make_new_application("shared-name");
        second.app_type = "gateway".into();

        let created_id = save_application_inner("test", &conn, second)
            .expect("same name/env with different type should be allowed");
        assert!(!created_id.is_empty());
    }

    #[test]
    fn save_application_inner_should_reject_same_name_env_and_type() {
        let conn = setup_test_db();
        let first = make_new_application("duplicate-key");
        save_application_inner("test", &conn, first).expect("create first application");

        let second = make_new_application("duplicate-key");
        let err = save_application_inner("test", &conn, second)
            .expect_err("same name/env/type should be rejected");
        assert_eq!(err.code, AppErrorCode::Conflict);
        assert_eq!(
            err.message,
            "保存失败，服务名称+环境+类型已存在，请调整后重试。"
        );
    }
}
