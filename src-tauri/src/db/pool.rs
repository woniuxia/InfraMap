use crate::error::{AppError, AppResult};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConn = PooledConnection<SqliteConnectionManager>;

pub fn init_db_pool(db_path: &str) -> AppResult<DbPool> {
    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
                 PRAGMA foreign_keys=ON;
                 PRAGMA busy_timeout=5000;",
        )
    });
    let pool = Pool::builder()
        .max_size(8)
        .build(manager)
        .map_err(|error| {
            AppError::db_unavailable("init_db_pool", format!("Pool error: {}", error))
        })?;

    Ok(pool)
}

pub fn get_conn(pool: &DbPool, command: &str) -> AppResult<DbConn> {
    pool.get()
        .map_err(|error| AppError::db_unavailable(command, format!("Pool error: {}", error)))
}
