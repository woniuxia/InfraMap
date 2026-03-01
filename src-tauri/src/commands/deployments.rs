use std::net::Ipv4Addr;
use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::deployment::Deployment;
use crate::validation::validate_deployment;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeployContext {
    pub resource_type: String,
    pub resource_id: String,
    pub address: Option<String>,
    pub resource_env: Option<String>,
    pub parsed_ip: Option<String>,
    pub matched_host_id: Option<String>,
    pub matched_host_name: Option<String>,
}

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

fn normalize_ipv4(candidate: &str) -> Option<String> {
    candidate
        .trim()
        .parse::<Ipv4Addr>()
        .ok()
        .map(|ip| ip.to_string())
}

fn extract_ipv4_from_address(address: &str) -> Option<String> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(ip) = normalize_ipv4(trimmed) {
        return Some(ip);
    }

    if let Some(rest) = trimmed.split("://").nth(1) {
        let host_port = rest.split('/').next().unwrap_or_default();
        let host_port = host_port.split('@').last().unwrap_or_default();
        let host = host_port.split(':').next().unwrap_or_default();
        if let Some(ip) = normalize_ipv4(host) {
            return Some(ip);
        }
    }

    let colon_segments: Vec<&str> = trimmed.split(':').collect();
    if colon_segments.len() == 2 {
        if let Some(ip) = normalize_ipv4(colon_segments[0]) {
            return Some(ip);
        }
    }

    for token in trimmed.split(|c: char| !c.is_ascii_digit() && c != '.') {
        if let Some(ip) = normalize_ipv4(token) {
            return Some(ip);
        }
    }

    None
}

fn query_resource_address_env(
    command: &str,
    conn: &rusqlite::Connection,
    resource_type: &str,
    resource_id: &str,
) -> AppResult<(Option<String>, Option<String>)> {
    match resource_type {
        "application" => conn
            .query_row(
                "SELECT address, env FROM applications WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![resource_id],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?,
                        Some(row.get::<_, String>(1)?),
                    ))
                },
            )
            .map_err(|e| AppError::not_found(command, "应用不存在或已删除。", Some(e.to_string()))),
        "middleware" => conn
            .query_row(
                "SELECT address, env FROM middlewares WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![resource_id],
                |row| {
                    Ok((
                        Some(row.get::<_, String>(0)?),
                        Some(row.get::<_, String>(1)?),
                    ))
                },
            )
            .map_err(|e| {
                AppError::not_found(command, "中间件不存在或已删除。", Some(e.to_string()))
            }),
        "nginx" => conn
            .query_row(
                "SELECT address, env FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![resource_id],
                |row| {
                    Ok((
                        Some(row.get::<_, String>(0)?),
                        Some(row.get::<_, String>(1)?),
                    ))
                },
            )
            .map_err(|e| {
                AppError::not_found(command, "网关配置不存在或已删除。", Some(e.to_string()))
            }),
        _ => Err(AppError::validation(
            command,
            format!(
                "resource_type must be one of application/middleware/nginx, got {}",
                resource_type
            ),
        )),
    }
}

fn build_resource_deploy_context(
    command: &str,
    conn: &rusqlite::Connection,
    resource_type: &str,
    resource_id: &str,
) -> AppResult<ResourceDeployContext> {
    let (address, resource_env) =
        query_resource_address_env(command, conn, resource_type, resource_id)?;
    let parsed_ip = address.as_deref().and_then(extract_ipv4_from_address);

    let (matched_host_id, matched_host_name) = if let Some(ref ip) = parsed_ip {
        if let Some(env) = resource_env.as_deref() {
            conn.query_row(
                "SELECT h.id, h.hostname
                 FROM ip_addresses ia
                 JOIN host_ip_bindings hb ON hb.ip_id = ia.id AND hb.is_deleted = 0
                 JOIN hosts h ON h.id = hb.host_id AND h.is_deleted = 0
                 WHERE ia.ip_address = ?1 AND ia.is_deleted = 0
                 ORDER BY CASE WHEN ia.env = ?2 THEN 0 ELSE 1 END, ia.created_at ASC, h.created_at ASC
                 LIMIT 1",
                rusqlite::params![ip, env],
                |row| {
                    Ok((
                        Some(row.get::<_, String>(0)?),
                        Some(row.get::<_, String>(1)?),
                    ))
                },
            )
            .optional()
            .map_err(|e| AppError::from_db_error(command, "查询匹配服务器", e))?
            .unwrap_or((None, None))
        } else {
            conn.query_row(
                "SELECT h.id, h.hostname
                 FROM ip_addresses ia
                 JOIN host_ip_bindings hb ON hb.ip_id = ia.id AND hb.is_deleted = 0
                 JOIN hosts h ON h.id = hb.host_id AND h.is_deleted = 0
                 WHERE ia.ip_address = ?1 AND ia.is_deleted = 0
                 ORDER BY ia.created_at ASC, h.created_at ASC
                 LIMIT 1",
                rusqlite::params![ip],
                |row| {
                    Ok((
                        Some(row.get::<_, String>(0)?),
                        Some(row.get::<_, String>(1)?),
                    ))
                },
            )
            .optional()
            .map_err(|e| AppError::from_db_error(command, "查询匹配服务器", e))?
            .unwrap_or((None, None))
        }
    } else {
        (None, None)
    };

    Ok(ResourceDeployContext {
        resource_type: resource_type.to_string(),
        resource_id: resource_id.to_string(),
        address,
        resource_env,
        parsed_ip,
        matched_host_id,
        matched_host_name,
    })
}

