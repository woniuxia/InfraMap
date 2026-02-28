use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::dependency::Dependency;
use crate::validation::validate_dependency;

fn row_to_dependency(row: &rusqlite::Row) -> rusqlite::Result<Dependency> {
    Ok(Dependency {
        id: row.get(0)?,
        source_id: row.get(1)?,
        source_type: row.get(2)?,
        target_id: row.get(3)?,
        target_type: row.get(4)?,
        relation_type: row.get(5)?,
        description: row.get(6)?,
        is_deleted: row.get(7)?,
        deleted_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

const SELECT_COLUMNS: &str = "id, source_id, source_type, target_id, target_type, relation_type, \
     description, is_deleted, deleted_at, created_at, updated_at";

#[tauri::command]
pub fn list_dependencies(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Dependency>> {
    let command = "list_dependencies";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let search_columns: &[&str] = &[];
    let filter_columns = &[
        "source_id",
        "source_type",
        "target_id",
        "target_type",
        "relation_type",
    ];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(&conn, "dependencies", &where_clause, &sql_params)
        .map_err(|e| AppError::from_db_error(command, "查询依赖关系数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM dependencies {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询依赖关系列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_dependency)
        .map_err(|e| AppError::from_db_error(command, "读取依赖关系列表", e))?;

    let data: Vec<Dependency> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn save_dependency(pool: State<DbPool>, data: Dependency) -> AppResult<()> {
    let command = "save_dependency";

    validate_dependency(&data).map_err(|e| AppError::validation(command, e))?;

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let now = chrono::Utc::now().to_rfc3339();

    let is_new = data.id.is_empty() || {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dependencies WHERE id = ?1 AND is_deleted = 0",
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
                "INSERT INTO dependencies (id, source_id, source_type, target_id, target_type, relation_type,
                                           description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,0,NULL,?8,?8)",
                rusqlite::params![
                    id,
                    data.source_id,
                    data.source_type,
                    data.target_id,
                    data.target_type,
                    data.relation_type,
                    data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建依赖关系", e))?;
            insert_audit_log(&conn, "create", "dependency", &id, None, None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        } else {
            conn.execute(
                "UPDATE dependencies SET source_id=?1, source_type=?2, target_id=?3, target_type=?4,
                                         relation_type=?5, description=?6, updated_at=?7
                 WHERE id=?8 AND is_deleted=0",
                rusqlite::params![
                    data.source_id,
                    data.source_type,
                    data.target_id,
                    data.target_type,
                    data.relation_type,
                    data.description,
                    now,
                    data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新依赖关系", e))?;
            insert_audit_log(&conn, "update", "dependency", &data.id, None, None)
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
pub fn soft_delete_dependency(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "soft_delete_dependency";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    match soft_delete(&conn, "dependencies", &id) {
        Ok(()) => match insert_audit_log(&conn, "delete", "dependency", &id, None, None) {
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
            Err(AppError::from_db_error(command, "删除依赖关系", e))
        }
    }
}
