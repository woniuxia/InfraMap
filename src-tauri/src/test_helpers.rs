use crate::db::schema::MIGRATIONS;
use rusqlite::Connection;

/// Create an in-memory SQLite database and apply all migrations.
pub fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory db");
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;",
    )
    .expect("Failed to set pragmas");

    // Create schema_version table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )
    .expect("Failed to create schema_version table");

    // Apply all migrations
    for (version, sql) in MIGRATIONS.iter() {
        conn.execute_batch(sql)
            .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, datetime('now'))",
            rusqlite::params![version],
        )
        .unwrap_or_else(|e| panic!("Failed to record migration v{}: {}", version, e));
    }

    conn
}

pub fn insert_test_host(conn: &Connection, id: &str, hostname: &str, ip: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO hosts (id, hostname, ip_address, status, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'running', 0, ?4, ?5)",
        rusqlite::params![id, hostname, ip, now, now],
    )
    .unwrap_or_else(|e| panic!("insert_test_host failed: {}", e));

    conn.execute(
        "INSERT OR IGNORE INTO ip_addresses (id, ip_address, env, is_vip, real_ips, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, 'prod', 0, NULL, 0, ?3, ?4)",
        rusqlite::params![format!("ip-{}", id), ip, now, now],
    )
    .unwrap_or_else(|e| panic!("insert_test_host ip insert failed: {}", e));

    let ip_id: String = conn
        .query_row(
            "SELECT id FROM ip_addresses WHERE ip_address = ?1 AND env = 'prod' AND is_deleted = 0 LIMIT 1",
            rusqlite::params![ip],
            |row| row.get(0),
        )
        .unwrap_or_else(|e| panic!("insert_test_host query ip id failed: {}", e));

    conn.execute(
        "INSERT OR IGNORE INTO host_ip_bindings (id, host_id, ip_id, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, ?4, ?5)",
        rusqlite::params![format!("hb-{}-{}", id, ip_id), id, ip_id, now, now],
    )
    .unwrap_or_else(|e| panic!("insert_test_host binding insert failed: {}", e));
}

pub fn insert_test_application(conn: &Connection, id: &str, name: &str, env: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO applications (id, name, type, env, status, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, 'backend', ?3, 'running', 0, ?4, ?5)",
        rusqlite::params![id, name, env, now, now],
    )
    .unwrap_or_else(|e| panic!("insert_test_application failed: {}", e));
}

pub fn insert_test_middleware(conn: &Connection, id: &str, name: &str, category: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO middlewares (id, name, category, type, address, env, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'redis', '127.0.0.1', 'prod', 0, ?4, ?5)",
        rusqlite::params![id, name, category, now, now],
    ).unwrap_or_else(|e| panic!("insert_test_middleware failed: {}", e));
}

pub fn insert_test_nginx_config(conn: &Connection, id: &str, name: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO nginx_configs (id, name, env, status, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, 'prod', 'running', 0, ?3, ?4)",
        rusqlite::params![id, name, now, now],
    )
    .unwrap_or_else(|e| panic!("insert_test_nginx_config failed: {}", e));
}

pub fn insert_test_dependency(
    conn: &Connection,
    id: &str,
    source_id: &str,
    source_type: &str,
    target_id: &str,
    target_type: &str,
    relation_type: &str,
) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO dependencies (id, source_id, source_type, target_id, target_type, relation_type, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
        rusqlite::params![id, source_id, source_type, target_id, target_type, relation_type, now, now],
    ).unwrap_or_else(|e| panic!("insert_test_dependency failed: {}", e));
}

pub fn insert_test_deployment(
    conn: &Connection,
    id: &str,
    resource_id: &str,
    resource_type: &str,
    host_id: &str,
) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO deployments (id, resource_id, resource_type, host_id, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
        rusqlite::params![id, resource_id, resource_type, host_id, now, now],
    ).unwrap_or_else(|e| panic!("insert_test_deployment failed: {}", e));
}
