use std::collections::{HashMap, HashSet};
use tauri::State;

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
        status: row.get(10)?,
        description: row.get(11)?,
        is_deleted: row.get(12)?,
        deleted_at: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

const SELECT_COLUMNS: &str = "id, name, type, address, port, tech_stack, deploy_mode, \
     env, git_repo, owner, status, description, is_deleted, deleted_at, created_at, updated_at";

fn parse_tech_stack_items(value: &str) -> Vec<String> {
    value
        .split(|ch| matches!(ch, ',' | ';' | '|' | '/' | '\u{FF0C}' | '\u{FF1B}'))
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

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

fn resolve_tech_stack_side(app_type: Option<&str>) -> &'static str {
    match app_type.unwrap_or("").trim() {
        "frontend" => "frontend",
        _ => "backend",
    }
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

fn parse_owner_filter_values(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if trimmed.starts_with('[') {
        if let Ok(values) = serde_json::from_str::<Vec<String>>(trimmed) {
            return normalize_owner_names(Some(values));
        }
    }

    vec![trimmed.to_string()]
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
        "SELECT application_id, owner_name
         FROM application_owners
         WHERE is_deleted = 0 AND application_id IN ({})
         ORDER BY owner_name ASC",
        placeholders
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
                 ))"
            .to_string(),
        );
        for _ in 0..6 {
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
                let values = parse_owner_filter_values(value);
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
            let owners = parse_owner_filter_values(owner_filter);
            if !owners.is_empty() {
                if owners.len() == 1 {
                    conditions.push(
                        "(applications.owner = ? \
                          OR EXISTS ( \
                                SELECT 1 FROM application_owners ao \
                                WHERE ao.application_id = applications.id \
                                  AND ao.is_deleted = 0 \
                                  AND ao.owner_name = ? \
                             ))"
                        .to_string(),
                    );
                    sql_params.push(Box::new(owners[0].clone()));
                    sql_params.push(Box::new(owners[0].clone()));
                } else {
                    let placeholders = vec!["?"; owners.len()].join(", ");
                    conditions.push(format!(
                        "(applications.owner IN ({}) \
                          OR EXISTS ( \
                                SELECT 1 FROM application_owners ao \
                                WHERE ao.application_id = applications.id \
                                  AND ao.is_deleted = 0 \
                                  AND ao.owner_name IN ({}) \
                             ))",
                        placeholders, placeholders
                    ));
                    for owner in &owners {
                        sql_params.push(Box::new(owner.clone()));
                    }
                    for owner in &owners {
                        sql_params.push(Box::new(owner.clone()));
                    }
                }
            }
        }
    }

    (format!("WHERE {}", conditions.join(" AND ")), sql_params)
}

#[tauri::command]
pub fn list_top_application_tech_stacks(
    pool: State<DbPool>,
    limit: Option<u32>,
    app_type: Option<String>,
) -> AppResult<Vec<String>> {
    let command = "list_top_application_tech_stacks";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let side = resolve_tech_stack_side(app_type.as_deref());
    let query_sql = match side {
        "frontend" => {
            "SELECT tech_stack
             FROM applications
             WHERE is_deleted = 0
               AND type = 'frontend'
               AND tech_stack IS NOT NULL
               AND TRIM(tech_stack) <> ''"
        }
        _ => {
            "SELECT tech_stack
             FROM applications
             WHERE is_deleted = 0
               AND type <> 'frontend'
               AND tech_stack IS NOT NULL
               AND TRIM(tech_stack) <> ''"
        }
    };

    let mut stmt = conn
        .prepare(query_sql)
        .map_err(|e| AppError::from_db_error(command, "query application tech stacks", e))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, Option<String>>(0))
        .map_err(|e| AppError::from_db_error(command, "read application tech stacks", e))?;

    let mut tech_stack_rows: Vec<String> = Vec::new();
    for row in rows {
        let value =
            row.map_err(|e| AppError::from_db_error(command, "decode application tech stacks", e))?;
        if let Some(text) = value {
            tech_stack_rows.push(text);
        }
    }

    let top_limit = limit.unwrap_or(10).clamp(1, 100) as usize;
    Ok(collect_top_tech_stacks(tech_stack_rows, top_limit))
}

