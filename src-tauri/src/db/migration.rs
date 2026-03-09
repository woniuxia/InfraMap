use super::migrations::hooks::{run_post_migration_hook, run_post_migration_repairs};
use super::migrations::MIGRATIONS;
use super::pool::DbPool;
use super::{get_conn, transaction::with_transaction};
use crate::error::{AppError, AppResult};

pub fn run_migrations(pool: &DbPool) -> AppResult<()> {
    let command = "run_migrations";
    let conn = get_conn(pool, command)?;

    // Ensure schema_version table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|error| AppError::from_db_error(command, "初始化 schema_version 表", error))?;

    // Get current version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .map_err(|error| AppError::from_db_error(command, "读取当前迁移版本", error))?;

    let target_version = MIGRATIONS.last().map(|(v, _)| *v).unwrap_or(0);

    if current_version < target_version {
        // Execute pending migrations in order
        for (version, sql) in MIGRATIONS.iter() {
            if *version <= current_version {
                continue;
            }

            with_transaction(&conn, command, |conn| {
                conn.execute_batch(sql).map_err(|error| {
                    AppError::from_db_error(command, &format!("执行迁移 v{version}"), error)
                })?;

                let now = chrono::Utc::now().to_rfc3339();
                conn.execute(
                    "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
                    rusqlite::params![version, now],
                )
                .map_err(|error| {
                    AppError::from_db_error(command, &format!("记录迁移版本 v{version}"), error)
                })?;

                Ok(())
            })?;

            run_post_migration_hook(command, &conn, *version)?;
        }
    }

    run_post_migration_repairs(command, &conn);

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
