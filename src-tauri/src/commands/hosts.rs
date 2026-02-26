use tauri::State;
use crate::db::DbPool;
use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::models::common::{QueryParams, PagedResult};
use crate::models::host::Host;
use crate::validation::validate_host;

fn row_to_host(row: &rusqlite::Row) -> rusqlite::Result<Host> {
    Ok(Host {
        id: row.get(0)?,
        hostname: row.get(1)?,
        ip_address: row.get(2)?,
        os_type: row.get(3)?,
        cpu_info: row.get(4)?,
        ram_gb: row.get(5)?,
        disk_gb: row.get(6)?,
        status: row.get(7)?,
        tags: row.get(8)?,
        description: row.get(9)?,
        is_deleted: row.get(10)?,
        deleted_at: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

const SELECT_COLUMNS: &str =
    "id, hostname, ip_address, os_type, cpu_info, ram_gb, disk_gb, \
     status, tags, description, is_deleted, deleted_at, created_at, updated_at";

#[tauri::command]
pub fn list_hosts(pool: State<DbPool>, params: QueryParams) -> Result<PagedResult<Host>, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let search_columns = &["hostname", "ip_address"];
    let filter_columns = &["status", "os_type"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(&conn, "hosts", &where_clause, &sql_params)?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM hosts {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(param_refs.as_slice(), row_to_host)
        .map_err(|e| e.to_string())?;

    let data: Vec<Host> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult { data, total, page, page_size })
}

#[tauri::command]
pub fn get_host(pool: State<DbPool>, id: String) -> Result<Host, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    let sql = format!(
        "SELECT {} FROM hosts WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_host)
        .map_err(|e| format!("Host not found: {}", e))
}

#[tauri::command]
pub fn save_host(pool: State<DbPool>, data: Host) -> Result<(), String> {
    validate_host(&data)?;

    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    let is_new = data.id.is_empty() || {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM hosts WHERE id = ?1 AND is_deleted = 0",
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
                "INSERT INTO hosts (id, hostname, ip_address, os_type, cpu_info, ram_gb, disk_gb,
                                    status, tags, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,0,NULL,?11,?11)",
                rusqlite::params![id, data.hostname, data.ip_address, data.os_type, data.cpu_info,
                                 data.ram_gb, data.disk_gb, data.status, data.tags, data.description, now],
            ).map_err(|e| format!("Insert failed: {}", e))?;
            insert_audit_log(&conn, "create", "host", &id, Some(&data.hostname), None)?;
        } else {
            conn.execute(
                "UPDATE hosts SET hostname=?1, ip_address=?2, os_type=?3, cpu_info=?4, ram_gb=?5, disk_gb=?6,
                                  status=?7, tags=?8, description=?9, updated_at=?10
                 WHERE id=?11 AND is_deleted=0",
                rusqlite::params![data.hostname, data.ip_address, data.os_type, data.cpu_info,
                                 data.ram_gb, data.disk_gb, data.status, data.tags, data.description, now, data.id],
            ).map_err(|e| format!("Update failed: {}", e))?;
            insert_audit_log(&conn, "update", "host", &data.id, Some(&data.hostname), None)?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => { conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?; Ok(()) }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
    }
}

#[tauri::command]
pub fn soft_delete_host(pool: State<DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let name: Option<String> = conn.query_row(
        "SELECT hostname FROM hosts WHERE id = ?1 AND is_deleted = 0",
        rusqlite::params![id],
        |row| row.get(0),
    ).ok();

    conn.execute_batch("BEGIN TRANSACTION;").map_err(|e| e.to_string())?;

    match soft_delete(&conn, "hosts", &id) {
        Ok(()) => {
            match insert_audit_log(&conn, "delete", "host", &id, name.as_deref(), None) {
                Ok(()) => { conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?; Ok(()) }
                Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
            }
        }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK;"); Err(e) }
    }
}
