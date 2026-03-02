use crate::error::{AppError, AppResult};

pub fn with_transaction<T, F>(
    conn: &rusqlite::Connection,
    command: &str,
    callback: F,
) -> AppResult<T>
where
    F: FnOnce(&rusqlite::Connection) -> AppResult<T>,
{
    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    match callback(conn) {
        Ok(value) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(value)
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::with_transaction;
    use crate::error::{AppError, AppErrorCode};

    #[test]
    fn with_transaction_should_commit_when_callback_succeeds() {
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        conn.execute("CREATE TABLE t (id INTEGER PRIMARY KEY)", [])
            .expect("create table");

        with_transaction(&conn, "test_tx", |db| {
            db.execute("INSERT INTO t (id) VALUES (1)", [])
                .map_err(|e| AppError::from_db_error("test_tx", "插入测试数据", e))?;
            Ok(())
        })
        .expect("transaction should commit");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM t", [], |row| row.get(0))
            .expect("count rows");
        assert_eq!(count, 1);
    }

    #[test]
    fn with_transaction_should_rollback_when_callback_fails() {
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        conn.execute("CREATE TABLE t (id INTEGER PRIMARY KEY)", [])
            .expect("create table");

        let err = with_transaction(&conn, "test_tx", |db| -> Result<(), AppError> {
            db.execute("INSERT INTO t (id) VALUES (1)", [])
                .map_err(|e| AppError::from_db_error("test_tx", "插入测试数据", e))?;
            Err(AppError::validation("test_tx", "boom"))
        })
        .expect_err("transaction should rollback");
        assert_eq!(err.code, AppErrorCode::ValidationError);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM t", [], |row| row.get(0))
            .expect("count rows");
        assert_eq!(count, 0);
    }
}
