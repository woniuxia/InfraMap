use tauri::State;
use serde::Serialize;
use crate::db::DbPool;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub db_connected: bool,
    pub db_path: String,
    pub table_count: i32,
    pub schema_version: i32,
}

#[tauri::command]
pub fn health_check(pool: State<DbPool>) -> Result<HealthStatus, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let table_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let schema_version: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    Ok(HealthStatus {
        status: "ok".to_string(),
        db_connected: true,
        db_path: "inframap.db".to_string(),
        table_count,
        schema_version,
    })
}

#[derive(Serialize)]
pub struct DbInfo {
    pub tables: Vec<String>,
    pub index_count: i32,
}

#[tauri::command]
pub fn get_db_info(pool: State<DbPool>) -> Result<DbInfo, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
    ).map_err(|e| e.to_string())?;

    let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let index_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    Ok(DbInfo { tables, index_count })
}
