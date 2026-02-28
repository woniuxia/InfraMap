use super::pool::DbPool;
use super::schema::MIGRATIONS;

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

    if current_version >= target_version {
        return Ok(());
    }

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

    Ok(())
}
