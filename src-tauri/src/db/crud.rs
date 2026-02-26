use rusqlite::Connection;
use crate::models::common::QueryParams;

/// Build WHERE clause for soft-delete filtering + search + filters.
/// Returns (where_clause_string, params_vec).
pub fn build_where_clause(
    params: &QueryParams,
    search_columns: &[&str],
    filter_columns: &[&str],
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions = vec!["is_deleted = 0".to_string()];
    let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    // Search: OR across search_columns with LIKE
    if let Some(ref search) = params.search {
        if !search.trim().is_empty() {
            let like_conditions: Vec<String> = search_columns
                .iter()
                .map(|col| format!("{} LIKE ?", col))
                .collect();
            if !like_conditions.is_empty() {
                conditions.push(format!("({})", like_conditions.join(" OR ")));
                let like_val = format!("%{}%", search.trim());
                for _ in search_columns {
                    sql_params.push(Box::new(like_val.clone()));
                }
            }
        }
    }

    // Filters: exact match on allowed filter_columns
    if let Some(ref filters) = params.filters {
        for col in filter_columns {
            if let Some(val) = filters.get(*col) {
                if !val.is_empty() {
                    conditions.push(format!("{} = ?", col));
                    sql_params.push(Box::new(val.clone()));
                }
            }
        }
    }

    let where_clause = format!("WHERE {}", conditions.join(" AND "));
    (where_clause, sql_params)
}

/// Execute a count query for pagination total.
pub fn count_query(
    conn: &Connection,
    table: &str,
    where_clause: &str,
    sql_params: &[Box<dyn rusqlite::types::ToSql>],
) -> Result<u64, String> {
    let sql = format!("SELECT COUNT(*) FROM {} {}", table, where_clause);
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        sql_params.iter().map(|p| p.as_ref()).collect();
    conn.query_row(&sql, param_refs.as_slice(), |row| row.get::<_, i64>(0))
        .map(|c| c as u64)
        .map_err(|e| format!("Count query failed: {}", e))
}

/// Soft delete: set is_deleted=1, deleted_at=now, updated_at=now.
pub fn soft_delete(conn: &Connection, table: &str, id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let affected = conn
        .execute(
            &format!(
                "UPDATE {} SET is_deleted = 1, deleted_at = ?1, updated_at = ?2 WHERE id = ?3 AND is_deleted = 0",
                table
            ),
            rusqlite::params![now, now, id],
        )
        .map_err(|e| format!("Soft delete failed: {}", e))?;
    if affected == 0 {
        return Err(format!("Record not found or already deleted: {}", id));
    }
    Ok(())
}
