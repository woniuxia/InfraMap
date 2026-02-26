use tauri::State;
use crate::db::DbPool;
use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::models::common::{QueryParams, PagedResult};
use crate::models::application::Application;
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
        status: row.get(10)?,
        description: row.get(11)?,
        is_deleted: row.get(12)?,
        deleted_at: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

const SELECT_COLUMNS: &str =
    "id, name, type, address, port, tech_stack, deploy_mode, \
     env, git_repo, owner, status, description, is_deleted, deleted_at, created_at, updated_at";

#[tauri::command]
pub fn list_applications(pool: State<DbPool>, params: QueryParams) -> Result<PagedResult<Application>, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let search_columns = &["name", "address"];
    let filter_columns = &["type", "env", "status", "owner"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(&conn, "applications", &where_clause, &sql_params)?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM applications {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(param_refs.as_slice(), row_to_application)
        .map_err(|e| e.to_string())?;

    let data: Vec<Application> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult { data, total, page, page_size })
}

#[tauri::command]
pub fn get_application(pool: State<DbPool>, id: String) -> Result<Application, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    let sql = format!(
        "SELECT {} FROM applications WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_application)
        .map_err(|e| format!("Application not found: {}", e))
}

#[tauri::command]
pub fn save_application(pool: State<DbPool>, data: Application) -> Result<(), String> {
    validate_application(&data)?;

    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    let is_new = data.id.is_empty() || {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM applications WHERE id = ?1 AND is_deleted = 0",
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
                "INSERT INTO applications (id, name, type, address, port, tech_stack, deploy_mode,
                                           env, git_repo, owner, status, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,0,NULL,?13,?13)",
                rusqlite::params![id, data.name, data.app_type, data.address, data.port, data.tech_stack,
                                 data.deploy_mode, data.env, data.git_repo, data.owner, data.status, data.description, now],
            ).map_err(|e| format!("Insert failed: {}", e))?;
            insert_audit_log(&conn, "create", "application", &id, Some(&data.name), None)?;
        } else {
            conn.execute(
                "UPDATE applications SET name=?1, type=?2, address=?3, port=?4, tech_stack=?5, deploy_mode=?6,
                                         env=?7, git_repo=?8, owner=?9, status=?10, description=?11, updated_at=?12
                 WHERE id=?13 AND is_deleted=0",
                rusqlite::params![data.name, data.app_type, data.address, data.port, data.tech_stack,
                                 data.deploy_mode, data.env, data.git_repo, data.owner, data.status, data.description, now, data.id],
            ).map_err(|e| format!("Update failed: {}", e))?;
            insert_audit_log(&conn, "update", "application", &data.id, Some(&data.name), None)?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => { conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?; Ok(()) }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
    }
}

#[tauri::command]
pub fn soft_delete_application(pool: State<DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let name: Option<String> = conn.query_row(
        "SELECT name FROM applications WHERE id = ?1 AND is_deleted = 0",
        rusqlite::params![id],
        |row| row.get(0),
    ).ok();

    conn.execute_batch("BEGIN TRANSACTION;").map_err(|e| e.to_string())?;

    match soft_delete(&conn, "applications", &id) {
        Ok(()) => {
            match insert_audit_log(&conn, "delete", "application", &id, name.as_deref(), None) {
                Ok(()) => { conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?; Ok(()) }
                Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
            }
        }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
    }
}
