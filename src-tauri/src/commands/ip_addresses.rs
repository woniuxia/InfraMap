use tauri::State;

use crate::commands::taxonomy::{
    build_taxonomy_exists_filter, delete_resource_terms, parse_filter_values,
    parse_json_string_array, save_resource_terms, FIELD_TAGS,
};
use crate::db::crud::{
    build_where_clause, count_query, delete_with_audit, resolve_upsert_state, write_audit_log_entry,
};
use crate::db::transaction::with_transaction;
use crate::db::{get_conn, DbPool};
use crate::error::{AppError, AppErrorCode, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::ip_address::IpAddress;
use crate::validation::{
    validate_enum, validate_ip_address_resource, validate_ipv4, validate_json_array,
};
use serde::{Deserialize, Serialize};

const MAX_BATCH_RANGE: u32 = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateIpParams {
    pub start_ip: String,
    pub end_ip: String,
    pub env: String,
    pub tags: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateIpResult {
    pub created_count: u64,
    pub skipped_count: u64,
}

fn row_to_ip_address(row: &rusqlite::Row) -> rusqlite::Result<IpAddress> {
    Ok(IpAddress {
        id: row.get(0)?,
        ip_address: row.get(1)?,
        env: row.get(2)?,
        is_vip: row.get(3)?,
        real_ips: row.get(4)?,
        tags: row.get(5)?,
        description: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

const SELECT_COLUMNS: &str =
    "id, ip_address, env, is_vip, real_ips, tags, description, created_at, updated_at";

fn ipv4_to_u32(ip: &str) -> Result<u32, String> {
    let parsed = ip
        .trim()
        .parse::<std::net::Ipv4Addr>()
        .map_err(|_| format!("Invalid IPv4 address: {}", ip))?;
    Ok(u32::from(parsed))
}

fn u32_to_ipv4(ip: u32) -> String {
    std::net::Ipv4Addr::from(ip).to_string()
}

fn expand_ipv4_range(start_ip: &str, end_ip: &str, max_count: u32) -> Result<Vec<String>, String> {
    let start = ipv4_to_u32(start_ip)?;
    let end = ipv4_to_u32(end_ip)?;
    if start > end {
        return Err("start_ip must be less than or equal to end_ip".into());
    }
    let total = end - start + 1;
    if total > max_count {
        return Err(format!(
            "IP range is too large: {} (max allowed {})",
            total, max_count
        ));
    }

    let mut ips = Vec::with_capacity(total as usize);
    for value in start..=end {
        ips.push(u32_to_ipv4(value));
    }
    Ok(ips)
}

fn save_ip_address_inner(
    command: &str,
    conn: &rusqlite::Connection,
    data: IpAddress,
) -> AppResult<String> {
    validate_ip_address_resource(&data).map_err(|e| AppError::validation(command, e))?;
    let now = chrono::Utc::now().to_rfc3339();
    let (persisted_id, is_new) = resolve_upsert_state(command, conn, "ip_addresses", &data.id)?;

    with_transaction(conn, command, |conn| {
        if is_new {
            conn.execute(
                "INSERT INTO ip_addresses (id, ip_address, env, is_vip, real_ips, tags, description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,0,NULL,?8,?8)",
                rusqlite::params![
                    &persisted_id,
                    data.ip_address,
                    data.env,
                    data.is_vip,
                    data.real_ips,
                    data.tags,
                    data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建IP资源", e))?;
            write_audit_log_entry(
                command,
                conn,
                "create",
                "ip_address",
                &persisted_id,
                Some(&data.ip_address),
            )?;

            let tag_values = parse_json_string_array(data.tags.as_deref());
            save_resource_terms(
                conn,
                "ip_address",
                &persisted_id,
                FIELD_TAGS,
                &tag_values,
                &now,
            )
            .map_err(|e| AppError::from_db_error(command, "同步标签词条", e))?;
            Ok(persisted_id)
        } else {
            conn.execute(
                "UPDATE ip_addresses SET ip_address=?1, env=?2, is_vip=?3, real_ips=?4, tags=?5, description=?6, updated_at=?7
                 WHERE id=?8 AND is_deleted=0",
                rusqlite::params![
                    data.ip_address,
                    data.env,
                    data.is_vip,
                    data.real_ips,
                    data.tags,
                    data.description,
                    now,
                    data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新IP资源", e))?;
            write_audit_log_entry(
                command,
                conn,
                "update",
                "ip_address",
                &data.id,
                Some(&data.ip_address),
            )?;

            let tag_values = parse_json_string_array(data.tags.as_deref());
            save_resource_terms(conn, "ip_address", &data.id, FIELD_TAGS, &tag_values, &now)
                .map_err(|e| AppError::from_db_error(command, "同步标签词条", e))?;
            Ok(data.id.clone())
        }
    })
}

fn batch_create_ip_addresses_inner(
    command: &str,
    conn: &rusqlite::Connection,
    params: BatchCreateIpParams,
) -> AppResult<BatchCreateIpResult> {
    validate_ipv4(&params.start_ip).map_err(|e| AppError::validation(command, e))?;
    validate_ipv4(&params.end_ip).map_err(|e| AppError::validation(command, e))?;
    validate_enum(&params.env, &["prod", "dev", "test"], "env")
        .map_err(|e| AppError::validation(command, e))?;
    if let Some(tags) = params.tags.as_deref() {
        if !tags.trim().is_empty() {
            validate_json_array(tags, "tags").map_err(|e| AppError::validation(command, e))?;
        }
    }

    let ips = expand_ipv4_range(&params.start_ip, &params.end_ip, MAX_BATCH_RANGE)
        .map_err(|e| AppError::validation(command, e))?;

    let mut created_count: u64 = 0;
    let mut skipped_count: u64 = 0;
    for ip in ips {
        let data = IpAddress {
            id: "".into(),
            ip_address: ip,
            env: params.env.clone(),
            is_vip: false,
            real_ips: None,
            tags: params.tags.clone(),
            description: params.description.clone(),
            created_at: "".into(),
            updated_at: "".into(),
        };

        match save_ip_address_inner(command, conn, data) {
            Ok(_) => created_count += 1,
            Err(error) if error.code == AppErrorCode::Conflict => skipped_count += 1,
            Err(error) => return Err(error),
        }
    }

    Ok(BatchCreateIpResult {
        created_count,
        skipped_count,
    })
}

#[tauri::command]
pub fn list_ip_addresses(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<IpAddress>> {
    let command = "list_ip_addresses";
    let conn = get_conn(pool.inner(), command)?;

    let search_columns = &["ip_address", "description", "real_ips", "tags"];
    let filter_columns = &["env", "is_vip"];
    let (mut where_clause, mut sql_params) =
        build_where_clause(&params, search_columns, filter_columns);
    if let Some(filters) = params.filters.as_ref() {
        if let Some(raw_tags) = filters.get("tags") {
            let tag_values = parse_filter_values(raw_tags);
            if !tag_values.is_empty() {
                if let Some(clause) = build_taxonomy_exists_filter(
                    "ip_address",
                    FIELD_TAGS,
                    "ip_addresses.id",
                    &tag_values,
                ) {
                    where_clause = format!("{} AND {}", where_clause, clause);
                }
                for value in tag_values {
                    sql_params.push(Box::new(value));
                }
            }
        }
    }
    let total = count_query(
        command,
        "查询IP资源数量",
        &conn,
        "ip_addresses",
        &where_clause,
        &sql_params,
    )?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let sql = format!(
        "SELECT {} FROM ip_addresses {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|param| param.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询IP资源列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_ip_address)
        .map_err(|e| AppError::from_db_error(command, "读取IP资源列表", e))?;
    let data: Vec<IpAddress> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::from_db_error(command, "读取IP资源列表", e))?;

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_ip_address(pool: State<DbPool>, id: String) -> AppResult<IpAddress> {
    let command = "get_ip_address";
    let conn = get_conn(pool.inner(), command)?;
    let sql = format!(
        "SELECT {} FROM ip_addresses WHERE id=?1 AND is_deleted=0",
        SELECT_COLUMNS
    );

    conn.query_row(&sql, rusqlite::params![id], row_to_ip_address)
        .map_err(|e| AppError::not_found(command, "IP资源不存在或已删除。", Some(e.to_string())))
}

#[tauri::command]
pub fn save_ip_address(pool: State<DbPool>, data: IpAddress) -> AppResult<String> {
    let command = "save_ip_address";
    let conn = get_conn(pool.inner(), command)?;
    save_ip_address_inner(command, &conn, data)
}

#[tauri::command]
pub fn batch_create_ip_addresses(
    pool: State<DbPool>,
    params: BatchCreateIpParams,
) -> AppResult<BatchCreateIpResult> {
    let command = "batch_create_ip_addresses";
    let conn = get_conn(pool.inner(), command)?;
    batch_create_ip_addresses_inner(command, &conn, params)
}

fn delete_ip_address_inner(
    command: &str,
    conn: &rusqlite::Connection,
    id: String,
) -> AppResult<()> {
    let name: Option<String> = conn
        .query_row(
            "SELECT ip_address FROM ip_addresses WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    let binding_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM host_ip_bindings WHERE ip_id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::from_db_error(command, "查询IP绑定关系", e))?;
    if binding_count > 0 {
        return Err(AppError::dependency_conflict(
            command,
            "删除失败，IP 已绑定服务器，请先解绑后再删除。",
            None,
        ));
    }

    with_transaction(conn, command, |conn| {
        delete_with_audit(
            command,
            "删除IP资源",
            conn,
            "ip_addresses",
            "ip_address",
            &id,
            name.as_deref(),
        )?;
        let now = chrono::Utc::now().to_rfc3339();
        delete_resource_terms(conn, "ip_address", &id, &now)
            .map_err(|e| AppError::from_db_error(command, "删除标签绑定", e))?;
        Ok(())
    })
}

#[tauri::command]
pub fn delete_ip_address(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "delete_ip_address";
    let conn = get_conn(pool.inner(), command)?;
    delete_ip_address_inner(command, &conn, id)
}

#[cfg(test)]
mod tests {
    use super::{expand_ipv4_range, save_ip_address_inner, BatchCreateIpParams};
    use crate::commands::taxonomy::{list_terms_by_scope, FIELD_TAGS};
    use crate::error::AppErrorCode;
    use crate::models::ip_address::IpAddress;
    use crate::test_helpers::{insert_test_host, setup_test_db};

    fn make_test_ip() -> IpAddress {
        IpAddress {
            id: "".into(),
            ip_address: "10.0.0.10".into(),
            env: "prod".into(),
            is_vip: false,
            real_ips: None,
            tags: None,
            description: None,
            created_at: "".into(),
            updated_at: "".into(),
        }
    }

    #[test]
    fn save_ip_address_should_reject_invalid_vip_payload() {
        let conn = setup_test_db();
        let mut ip = make_test_ip();
        ip.is_vip = true;
        ip.real_ips = Some("[]".into());

        let err =
            save_ip_address_inner("test", &conn, ip).expect_err("vip without real_ips should fail");
        assert_eq!(err.code, AppErrorCode::ValidationError);
    }

    #[test]
    fn save_ip_address_should_enforce_unique_ip_and_env() {
        let conn = setup_test_db();
        save_ip_address_inner("test", &conn, make_test_ip()).expect("first save");

        let err = save_ip_address_inner("test", &conn, make_test_ip())
            .expect_err("duplicate ip+env should fail");
        assert_eq!(err.code, AppErrorCode::Conflict);
    }

    #[test]
    fn delete_ip_address_should_block_when_bound_to_host() {
        let conn = setup_test_db();
        insert_test_host(&conn, "h1", "server-1", "10.0.0.2");
        let ip_id: String = conn
            .query_row(
                "SELECT id FROM ip_addresses WHERE ip_address='10.0.0.2' AND env='prod' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("query ip id");

        let err = super::delete_ip_address_inner("test", &conn, "hb-unknown".to_string())
            .expect_err("deleting unknown id should fail first");
        assert_eq!(err.code, AppErrorCode::NotFound);

        let err = super::delete_ip_address_inner("test", &conn, ip_id)
            .expect_err("bound ip should not be deleted");
        assert_eq!(err.code, AppErrorCode::DependencyConflict);
    }

    #[test]
    fn expand_ipv4_range_should_include_boundaries() {
        let ips = expand_ipv4_range("10.0.0.10", "10.0.0.12", 1024).expect("expand range");
        assert_eq!(
            ips,
            vec![
                "10.0.0.10".to_string(),
                "10.0.0.11".to_string(),
                "10.0.0.12".to_string()
            ]
        );
    }

    #[test]
    fn batch_create_ip_addresses_should_create_all_ips_with_tags() {
        let conn = setup_test_db();
        let params = BatchCreateIpParams {
            start_ip: "10.0.1.1".into(),
            end_ip: "10.0.1.3".into(),
            env: "prod".into(),
            tags: Some(r#"["batch","generated"]"#.into()),
            description: Some("batch-created".into()),
        };

        let result = super::batch_create_ip_addresses_inner("test", &conn, params)
            .expect("batch create should succeed");
        assert_eq!(result.created_count, 3);
        assert_eq!(result.skipped_count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ip_addresses WHERE ip_address LIKE '10.0.1.%' AND env='prod' AND tags='[\"batch\",\"generated\"]' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("query inserted rows");
        assert_eq!(count, 3);

        let tags =
            list_terms_by_scope(&conn, "ip_address", FIELD_TAGS, 100).expect("query taxonomy tags");
        assert_eq!(tags, vec!["batch".to_string(), "generated".to_string()]);
    }

    #[test]
    fn save_ip_address_should_sync_taxonomy_tags() {
        let conn = setup_test_db();
        let mut ip = make_test_ip();
        ip.tags = Some(r#"["core","gateway"]"#.into());
        save_ip_address_inner("test", &conn, ip).expect("save ip with tags");

        let tags =
            list_terms_by_scope(&conn, "ip_address", FIELD_TAGS, 100).expect("query taxonomy tags");
        assert_eq!(tags, vec!["core".to_string(), "gateway".to_string()]);
    }
}
