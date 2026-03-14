use crate::db::migrations::MIGRATIONS;
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

        // Run post-migration hooks
        if *version == 12 {
            crate::db::schema::migrate_taxonomy_v2(&conn)
                .unwrap_or_else(|e| panic!("Post-migration hook v{} failed: {}", version, e));
        }
    }

    conn
}

pub fn insert_test_host(conn: &Connection, id: &str, hostname: &str, ip: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO hosts (id, hostname, env, status, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, 'prod', 'running', 0, ?3, ?4)",
        rusqlite::params![id, hostname, now, now],
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

pub fn insert_test_system(conn: &Connection, id: &str, name: &str, env: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO systems (id, name, code, description, env, status, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, NULL, NULL, ?3, 'active', 0, ?4, ?5)",
        rusqlite::params![id, name, env, now, now],
    )
    .unwrap_or_else(|e| panic!("insert_test_system failed: {}", e));
}

pub fn insert_test_service(conn: &Connection, id: &str, name: &str, env: &str, system_id: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO services (id, name, type, env, system_id, status, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, 'backend', ?3, ?4, 'running', 0, ?5, ?6)",
        rusqlite::params![id, name, env, system_id, now, now],
    )
    .unwrap_or_else(|e| panic!("insert_test_service failed: {}", e));
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
        "INSERT INTO nginx_configs (id, name, endpoints, env, status, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, '[]', 'prod', 'running', 0, ?3, ?4)",
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
    let source_ref = format!("{}:{}", source_type, source_id);
    let target_ref = format!("{}:{}", target_type, target_id);
    let pair_key = if source_ref <= target_ref {
        format!("{}|{}|{}", source_ref, target_ref, relation_type)
    } else {
        format!("{}|{}|{}", target_ref, source_ref, relation_type)
    };

    conn.execute(
        "INSERT INTO call_relations (id, pair_key, owner_id, owner_type, peer_id, peer_type, direction, relation_type, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'upstream', ?7, 0, ?8, ?9)",
        rusqlite::params![
            format!("{}-a", id),
            pair_key,
            source_id,
            source_type,
            target_id,
            target_type,
            relation_type,
            now,
            now
        ],
    )
    .unwrap_or_else(|e| panic!("insert_test_dependency upstream failed: {}", e));

    conn.execute(
        "INSERT INTO call_relations (id, pair_key, owner_id, owner_type, peer_id, peer_type, direction, relation_type, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'downstream', ?7, 0, ?8, ?9)",
        rusqlite::params![
            format!("{}-b", id),
            pair_key,
            target_id,
            target_type,
            source_id,
            source_type,
            relation_type,
            now,
            now
        ],
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
