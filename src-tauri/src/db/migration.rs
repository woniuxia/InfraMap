use super::pool::DbPool;
use super::schema::{ensure_taxonomy_schema_consistency, MIGRATIONS};

fn run_post_migration_hook(conn: &rusqlite::Connection, version: i32) -> Result<(), String> {
    match version {
        12 => super::schema::migrate_taxonomy_v2(conn)
            .map_err(|e| format!("Post-migration hook v{} failed: {}", version, e)),
        _ => Ok(()),
    }
}

pub fn run_migrations(pool: &DbPool) -> Result<(), String> {
    let conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    // Ensure schema_version table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| format!("Failed to create schema_version table: {}", e))?;

    // Get current version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to query schema version: {}", e))?;

    let target_version = MIGRATIONS.last().map(|(v, _)| *v).unwrap_or(0);

    if current_version < target_version {
        // Execute pending migrations in order
        for (version, sql) in MIGRATIONS.iter() {
            if *version <= current_version {
                continue;
            }

            // Each migration in a transaction
            conn.execute_batch("BEGIN TRANSACTION;").map_err(|e| {
                format!(
                    "Failed to begin transaction for migration v{}: {}",
                    version, e
                )
            })?;

            match conn.execute_batch(sql) {
                Ok(_) => {
                    let now = chrono::Utc::now().to_rfc3339();
                    conn.execute(
                        "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
                        rusqlite::params![version, now],
                    )
                    .map_err(|e| {
                        let _ = conn.execute_batch("ROLLBACK;");
                        format!("Failed to record migration v{}: {}", version, e)
                    })?;
                    conn.execute_batch("COMMIT;")
                        .map_err(|e| format!("Failed to commit migration v{}: {}", version, e))?;
                    run_post_migration_hook(&conn, *version)?;
                }
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK;");
                    return Err(format!(
                        "Migration v{} failed: {}. Startup aborted.",
                        version, e
                    ));
                }
            }
        }
    }

    if let Err(error) = ensure_taxonomy_schema_consistency(&conn) {
        eprintln!(
            "[run_migrations] taxonomy schema consistency check failed: {}",
            error
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_migrations;
    use crate::db::init_db_pool;

    #[test]
    fn run_migrations_should_repair_taxonomy_schema_even_when_up_to_date() {
        let db_path =
            std::env::temp_dir().join(format!("inframap-migration-{}.db", uuid::Uuid::new_v4()));
        let db_path_str = db_path.to_string_lossy().to_string();

        let pool = init_db_pool(&db_path_str).expect("init db pool");
        run_migrations(&pool).expect("run initial migrations");

        {
            let conn = pool.get().expect("get connection");
            conn.execute("DROP INDEX IF EXISTS idx_taxonomy_term_stats_recent", [])
                .expect("drop taxonomy index");
        }

        run_migrations(&pool).expect("run migrations again");

        let conn = pool.get().expect("get connection");
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_taxonomy_term_stats_recent'",
                [],
                |row| row.get(0),
            )
            .expect("query index existence");
        assert_eq!(exists, 1, "missing taxonomy index should be repaired");

        drop(conn);
        drop(pool);
        let _ = std::fs::remove_file(db_path);
    }
}
