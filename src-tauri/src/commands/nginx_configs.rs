use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::nginx_config::NginxConfig;
use crate::validation::validate_nginx_config;

fn row_to_nginx_config(row: &rusqlite::Row) -> rusqlite::Result<NginxConfig> {
    Ok(NginxConfig {
        id: row.get(0)?,
        name: row.get(1)?,
        address: row.get(2)?,
        listen_port: row.get(3)?,
        strategy: row.get(4)?,
        upstream_servers: row.get(5)?,
        env: row.get(6)?,
        status: row.get(7)?,
        description: row.get(8)?,
        is_deleted: row.get(9)?,
        deleted_at: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

const SELECT_COLUMNS: &str = "id, name, address, listen_port, strategy, upstream_servers, \
     env, status, description, is_deleted, deleted_at, created_at, updated_at";

#[tauri::command]
pub fn list_nginx_configs(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<NginxConfig>> {
    let command = "list_nginx_configs";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let search_columns = &["name", "address"];
    let filter_columns = &["env", "status", "strategy"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(&conn, "nginx_configs", &where_clause, &sql_params)
        .map_err(|e| AppError::from_db_error(command, "查询网关配置数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM nginx_configs {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询网关配置列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_nginx_config)
        .map_err(|e| AppError::from_db_error(command, "读取网关配置列表", e))?;

    let data: Vec<NginxConfig> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_nginx_config(pool: State<DbPool>, id: String) -> AppResult<NginxConfig> {
    let command = "get_nginx_config";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let sql = format!(
        "SELECT {} FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_nginx_config)
        .map_err(|e| AppError::not_found(command, "网关配置不存在或已删除。", Some(e.to_string())))
}

#[tauri::command]
pub fn save_nginx_config(pool: State<DbPool>, data: NginxConfig) -> AppResult<String> {
    let command = "save_nginx_config";

    validate_nginx_config(&data).map_err(|e| AppError::validation(command, e))?;

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let now = chrono::Utc::now().to_rfc3339();

    let is_new = data.id.is_empty() || {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![data.id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        count == 0
    };

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<String> = (|| {
        if is_new {
            let id = if data.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                data.id.clone()
            };
            conn.execute(
                "INSERT INTO nginx_configs (id, name, address, listen_port, strategy, upstream_servers,
                                            env, status, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,0,NULL,?10,?10)",
                rusqlite::params![
                    id,
                    data.name,
                    data.address,
                    data.listen_port,
                    data.strategy,
                    data.upstream_servers,
                    data.env,
                    data.status,
                    data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建网关配置", e))?;
            insert_audit_log(&conn, "create", "nginx", &id, Some(&data.name), None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
            Ok(id)
        } else {
            conn.execute(
                "UPDATE nginx_configs SET name=?1, address=?2, listen_port=?3, strategy=?4, upstream_servers=?5,
                                          env=?6, status=?7, description=?8, updated_at=?9
                 WHERE id=?10 AND is_deleted=0",
                rusqlite::params![
                    data.name,
                    data.address,
                    data.listen_port,
                    data.strategy,
                    data.upstream_servers,
                    data.env,
                    data.status,
                    data.description,
                    now,
                    data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新网关配置", e))?;
            insert_audit_log(&conn, "update", "nginx", &data.id, Some(&data.name), None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
            Ok(data.id)
        }
    })();

    match result {
        Ok(id) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(id)
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

#[tauri::command]
pub fn soft_delete_nginx_config(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "soft_delete_nginx_config";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    match soft_delete(&conn, "nginx_configs", &id) {
        Ok(()) => match insert_audit_log(&conn, "delete", "nginx", &id, name.as_deref(), None) {
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
            Err(AppError::from_db_error(command, "删除网关配置", e))
        }
    }
}
