use std::collections::HashSet;
use tauri::State;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::validation::validate_enum;

pub const FIELD_TAGS: &str = "tags";
pub const FIELD_OWNER: &str = "owner";
pub const FIELD_TECH_STACK: &str = "tech_stack";
pub const SORT_ALPHA: &str = "alpha";
pub const SORT_RECENT: &str = "recent";
pub const RECENCY_SCOPE_GLOBAL: &str = "global";
pub const RECENCY_SCOPE_RESOURCE_TYPE: &str = "resource_type";

const ALLOWED_RESOURCE_TYPES: [&str; 4] =
    ["host", "ip_address", "application", "business_application"];
const ALLOWED_SORT_BY: [&str; 2] = [SORT_ALPHA, SORT_RECENT];
const ALLOWED_RECENCY_SCOPE: [&str; 2] = [RECENCY_SCOPE_GLOBAL, RECENCY_SCOPE_RESOURCE_TYPE];
const ALLOWED_APP_TYPES: [&str; 2] = ["frontend", "backend"];

struct TaxonomyFieldSpec {
    resource_type: &'static str,
    field_key: &'static str,
}

const TAXONOMY_FIELD_SPECS: [TaxonomyFieldSpec; 5] = [
    TaxonomyFieldSpec {
        resource_type: "host",
        field_key: FIELD_TAGS,
    },
    TaxonomyFieldSpec {
        resource_type: "ip_address",
        field_key: FIELD_TAGS,
    },
    TaxonomyFieldSpec {
        resource_type: "application",
        field_key: FIELD_OWNER,
    },
    TaxonomyFieldSpec {
        resource_type: "application",
        field_key: FIELD_TECH_STACK,
    },
    TaxonomyFieldSpec {
        resource_type: "business_application",
        field_key: FIELD_OWNER,
    },
];

fn allowed_field_keys(resource_type: &str) -> Vec<&'static str> {
    TAXONOMY_FIELD_SPECS
        .iter()
        .filter(|spec| spec.resource_type == resource_type)
        .map(|spec| spec.field_key)
        .collect()
}

fn validate_resource_field(command: &str, resource_type: &str, field_key: &str) -> AppResult<()> {
    validate_enum(resource_type, &ALLOWED_RESOURCE_TYPES, "resource_type")
        .map_err(|e| AppError::validation(command, e))?;

    let fields = allowed_field_keys(resource_type);
    if !fields.contains(&field_key) {
        return Err(AppError::validation(
            command,
            format!(
                "field_key must be one of {:?} for resource_type '{}', got '{}'",
                fields, resource_type, field_key
            ),
        ));
    }

    Ok(())
}

fn normalize_term(value: &str) -> Option<(String, String)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some((trimmed.to_string(), trimmed.to_lowercase()))
}

pub fn parse_json_string_array(raw: Option<&str>) -> Vec<String> {
    let text = raw.unwrap_or("").trim();
    if text.is_empty() {
        return Vec::new();
    }

    if let Ok(values) = serde_json::from_str::<Vec<String>>(text) {
        return values;
    }

    Vec::new()
}

pub fn parse_filter_values(raw: &str) -> Vec<String> {
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

pub fn parse_tech_stack_terms(raw: Option<&str>) -> Vec<String> {
    let text = raw.unwrap_or("").trim();
    if text.is_empty() {
        return Vec::new();
    }
    text.split(|ch| matches!(ch, ',' | ';' | '|' | '/' | '\u{FF0C}' | '\u{FF1B}'))
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

pub fn normalize_terms(values: &[String]) -> Vec<(String, String)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut normalized: Vec<(String, String)> = Vec::new();
    for value in values {
        if let Some((display, normalized_value)) = normalize_term(value) {
            if seen.insert(normalized_value.clone()) {
                normalized.push((display, normalized_value));
            }
        }
    }
    normalized
}

/// Find an existing term by (field_key, normalized_value) or create a new one.
/// In v2, terms are keyed by (field_key, normalized_value) without resource_type.
fn find_or_create_term_id(
    conn: &rusqlite::Connection,
    field_key: &str,
    display: &str,
    normalized_value: &str,
    now: &str,
) -> Result<String, rusqlite::Error> {
    let existing_id = conn.query_row(
        "SELECT id
         FROM taxonomy_terms
         WHERE field_key = ?1
           AND normalized_value = ?2
           AND is_deleted = 0
         LIMIT 1",
        rusqlite::params![field_key, normalized_value],
        |row| row.get::<_, String>(0),
    );

    match existing_id {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 0, NULL, ?5, ?5)",
                rusqlite::params![id, field_key, normalized_value, display, now],
            )?;
            Ok(id)
        }
        Err(error) => Err(error),
    }
}

