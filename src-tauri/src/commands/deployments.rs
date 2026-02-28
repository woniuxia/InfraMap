use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::deployment::Deployment;
use crate::validation::validate_deployment;

fn row_to_deployment(row: &rusqlite::Row) -> rusqlite::Result<Deployment> {
    Ok(Deployment {
        id: row.get(0)?,
        resource_id: row.get(1)?,
        resource_type: row.get(2)?,
        host_id: row.get(3)?,
        port: row.get(4)?,
        is_deleted: row.get(5)?,
        deleted_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

const SELECT_COLUMNS: &str = "id, resource_id, resource_type, host_id, port, \
     is_deleted, deleted_at, created_at, updated_at";

#[tauri::command]
pub fn list_deployments(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Deployment>> {
    let command = "list_deployments";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let search_columns: &[&str] = &[];
    let filter_columns = &["resource_id", "resource_type", "host_id"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(&conn, "deployments", &where_clause, &sql_params)
        .map_err(|e| AppError::from_db_error(command, "查询部署关系数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM deployments {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询部署关系列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_deployment)
        .map_err(|e| AppError::from_db_error(command, "读取部署关系列表", e))?;

    let data: Vec<Deployment> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn save_deployment(pool: State<DbPool>, data: Deployment) -> AppResult<()> {
    let command = "save_deployment";

    validate_deployment(&data).map_err(|e| AppError::validation(command, e))?;

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let now = chrono::Utc::now().to_rfc3339();

    let is_new = data.id.is_empty() || {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM deployments WHERE id = ?1 AND is_deleted = 0",
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
                "INSERT INTO deployments (id, resource_id, resource_type, host_id, port,
                                          is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,0,NULL,?6,?6)",
                rusqlite::params![
                    id,
                    data.resource_id,
                    data.resource_type,
                    data.host_id,
                    data.port,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建部署关系", e))?;
            insert_audit_log(&conn, "create", "deployment", &id, None, None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        } else {
            conn.execute(
                "UPDATE deployments SET resource_id=?1, resource_type=?2, host_id=?3, port=?4, updated_at=?5
                 WHERE id=?6 AND is_deleted=0",
                rusqlite::params![data.resource_id, data.resource_type, data.host_id, data.port, now, data.id],
            )
            .map_err(|e| AppError::from_db_error(command, "更新部署关系", e))?;
            insert_audit_log(&conn, "update", "deployment", &data.id, None, None)
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
pub fn soft_delete_deployment(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "soft_delete_deployment";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    match soft_delete(&conn, "deployments", &id) {
        Ok(()) => match insert_audit_log(&conn, "delete", "deployment", &id, None, None) {
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
            Err(AppError::from_db_error(command, "删除部署关系", e))
        }
    }
}
