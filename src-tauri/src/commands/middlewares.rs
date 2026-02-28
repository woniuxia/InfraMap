use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::middleware::Middleware;
use crate::validation::validate_middleware;

fn row_to_middleware(row: &rusqlite::Row) -> rusqlite::Result<Middleware> {
    Ok(Middleware {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(2)?,
        mw_type: row.get(3)?,
        address: row.get(4)?,
        port: row.get(5)?,
        version: row.get(6)?,
        env: row.get(7)?,
        description: row.get(8)?,
        is_deleted: row.get(9)?,
        deleted_at: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

const SELECT_COLUMNS: &str = "id, name, category, type, address, port, version, \
     env, description, is_deleted, deleted_at, created_at, updated_at";

#[tauri::command]
pub fn list_middlewares(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Middleware>> {
    let command = "list_middlewares";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let search_columns = &["name", "address"];
    let filter_columns = &["category", "type", "env"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(&conn, "middlewares", &where_clause, &sql_params)
        .map_err(|e| AppError::from_db_error(command, "查询中间件数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM middlewares {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询中间件列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_middleware)
        .map_err(|e| AppError::from_db_error(command, "读取中间件列表", e))?;

    let data: Vec<Middleware> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_middleware(pool: State<DbPool>, id: String) -> AppResult<Middleware> {
    let command = "get_middleware";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let sql = format!(
        "SELECT {} FROM middlewares WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_middleware)
        .map_err(|e| AppError::not_found(command, "中间件不存在或已删除。", Some(e.to_string())))
}

#[tauri::command]
pub fn save_middleware(pool: State<DbPool>, data: Middleware) -> AppResult<()> {
    let command = "save_middleware";

    validate_middleware(&data).map_err(|e| AppError::validation(command, e))?;

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let now = chrono::Utc::now().to_rfc3339();

    let is_new = data.id.is_empty() || {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM middlewares WHERE id = ?1 AND is_deleted = 0",
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
                "INSERT INTO middlewares (id, name, category, type, address, port, version,
                                         env, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,0,NULL,?10,?10)",
                rusqlite::params![
                    id,
                    data.name,
                    data.category,
                    data.mw_type,
                    data.address,
                    data.port,
                    data.version,
                    data.env,
                    data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建中间件", e))?;
            insert_audit_log(&conn, "create", "middleware", &id, Some(&data.name), None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        } else {
            conn.execute(
                "UPDATE middlewares SET name=?1, category=?2, type=?3, address=?4, port=?5, version=?6,
                                       env=?7, description=?8, updated_at=?9
                 WHERE id=?10 AND is_deleted=0",
                rusqlite::params![
                    data.name,
                    data.category,
                    data.mw_type,
                    data.address,
                    data.port,
                    data.version,
                    data.env,
                    data.description,
                    now,
                    data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新中间件", e))?;
            insert_audit_log(
                &conn,
                "update",
                "middleware",
                &data.id,
                Some(&data.name),
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
pub fn soft_delete_middleware(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "soft_delete_middleware";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM middlewares WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    match soft_delete(&conn, "middlewares", &id) {
        Ok(()) => match insert_audit_log(&conn, "delete", "middleware", &id, name.as_deref(), None)
        {
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
            Err(AppError::from_db_error(command, "删除中间件", e))
        }
    }
}
