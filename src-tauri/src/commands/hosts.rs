use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::soft_delete;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::host::Host;
use crate::validation::validate_host;

fn parse_filter_values(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if trimmed.starts_with('[') {
        if let Ok(values) = serde_json::from_str::<Vec<String>>(trimmed) {
            let normalized: Vec<String> = values
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect();
            if !normalized.is_empty() {
                return normalized;
            }
        }
    }

    vec![trimmed.to_string()]
}

fn build_hosts_where_clause(
    params: &QueryParams,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions = vec!["h.is_deleted = 0".to_string()];
    let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(search) = params.search.as_deref() {
        let search_text = search.trim();
        if !search_text.is_empty() {
            let like_value = format!("%{}%", search_text);
            conditions.push(
                "(h.hostname LIKE ? OR EXISTS (
                    SELECT 1
                    FROM host_ip_bindings hb
                    JOIN ip_addresses ia ON ia.id = hb.ip_id
                    WHERE hb.host_id = h.id
                      AND hb.is_deleted = 0
                      AND ia.is_deleted = 0
                      AND ia.ip_address LIKE ?
                ))"
                .to_string(),
            );
            sql_params.push(Box::new(like_value.clone()));
            sql_params.push(Box::new(like_value));
        }
    }

    if let Some(filters) = params.filters.as_ref() {
        for column in ["status", "os_type", "env"] {
            if let Some(raw_value) = filters.get(column) {
                let values = parse_filter_values(raw_value);
                if values.is_empty() {
                    continue;
                }

                if values.len() == 1 {
                    conditions.push(format!("h.{} = ?", column));
                    sql_params.push(Box::new(values[0].clone()));
                } else {
                    let placeholders = vec!["?"; values.len()].join(", ");
                    conditions.push(format!("h.{} IN ({})", column, placeholders));
                    for value in values {
                        sql_params.push(Box::new(value));
                    }
                }
            }
        }
    }

    (format!("WHERE {}", conditions.join(" AND ")), sql_params)
}

fn row_to_host(row: &rusqlite::Row) -> rusqlite::Result<Host> {
    Ok(Host {
        id: row.get(0)?,
        hostname: row.get(1)?,
        ip_address: row.get(2)?,
        ip_display: row.get(3)?,
        env: row.get(4)?,
        os_type: row.get(5)?,
        cpu_model: row.get(6)?,
        cpu_cores: row.get(7)?,
        cpu_threads: row.get(8)?,
        cpu_freq: row.get(9)?,
        ram_gb: row.get(10)?,
        disk_gb: row.get(11)?,
        status: row.get(12)?,
        tags: row.get(13)?,
        description: row.get(14)?,
        is_deleted: row.get(15)?,
        deleted_at: row.get(16)?,
        created_at: row.get(17)?,
        updated_at: row.get(18)?,
    })
}

const SELECT_COLUMNS: &str = "h.id, h.hostname,
     (SELECT ia.ip_address
      FROM host_ip_bindings hb
      JOIN ip_addresses ia ON ia.id = hb.ip_id
      WHERE hb.host_id = h.id AND hb.is_deleted = 0 AND ia.is_deleted = 0
      ORDER BY ia.created_at ASC
      LIMIT 1) AS ip_address,
     NULLIF((
      SELECT GROUP_CONCAT(ia.ip_address, ', ')
      FROM host_ip_bindings hb
      JOIN ip_addresses ia ON ia.id = hb.ip_id
      WHERE hb.host_id = h.id AND hb.is_deleted = 0 AND ia.is_deleted = 0
     ), '') AS ip_display,
     h.env, h.os_type, h.cpu_model, h.cpu_cores, h.cpu_threads, h.cpu_freq,
     h.ram_gb, h.disk_gb, h.status, h.tags, h.description,
     h.is_deleted, h.deleted_at, h.created_at, h.updated_at";