/// Refresh stats for a single term_id, grouped by resource_type.
fn refresh_term_stats(
    conn: &rusqlite::Connection,
    term_id: &str,
    now: &str,
) -> Result<(), rusqlite::Error> {
    // Delete existing stats for this term
    conn.execute(
        "DELETE FROM taxonomy_term_stats WHERE term_id = ?1",
        rusqlite::params![term_id],
    )?;

    // Re-insert stats grouped by resource_type
    conn.execute(
        "INSERT INTO taxonomy_term_stats (term_id, resource_type, usage_count, last_used_at, updated_at)
         SELECT tb.term_id, tb.resource_type, COUNT(*) as usage_count, MAX(tb.updated_at) as last_used_at, ?2
         FROM taxonomy_bindings tb
         JOIN taxonomy_terms tt ON tt.id = tb.term_id
         WHERE tb.term_id = ?1
           AND tb.is_deleted = 0
           AND tt.is_deleted = 0
         GROUP BY tb.resource_type",
        rusqlite::params![term_id, now],
    )?;

    Ok(())
}

fn refresh_term_stats_for_term_ids(
    conn: &rusqlite::Connection,
    term_ids: &[String],
    now: &str,
) -> Result<(), rusqlite::Error> {
    let mut unique_ids: HashSet<String> = HashSet::new();
    for term_id in term_ids {
        if unique_ids.insert(term_id.clone()) {
            refresh_term_stats(conn, term_id, now)?;
        }
    }
    Ok(())
}

pub fn rebuild_taxonomy_term_stats(
    conn: &rusqlite::Connection,
    now: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM taxonomy_term_stats", [])?;
    conn.execute(
        "INSERT INTO taxonomy_term_stats (
            term_id,
            resource_type,
            usage_count,
            last_used_at,
            updated_at
         )
         SELECT
            tb.term_id,
            tb.resource_type,
            COUNT(*) AS usage_count,
            MAX(tb.updated_at) AS last_used_at,
            ?1 AS updated_at
         FROM taxonomy_bindings tb
         JOIN taxonomy_terms tt ON tt.id = tb.term_id
         WHERE tb.is_deleted = 0
           AND tt.is_deleted = 0
         GROUP BY tb.term_id, tb.resource_type",
        rusqlite::params![now],
    )?;
    Ok(())
}

pub fn delete_resource_terms(
    conn: &rusqlite::Connection,
    resource_type: &str,
    resource_id: &str,
    now: &str,
) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT term_id
         FROM taxonomy_bindings
         WHERE resource_type = ?1
           AND resource_id = ?2
           AND is_deleted = 0",
    )?;
    let rows = stmt.query_map(rusqlite::params![resource_type, resource_id], |row| {
        row.get::<_, String>(0)
    })?;
    let mut affected_term_ids: Vec<String> = Vec::new();
    for row in rows {
        affected_term_ids.push(row?);
    }

    conn.execute(
        "DELETE FROM taxonomy_bindings
         WHERE resource_type = ?1
           AND resource_id = ?2
           AND is_deleted = 0",
        rusqlite::params![resource_type, resource_id],
    )?;

    refresh_term_stats_for_term_ids(conn, &affected_term_ids, now)?;
    Ok(())
}

