use tauri::State;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::dashboard::{DashboardStats, EnvCount};

fn count_active(conn: &rusqlite::Connection, table: &str) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {} WHERE is_deleted = 0", table),
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c as u64)
}

fn count_abnormal(conn: &rusqlite::Connection, table: &str) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM {} WHERE is_deleted = 0 AND status IN ('stopped', 'maintenance')",
            table
        ),
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c as u64)
}

fn query_env_distribution(conn: &rusqlite::Connection) -> Result<Vec<EnvCount>, rusqlite::Error> {
    let sql = "SELECT env, COUNT(*) as cnt FROM (
        SELECT env FROM applications WHERE is_deleted = 0
        UNION ALL
        SELECT env FROM middlewares WHERE is_deleted = 0
        UNION ALL
        SELECT env FROM nginx_configs WHERE is_deleted = 0
    ) GROUP BY env ORDER BY cnt DESC";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(EnvCount {
            env: row.get(0)?,
            count: row.get::<_, i64>(1)? as u64,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn get_dashboard_stats(pool: State<DbPool>) -> AppResult<DashboardStats> {
    let command = "get_dashboard_stats";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let host_total = count_active(&conn, "hosts")
        .map_err(|e| AppError::from_db_error(command, "统计主机总数", e))?;
    let host_abnormal = count_abnormal(&conn, "hosts")
        .map_err(|e| AppError::from_db_error(command, "统计主机异常数量", e))?;
    let application_total = count_active(&conn, "applications")
        .map_err(|e| AppError::from_db_error(command, "统计应用总数", e))?;
    let application_abnormal = count_abnormal(&conn, "applications")
        .map_err(|e| AppError::from_db_error(command, "统计应用异常数量", e))?;
    let middleware_total = count_active(&conn, "middlewares")
        .map_err(|e| AppError::from_db_error(command, "统计中间件总数", e))?;
    let nginx_total = count_active(&conn, "nginx_configs")
        .map_err(|e| AppError::from_db_error(command, "统计网关总数", e))?;
    let nginx_abnormal = count_abnormal(&conn, "nginx_configs")
        .map_err(|e| AppError::from_db_error(command, "统计网关异常数量", e))?;
    let deployment_total = count_active(&conn, "deployments")
        .map_err(|e| AppError::from_db_error(command, "统计部署总数", e))?;
    let dependency_total: u64 = conn
        .query_row(
            "SELECT COUNT(*) FROM call_relations WHERE is_deleted = 0 AND direction = 'upstream'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count as u64)
        .map_err(|e| AppError::from_db_error(command, "统计依赖总数", e))?;
    let env_distribution = query_env_distribution(&conn)
        .map_err(|e| AppError::from_db_error(command, "统计环境分布", e))?;

    Ok(DashboardStats {
        host_total,
        host_abnormal,
        application_total,
        application_abnormal,
        middleware_total,
        nginx_total,
        nginx_abnormal,
        deployment_total,
        dependency_total,
        env_distribution,
    })
}
