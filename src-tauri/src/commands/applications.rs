use std::collections::{HashMap, HashSet};
use tauri::State;

use crate::commands::taxonomy::{
    build_taxonomy_exists_filter, parse_filter_values, parse_tech_stack_terms, save_resource_terms,
    soft_delete_resource_terms, FIELD_OWNER, FIELD_TECH_STACK,
};
use crate::db::audit::insert_audit_log;
use crate::db::crud::soft_delete;
use crate::db::DbPool;
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
        owner: row.get(9)?,
        owners: Some(Vec::new()),
        business_application_id: row.get(10)?,
        business_application_name: row.get(11)?,
        status: row.get(12)?,
        description: row.get(13)?,
        is_deleted: row.get(14)?,
        deleted_at: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

const SELECT_COLUMNS: &str = "id, name, type, address, port, tech_stack, deploy_mode, \
     env, git_repo, owner, business_application_id, \
     (SELECT ba.name FROM business_applications ba WHERE ba.id = applications.business_application_id AND ba.is_deleted = 0) AS business_application_name, \
     status, description, is_deleted, deleted_at, created_at, updated_at";

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

fn normalize_owner_names(owners: Option<Vec<String>>) -> Vec<String> {
    let mut normalized: Vec<String> = Vec::new();
    let mut dedupe: HashSet<String> = HashSet::new();

    if let Some(items) = owners {
        for item in items {
            let owner = item.trim().to_string();
            if owner.is_empty() {
                continue;
            }
            if dedupe.insert(owner.clone()) {
                normalized.push(owner);
            }
        }
    }

    normalized
}

fn load_owner_map(
    conn: &rusqlite::Connection,
    app_ids: &[String],
) -> Result<HashMap<String, Vec<String>>, rusqlite::Error> {
    if app_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = vec!["?"; app_ids.len()].join(", ");
    let sql = format!(
        "SELECT tb.resource_id, tt.display_name
         FROM taxonomy_bindings tb
         JOIN taxonomy_terms tt ON tt.id = tb.term_id
         WHERE tb.is_deleted = 0
           AND tt.is_deleted = 0
           AND tb.resource_type = 'application'
           AND tt.field_key = '{}'
           AND tb.resource_id IN ({})
         ORDER BY tt.display_name ASC",
        FIELD_OWNER, placeholders
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = app_ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (app_id, owner_name) = row?;
        map.entry(app_id).or_default().push(owner_name);
    }

    Ok(map)
}

fn merge_owner_fields(app: &mut Application, owner_map: &HashMap<String, Vec<String>>) {
    if let Some(owners) = owner_map.get(&app.id) {
        app.owners = Some(owners.clone());
        app.owner = owners.first().cloned();
        return;
    }

    let fallback = app.owner.as_deref().unwrap_or("").trim();
    if fallback.is_empty() {
        app.owners = Some(Vec::new());
    } else {
        app.owners = Some(vec![fallback.to_string()]);
    }
}

fn replace_application_owners(
    conn: &rusqlite::Connection,
    application_id: &str,
    owners: &[String],
    now: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE application_owners
         SET is_deleted = 1, deleted_at = ?1, updated_at = ?1
         WHERE application_id = ?2 AND is_deleted = 0",
        rusqlite::params![now, application_id],
    )?;

    for owner in owners {
        conn.execute(
            "INSERT INTO application_owners (id, application_id, owner_name, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, NULL, ?4, ?4)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), application_id, owner, now],
        )?;
    }

    Ok(())
}

fn soft_delete_application_owners(
    conn: &rusqlite::Connection,
    application_id: &str,
    now: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE application_owners
         SET is_deleted = 1, deleted_at = ?1, updated_at = ?1
         WHERE application_id = ?2 AND is_deleted = 0",
        rusqlite::params![now, application_id],
    )?;
    Ok(())
}