pub fn save_resource_terms(
    conn: &rusqlite::Connection,
    resource_type: &str,
    resource_id: &str,
    field_key: &str,
    values: &[String],
    now: &str,
) -> Result<(), rusqlite::Error> {
    let normalized_values = normalize_terms(values);
    let mut affected_term_ids: Vec<String> = Vec::new();

    // Find old bindings for this (resource_type, resource_id, field_key) via JOIN
    let mut old_stmt = conn.prepare(
        "SELECT tb.term_id
         FROM taxonomy_bindings tb
         JOIN taxonomy_terms tt ON tt.id = tb.term_id
         WHERE tb.resource_type = ?1
           AND tb.resource_id = ?2
           AND tt.field_key = ?3
           AND tb.is_deleted = 0",
    )?;
    let old_rows = old_stmt.query_map(
        rusqlite::params![resource_type, resource_id, field_key],
        |row| row.get::<_, String>(0),
    )?;
    for row in old_rows {
        affected_term_ids.push(row?);
    }

    conn.execute(
        "DELETE FROM taxonomy_bindings
         WHERE resource_type = ?1
           AND resource_id = ?2
           AND is_deleted = 0
           AND term_id IN (
               SELECT tt.id FROM taxonomy_terms tt WHERE tt.field_key = ?3 AND tt.is_deleted = 0
           )",
        rusqlite::params![resource_type, resource_id, field_key],
    )?;

    for (display, normalized_value) in normalized_values {
        let term_id = find_or_create_term_id(conn, field_key, &display, &normalized_value, now)?;
        conn.execute(
            "INSERT INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0, NULL, ?5, ?5)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                term_id,
                resource_type,
                resource_id,
                now
            ],
        )?;
        affected_term_ids.push(term_id);
    }

    refresh_term_stats_for_term_ids(conn, &affected_term_ids, now)?;

    Ok(())
}

fn list_terms_by_scope_alpha(
    conn: &rusqlite::Connection,
    resource_type: &str,
    field_key: &str,
    limit: usize,
    app_type: Option<&str>,
) -> Result<Vec<String>, rusqlite::Error> {
    if let Some(side) = app_type {
        let app_filter = if side == "frontend" {
            "a.type = 'frontend'"
        } else {
            "a.type <> 'frontend'"
        };
        let sql = format!(
            "SELECT DISTINCT tt.display_name
             FROM taxonomy_bindings tb
             INNER JOIN taxonomy_terms tt ON tt.id = tb.term_id
             INNER JOIN applications a ON a.id = tb.resource_id
             WHERE tt.field_key = 'tech_stack'
               AND tb.resource_type = 'application'
               AND tb.is_deleted = 0
               AND tt.is_deleted = 0
               AND a.is_deleted = 0
               AND {}
             ORDER BY tt.display_name ASC
             LIMIT ?1",
            app_filter
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
            row.get::<_, String>(0)
        })?;
        return rows.collect();
    }

    // No app_type: use stats table
    let mut stmt = conn.prepare(
        "SELECT tt.display_name
         FROM taxonomy_term_stats ts
         INNER JOIN taxonomy_terms tt ON tt.id = ts.term_id
         WHERE ts.resource_type = ?1
           AND tt.field_key = ?2
           AND tt.is_deleted = 0
           AND ts.usage_count > 0
         ORDER BY tt.display_name ASC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![resource_type, field_key, limit as i64],
        |row| row.get::<_, String>(0),
    )?;
    rows.collect()
}