#[tauri::command]
pub fn list_hosts(pool: State<DbPool>, params: QueryParams) -> AppResult<PagedResult<Host>> {
    let command = "list_hosts";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let (where_clause, sql_params) = build_hosts_where_clause(&params);
    let count_sql = format!("SELECT COUNT(*) FROM hosts h {}", where_clause);
    let count_param_refs: Vec<&dyn rusqlite::types::ToSql> =
        sql_params.iter().map(|param| param.as_ref()).collect();
    let total: u64 = conn
        .query_row(&count_sql, count_param_refs.as_slice(), |row| {
            row.get::<_, i64>(0)
        })
        .map(|value| value as u64)
        .map_err(|e| AppError::from_db_error(command, "查询主机数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM hosts h {} ORDER BY h.created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );
    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|param| param.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询主机列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_host)
        .map_err(|e| AppError::from_db_error(command, "读取主机列表", e))?;
    let data: Vec<Host> = rows.filter_map(|row| row.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_host(pool: State<DbPool>, id: String) -> AppResult<Host> {
    let command = "get_host";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let sql = format!(
        "SELECT {} FROM hosts h WHERE h.id = ?1 AND h.is_deleted = 0",
        SELECT_COLUMNS
    );

    conn.query_row(&sql, rusqlite::params![id], row_to_host)
        .map_err(|e| AppError::not_found(command, "主机不存在或已删除。", Some(e.to_string())))
}

#[tauri::command]
pub fn save_host(pool: State<DbPool>, data: Host) -> AppResult<()> {
    let command = "save_host";
    validate_host(&data).map_err(|e| AppError::validation(command, e))?;

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let now = chrono::Utc::now().to_rfc3339();
    let ip_address_for_legacy = data.ip_address.clone().unwrap_or_default();

    let is_new = data.id.is_empty() || {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM hosts WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![data.id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        count == 0
    };

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<()> = (|| {
        if is_new {
            let id = if data.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                data.id.clone()
            };
            conn.execute(
                "INSERT INTO hosts (id, hostname, ip_address, env, os_type, cpu_model, cpu_cores, cpu_threads, cpu_freq,
                                    ram_gb, disk_gb, status, tags, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,0,NULL,?15,?15)",
                rusqlite::params![
                    id,
                    data.hostname,
                    ip_address_for_legacy,
                    data.env,
                    data.os_type,
                    data.cpu_model,
                    data.cpu_cores,
                    data.cpu_threads,
                    data.cpu_freq,
                    data.ram_gb,
                    data.disk_gb,
                    data.status,
                    data.tags,
                    data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建主机", e))?;
            insert_audit_log(&conn, "create", "host", &id, Some(&data.hostname), None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        } else {
            conn.execute(
                "UPDATE hosts SET hostname=?1, ip_address=?2, env=?3, os_type=?4, cpu_model=?5, cpu_cores=?6,
                                  cpu_threads=?7, cpu_freq=?8, ram_gb=?9, disk_gb=?10,
                                  status=?11, tags=?12, description=?13, updated_at=?14
                 WHERE id=?15 AND is_deleted=0",
                rusqlite::params![
                    data.hostname,
                    ip_address_for_legacy,
                    data.env,
                    data.os_type,
                    data.cpu_model,
                    data.cpu_cores,
                    data.cpu_threads,
                    data.cpu_freq,
                    data.ram_gb,
                    data.disk_gb,
                    data.status,
                    data.tags,
                    data.description,
                    now,
                    data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新主机", e))?;
            insert_audit_log(
                &conn,
                "update",
                "host",
                &data.id,
                Some(&data.hostname),
                None,
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        }
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
pub fn soft_delete_host(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "soft_delete_host";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let name: Option<String> = conn
        .query_row(
            "SELECT hostname FROM hosts WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    match soft_delete(&conn, "hosts", &id) {
        Ok(()) => match insert_audit_log(&conn, "delete", "host", &id, name.as_deref(), None) {
            Ok(()) => {
                conn.execute_batch("COMMIT;")
                    .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(AppError::from_db_error(command, "写入审计日志", e))
            }
        },
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(AppError::from_db_error(command, "删除主机", e))
        }
    }
}
