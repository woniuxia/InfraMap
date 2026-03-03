use tauri::State;

use crate::commands::taxonomy::{
    build_taxonomy_exists_filter, delete_resource_terms, parse_filter_values,
    parse_json_string_array, save_resource_terms, FIELD_CPU_MODEL, FIELD_OS_TYPE, FIELD_TAGS,
};
use crate::db::audit::insert_audit_log;
use crate::db::crud::delete_by_id;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::host::Host;
use crate::validation::validate_host;

fn parse_single_term(value: Option<&str>) -> Vec<String> {
    match value
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
    {
        Some(term) => vec![term.to_string()],
        None => Vec::new(),
    }
}

fn sync_host_taxonomy_terms(
    conn: &rusqlite::Connection,
    host_id: &str,
    host: &Host,
    now: &str,
) -> Result<(), rusqlite::Error> {
    let tag_values = parse_json_string_array(host.tags.as_deref());
    save_resource_terms(conn, "host", host_id, FIELD_TAGS, &tag_values, now)?;

    let os_type_values = parse_single_term(host.os_type.as_deref());
    save_resource_terms(conn, "host", host_id, FIELD_OS_TYPE, &os_type_values, now)?;

    let cpu_model_values = parse_single_term(host.cpu_model.as_deref());
    save_resource_terms(conn, "host", host_id, FIELD_CPU_MODEL, &cpu_model_values, now)?;

    Ok(())
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
        for column in ["status", "env"] {
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

        if let Some(raw_tags) = filters.get("tags") {
            let values = parse_filter_values(raw_tags);
            if !values.is_empty() {
                if let Some(clause) =
                    build_taxonomy_exists_filter("host", FIELD_TAGS, "h.id", &values)
                {
                    conditions.push(clause);
                }
                for value in values {
                    sql_params.push(Box::new(value));
                }
            }
        }

        if let Some(raw_os_type) = filters.get("os_type") {
            let values = parse_filter_values(raw_os_type);
            if !values.is_empty() {
                if let Some(clause) =
                    build_taxonomy_exists_filter("host", FIELD_OS_TYPE, "h.id", &values)
                {
                    conditions.push(clause);
                }
                for value in values {
                    sql_params.push(Box::new(value));
                }
            }
        }

        if let Some(raw_cpu_model) = filters.get("cpu_model") {
            let values = parse_filter_values(raw_cpu_model);
            if !values.is_empty() {
                if let Some(clause) =
                    build_taxonomy_exists_filter("host", FIELD_CPU_MODEL, "h.id", &values)
                {
                    conditions.push(clause);
                }
                for value in values {
                    sql_params.push(Box::new(value));
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
        ip_display: row.get(2)?,
        env: row.get(3)?,
        os_type: row.get(4)?,
        cpu_model: row.get(5)?,
        cpu_cores: row.get(6)?,
        cpu_threads: row.get(7)?,
        cpu_freq: row.get(8)?,
        ram_gb: row.get(9)?,
        disk_gb: row.get(10)?,
        status: row.get(11)?,
        tags: row.get(12)?,
        description: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

const SELECT_COLUMNS: &str = "h.id, h.hostname,
     NULLIF((
      SELECT GROUP_CONCAT(ia.ip_address, ', ')
      FROM host_ip_bindings hb
      JOIN ip_addresses ia ON ia.id = hb.ip_id
      WHERE hb.host_id = h.id AND hb.is_deleted = 0 AND ia.is_deleted = 0
     ), '') AS ip_display,
     h.env, h.os_type, h.cpu_model, h.cpu_cores, h.cpu_threads, h.cpu_freq,
     h.ram_gb, h.disk_gb, h.status, h.tags, h.description,
     h.created_at, h.updated_at";

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
                "INSERT INTO hosts (id, hostname, env, os_type, cpu_model, cpu_cores, cpu_threads, cpu_freq,
                                    ram_gb, disk_gb, status, tags, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,0,NULL,?14,?14)",
                rusqlite::params![
                    id,
                    data.hostname,
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
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建主机", e))?;
            insert_audit_log(&conn, "create", "host", &id, Some(&data.hostname), None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
            sync_host_taxonomy_terms(&conn, &id, &data, &now)
                .map_err(|e| AppError::from_db_error(command, "同步主机词条", e))?;
        } else {
            conn.execute(
                "UPDATE hosts SET hostname=?1, env=?2, os_type=?3, cpu_model=?4, cpu_cores=?5,
                                  cpu_threads=?6, cpu_freq=?7, ram_gb=?8, disk_gb=?9,
                                  status=?10, tags=?11, description=?12, updated_at=?13
                 WHERE id=?14 AND is_deleted=0",
                rusqlite::params![
                    data.hostname,
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
                    data.id,
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
            sync_host_taxonomy_terms(&conn, &data.id, &data, &now)
                .map_err(|e| AppError::from_db_error(command, "同步主机词条", e))?;
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

#[cfg(test)]
mod tests {
    use super::build_hosts_where_clause;
    use crate::models::common::QueryParams;
    use std::collections::HashMap;

    #[test]
    fn build_hosts_where_clause_should_support_tag_filter() {
        let mut filters = HashMap::new();
        filters.insert("tags".to_string(), r#"["core","edge"]"#.to_string());
        let params = QueryParams {
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_hosts_where_clause(&params);
        assert!(where_clause.contains("taxonomy_bindings"));
        assert!(where_clause.contains("tt.field_key = 'tags'"));
        assert!(where_clause.contains("tt.display_name IN (?, ?)"));
        assert_eq!(sql_params.len(), 2);
    }

    #[test]
    fn build_hosts_where_clause_should_combine_status_and_tags() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "running".to_string());
        filters.insert("tags".to_string(), "core".to_string());
        let params = QueryParams {
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_hosts_where_clause(&params);
        assert!(where_clause.contains("h.status = ?"));
        assert!(where_clause.contains("taxonomy_bindings"));
        assert_eq!(sql_params.len(), 2);
    }

    #[test]
    fn build_hosts_where_clause_should_filter_os_type_with_taxonomy() {
        let mut filters = HashMap::new();
        filters.insert(
            "os_type".to_string(),
            r#"["openEuler","Ubuntu 22.04"]"#.to_string(),
        );
        let params = QueryParams {
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_hosts_where_clause(&params);
        assert!(where_clause.contains("taxonomy_bindings"));
        assert!(where_clause.contains("tt.field_key = 'os_type'"));
        assert!(where_clause.contains("tt.display_name IN (?, ?)"));
        assert!(!where_clause.contains("h.os_type IN"));
        assert_eq!(sql_params.len(), 2);
    }

    #[test]
    fn build_hosts_where_clause_should_filter_cpu_model_with_taxonomy() {
        let mut filters = HashMap::new();
        filters.insert("cpu_model".to_string(), "Hygon C86 7285".to_string());
        let params = QueryParams {
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_hosts_where_clause(&params);
        assert!(where_clause.contains("taxonomy_bindings"));
        assert!(where_clause.contains("tt.field_key = 'cpu_model'"));
        assert!(where_clause.contains("tt.display_name IN (?)"));
        assert_eq!(sql_params.len(), 1);
    }
}

fn delete_host_inner(command: &str, conn: &rusqlite::Connection, id: String) -> AppResult<()> {
    let name: Option<String> = conn
        .query_row(
            "SELECT hostname FROM hosts WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    conn.execute(
        "DELETE FROM host_ip_bindings WHERE host_id = ?1 AND is_deleted = 0",
        rusqlite::params![id],
    )
    .map_err(|e| AppError::from_db_error(command, "删除主机IP绑定", e))?;

    match delete_by_id(&conn, "hosts", &id) {
        Ok(()) => match insert_audit_log(&conn, "delete", "host", &id, name.as_deref(), None) {
            Ok(()) => {
                let now = chrono::Utc::now().to_rfc3339();
                delete_resource_terms(&conn, "host", &id, &now)
                    .map_err(|e| AppError::from_db_error(command, "删除标签绑定", e))?;
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

#[tauri::command]
pub fn delete_host(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "delete_host";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    delete_host_inner(command, &conn, id)
}

#[cfg(test)]
mod delete_tests {
    use super::delete_host_inner;
    use crate::test_helpers::{insert_test_host, setup_test_db};

    #[test]
    fn delete_host_should_cleanup_ip_bindings() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "server-1", "10.0.0.2");

        delete_host_inner("test", &conn, "h1".to_string()).expect("delete host");

        let host_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM hosts WHERE id = 'h1'", [], |row| {
                row.get(0)
            })
            .expect("query host count");
        assert_eq!(host_count, 0);

        let binding_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM host_ip_bindings WHERE host_id = 'h1'",
                [],
                |row| row.get(0),
            )
            .expect("query binding count");
        assert_eq!(binding_count, 0);
    }
}