fn list_terms_by_scope_recent(
    conn: &rusqlite::Connection,
    resource_type: &str,
    field_key: &str,
    limit: usize,
    recency_scope: &str,
    app_type: Option<&str>,
) -> Result<Vec<String>, rusqlite::Error> {
    if let Some(side) = app_type {
        let app_filter = if side == "frontend" {
            "a.type = 'frontend'"
        } else {
            "a.type <> 'frontend'"
        };
        let sql = format!(
            "SELECT tt.display_name
             FROM taxonomy_bindings tb
             INNER JOIN taxonomy_terms tt ON tt.id = tb.term_id
             INNER JOIN applications a ON a.id = tb.resource_id
             WHERE tb.is_deleted = 0
               AND tt.is_deleted = 0
               AND tb.resource_type = 'application'
               AND tt.field_key = 'tech_stack'
               AND a.is_deleted = 0
               AND {}
             GROUP BY tt.display_name
             ORDER BY MAX(tb.updated_at) DESC, tt.display_name ASC
             LIMIT ?1",
            app_filter
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
            row.get::<_, String>(0)
        })?;
        return rows.collect();
    }

    if recency_scope == RECENCY_SCOPE_GLOBAL {
        // GLOBAL: aggregate across all resource_types, pick max last_used_at
        let mut stmt = conn.prepare(
            "SELECT tt.display_name
             FROM taxonomy_term_stats ts
             INNER JOIN taxonomy_terms tt ON tt.id = ts.term_id
             WHERE tt.field_key = ?1
               AND tt.is_deleted = 0
               AND ts.usage_count > 0
             GROUP BY tt.display_name
             ORDER BY MAX(ts.last_used_at) DESC, tt.display_name ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![field_key, limit as i64], |row| {
            row.get::<_, String>(0)
        })?;
        return rows.collect();
    }

    // RESOURCE_TYPE scope: filter by resource_type
    let mut stmt = conn.prepare(
        "SELECT tt.display_name
         FROM taxonomy_term_stats ts
         INNER JOIN taxonomy_terms tt ON tt.id = ts.term_id
         WHERE ts.resource_type = ?1
           AND tt.field_key = ?2
           AND tt.is_deleted = 0
           AND ts.usage_count > 0
         ORDER BY ts.last_used_at DESC, tt.display_name ASC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![resource_type, field_key, limit as i64],
        |row| row.get::<_, String>(0),
    )?;
    rows.collect()
}

#[cfg(test)]
pub fn list_terms_by_scope(
    conn: &rusqlite::Connection,
    resource_type: &str,
    field_key: &str,
    limit: usize,
) -> Result<Vec<String>, rusqlite::Error> {
    list_terms_by_scope_alpha(conn, resource_type, field_key, limit, None)
}

pub fn list_terms_by_scope_with_options(
    conn: &rusqlite::Connection,
    resource_type: &str,
    field_key: &str,
    limit: usize,
    sort_by: &str,
    recency_scope: &str,
    app_type: Option<&str>,
) -> Result<Vec<String>, rusqlite::Error> {
    if sort_by == SORT_RECENT {
        return list_terms_by_scope_recent(
            conn,
            resource_type,
            field_key,
            limit,
            recency_scope,
            app_type,
        );
    }

    list_terms_by_scope_alpha(conn, resource_type, field_key, limit, app_type)
}

fn is_taxonomy_schema_error(raw: &str) -> bool {
    let message = raw.to_lowercase();
    message.contains("no such table")
        || message.contains("no such column")
        || message.contains("has no column named")
        || message.contains("database schema has changed")
        || (message.contains("has")
            && message.contains("columns")
            && message.contains("values were supplied"))
}

fn list_terms_with_fallback(
    conn: &rusqlite::Connection,
    resource_type: &str,
    field_key: &str,
    limit: usize,
    sort_by: &str,
    recency_scope: &str,
    app_type: Option<&str>,
) -> Result<Vec<String>, rusqlite::Error> {
    match list_terms_by_scope_with_options(
        conn,
        resource_type,
        field_key,
        limit,
        sort_by,
        recency_scope,
        app_type,
    ) {
        Ok(terms) => Ok(terms),
        Err(error) => {
            if is_taxonomy_schema_error(&error.to_string()) {
                eprintln!(
                    "[list_taxonomy_terms] taxonomy schema mismatch, fallback to empty list. resource_type={}, field_key={}, sort_by={}, recency_scope={}, app_type={:?}, error={}",
                    resource_type,
                    field_key,
                    sort_by,
                    recency_scope,
                    app_type,
                    error
                );
                return Ok(Vec::new());
            }
            Err(error)
        }
    }
}

pub fn list_resource_terms_by_field(
    conn: &rusqlite::Connection,
    resource_type: &str,
    resource_id: &str,
    field_key: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT tt.display_name
         FROM taxonomy_bindings tb
         INNER JOIN taxonomy_terms tt ON tt.id = tb.term_id
         WHERE tb.resource_type = ?1
           AND tb.resource_id = ?2
           AND tt.field_key = ?3
           AND tb.is_deleted = 0
           AND tt.is_deleted = 0
         ORDER BY tt.display_name ASC",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![resource_type, resource_id, field_key],
        |row| row.get::<_, String>(0),
    )?;
    rows.collect()
}

