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
use crate::models::service::Service;
use crate::models::common::{PagedResult, QueryParams};
use crate::validation::validate_service;

fn row_to_service(row: &rusqlite::Row) -> rusqlite::Result<Service> {
    Ok(Service {
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
        system_id: row.get(9)?,
        system_name: row.get(10)?,
        status: row.get(11)?,
        description: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

const SELECT_COLUMNS: &str = "id, name, type, address, port, tech_stack, deploy_mode, \
     env, git_repo, system_id, \
     (SELECT s.name FROM systems s WHERE s.id = services.system_id AND s.is_deleted = 0) AS system_name, \
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
    svc_ids: &[String],
) -> Result<HashMap<String, Vec<(String, String)>>, rusqlite::Error> {
    if svc_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = vec!["?"; svc_ids.len()].join(", ");
    let sql = format!(
        "SELECT soc.service_id, c.id, c.name
         FROM service_owner_contacts soc
         JOIN contacts c ON c.id = soc.contact_id
         WHERE soc.is_deleted = 0
           AND c.is_deleted = 0
           AND soc.service_id IN ({})
         ORDER BY c.name COLLATE NOCASE ASC, c.id ASC",
        placeholders
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = svc_ids
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
        let (svc_id, contact_id, contact_name) = row?;
        map.entry(svc_id)
            .or_default()
            .push((contact_id, contact_name));
    }

    Ok(map)
}

fn merge_owner_fields(
    svc: &mut Service,
    owner_map: &HashMap<String, Vec<(String, String)>>,
) {
    if let Some(items) = owner_map.get(&svc.id) {
        svc.owner_contact_ids = Some(items.iter().map(|(id, _)| id.clone()).collect());
        svc.owners = Some(items.iter().map(|(_, name)| name.clone()).collect());
    } else {
        svc.owner_contact_ids = Some(Vec::new());
        svc.owners = Some(Vec::new());
    }
}

fn build_services_where_clause(params: &QueryParams) -> (String, Vec<SqlParam>) {
    let owner_name_search_clause = build_exists_like_clause(
        "service_owner_contacts soc JOIN contacts c ON c.id = soc.contact_id",
        &[
            "soc.service_id = services.id",
            "soc.is_deleted = 0",
            "c.is_deleted = 0",
        ],
        "c.name",
    );
    let owner_phone_search_clause = build_exists_like_clause(
        "service_owner_contacts soc JOIN contacts c ON c.id = soc.contact_id",
        &[
            "soc.service_id = services.id",
            "soc.is_deleted = 0",
            "c.is_deleted = 0",
        ],
        "c.phone",
    );
    let owner_email_search_clause = build_exists_like_clause(
        "service_owner_contacts soc JOIN contacts c ON c.id = soc.contact_id",
        &[
            "soc.service_id = services.id",
            "soc.is_deleted = 0",
            "c.is_deleted = 0",
        ],
        "c.email",
    );

    let (mut where_clause, mut sql_params) = build_resource_where_clause(
        &["services.is_deleted = 0"],
        params,
        &[
            "services.name",
            "services.address",
            "services.tech_stack",
            "services.git_repo",
        ],
        &[
            owner_name_search_clause,
            owner_phone_search_clause,
            owner_email_search_clause,
        ],
        &[
            ("type", "services.type"),
            ("env", "services.env"),
            ("status", "services.status"),
            ("deploy_mode", "services.deploy_mode"),
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
                        FROM service_owner_contacts soc
                        JOIN contacts c ON c.id = soc.contact_id
                        WHERE soc.service_id = services.id
                          AND soc.is_deleted = 0
                          AND c.is_deleted = 0
                          AND soc.contact_id IN ({})
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
pub fn list_services(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Service>> {
    let command = "list_services";
    let conn = get_conn(pool.inner(), command)?;

    let (where_clause, sql_params) = build_services_where_clause(&params);
    let total = count_query(
        command,
        "查询服务数量",
        &conn,
        "services",
        &where_clause,
        &sql_params,
    )?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM services {} ORDER BY services.created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );
    let mut query_params = sql_params;
    query_params.push(Box::new(page_size as i64));
    query_params.push(Box::new(offset as i64));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        query_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询服务列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_service)
        .map_err(|e| AppError::from_db_error(command, "读取服务列表", e))?;
    let mut data: Vec<Service> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::from_db_error(command, "读取服务列表", e))?;

    let svc_ids: Vec<String> = data.iter().map(|svc| svc.id.clone()).collect();
    let owner_map = load_owner_contacts_map(&conn, &svc_ids)
        .map_err(|e| AppError::from_db_error(command, "读取服务负责人", e))?;
    for svc in &mut data {
        merge_owner_fields(svc, &owner_map);
    }

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_service(pool: State<DbPool>, id: String) -> AppResult<Service> {
    let command = "get_service";
    let conn = get_conn(pool.inner(), command)?;

    let sql = format!(
        "SELECT {} FROM services WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    let mut svc = conn
        .query_row(&sql, rusqlite::params![id], row_to_service)
        .map_err(|e| AppError::not_found(command, "服务不存在或已删除。", Some(e.to_string())))?;

    let owner_map = load_owner_contacts_map(&conn, &[svc.id.clone()])
        .map_err(|e| AppError::from_db_error(command, "读取服务负责人", e))?;
    merge_owner_fields(&mut svc, &owner_map);
    Ok(svc)
}

fn sync_service_owner_contacts(
    command: &str,
    conn: &rusqlite::Connection,
    service_id: &str,
    owner_contact_ids: &[String],
    now: &str,
) -> AppResult<()> {
    conn.execute(
        "DELETE FROM service_owner_contacts WHERE service_id = ?1",
        rusqlite::params![service_id],
    )
    .map_err(|e| AppError::from_db_error(command, "清理服务负责人绑定", e))?;

    if owner_contact_ids.is_empty() {
        return Ok(());
    }

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
            "INSERT INTO service_owner_contacts (id, service_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, NULL, ?4, ?4)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), service_id, contact_id, now],
        )
        .map_err(|e| AppError::from_db_error(command, "创建服务负责人绑定", e))?;
    }

    Ok(())
}

fn save_service_inner(
    command: &str,
    conn: &rusqlite::Connection,
    data: Service,
) -> AppResult<String> {
    let mut normalized_data = data.clone();
    let normalized_owner_contact_ids = normalize_owner_contact_ids(data.owner_contact_ids.clone());
    normalized_data.owner_contact_ids = Some(normalized_owner_contact_ids.clone());
    let tech_stack_terms = parse_tech_stack_terms(normalized_data.tech_stack.as_deref());
    validate_service(&normalized_data).map_err(|e| AppError::validation(command, e))?;

    // Auto-fill name and env from the parent system
    let (sys_name, sys_env): (String, String) = conn
        .query_row(
            "SELECT name, env FROM systems WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![normalized_data.system_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::validation(command, "所属系统不存在或已删除。"))?;
    normalized_data.name = sys_name;
    normalized_data.env = sys_env;

    let now = chrono::Utc::now().to_rfc3339();
    let (persisted_id, is_new) =
        resolve_upsert_state(command, conn, "services", &normalized_data.id)?;

    with_transaction(conn, command, |conn| {
        if is_new {
            conn.execute(
                "INSERT INTO services (id, name, type, address, port, tech_stack, deploy_mode,
                                       env, git_repo, system_id, status, description, is_deleted, deleted_at, created_at, updated_at)
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
                    normalized_data.system_id,
                    normalized_data.status,
                    normalized_data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建服务", e))?;
            sync_service_owner_contacts(
                command,
                conn,
                &persisted_id,
                &normalized_owner_contact_ids,
                &now,
            )?;
            save_resource_terms(
                conn,
                "service",
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
                "service",
                &persisted_id,
                Some(&normalized_data.name),
            )?;
            Ok(persisted_id)
        } else {
            conn.execute(
                "UPDATE services SET name=?1, type=?2, address=?3, port=?4, tech_stack=?5, deploy_mode=?6,
                                     env=?7, git_repo=?8, system_id=?9, status=?10, description=?11, updated_at=?12
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
                    normalized_data.system_id,
                    normalized_data.status,
                    normalized_data.description,
                    now,
                    normalized_data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新服务", e))?;
            sync_service_owner_contacts(
                command,
                conn,
                &normalized_data.id,
                &normalized_owner_contact_ids,
                &now,
            )?;
            save_resource_terms(
                conn,
                "service",
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
                "service",
                &normalized_data.id,
                Some(&normalized_data.name),
            )?;
            Ok(normalized_data.id.clone())
        }
    })
}

#[tauri::command]
pub fn save_service(pool: State<DbPool>, data: Service) -> AppResult<String> {
    let command = "save_service";
    let conn = get_conn(pool.inner(), command)?;
    save_service_inner(command, &conn, data)
}

#[tauri::command]
pub fn delete_service(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "delete_service";
    let conn = get_conn(pool.inner(), command)?;

    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM services WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();
    let now = chrono::Utc::now().to_rfc3339();

    with_transaction(&conn, command, |conn| {
        delete_with_audit(
            command,
            "删除服务",
            conn,
            "services",
            "service",
            &id,
            name.as_deref(),
        )?;
        conn.execute(
            "DELETE FROM service_owner_contacts WHERE service_id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| AppError::from_db_error(command, "删除服务负责人绑定", e))?;
        delete_resource_terms(conn, "service", &id, &now)
            .map_err(|e| AppError::from_db_error(command, "删除服务词条绑定", e))?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_services_where_clause, collect_top_tech_stacks, merge_owner_fields,
        normalize_owner_contact_ids, row_to_service, save_service_inner, SELECT_COLUMNS,
    };
    use crate::error::AppErrorCode;
    use crate::models::service::Service;
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
    fn build_services_where_clause_should_support_owner_contact_filter_and_search() {
        let mut filters = HashMap::new();
        filters.insert("owner".to_string(), r#"["c-alice","c-bob"]"#.to_string());
        let params = QueryParams {
            search: Some("alice".to_string()),
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_services_where_clause(&params);
        assert!(where_clause.contains("EXISTS"));
        assert!(where_clause.contains("service_owner_contacts"));
        assert!(where_clause.contains("contacts"));
        assert!(!where_clause.contains("taxonomy_bindings"));
        assert!(!where_clause.contains("services.owner"));
        assert!(sql_params.len() >= 9);
    }

    #[test]
    fn merge_owner_fields_should_default_to_empty_owner_lists_when_binding_missing() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            "CREATE TABLE services (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                address TEXT,
                port INTEGER,
                tech_stack TEXT,
                deploy_mode TEXT,
                env TEXT NOT NULL,
                git_repo TEXT,
                system_id TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE systems (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0
            );",
        )
        .expect("create service tables");
        conn.execute(
            "INSERT INTO services (
                id, name, type, address, port, tech_stack, deploy_mode,
                env, git_repo, system_id, status, description, created_at, updated_at
             ) VALUES (
                'svc-1', 'test-svc', 'backend', NULL, NULL, NULL, NULL,
                'prod', NULL, 'sys-1', 'running', NULL, datetime('now'), datetime('now')
             )",
            [],
        )
        .expect("insert service row");

        let sql = format!(
            "SELECT {} FROM services WHERE id = 'svc-1'",
            SELECT_COLUMNS
        );
        let mut svc = conn
            .query_row(&sql, [], row_to_service)
            .expect("query service row");

        merge_owner_fields(&mut svc, &HashMap::new());
        assert_eq!(svc.owner_contact_ids, Some(Vec::<String>::new()));
        assert_eq!(svc.owners, Some(Vec::<String>::new()));
    }

    fn insert_test_system(conn: &rusqlite::Connection, id: &str, name: &str, env: &str) {
        conn.execute(
            "INSERT INTO systems (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, NULL, NULL, ?3, 'active', 0, NULL, datetime('now'), datetime('now'))",
            rusqlite::params![id, name, env],
        )
        .expect("insert system");
    }

    #[test]
    fn save_service_inner_should_sync_owner_contact_bindings_when_owner_contact_ids_provided()
    {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        insert_test_system(&conn, "sys-1", "test-system", "prod");
        conn.execute(
            "INSERT INTO contacts (id, name, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('c-alice', 'alice', NULL, NULL, NULL, 0, NULL, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("seed contact");

        let svc: Service = serde_json::from_value(json!({
            "id": "",
            "name": "",
            "type": "backend",
            "env": "prod",
            "system_id": "sys-1",
            "status": "running",
            "owner_contact_ids": ["c-alice"]
        }))
        .expect("deserialize service");

        let created_id = save_service_inner("test", &conn, svc).expect("create service");

        let owner_binding_count: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                 FROM service_owner_contacts
                 WHERE service_id = ?1 AND contact_id = 'c-alice' AND is_deleted = 0",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("count owner bindings");
        assert_eq!(owner_binding_count, 1);
    }

    fn make_new_service(system_id: &str) -> Service {
        Service {
            id: "".into(),
            name: "".into(),
            app_type: "backend".into(),
            address: Some("127.0.0.1".into()),
            port: Some(8080),
            tech_stack: Some("Rust".into()),
            deploy_mode: Some("docker".into()),
            env: "".into(),
            git_repo: None,
            owner_contact_ids: None,
            owners: None,
            system_id: system_id.into(),
            system_name: None,
            status: "running".into(),
            description: None,
            created_at: "".into(),
            updated_at: "".into(),
        }
    }

    #[test]
    fn save_service_inner_should_return_generated_id_for_create() {
        let conn = setup_test_db();
        insert_test_system(&conn, "sys-1", "test-system", "prod");
        let svc = make_new_service("sys-1");

        let id =
            save_service_inner("test", &conn, svc).expect("create service should pass");
        assert!(!id.is_empty());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM services WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .expect("query inserted service");
        assert_eq!(count, 1);
    }

    #[test]
    fn save_service_inner_should_auto_fill_name_and_env_from_system() {
        let conn = setup_test_db();
        insert_test_system(&conn, "sys-1", "支付中心", "test");
        let svc = make_new_service("sys-1");

        let id = save_service_inner("test", &conn, svc).expect("create service");

        let (name, env): (String, String) = conn
            .query_row(
                "SELECT name, env FROM services WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("query service");
        assert_eq!(name, "支付中心");
        assert_eq!(env, "test");
    }

    #[test]
    fn save_service_inner_should_reject_nonexistent_system() {
        let conn = setup_test_db();
        let svc = make_new_service("nonexistent-sys");
        let err = save_service_inner("test", &conn, svc)
            .expect_err("should reject nonexistent system");
        assert_eq!(err.code, AppErrorCode::ValidationError);
    }
}
