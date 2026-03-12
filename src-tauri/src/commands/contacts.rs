use tauri::State;

use crate::db::crud::{build_resource_where_clause, count_query, delete_with_audit, SqlParam};
use crate::db::{get_conn, DbPool};
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::contact::Contact;

fn row_to_contact(row: &rusqlite::Row) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: row.get(0)?,
        name: row.get(1)?,
        company: row.get(2)?,
        phone: row.get(3)?,
        email: row.get(4)?,
        remark: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

const SELECT_COLUMNS: &str = "c.id, c.name, c.company, c.phone, c.email, c.remark, c.created_at, c.updated_at";

fn build_contacts_where_clause(params: &QueryParams) -> (String, Vec<SqlParam>) {
    build_resource_where_clause(
        &["c.is_deleted = 0"],
        params,
        &["c.name", "c.company", "c.phone", "c.email", "c.remark"],
        &[],
        &[],
        &[],
    )
}

#[tauri::command]
pub fn list_contacts(pool: State<DbPool>, params: QueryParams) -> AppResult<PagedResult<Contact>> {
    let command = "list_contacts";
    let conn = get_conn(pool.inner(), command)?;

    let (where_clause, sql_params) = build_contacts_where_clause(&params);
    let total = count_query(
        command,
        "查询联系人数量",
        &conn,
        "contacts c",
        &where_clause,
        &sql_params,
    )?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM contacts c {} ORDER BY c.name COLLATE NOCASE ASC, c.updated_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );
    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|param| param.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询联系人列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_contact)
        .map_err(|e| AppError::from_db_error(command, "读取联系人列表", e))?;
    let data: Vec<Contact> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::from_db_error(command, "读取联系人列表", e))?;

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_contact(pool: State<DbPool>, id: String) -> AppResult<Contact> {
    let command = "get_contact";
    let conn = get_conn(pool.inner(), command)?;

    let sql = format!(
        "SELECT {} FROM contacts c WHERE c.id = ?1 AND c.is_deleted = 0",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_contact)
        .map_err(|e| AppError::not_found(command, "联系人不存在或已删除。", Some(e.to_string())))
}

impl_save_command!(
    fn_name: save_contact,
    model: Contact,
    table: "contacts",
    resource_type: "contact",
    validator: crate::validation::validate_contact,
    create_label: "创建联系人",
    update_label: "更新联系人",
    |data, persisted_id, now, _command| {
        prepare: {
            let name = data.name.trim().to_string();
            let company = data.company.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(|v| v.to_string());
            let phone = data.phone.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(|v| v.to_string());
            let email = data.email.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(|v| v.to_string());
            let remark = data.remark.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(|v| v.to_string());
        },
        insert: (
            "INSERT INTO contacts (id, name, company, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,0,NULL,?7,?7)",
            [&persisted_id, &name, company, phone, email, remark, now]
        ),
        update: (
            "UPDATE contacts SET name=?1, company=?2, phone=?3, email=?4, remark=?5, updated_at=?6
             WHERE id=?7 AND is_deleted=0",
            [&name, company, phone, email, remark, now, data.id]
        ),
        display_name: name,
    },
);

#[tauri::command]
pub fn delete_contact(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "delete_contact";
    let conn = get_conn(pool.inner(), command)?;

    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM contacts WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    let application_refs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM application_owner_contacts WHERE contact_id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::from_db_error(command, "查询应用负责人引用", e))?;

    let business_refs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM business_application_owner_contacts WHERE contact_id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::from_db_error(command, "查询业务应用负责人引用", e))?;

    if application_refs > 0 || business_refs > 0 {
        return Err(AppError::dependency_conflict(
            command,
            "删除失败，联系人仍被负责人引用，请先解除引用后再删除。",
            Some(format!(
                "application_refs={}, business_refs={}",
                application_refs, business_refs
            )),
        ));
    }

    delete_with_audit(
        command,
        "删除联系人",
        &conn,
        "contacts",
        "contact",
        &id,
        name.as_deref(),
    )
}