#[tauri::command]
pub fn list_taxonomy_terms(
    pool: State<DbPool>,
    resource_type: String,
    field_key: String,
    limit: Option<u32>,
    sort_by: Option<String>,
    recency_scope: Option<String>,
    app_type: Option<String>,
) -> AppResult<Vec<String>> {
    let command = "list_taxonomy_terms";
    validate_resource_field(command, &resource_type, &field_key)?;

    let sort_by = sort_by.unwrap_or_else(|| SORT_ALPHA.to_string());
    validate_enum(&sort_by, &ALLOWED_SORT_BY, "sort_by")
        .map_err(|e| AppError::validation(command, e))?;

    let recency_scope = recency_scope.unwrap_or_else(|| RECENCY_SCOPE_RESOURCE_TYPE.to_string());
    validate_enum(&recency_scope, &ALLOWED_RECENCY_SCOPE, "recency_scope")
        .map_err(|e| AppError::validation(command, e))?;

    if let Some(side) = app_type.as_deref() {
        validate_enum(side, &ALLOWED_APP_TYPES, "app_type")
            .map_err(|e| AppError::validation(command, e))?;
        if resource_type != "application" || field_key != FIELD_TECH_STACK {
            return Err(AppError::validation(
                command,
                "app_type is only supported for resource_type=application and field_key=tech_stack",
            ));
        }
    }

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let max_limit = limit.unwrap_or(100).clamp(1, 500) as usize;
    list_terms_with_fallback(
        &conn,
        &resource_type,
        &field_key,
        max_limit,
        &sort_by,
        &recency_scope,
        app_type.as_deref(),
    )
    .map_err(|e| AppError::from_db_error(command, "读取词条候选", e))
}

#[tauri::command]
pub fn save_resource_terms_command(
    pool: State<DbPool>,
    resource_type: String,
    resource_id: String,
    field_key: String,
    values: Vec<String>,
) -> AppResult<()> {
    let command = "save_resource_terms_command";
    validate_resource_field(command, &resource_type, &field_key)?;
    if resource_id.trim().is_empty() {
        return Err(AppError::validation(command, "resource_id is required"));
    }

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let now = chrono::Utc::now().to_rfc3339();
    save_resource_terms(
        &conn,
        &resource_type,
        resource_id.trim(),
        &field_key,
        &values,
        &now,
    )
    .map_err(|e| AppError::from_db_error(command, "保存资源词条", e))
}

#[tauri::command]
pub fn list_resource_terms(
    pool: State<DbPool>,
    resource_type: String,
    resource_id: String,
    field_key: String,
) -> AppResult<Vec<String>> {
    let command = "list_resource_terms";
    validate_resource_field(command, &resource_type, &field_key)?;
    if resource_id.trim().is_empty() {
        return Err(AppError::validation(command, "resource_id is required"));
    }

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    list_resource_terms_by_field(&conn, &resource_type, resource_id.trim(), &field_key)
        .map_err(|e| AppError::from_db_error(command, "读取资源词条", e))
}

#[cfg(test)]
mod tests {
    use super::{
        is_taxonomy_schema_error, list_resource_terms_by_field, list_terms_by_scope,
        list_terms_by_scope_with_options, list_terms_with_fallback, normalize_terms,
        parse_json_string_array, parse_tech_stack_terms, save_resource_terms,
        validate_resource_field, FIELD_OWNER, FIELD_TAGS, FIELD_TECH_STACK, RECENCY_SCOPE_GLOBAL,
        RECENCY_SCOPE_RESOURCE_TYPE, SORT_RECENT,
    };
    use crate::test_helpers::setup_test_db;

    #[test]
    fn save_resource_terms_should_replace_existing_bindings() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        save_resource_terms(
            &conn,
            "ip_address",
            "ip-1",
            FIELD_TAGS,
            &["core".to_string(), "gateway".to_string()],
            &now,
        )
        .expect("first save");

