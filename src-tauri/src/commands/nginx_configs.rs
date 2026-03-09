use tauri::State;

use crate::db::crud::{build_where_clause, count_query};
use crate::db::{get_conn, DbPool};
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::nginx_config::{NginxConfig, NginxEndpoint};

fn row_to_nginx_config(row: &rusqlite::Row) -> rusqlite::Result<NginxConfig> {
    let endpoints_raw: String = row.get(2)?;
    let endpoints = serde_json::from_str::<Vec<NginxEndpoint>>(&endpoints_raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(err))
    })?;
    Ok(NginxConfig {
        id: row.get(0)?,
        name: row.get(1)?,
        endpoints,
        strategy: row.get(3)?,
        env: row.get(4)?,
        status: row.get(5)?,
        description: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

const SELECT_COLUMNS: &str =
    "id, name, endpoints, strategy, env, status, description, created_at, updated_at";

#[tauri::command]
pub fn list_nginx_configs(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<NginxConfig>> {
    let command = "list_nginx_configs";
    let conn = get_conn(pool.inner(), command)?;

    let search_columns = &["name"];
    let filter_columns = &["env", "status", "strategy"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(
        command,
        "查询网关配置数量",
        &conn,
        "nginx_configs",
        &where_clause,
        &sql_params,
    )?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM nginx_configs {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询网关配置列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_nginx_config)
        .map_err(|e| AppError::from_db_error(command, "读取网关配置列表", e))?;

    let data: Vec<NginxConfig> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_nginx_config(pool: State<DbPool>, id: String) -> AppResult<NginxConfig> {
    let command = "get_nginx_config";
    let conn = get_conn(pool.inner(), command)?;

    let sql = format!(
        "SELECT {} FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_nginx_config)
        .map_err(|e| AppError::not_found(command, "网关配置不存在或已删除。", Some(e.to_string())))
}

impl_save_command!(
    fn_name: save_nginx_config,
    model: NginxConfig,
    table: "nginx_configs",
    resource_type: "nginx",
    validator: crate::validation::validate_nginx_config,
    create_label: "创建网关配置",
    update_label: "更新网关配置",
    |data, persisted_id, now, command| {
        prepare: {
            let endpoints_json = serde_json::to_string(&data.endpoints)
                .map_err(|e| crate::error::AppError::internal(command, format!("serialize nginx endpoints: {}", e)))?;
        },
        insert: (
            "INSERT INTO nginx_configs (id, name, endpoints, strategy, env, status, description,
                                        is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,0,NULL,?8,?8)",
            [&persisted_id, data.name, &endpoints_json, data.strategy,
             data.env, data.status, data.description, now]
        ),
        update: (
            "UPDATE nginx_configs SET name=?1, endpoints=?2, strategy=?3,
                                      env=?4, status=?5, description=?6, updated_at=?7
             WHERE id=?8 AND is_deleted=0",
            [data.name, &endpoints_json, data.strategy, data.env,
             data.status, data.description, now, data.id]
        ),
        display_name: data.name,
    },
);

impl_delete_command!(
    fn_name: delete_nginx_config,
    table: "nginx_configs",
    resource_type: "nginx",
    delete_label: "删除网关配置",
    name_column: "name",
);