#[tauri::command]
pub fn get_resource_deploy_context(
    pool: State<DbPool>,
    resource_type: String,
    resource_id: String,
) -> AppResult<ResourceDeployContext> {
    let command = "get_resource_deploy_context";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    build_resource_deploy_context(command, &conn, &resource_type, &resource_id)
}

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

#[cfg(test)]
mod tests {
    use crate::error::AppErrorCode;
    use crate::test_helpers::{
        insert_test_application, insert_test_host, insert_test_middleware,
        insert_test_nginx_config, setup_test_db,
    };

    use super::{build_resource_deploy_context, extract_ipv4_from_address};

    #[test]
    fn extract_ipv4_from_address_should_support_common_patterns() {
        assert_eq!(
            extract_ipv4_from_address("10.0.0.1"),
            Some("10.0.0.1".to_string())
        );
        assert_eq!(
            extract_ipv4_from_address("redis://10.0.0.2:6379"),
            Some("10.0.0.2".to_string())
        );
        assert_eq!(
            extract_ipv4_from_address("10.0.0.3:8080"),
            Some("10.0.0.3".to_string())
        );
        assert_eq!(extract_ipv4_from_address("api.example.com"), None);
    }

    #[test]
    fn build_resource_deploy_context_should_match_existing_host_for_middleware() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "redis-host", "10.0.0.8");
        insert_test_middleware(&conn, "mw1", "redis-main", "cache");
        conn.execute(
            "UPDATE middlewares SET address = 'redis://10.0.0.8:6379', env = 'prod' WHERE id = 'mw1'",
            [],
        )
        .expect("update middleware address");

        let context = build_resource_deploy_context("test", &conn, "middleware", "mw1")
            .expect("build context");
        assert_eq!(context.parsed_ip.as_deref(), Some("10.0.0.8"));
        assert_eq!(context.matched_host_id.as_deref(), Some("h1"));
        assert_eq!(context.resource_env.as_deref(), Some("prod"));
    }

    #[test]
    fn build_resource_deploy_context_should_return_unmatched_ip_for_nginx() {
        let conn = setup_test_db();
        insert_test_nginx_config(&conn, "ng1", "gateway-1");
        conn.execute(
            "UPDATE nginx_configs SET address = 'http://10.9.9.9:80', env='dev' WHERE id = 'ng1'",
            [],
        )
        .expect("update nginx address");

        let context =
            build_resource_deploy_context("test", &conn, "nginx", "ng1").expect("build context");
        assert_eq!(context.parsed_ip.as_deref(), Some("10.9.9.9"));
        assert_eq!(context.matched_host_id, None);
        assert_eq!(context.resource_env.as_deref(), Some("dev"));
    }

    #[test]
    fn build_resource_deploy_context_should_allow_non_ip_application_address() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app1", "payment", "prod");
        conn.execute(
            "UPDATE applications SET address = 'https://api.example.com' WHERE id = 'app1'",
            [],
        )
        .expect("update app address");

        let context = build_resource_deploy_context("test", &conn, "application", "app1")
            .expect("build context");
        assert_eq!(context.address.as_deref(), Some("https://api.example.com"));
        assert_eq!(context.parsed_ip, None);
        assert_eq!(context.matched_host_id, None);
    }

    #[test]
    fn build_resource_deploy_context_should_fail_for_invalid_type() {
        let conn = setup_test_db();
        let err = build_resource_deploy_context("test", &conn, "invalid", "x")
            .expect_err("invalid type should fail");
        assert_eq!(err.code, AppErrorCode::ValidationError);
    }

    #[test]
    fn build_resource_deploy_context_should_prefer_same_env_host_when_ip_reused() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h-prod", "redis-prod", "10.0.0.9");
        insert_test_host(&conn, "h-dev", "redis-dev", "10.0.0.9");
        conn.execute("UPDATE hosts SET env='prod' WHERE id='h-prod'", [])
            .expect("update prod env");
        conn.execute("UPDATE hosts SET env='dev' WHERE id='h-dev'", [])
            .expect("update dev env");
        let ip_prod_id: String = conn
            .query_row(
                "SELECT id FROM ip_addresses WHERE ip_address='10.0.0.9' AND env='prod' AND is_deleted=0 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("query prod ip id");
        conn.execute(
            "INSERT INTO ip_addresses (id, ip_address, env, is_vip, real_ips, is_deleted, created_at, updated_at)
             VALUES ('ip-dev', '10.0.0.9', 'dev', 0, NULL, 0, datetime('now'), datetime('now'))",
            [],
        )
        .expect("insert ip addresses");
        conn.execute(
            "DELETE FROM host_ip_bindings WHERE host_id IN ('h-prod','h-dev')",
            [],
        )
        .expect("clear existing bindings");
        conn.execute(
            "INSERT INTO host_ip_bindings (id, host_id, ip_id, is_deleted, created_at, updated_at)
             VALUES ('hb-prod', 'h-prod', ?1, 0, datetime('now'), datetime('now')),
                    ('hb-dev', 'h-dev', 'ip-dev', 0, datetime('now'), datetime('now'))",
            rusqlite::params![ip_prod_id],
        )
        .expect("insert bindings");
        insert_test_middleware(&conn, "mw2", "redis-env", "cache");
        conn.execute(
            "UPDATE middlewares SET address = 'redis://10.0.0.9:6379', env = 'dev' WHERE id = 'mw2'",
            [],
        )
        .expect("update middleware");

        let context = build_resource_deploy_context("test", &conn, "middleware", "mw2")
            .expect("build context");
        assert_eq!(context.parsed_ip.as_deref(), Some("10.0.0.9"));
        assert_eq!(context.matched_host_id.as_deref(), Some("h-dev"));
        assert_eq!(context.resource_env.as_deref(), Some("dev"));
    }
}
