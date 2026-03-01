use tauri::State;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::audit_log::AuditLog;
use crate::models::common::{PagedResult, QueryParams};

fn row_to_audit_log(row: &rusqlite::Row) -> rusqlite::Result<AuditLog> {
    Ok(AuditLog {
        id: row.get(0)?,
        action: row.get(1)?,
        resource_type: row.get(2)?,
        resource_id: row.get(3)?,
        resource_name: row.get(4)?,
        changes: row.get(5)?,
        created_at: row.get(6)?,
    })
}

const SELECT_COLUMNS: &str =
    "id, action, resource_type, resource_id, resource_name, changes, created_at";

#[tauri::command]
pub fn list_audit_logs(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<AuditLog>> {
    let command = "list_audit_logs";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let mut conditions: Vec<String> = Vec::new();
    let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref search) = params.search {
        if !search.trim().is_empty() {
            conditions.push("resource_name LIKE ?".to_string());
            sql_params.push(Box::new(format!("%{}%", search.trim())));
        }
    }

    if let Some(ref filters) = params.filters {
        if let Some(action) = filters.get("action") {
            if !action.is_empty() {
                conditions.push("action = ?".to_string());
                sql_params.push(Box::new(action.clone()));
            }
        }
        if let Some(resource_type) = filters.get("resource_type") {
            if !resource_type.is_empty() {
                conditions.push("resource_type = ?".to_string());
                sql_params.push(Box::new(resource_type.clone()));
            }
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) FROM audit_logs {}", where_clause);
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        sql_params.iter().map(|p| p.as_ref()).collect();
    let total: u64 = conn
        .query_row(&count_sql, param_refs.as_slice(), |row| {
            row.get::<_, i64>(0)
        })
        .map(|c| c as u64)
        .map_err(|e| AppError::from_db_error(command, "统计审计日志数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM audit_logs {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "读取审计日志", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_audit_log)
        .map_err(|e| AppError::from_db_error(command, "遍历审计日志", e))?;

    let data: Vec<AuditLog> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}