fn build_applications_where_clause(
    params: &QueryParams,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions: Vec<String> = vec!["applications.is_deleted = 0".to_string()];
    let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(search) = params
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let like_value = format!("%{}%", search);
        conditions.push(
            "(applications.name LIKE ? \
              OR applications.address LIKE ? \
              OR applications.tech_stack LIKE ? \
              OR applications.git_repo LIKE ? \
              OR applications.owner LIKE ? \
              OR EXISTS ( \
                    SELECT 1 FROM application_owners ao \
                    WHERE ao.application_id = applications.id \
                      AND ao.is_deleted = 0 \
                      AND ao.owner_name LIKE ? \
                 ) \
              OR EXISTS ( \
                    SELECT 1 \
                    FROM taxonomy_bindings tb \
                    JOIN taxonomy_terms tt ON tt.id = tb.term_id \
                    WHERE tb.resource_type = 'application' \
                      AND tb.resource_id = applications.id \
                      AND tb.is_deleted = 0 \
                      AND tt.is_deleted = 0 \
                      AND tt.field_key = 'owner' \
                      AND tt.display_name LIKE ? \
                 ))"
            .to_string(),
        );
        for _ in 0..7 {
            sql_params.push(Box::new(like_value.clone()));
        }
    }

    if let Some(filters) = &params.filters {
        for (column, key) in [
            ("applications.type", "type"),
            ("applications.env", "env"),
            ("applications.status", "status"),
            ("applications.deploy_mode", "deploy_mode"),
        ] {
            if let Some(value) = filters.get(key) {
                let values = parse_filter_values(value);
                if values.is_empty() {
                    continue;
                }
                if values.len() == 1 {
                    conditions.push(format!("{} = ?", column));
                    sql_params.push(Box::new(values[0].clone()));
                } else {
                    let placeholders = vec!["?"; values.len()].join(", ");
                    conditions.push(format!("{} IN ({})", column, placeholders));
                    for item in values {
                        sql_params.push(Box::new(item));
                    }
                }
            }
        }

        if let Some(owner_filter) = filters.get("owner") {
            let owners = parse_filter_values(owner_filter);
            if !owners.is_empty() {
                if let Some(clause) = build_taxonomy_exists_filter(
                    "application",
                    FIELD_OWNER,
                    "applications.id",
                    &owners,
                ) {
                    conditions.push(clause);
                    for owner in owners {
                        sql_params.push(Box::new(owner));
                    }
                }
            }
        }
    }

    (format!("WHERE {}", conditions.join(" AND ")), sql_params)
}

