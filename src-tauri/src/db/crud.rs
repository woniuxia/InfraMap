use crate::db::audit::insert_audit_log;
use crate::error::{AppError, AppResult};
use crate::models::common::QueryParams;
use rusqlite::Connection;
use std::collections::HashMap;

pub type SqlParam = Box<dyn rusqlite::types::ToSql>;

pub fn parse_filter_values(raw: &str) -> Vec<String> {
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

/// Build WHERE clause for search + filters.
/// Returns (where_clause_string, params_vec).
pub fn build_where_clause(
    params: &QueryParams,
    search_columns: &[&str],
    filter_columns: &[&str],
) -> (String, Vec<SqlParam>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut sql_params: Vec<SqlParam> = Vec::new();

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

    let where_clause = if conditions.is_empty() {
        "WHERE 1 = 1".to_string()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    (where_clause, sql_params)
}

pub fn build_exists_like_clause(
    from_clause: &str,
    predicates: &[&str],
    like_column: &str,
) -> String {
    let where_clause = predicates.join("\n              AND ");

    format!(
        "EXISTS (\n              SELECT 1\n              FROM {from_clause}\n              WHERE {where_clause}\n              AND {like_column} LIKE ?\n         )"
    )
}

pub fn build_or_like_clause(columns: &[&str], extra_clauses: &[String]) -> Option<String> {
    let mut fragments: Vec<String> = columns
        .iter()
        .map(|column| format!("{column} LIKE ?"))
        .collect();
    fragments.extend(extra_clauses.iter().cloned());

    if fragments.is_empty() {
        None
    } else {
        Some(format!("({})", fragments.join(" OR ")))
    }
}

pub fn push_like_search_conditions(
    conditions: &mut Vec<String>,
    sql_params: &mut Vec<SqlParam>,
    search_text: Option<&str>,
    columns: &[&str],
    extra_clauses: &[String],
) {
    let Some(search_text) = search_text.map(str::trim).filter(|text| !text.is_empty()) else {
        return;
    };

    let Some(clause) = build_or_like_clause(columns, extra_clauses) else {
        return;
    };

    let like_value = format!("%{}%", search_text);
    conditions.push(clause);
    for _ in 0..(columns.len() + extra_clauses.len()) {
        sql_params.push(Box::new(like_value.clone()));
    }
}

pub fn push_multi_value_filter(
    conditions: &mut Vec<String>,
    sql_params: &mut Vec<SqlParam>,
    column: &str,
    values: &[String],
) {
    if values.is_empty() {
        return;
    }

    if values.len() == 1 {
        conditions.push(format!("{column} = ?"));
        sql_params.push(Box::new(values[0].clone()));
        return;
    }

    let placeholders = vec!["?"; values.len()].join(", ");
    conditions.push(format!("{column} IN ({placeholders})"));
    for value in values {
        sql_params.push(Box::new(value.clone()));
    }
}

pub fn apply_column_filters(
    conditions: &mut Vec<String>,
    sql_params: &mut Vec<SqlParam>,
    filters: Option<&HashMap<String, String>>,
    mappings: &[(&str, &str)],
) {
    let Some(filters) = filters else {
        return;
    };

    for (filter_key, column_name) in mappings {
        if let Some(raw_value) = filters.get(*filter_key) {
            let values = parse_filter_values(raw_value);
            push_multi_value_filter(conditions, sql_params, column_name, &values);
        }
    }
}

pub fn build_taxonomy_exists_filter(
    resource_type: &str,
    field_key: &str,
    resource_id_expr: &str,
    values: &[String],
) -> Option<String> {
    if values.is_empty() {
        return None;
    }

    let placeholders = vec!["?"; values.len()].join(", ");
    Some(format!(
        "EXISTS (
            SELECT 1
            FROM taxonomy_bindings tb
            JOIN taxonomy_terms tt ON tt.id = tb.term_id
            WHERE tb.resource_type = '{resource_type}'
              AND tb.resource_id = {resource_id_expr}
              AND tb.is_deleted = 0
              AND tt.is_deleted = 0
              AND tt.field_key = '{field_key}'
              AND tt.display_name IN ({placeholders})
        )"
    ))
}

pub fn apply_taxonomy_filter(
    conditions: &mut Vec<String>,
    sql_params: &mut Vec<SqlParam>,
    raw_value: Option<&String>,
    resource_type: &str,
    field_key: &str,
    resource_id_expr: &str,
) {
    let Some(raw_value) = raw_value else {
        return;
    };

    let values = parse_filter_values(raw_value);
    if values.is_empty() {
        return;
    }

    if let Some(clause) =
        build_taxonomy_exists_filter(resource_type, field_key, resource_id_expr, &values)
    {
        conditions.push(clause);
    }

    for value in values {
        sql_params.push(Box::new(value));
    }
}

pub fn build_resource_where_clause(
    base_conditions: &[&str],
    params: &QueryParams,
    search_columns: &[&str],
    extra_search_clauses: &[String],
    column_filters: &[(&str, &str)],
    taxonomy_filters: &[(&str, &str, &str, &str)],
) -> (String, Vec<SqlParam>) {
    let mut conditions: Vec<String> = base_conditions.iter().map(|item| (*item).to_string()).collect();
    let mut sql_params: Vec<SqlParam> = Vec::new();

    push_like_search_conditions(
        &mut conditions,
        &mut sql_params,
        params.search.as_deref(),
        search_columns,
        extra_search_clauses,
    );

    if let Some(filters) = params.filters.as_ref() {
        apply_column_filters(
            &mut conditions,
            &mut sql_params,
            Some(filters),
            column_filters,
        );

        for (filter_key, resource_type, field_key, resource_id_expr) in taxonomy_filters {
            apply_taxonomy_filter(
                &mut conditions,
                &mut sql_params,
                filters.get(*filter_key),
                resource_type,
                field_key,
                resource_id_expr,
            );
        }
    }

    (format!("WHERE {}", conditions.join(" AND ")), sql_params)
}

pub fn resolve_upsert_state(
    command: &str,
    conn: &Connection,
    table: &str,
    requested_id: &str,
) -> AppResult<(String, bool)> {
    if requested_id.trim().is_empty() {
        return Ok((uuid::Uuid::new_v4().to_string(), true));
    }

    let sql = format!("SELECT COUNT(*) FROM {table} WHERE id = ?1 AND is_deleted = 0");
    let count: i64 = conn
        .query_row(&sql, rusqlite::params![requested_id], |row| row.get(0))
        .map_err(|error| {
            AppError::from_db_error(command, &format!("检查 {table} 是否存在"), error)
        })?;

    Ok((requested_id.to_string(), count == 0))
}

pub fn write_audit_log_entry(
    command: &str,
    conn: &Connection,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    resource_name: Option<&str>,
) -> AppResult<()> {
    insert_audit_log(
        conn,
        action,
        resource_type,
        resource_id,
        resource_name,
        None,
    )
    .map_err(|error| AppError::from_db_error(command, "写入审计日志", error))
}

pub fn delete_with_audit(
    command: &str,
    action: &str,
    conn: &Connection,
    table: &str,
    resource_type: &str,
    resource_id: &str,
    resource_name: Option<&str>,
) -> AppResult<()> {
    delete_by_id(command, action, conn, table, resource_id)?;
    write_audit_log_entry(
        command,
        conn,
        "delete",
        resource_type,
        resource_id,
        resource_name,
    )
}

/// Execute a count query for pagination total.
pub fn count_query(
    command: &str,
    action: &str,
    conn: &Connection,
    table: &str,
    where_clause: &str,
    sql_params: &[SqlParam],
) -> AppResult<u64> {
    let sql = format!("SELECT COUNT(*) FROM {} {}", table, where_clause);
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        sql_params.iter().map(|p| p.as_ref()).collect();

    conn.query_row(&sql, param_refs.as_slice(), |row| row.get::<_, i64>(0))
        .map(|c| c as u64)
        .map_err(|error| AppError::from_db_error(command, action, error))
}

/// Hard delete by id.
pub fn delete_by_id(
    command: &str,
    action: &str,
    conn: &Connection,
    table: &str,
    id: &str,
) -> AppResult<()> {
    let affected = conn
        .execute(
            &format!("DELETE FROM {} WHERE id = ?1", table),
            rusqlite::params![id],
        )
        .map_err(|error| AppError::from_db_error(command, action, error))?;

    if affected == 0 {
        return Err(AppError::not_found(
            command,
            "目标记录不存在或已被删除。",
            Some(format!("Record not found: {id}")),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::QueryParams;
    use crate::test_helpers::*;
    use std::collections::HashMap;

    #[test]
    fn test_build_where_clause_no_filters() {
        let params = QueryParams::default();
        let (clause, sql_params) = build_where_clause(&params, &["name"], &["status"]);
        assert_eq!(clause, "WHERE 1 = 1");
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
        filters.insert(
            "status".to_string(),
            r#"["running","maintenance"]"#.to_string(),
        );
        let params = QueryParams {
            filters: Some(filters),
            ..Default::default()
        };
        let (clause, sql_params) = build_where_clause(&params, &["name"], &["status"]);
        assert!(clause.contains("status IN (?, ?)"));
        assert_eq!(sql_params.len(), 2);
    }

    #[test]
    fn test_push_like_search_conditions_with_extra_clause() {
        let mut conditions = Vec::new();
        let mut sql_params: Vec<SqlParam> = Vec::new();

        push_like_search_conditions(
            &mut conditions,
            &mut sql_params,
            Some("gateway"),
            &["applications.name", "applications.address"],
            &["EXISTS (SELECT 1 FROM taxonomy_terms tt WHERE tt.display_name LIKE ?)".to_string()],
        );

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("applications.name LIKE ?"));
        assert!(conditions[0].contains("taxonomy_terms"));
        assert_eq!(sql_params.len(), 3);
    }

    #[test]
    fn test_apply_taxonomy_filter_should_append_clause_and_params() {
        let mut conditions = vec!["hosts.is_deleted = 0".to_string()];
        let mut sql_params: Vec<SqlParam> = Vec::new();
        let raw_value = r#"["core","edge"]"#.to_string();

        apply_taxonomy_filter(
            &mut conditions,
            &mut sql_params,
            Some(&raw_value),
            "host",
            "tags",
            "hosts.id",
        );

        assert_eq!(conditions.len(), 2);
        assert!(conditions[1].contains("taxonomy_bindings"));
        assert!(conditions[1].contains("tt.field_key = 'tags'"));
        assert_eq!(sql_params.len(), 2);
    }

    #[test]
    fn test_resolve_upsert_state_should_generate_id_for_new_record() {
        let conn = setup_test_db();

        let (persisted_id, is_new) =
            resolve_upsert_state("crud_test", &conn, "hosts", "").expect("resolve upsert state");

        assert!(is_new);
        assert!(!persisted_id.is_empty());
    }

    #[test]
    fn test_resolve_upsert_state_should_detect_existing_record() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h-existing", "server1", "10.0.0.1");

        let (persisted_id, is_new) =
            resolve_upsert_state("crud_test", &conn, "hosts", "h-existing")
                .expect("resolve upsert state");

        assert_eq!(persisted_id, "h-existing");
        assert!(!is_new);
    }

    #[test]
    fn test_count_query_empty_table() {
        let conn = setup_test_db();
        let count = count_query(
            "crud_test",
            "count hosts",
            &conn,
            "hosts",
            "WHERE 1 = 1",
            &[],
        )
        .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_query_with_data() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "server1", "10.0.0.1");
        insert_test_host(&conn, "h2", "server2", "10.0.0.2");
        let count = count_query(
            "crud_test",
            "count hosts",
            &conn,
            "hosts",
            "WHERE 1 = 1",
            &[],
        )
        .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_delete_by_id_success() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "server1", "10.0.0.1");
        assert!(delete_by_id("crud_test", "delete host", &conn, "hosts", "h1").is_ok());

        let count = count_query(
            "crud_test",
            "count hosts",
            &conn,
            "hosts",
            "WHERE 1 = 1",
            &[],
        )
        .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_delete_by_id_non_existing_should_fail() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "server1", "10.0.0.1");
        delete_by_id("crud_test", "delete host", &conn, "hosts", "h1").unwrap();
        assert!(delete_by_id("crud_test", "delete host", &conn, "hosts", "h1").is_err());
    }
}
