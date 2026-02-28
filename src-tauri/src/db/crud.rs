use rusqlite::Connection;
use crate::models::common::QueryParams;

fn parse_filter_values(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if trimmed.starts_with('[') {
        if let Ok(values) = serde_json::from_str::<Vec<String>>(trimmed) {
            let normalized: Vec<String> = values
                .into_iter()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect();
            if !normalized.is_empty() {
                return normalized;
            }
        }
    }

    vec![trimmed.to_string()]
}

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
                let values = parse_filter_values(val);
                if !values.is_empty() {
                    if values.len() == 1 {
                        conditions.push(format!("{} = ?", col));
                        sql_params.push(Box::new(values[0].clone()));
                    } else {
                        let placeholders = vec!["?"; values.len()].join(", ");
                        conditions.push(format!("{} IN ({})", col, placeholders));
                        for value in values {
                            sql_params.push(Box::new(value));
                        }
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use crate::models::common::QueryParams;
    use std::collections::HashMap;

    #[test]
    fn test_build_where_clause_no_filters() {
        let params = QueryParams::default();
        let (clause, sql_params) = build_where_clause(&params, &["name"], &["status"]);
        assert_eq!(clause, "WHERE is_deleted = 0");
        assert_eq!(sql_params.len(), 0);
    }

    #[test]
    fn test_build_where_clause_with_search() {
        let params = QueryParams {
            search: Some("test".into()),
            ..Default::default()
        };
        let (clause, sql_params) = build_where_clause(&params, &["name", "description"], &[]);
        assert!(clause.contains("name LIKE ?"));
        assert!(clause.contains("description LIKE ?"));
        assert_eq!(sql_params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_with_filter() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "running".to_string());
        let params = QueryParams {
            filters: Some(filters),
            ..Default::default()
        };
        let (clause, sql_params) = build_where_clause(&params, &["name"], &["status"]);
        assert!(clause.contains("status = ?"));
        assert_eq!(sql_params.len(), 1);
    }

    #[test]
    fn test_build_where_clause_search_and_filter() {
        let mut filters = HashMap::new();
        filters.insert("env".to_string(), "prod".to_string());
        let params = QueryParams {
            search: Some("web".into()),
            filters: Some(filters),
            ..Default::default()
        };
        let (clause, sql_params) = build_where_clause(&params, &["name"], &["env"]);
        assert!(clause.contains("name LIKE ?"));
        assert!(clause.contains("env = ?"));
        assert_eq!(sql_params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_with_multi_value_filter() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), r#"["running","maintenance"]"#.to_string());
        let params = QueryParams {
            filters: Some(filters),
            ..Default::default()
        };
        let (clause, sql_params) = build_where_clause(&params, &["name"], &["status"]);
        assert!(clause.contains("status IN (?, ?)"));
        assert_eq!(sql_params.len(), 2);
    }

    #[test]
    fn test_count_query_empty_table() {
        let conn = setup_test_db();
        let count = count_query(&conn, "hosts", "WHERE is_deleted = 0", &[]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_query_with_data() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "server1", "10.0.0.1");
        insert_test_host(&conn, "h2", "server2", "10.0.0.2");
        let count = count_query(&conn, "hosts", "WHERE is_deleted = 0", &[]).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_soft_delete_success() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "server1", "10.0.0.1");
        assert!(soft_delete(&conn, "hosts", "h1").is_ok());

        // Verify it's marked as deleted
        let count = count_query(&conn, "hosts", "WHERE is_deleted = 0", &[]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_soft_delete_already_deleted() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "server1", "10.0.0.1");
        soft_delete(&conn, "hosts", "h1").unwrap();
        // Second delete should fail
        assert!(soft_delete(&conn, "hosts", "h1").is_err());
    }
}