#[tauri::command]
pub fn list_application_owner_candidates(
    pool: State<DbPool>,
    limit: Option<u32>,
) -> AppResult<Vec<String>> {
    let command = "list_application_owner_candidates";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let max_limit = limit.unwrap_or(100).clamp(1, 500) as i64;

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT ao.owner_name
             FROM application_owners ao
             INNER JOIN applications a ON a.id = ao.application_id
             WHERE ao.is_deleted = 0
               AND a.is_deleted = 0
               AND TRIM(ao.owner_name) <> ''
             ORDER BY ao.owner_name ASC
             LIMIT ?1",
        )
        .map_err(|e| AppError::from_db_error(command, "query owner candidates", e))?;

    let rows = stmt
        .query_map(rusqlite::params![max_limit], |row| row.get::<_, String>(0))
        .map_err(|e| AppError::from_db_error(command, "read owner candidates", e))?;

    let mut candidates: Vec<String> = Vec::new();
    for row in rows {
        candidates
            .push(row.map_err(|e| AppError::from_db_error(command, "decode owner candidates", e))?);
    }
    Ok(candidates)
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

#[tauri::command]
pub fn save_application(pool: State<DbPool>, data: Application) -> AppResult<()> {
    let command = "save_application";
    let owners_input = if data.owners.is_some() {
        data.owners.clone()
    } else {
        data.owner.clone().map(|item| vec![item])
    };
    let normalized_owners = normalize_owner_names(owners_input);

    let mut normalized_data = data.clone();
    normalized_data.owners = Some(normalized_owners.clone());
    normalized_data.owner = normalized_owners.first().cloned();
    validate_application(&normalized_data).map_err(|e| AppError::validation(command, e))?;

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
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

    let result: AppResult<()> = (|| {
        if is_new {
            let id = if normalized_data.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                normalized_data.id.clone()
            };
            conn.execute(
                "INSERT INTO applications (id, name, type, address, port, tech_stack, deploy_mode,
                                           env, git_repo, owner, status, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,0,NULL,?13,?13)",
                rusqlite::params![
                    id,
                    normalized_data.name,
                    normalized_data.app_type,
                    normalized_data.address,
                    normalized_data.port,
                    normalized_data.tech_stack,
                    normalized_data.deploy_mode,
                    normalized_data.env,
                    normalized_data.git_repo,
                    normalized_data.owner,
                    normalized_data.status,
                    normalized_data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建应用", e))?;
            replace_application_owners(&conn, &id, &normalized_owners, &now)
                .map_err(|e| AppError::from_db_error(command, "保存应用负责人", e))?;
            insert_audit_log(
                &conn,
                "create",
                "application",
                &id,
                Some(&normalized_data.name),
                None,
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        } else {
            conn.execute(
                "UPDATE applications SET name=?1, type=?2, address=?3, port=?4, tech_stack=?5, deploy_mode=?6,
                                         env=?7, git_repo=?8, owner=?9, status=?10, description=?11, updated_at=?12
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
                    normalized_data.owner,
                    normalized_data.status,
                    normalized_data.description,
                    now,
                    normalized_data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新应用", e))?;
            replace_application_owners(&conn, &normalized_data.id, &normalized_owners, &now)
                .map_err(|e| AppError::from_db_error(command, "保存应用负责人", e))?;
            insert_audit_log(
                &conn,
                "update",
                "application",
                &normalized_data.id,
                Some(&normalized_data.name),
                None,
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(())
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
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
        parse_owner_filter_values, resolve_tech_stack_side,
    };
    use crate::models::common::QueryParams;
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
    fn resolve_tech_stack_side_should_fallback_to_backend_for_unknown_values() {
        assert_eq!(resolve_tech_stack_side(Some("frontend")), "frontend");
        assert_eq!(resolve_tech_stack_side(Some("backend")), "backend");
        assert_eq!(resolve_tech_stack_side(Some("gateway")), "backend");
        assert_eq!(resolve_tech_stack_side(None), "backend");
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
    fn parse_owner_filter_values_should_support_json_array_and_plain_text() {
        assert_eq!(
            parse_owner_filter_values(r#"["alice","bob"]"#),
            vec!["alice", "bob"]
        );
        assert_eq!(parse_owner_filter_values("alice"), vec!["alice"]);
        assert_eq!(parse_owner_filter_values("   "), Vec::<String>::new());
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
}
