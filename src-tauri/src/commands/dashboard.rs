use tauri::State;
use crate::db::DbPool;
use crate::models::dashboard::{DashboardStats, EnvCount};

fn count_active(conn: &rusqlite::Connection, table: &str) -> Result<u64, String> {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {} WHERE is_deleted = 0", table),
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c as u64)
    .map_err(|e| format!("Count failed for {}: {}", table, e))
}

fn count_abnormal(conn: &rusqlite::Connection, table: &str) -> Result<u64, String> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM {} WHERE is_deleted = 0 AND status IN ('stopped', 'maintenance')",
            table
        ),
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c as u64)
    .map_err(|e| format!("Abnormal count failed for {}: {}", table, e))
}

fn query_env_distribution(conn: &rusqlite::Connection) -> Result<Vec<EnvCount>, String> {
    let sql = "SELECT env, COUNT(*) as cnt FROM (
        SELECT env FROM applications WHERE is_deleted = 0
        UNION ALL
        SELECT env FROM middlewares WHERE is_deleted = 0
        UNION ALL
        SELECT env FROM nginx_configs WHERE is_deleted = 0
    ) GROUP BY env ORDER BY cnt DESC";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(EnvCount {
                env: row.get(0)?,
                count: row.get::<_, i64>(1)? as u64,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn get_dashboard_stats(pool: State<DbPool>) -> Result<DashboardStats, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;

    let host_total = count_active(&conn, "hosts")?;
    let host_abnormal = count_abnormal(&conn, "hosts")?;
    let application_total = count_active(&conn, "applications")?;
    let application_abnormal = count_abnormal(&conn, "applications")?;
    let middleware_total = count_active(&conn, "middlewares")?;
    let nginx_total = count_active(&conn, "nginx_configs")?;
    let nginx_abnormal = count_abnormal(&conn, "nginx_configs")?;
    let deployment_total = count_active(&conn, "deployments")?;
    let dependency_total = count_active(&conn, "dependencies")?;
    let env_distribution = query_env_distribution(&conn)?;

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