        save_resource_terms(
            &conn,
            "ip_address",
            "ip-1",
            FIELD_TAGS,
            &["edge".to_string()],
            &now,
        )
        .expect("second save");

        let values = list_resource_terms_by_field(&conn, "ip_address", "ip-1", FIELD_TAGS)
            .expect("query terms");
        assert_eq!(values, vec!["edge".to_string()]);
    }

    #[test]
    fn list_terms_by_scope_should_filter_resource_and_field() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        save_resource_terms(
            &conn,
            "application",
            "app-1",
            FIELD_OWNER,
            &["alice".to_string()],
            &now,
        )
        .expect("save owners");
        save_resource_terms(
            &conn,
            "application",
            "app-1",
            FIELD_TECH_STACK,
            &["Rust".to_string()],
            &now,
        )
        .expect("save tech stack");

        let owners =
            list_terms_by_scope(&conn, "application", FIELD_OWNER, 100).expect("list owners");
        assert_eq!(owners, vec!["alice".to_string()]);
    }

    #[test]
    fn validate_resource_field_should_allow_business_application_owner_only() {
        assert!(validate_resource_field("test", "business_application", FIELD_OWNER).is_ok());
        assert!(validate_resource_field("test", "business_application", FIELD_TECH_STACK).is_err());
    }

    #[test]
    fn parse_helpers_should_parse_expected_values() {
        assert_eq!(
            parse_json_string_array(Some(r#"["a","b"]"#)),
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            parse_tech_stack_terms(Some("Vue, TypeScript; Rust")),
            vec![
                "Vue".to_string(),
                "TypeScript".to_string(),
                "Rust".to_string()
            ]
        );
        assert_eq!(
            normalize_terms(&[
                "core".to_string(),
                " core ".to_string(),
                "".to_string(),
                "EDGE".to_string()
            ]),
            vec![
                ("core".to_string(), "core".to_string()),
                ("EDGE".to_string(), "edge".to_string())
            ]
        );
    }

    #[test]
    fn list_terms_by_scope_with_options_should_order_recent_first() {
        let conn = setup_test_db();
        save_resource_terms(
            &conn,
            "ip_address",
            "ip-old",
            FIELD_TAGS,
            &["old-tag".to_string()],
            "2025-01-01T00:00:00Z",
        )
        .expect("save old tag");
        save_resource_terms(
            &conn,
            "ip_address",
            "ip-new",
            FIELD_TAGS,
            &["new-tag".to_string()],
            "2025-01-02T00:00:00Z",
        )
        .expect("save new tag");

        let tags = list_terms_by_scope_with_options(
            &conn,
            "ip_address",
            FIELD_TAGS,
            100,
            SORT_RECENT,
            RECENCY_SCOPE_RESOURCE_TYPE,
            None,
        )
        .expect("list recent tags");
        assert_eq!(tags, vec!["new-tag".to_string(), "old-tag".to_string()]);
    }

    #[test]
    fn list_terms_by_scope_with_options_should_support_recency_scope() {
        let conn = setup_test_db();
        save_resource_terms(
            &conn,
            "host",
            "host-1",
            FIELD_TAGS,
            &["host-tag".to_string()],
            "2025-01-01T00:00:00Z",
        )
        .expect("save host tag");
        save_resource_terms(
            &conn,
            "ip_address",
            "ip-1",
            FIELD_TAGS,
            &["ip-tag".to_string()],
            "2025-01-02T00:00:00Z",
        )
        .expect("save ip tag");

        // GLOBAL scope should show tags from all resource types
        let global_tags = list_terms_by_scope_with_options(
            &conn,
            "host",
            FIELD_TAGS,
            100,
            SORT_RECENT,
            RECENCY_SCOPE_GLOBAL,
            None,
        )
        .expect("list global tags");
        assert_eq!(
            global_tags,
            vec!["ip-tag".to_string(), "host-tag".to_string()]
        );

        // RESOURCE_TYPE scope should only show host tags
        let host_only_tags = list_terms_by_scope_with_options(
            &conn,
            "host",
            FIELD_TAGS,
            100,
            SORT_RECENT,
            RECENCY_SCOPE_RESOURCE_TYPE,
            None,
        )
        .expect("list host scoped tags");
        assert_eq!(host_only_tags, vec!["host-tag".to_string()]);
    }

    #[test]
    fn list_terms_by_scope_with_options_should_filter_tech_stack_by_app_type() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO applications (id, name, type, env, status, is_deleted, created_at, updated_at)
             VALUES ('app-fe', 'frontend-app', 'frontend', 'prod', 'running', 0, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("insert frontend app");
        conn.execute(
            "INSERT INTO applications (id, name, type, env, status, is_deleted, created_at, updated_at)
             VALUES ('app-be', 'backend-app', 'backend', 'prod', 'running', 0, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("insert backend app");

        save_resource_terms(
            &conn,
            "application",
            "app-fe",
            FIELD_TECH_STACK,
            &["Vue".to_string()],
            "2025-01-01T00:00:00Z",
        )
        .expect("save frontend stack");
        save_resource_terms(
            &conn,
            "application",
            "app-be",
            FIELD_TECH_STACK,
            &["Rust".to_string()],
            "2025-01-02T00:00:00Z",
        )
        .expect("save backend stack");

        let frontend_terms = list_terms_by_scope_with_options(
            &conn,
            "application",
            FIELD_TECH_STACK,
            100,
            SORT_RECENT,
            RECENCY_SCOPE_RESOURCE_TYPE,
            Some("frontend"),
        )
        .expect("list frontend stacks");
        assert_eq!(frontend_terms, vec!["Vue".to_string()]);

        let backend_terms = list_terms_by_scope_with_options(
            &conn,
            "application",
            FIELD_TECH_STACK,
            100,
            SORT_RECENT,
            RECENCY_SCOPE_RESOURCE_TYPE,
            Some("backend"),
        )
        .expect("list backend stacks");
        assert_eq!(backend_terms, vec!["Rust".to_string()]);
    }

    #[test]
    fn cross_resource_type_terms_should_share_same_term_record() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();

        // Save "production" tag for both host and ip_address
        save_resource_terms(
            &conn,
            "host",
            "host-1",
            FIELD_TAGS,
            &["production".to_string()],
            &now,
        )
        .expect("save host tag");
        save_resource_terms(
            &conn,
            "ip_address",
            "ip-1",
            FIELD_TAGS,
            &["production".to_string()],
            &now,
        )
        .expect("save ip tag");

        // There should be only ONE term record for "production" in tags
        let term_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM taxonomy_terms WHERE field_key = 'tags' AND normalized_value = 'production' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("count terms");
        assert_eq!(
            term_count, 1,
            "should share a single term record across resource types"
        );

        // Both resource types should see it via their bindings
        let host_tags = list_resource_terms_by_field(&conn, "host", "host-1", FIELD_TAGS)
            .expect("list host tags");
        let ip_tags = list_resource_terms_by_field(&conn, "ip_address", "ip-1", FIELD_TAGS)
            .expect("list ip tags");
        assert_eq!(host_tags, vec!["production".to_string()]);
        assert_eq!(ip_tags, vec!["production".to_string()]);
    }

    #[test]
    fn taxonomy_schema_error_classifier_should_detect_schema_mismatch() {
        assert!(is_taxonomy_schema_error(
            "no such table: taxonomy_term_stats"
        ));
        assert!(is_taxonomy_schema_error(
            "table taxonomy_bindings has no column named field_key"
        ));
        assert!(!is_taxonomy_schema_error(
            "UNIQUE constraint failed: taxonomy_terms.field_key, taxonomy_terms.normalized_value"
        ));
    }

    #[test]
    fn list_terms_with_fallback_should_return_empty_on_schema_error() {
        let conn = setup_test_db();
        conn.execute("DROP TABLE taxonomy_term_stats", [])
            .expect("drop stats table");

        let terms = list_terms_with_fallback(
            &conn,
            "ip_address",
            FIELD_TAGS,
            50,
            SORT_RECENT,
            RECENCY_SCOPE_RESOURCE_TYPE,
            None,
        )
        .expect("schema error should fallback to empty list");
        assert!(terms.is_empty());
    }
}
