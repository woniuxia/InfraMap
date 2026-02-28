use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub db_connected: bool,
    pub db_path: String,
    pub table_count: i32,
    pub schema_version: i32,
}

#[tauri::command]
pub fn health_check(pool: State<DbPool>) -> AppResult<HealthStatus> {
    let command = "health_check";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let table_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| AppError::db_query_failed(command, "查询表数量", e))?;

    let schema_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .map_err(|e| AppError::db_query_failed(command, "查询 schema 版本", e))?;

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
pub fn get_db_info(pool: State<DbPool>) -> AppResult<DbInfo> {
    let command = "get_db_info";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
    )
    .map_err(|e| AppError::db_query_failed(command, "读取数据库表", e))?;

    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| AppError::db_query_failed(command, "遍历数据库表", e))?
        .filter_map(|r| r.ok())
        .collect();

    let index_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| AppError::db_query_failed(command, "查询索引数量", e))?;

    Ok(DbInfo {
        tables,
        index_count,
    })
}
