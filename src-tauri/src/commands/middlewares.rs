use tauri::State;

use crate::db::crud::{build_where_clause, count_query};
use crate::db::{get_conn, DbPool};
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::middleware::Middleware;

fn row_to_middleware(row: &rusqlite::Row) -> rusqlite::Result<Middleware> {
    Ok(Middleware {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(2)?,
        mw_type: row.get(3)?,
        address: row.get(4)?,
        port: row.get(5)?,
        version: row.get(6)?,
        env: row.get(7)?,
        description: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

const SELECT_COLUMNS: &str = "id, name, category, type, address, port, version, \
     env, description, created_at, updated_at";

#[tauri::command]
pub fn list_middlewares(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Middleware>> {
    let command = "list_middlewares";
    let conn = get_conn(pool.inner(), command)?;

    let search_columns = &["name", "address"];
    let filter_columns = &["category", "type", "env"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(
        command,
        "查询中间件数量",
        &conn,
        "middlewares",
        &where_clause,
        &sql_params,
    )?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM middlewares {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询中间件列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_middleware)
        .map_err(|e| AppError::from_db_error(command, "读取中间件列表", e))?;

    let data: Vec<Middleware> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_middleware(pool: State<DbPool>, id: String) -> AppResult<Middleware> {
    let command = "get_middleware";
    let conn = get_conn(pool.inner(), command)?;
    let sql = format!(
        "SELECT {} FROM middlewares WHERE id = ?1 AND is_deleted = 0",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_middleware)
        .map_err(|e| AppError::not_found(command, "中间件不存在或已删除。", Some(e.to_string())))
}

impl_save_command!(
    fn_name: save_middleware,
    model: Middleware,
    table: "middlewares",
    resource_type: "middleware",
    validator: crate::validation::validate_middleware,
    create_label: "创建中间件",
    update_label: "更新中间件",
    |data, persisted_id, now, command| {
        insert: (
            "INSERT INTO middlewares (id, name, category, type, address, port, version,
                                     env, description, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,0,NULL,?10,?10)",
            [&persisted_id, data.name, data.category, data.mw_type, data.address,
             data.port, data.version, data.env, data.description, now]
        ),
        update: (
            "UPDATE middlewares SET name=?1, category=?2, type=?3, address=?4, port=?5, version=?6,
                                   env=?7, description=?8, updated_at=?9
             WHERE id=?10 AND is_deleted=0",
            [data.name, data.category, data.mw_type, data.address, data.port,
             data.version, data.env, data.description, now, data.id]
        ),
        display_name: data.name,
    },
);

impl_delete_command!(
    fn_name: delete_middleware,
    table: "middlewares",
    resource_type: "middleware",
    delete_label: "删除中间件",
    name_column: "name",
);
