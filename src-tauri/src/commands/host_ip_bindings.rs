use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::ip_address::IpAddress;

fn list_host_ip_bindings_inner(
    command: &str,
    conn: &rusqlite::Connection,
    host_id: &str,
) -> AppResult<Vec<IpAddress>> {
    let host_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM hosts WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![host_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::from_db_error(command, "查询服务器", e))?;
    if host_exists == 0 {
        return Err(AppError::not_found(command, "服务器不存在或已删除。", None));
    }

    let mut stmt = conn
        .prepare(
            "SELECT ia.id, ia.ip_address, ia.env, ia.is_vip, ia.real_ips, ia.tags, ia.description, ia.is_deleted, ia.deleted_at, ia.created_at, ia.updated_at
             FROM host_ip_bindings hb
             JOIN ip_addresses ia ON ia.id = hb.ip_id
             WHERE hb.host_id = ?1 AND hb.is_deleted = 0 AND ia.is_deleted = 0
             ORDER BY ia.created_at DESC",
        )
        .map_err(|e| AppError::from_db_error(command, "查询主机IP绑定", e))?;
    let rows = stmt
        .query_map(rusqlite::params![host_id], |row| {
            Ok(IpAddress {
                id: row.get(0)?,
                ip_address: row.get(1)?,
                env: row.get(2)?,
                is_vip: row.get(3)?,
                real_ips: row.get(4)?,
                tags: row.get(5)?,
                description: row.get(6)?,
                is_deleted: row.get(7)?,
                deleted_at: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| AppError::from_db_error(command, "读取主机IP绑定", e))?;

    Ok(rows.filter_map(|row| row.ok()).collect())
}

fn bind_host_ip_inner(
    command: &str,
    conn: &rusqlite::Connection,
    host_id: &str,
    ip_id: &str,
) -> AppResult<()> {
    let host_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM hosts WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![host_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::from_db_error(command, "查询服务器", e))?;
    if host_exists == 0 {
        return Err(AppError::not_found(command, "服务器不存在或已删除。", None));
    }

    let ip_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ip_addresses WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![ip_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::from_db_error(command, "查询IP资源", e))?;
    if ip_exists == 0 {
        return Err(AppError::not_found(command, "IP资源不存在或已删除。", None));
    }

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<()> = (|| {
        let existing = conn
            .query_row(
                "SELECT id, is_deleted FROM host_ip_bindings WHERE host_id = ?1 AND ip_id = ?2 LIMIT 1",
                rusqlite::params![host_id, ip_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )
            .ok();

        match existing {
            Some((id, is_deleted)) if is_deleted == 0 => {
                conn.execute(
                    "UPDATE host_ip_bindings SET updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![now, id],
                )
                .map_err(|e| AppError::from_db_error(command, "更新绑定时间", e))?;
            }
            Some((id, _)) => {
                conn.execute(
                    "UPDATE host_ip_bindings
                     SET is_deleted = 0, deleted_at = NULL, updated_at = ?1
                     WHERE id = ?2",
                    rusqlite::params![now, id],
                )
                .map_err(|e| AppError::from_db_error(command, "恢复IP绑定", e))?;
            }
            None => {
                conn.execute(
                    "INSERT INTO host_ip_bindings (id, host_id, ip_id, is_deleted, deleted_at, created_at, updated_at)
                     VALUES (?1, ?2, ?3, 0, NULL, ?4, ?4)",
                    rusqlite::params![uuid::Uuid::new_v4().to_string(), host_id, ip_id, now],
                )
                .map_err(|e| AppError::from_db_error(command, "创建IP绑定", e))?;
            }
        }

        insert_audit_log(
            conn,
            "update",
            "host_ip_binding",
            host_id,
            Some(host_id),
            Some(&format!("bind ip_id={}", ip_id)),
        )
        .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;

        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(())
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn unbind_host_ip_inner(
    command: &str,
    conn: &rusqlite::Connection,
    host_id: &str,
    ip_id: &str,
) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<()> = (|| {
        let affected = conn
            .execute(
                "UPDATE host_ip_bindings
                 SET is_deleted = 1, deleted_at = ?1, updated_at = ?2
                 WHERE host_id = ?3 AND ip_id = ?4 AND is_deleted = 0",
                rusqlite::params![now, now, host_id, ip_id],
            )
            .map_err(|e| AppError::from_db_error(command, "解绑主机IP", e))?;
        if affected == 0 {
            return Err(AppError::not_found(
                command,
                "主机与IP绑定关系不存在或已删除。",
                None,
            ));
        }

        insert_audit_log(
            conn,
            "update",
            "host_ip_binding",
            host_id,
            Some(host_id),
            Some(&format!("unbind ip_id={}", ip_id)),
        )
        .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(())
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

#[tauri::command]
pub fn list_host_ip_bindings(pool: State<DbPool>, host_id: String) -> AppResult<Vec<IpAddress>> {
    let command = "list_host_ip_bindings";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    list_host_ip_bindings_inner(command, &conn, &host_id)
}

#[tauri::command]
pub fn bind_host_ip(pool: State<DbPool>, host_id: String, ip_id: String) -> AppResult<()> {
    let command = "bind_host_ip";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    bind_host_ip_inner(command, &conn, &host_id, &ip_id)
}

#[tauri::command]
pub fn unbind_host_ip(pool: State<DbPool>, host_id: String, ip_id: String) -> AppResult<()> {
    let command = "unbind_host_ip";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    unbind_host_ip_inner(command, &conn, &host_id, &ip_id)
}

#[cfg(test)]
mod tests {
    use crate::error::AppErrorCode;
    use crate::test_helpers::setup_test_db;

    use super::{bind_host_ip_inner, list_host_ip_bindings_inner, unbind_host_ip_inner};

    fn seed_test_host_without_binding(conn: &rusqlite::Connection, id: &str, hostname: &str) {
        conn.execute(
            "INSERT INTO hosts (id, hostname, env, status, is_deleted, created_at, updated_at)
             VALUES (?1, ?2, 'prod', 'running', 0, datetime('now'), datetime('now'))",
            rusqlite::params![id, hostname],
        )
        .expect("seed host");
    }

    fn seed_test_ip(conn: &rusqlite::Connection, id: &str, ip: &str, env: &str) {
        conn.execute(
            "INSERT INTO ip_addresses (id, ip_address, env, is_vip, real_ips, is_deleted, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, NULL, 0, datetime('now'), datetime('now'))",
            rusqlite::params![id, ip, env],
        )
        .expect("seed ip");
    }

    #[test]
    fn bind_and_list_host_ip_bindings_should_work() {
        let conn = setup_test_db();
        seed_test_host_without_binding(&conn, "h1", "server-1");
        seed_test_ip(&conn, "ip-1", "10.0.0.21", "prod");

        bind_host_ip_inner("test", &conn, "h1", "ip-1").expect("bind");
        let bindings = list_host_ip_bindings_inner("test", &conn, "h1").expect("list");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].ip_address, "10.0.0.21");
    }

    #[test]
    fn unbind_host_ip_should_soft_delete_relation() {
        let conn = setup_test_db();
        seed_test_host_without_binding(&conn, "h1", "server-1");
        seed_test_ip(&conn, "ip-1", "10.0.0.21", "prod");
        bind_host_ip_inner("test", &conn, "h1", "ip-1").expect("bind");

        unbind_host_ip_inner("test", &conn, "h1", "ip-1").expect("unbind");
        let bindings = list_host_ip_bindings_inner("test", &conn, "h1").expect("list");
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn bind_host_ip_should_allow_same_ip_for_multiple_hosts() {
        let conn = setup_test_db();
        seed_test_host_without_binding(&conn, "h1", "server-1");
        seed_test_host_without_binding(&conn, "h2", "server-2");
        seed_test_ip(&conn, "ip-1", "10.0.0.21", "prod");

        bind_host_ip_inner("test", &conn, "h1", "ip-1").expect("bind h1");
        bind_host_ip_inner("test", &conn, "h2", "ip-1").expect("bind h2");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM host_ip_bindings WHERE ip_id='ip-1' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("count bindings");
        assert_eq!(count, 2);
    }

    #[test]
    fn unbind_non_existing_relation_should_return_not_found() {
        let conn = setup_test_db();
        let err =
            unbind_host_ip_inner("test", &conn, "h1", "ip-not-exist").expect_err("should fail");
        assert_eq!(err.code, AppErrorCode::NotFound);
    }
}
