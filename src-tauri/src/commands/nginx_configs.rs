use tauri::State;
use crate::db::DbPool;
use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::models::common::{QueryParams, PagedResult};
use crate::models::nginx_config::NginxConfig;
use crate::validation::validate_nginx_config;

fn row_to_nginx_config(row: &rusqlite::Row) -> rusqlite::Result<NginxConfig> {
    Ok(NginxConfig {
        id: row.get(0)?,
        name: row.get(1)?,
        listen_port: row.get(2)?,
        strategy: row.get(3)?,
        upstream_servers: row.get(4)?,
        env: row.get(5)?,
        status: row.get(6)?,
        description: row.get(7)?,
        is_deleted: row.get(8)?,
        deleted_at: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

const SELECT_COLUMNS: &str =
    "id, name, listen_port, strategy, upstream_servers, \
     env, status, description, is_deleted, deleted_at, created_at, updated_at";

#[tauri::command]
pub fn list_nginx_configs(pool: State<DbPool>, params: QueryParams) -> Result<PagedResult<NginxConfig>, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let search_columns = &["name"];
    let filter_columns = &["env", "status", "strategy"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(&conn, "nginx_configs", &where_clause, &sql_params)?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM nginx_configs {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(param_refs.as_slice(), row_to_nginx_config)
        .map_err(|e| e.to_string())?;

    let data: Vec<NginxConfig> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult { data, total, page, page_size })
}

#[tauri::command]
pub fn get_nginx_config(pool: State<DbPool>, id: String) -> Result<NginxConfig, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    let sql = format!(
        "SELECT {} FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_nginx_config)
        .map_err(|e| format!("NginxConfig not found: {}", e))
}

#[tauri::command]
pub fn save_nginx_config(pool: State<DbPool>, data: NginxConfig) -> Result<(), String> {
    validate_nginx_config(&data)?;

    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    let is_new = data.id.is_empty() || {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![data.id],
            |row| row.get(0),
        ).unwrap_or(0);
        count == 0
    };

    conn.execute_batch("BEGIN TRANSACTION;").map_err(|e| e.to_string())?;

    let result: Result<(), String> = (|| {
        if is_new {
            let id = if data.id.is_empty() { uuid::Uuid::new_v4().to_string() } else { data.id.clone() };
            conn.execute(
                "INSERT INTO nginx_configs (id, name, listen_port, strategy, upstream_servers,
                                            env, status, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,0,NULL,?9,?9)",
                rusqlite::params![id, data.name, data.listen_port, data.strategy, data.upstream_servers,
                                 data.env, data.status, data.description, now],
            ).map_err(|e| format!("Insert failed: {}", e))?;
            insert_audit_log(&conn, "create", "nginx", &id, Some(&data.name), None)?;
        } else {
            conn.execute(
                "UPDATE nginx_configs SET name=?1, listen_port=?2, strategy=?3, upstream_servers=?4,
                                          env=?5, status=?6, description=?7, updated_at=?8
                 WHERE id=?9 AND is_deleted=0",
                rusqlite::params![data.name, data.listen_port, data.strategy, data.upstream_servers,
                                 data.env, data.status, data.description, now, data.id],
            ).map_err(|e| format!("Update failed: {}", e))?;
            insert_audit_log(&conn, "update", "nginx", &data.id, Some(&data.name), None)?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => { conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?; Ok(()) }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
    }
}

#[tauri::command]
pub fn soft_delete_nginx_config(pool: State<DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let name: Option<String> = conn.query_row(
        "SELECT name FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
        rusqlite::params![id],
        |row| row.get(0),
    ).ok();

    conn.execute_batch("BEGIN TRANSACTION;").map_err(|e| e.to_string())?;

    match soft_delete(&conn, "nginx_configs", &id) {
        Ok(()) => {
            match insert_audit_log(&conn, "delete", "nginx", &id, name.as_deref(), None) {
                Ok(()) => { conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?; Ok(()) }
                Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
            }
        }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
    }
}