#[tauri::command]
pub fn list_applications(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Application>> {
    let command = "list_applications";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let (where_clause, sql_params) = build_applications_where_clause(&params);
    let count_sql = format!("SELECT COUNT(*) FROM applications {}", where_clause);
    let count_refs: Vec<&dyn rusqlite::types::ToSql> =
        sql_params.iter().map(|p| p.as_ref()).collect();
    let total: u64 = conn
        .query_row(&count_sql, count_refs.as_slice(), |row| {
            row.get::<_, i64>(0)
        })
        .map(|count| count as u64)
        .map_err(|e| AppError::from_db_error(command, "查询应用数量", e))?;

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
    let mut data: Vec<Application> = rows.filter_map(|r| r.ok()).collect();

    let app_ids: Vec<String> = data.iter().map(|app| app.id.clone()).collect();
    let owner_map = load_owner_map(&conn, &app_ids)
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
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let sql = format!(
        "SELECT {} FROM applications WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    let mut app = conn
        .query_row(&sql, rusqlite::params![id], row_to_application)
        .map_err(|e| AppError::not_found(command, "应用不存在或已删除。", Some(e.to_string())))?;

    let owner_map = load_owner_map(&conn, &[app.id.clone()])
        .map_err(|e| AppError::from_db_error(command, "读取应用负责人", e))?;
    merge_owner_fields(&mut app, &owner_map);
    Ok(app)
}

fn save_application_inner(
    command: &str,
    conn: &rusqlite::Connection,
    data: Application,
) -> AppResult<String> {
    let owners_input = if data.owners.is_some() {
        data.owners.clone()
    } else {
        data.owner.clone().map(|item| vec![item])
    };
    let normalized_owners = normalize_owner_names(owners_input);

    let mut normalized_data = data.clone();
    normalized_data.owners = Some(normalized_owners.clone());
    normalized_data.owner = normalized_owners.first().cloned();
    let tech_stack_terms = parse_tech_stack_terms(normalized_data.tech_stack.as_deref());
    validate_application(&normalized_data).map_err(|e| AppError::validation(command, e))?;
    let now = chrono::Utc::now().to_rfc3339();

    let is_new = normalized_data.id.is_empty() || {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![normalized_data.id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        count == 0
    };

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<String> = (|| {
        if is_new {
            let persisted_id = if normalized_data.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                normalized_data.id.clone()
            };
            conn.execute(
                "INSERT INTO applications (id, name, type, address, port, tech_stack, deploy_mode,
                                           env, git_repo, owner, business_application_id, status, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,0,NULL,?14,?14)",
                rusqlite::params![
                    persisted_id,
                    normalized_data.name,
                    normalized_data.app_type,
                    normalized_data.address,
                    normalized_data.port,
                    normalized_data.tech_stack,
                    normalized_data.deploy_mode,
                    normalized_data.env,
                    normalized_data.git_repo,
                    normalized_data.owner,
                    normalized_data.business_application_id,
                    normalized_data.status,
                    normalized_data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建应用", e))?;
            replace_application_owners(conn, &persisted_id, &normalized_owners, &now)
                .map_err(|e| AppError::from_db_error(command, "保存应用负责人", e))?;
            save_resource_terms(
                conn,
                "application",
                &persisted_id,
                FIELD_OWNER,
                &normalized_owners,
                &now,
            )
            .map_err(|e| AppError::from_db_error(command, "同步负责人词条", e))?;
            save_resource_terms(
                conn,
                "application",
                &persisted_id,
                FIELD_TECH_STACK,
                &tech_stack_terms,
                &now,
            )
            .map_err(|e| AppError::from_db_error(command, "同步技术栈词条", e))?;
            insert_audit_log(
                conn,
                "create",
                "application",
                &persisted_id,
                Some(&normalized_data.name),
                None,
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
            Ok(persisted_id)
        } else {
            conn.execute(
                "UPDATE applications SET name=?1, type=?2, address=?3, port=?4, tech_stack=?5, deploy_mode=?6,
                                         env=?7, git_repo=?8, owner=?9, business_application_id=?10, status=?11, description=?12, updated_at=?13
                 WHERE id=?14 AND is_deleted=0",
                rusqlite::params![
                    normalized_data.name,
                    normalized_data.app_type,
                    normalized_data.address,
                    normalized_data.port,
                    normalized_data.tech_stack,
                    normalized_data.deploy_mode,
                    normalized_data.env,
                    normalized_data.git_repo,
                    normalized_data.owner,
                    normalized_data.business_application_id,
                    normalized_data.status,
                    normalized_data.description,
                    now,
                    normalized_data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新应用", e))?;
            replace_application_owners(conn, &normalized_data.id, &normalized_owners, &now)
                .map_err(|e| AppError::from_db_error(command, "保存应用负责人", e))?;
            save_resource_terms(
                conn,
                "application",
                &normalized_data.id,
                FIELD_OWNER,
                &normalized_owners,
                &now,
            )
            .map_err(|e| AppError::from_db_error(command, "同步负责人词条", e))?;
            save_resource_terms(
                conn,
                "application",
                &normalized_data.id,
                FIELD_TECH_STACK,
                &tech_stack_terms,
                &now,
            )
            .map_err(|e| AppError::from_db_error(command, "同步技术栈词条", e))?;
            insert_audit_log(
                conn,
                "update",
                "application",
                &normalized_data.id,
                Some(&normalized_data.name),
                None,
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
            Ok(normalized_data.id.clone())
        }
    })();

    match result {
        Ok(persisted_id) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(persisted_id)
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

#[tauri::command]
pub fn save_application(pool: State<DbPool>, data: Application) -> AppResult<String> {
    let command = "save_application";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    save_application_inner(command, &conn, data)
}

#[tauri::command]
pub fn soft_delete_application(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "soft_delete_application";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM applications WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    match soft_delete(&conn, "applications", &id) {
        Ok(()) => {
            if let Err(e) = soft_delete_application_owners(&conn, &id, &now) {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(AppError::from_db_error(command, "删除应用负责人", e));
            }
            if let Err(e) = soft_delete_resource_terms(&conn, "application", &id, &now) {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(AppError::from_db_error(command, "删除应用词条绑定", e));
            }
            match insert_audit_log(&conn, "delete", "application", &id, name.as_deref(), None) {
                Ok(()) => {
                    conn.execute_batch("COMMIT;")
                        .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
                    Ok(())
                }
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK;");
                    Err(AppError::from_db_error(command, "写入审计日志", e))
                }
            }
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(AppError::from_db_error(command, "删除应用", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_applications_where_clause, collect_top_tech_stacks, normalize_owner_names,
        save_application_inner,
    };
    use crate::models::application::Application;
    use crate::models::common::QueryParams;
    use crate::test_helpers::setup_test_db;
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
    fn normalize_owner_names_should_trim_deduplicate_and_drop_empty() {
        let owners = vec![
            " alice ".to_string(),
            "".to_string(),
            "bob".to_string(),
            "alice".to_string(),
            "   ".to_string(),
            "bob ".to_string(),
            "carol".to_string(),
        ];

        let result = normalize_owner_names(Some(owners));
        assert_eq!(result, vec!["alice", "bob", "carol"]);
    }

    #[test]
    fn build_applications_where_clause_should_include_owner_exists_for_search_and_filter() {
        let mut filters = HashMap::new();
        filters.insert("owner".to_string(), r#"["alice","bob"]"#.to_string());
        let params = QueryParams {
            search: Some("alice".to_string()),
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_applications_where_clause(&params);
        assert!(where_clause.contains("EXISTS"));
        assert!(where_clause.contains("application_owners"));
        assert!(sql_params.len() >= 6);
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
            owner: Some("alice".into()),
            owners: Some(vec!["alice".into()]),
            business_application_id: None,
            business_application_name: None,
            status: "running".into(),
            description: None,
            is_deleted: 0,
            deleted_at: None,
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
        updated.owners = Some(vec!["alice".into(), "bob".into()]);

        let returned_id = save_application_inner("test", &conn, updated).expect("update");
        assert_eq!(returned_id, created_id);
    }
}
