use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::application::Application;
use crate::models::business_application::BusinessApplication;
use crate::models::common::{PagedResult, QueryParams};
use crate::validation::{validate_business_application, validate_required};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AttachServicesResult {
    pub attached_count: u64,
    pub skipped_count: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BusinessApplicationServices {
    pub frontend: Vec<Application>,
    pub backend: Vec<Application>,
}

fn row_to_business_application(row: &rusqlite::Row) -> rusqlite::Result<BusinessApplication> {
    Ok(BusinessApplication {
        id: row.get(0)?,
        name: row.get(1)?,
        code: row.get(2)?,
        owner: row.get(3)?,
        description: row.get(4)?,
        env: row.get(5)?,
        status: row.get(6)?,
        is_deleted: row.get(7)?,
        deleted_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_application_with_business(row: &rusqlite::Row) -> rusqlite::Result<Application> {
    Ok(Application {
        id: row.get(0)?,
        name: row.get(1)?,
        app_type: row.get(2)?,
        address: row.get(3)?,
        port: row.get(4)?,
        tech_stack: row.get(5)?,
        deploy_mode: row.get(6)?,
        env: row.get(7)?,
        git_repo: row.get(8)?,
        owner: row.get(9)?,
        owners: Some(Vec::new()),
        business_application_id: row.get(10)?,
        business_application_name: row.get(11)?,
        status: row.get(12)?,
        description: row.get(13)?,
        is_deleted: row.get(14)?,
        deleted_at: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

const SELECT_BUSINESS_APPLICATION_COLUMNS: &str =
    "id, name, code, owner, description, env, status, is_deleted, deleted_at, created_at, updated_at";
const SELECT_APPLICATION_COLUMNS: &str = "applications.id, applications.name, applications.type, applications.address, \
     applications.port, applications.tech_stack, applications.deploy_mode, applications.env, applications.git_repo, \
     applications.owner, applications.business_application_id, \
     (SELECT ba.name FROM business_applications ba WHERE ba.id = applications.business_application_id AND ba.is_deleted = 0) AS business_application_name, \
     applications.status, applications.description, applications.is_deleted, applications.deleted_at, applications.created_at, applications.updated_at";

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

fn build_unassigned_applications_where_clause(
    params: &QueryParams,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions: Vec<String> = vec![
        "applications.is_deleted = 0".to_string(),
        "(applications.business_application_id IS NULL OR TRIM(applications.business_application_id) = '')"
            .to_string(),
    ];
    let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(search) = params
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let like_value = format!("%{}%", search);
        conditions.push(
            "(applications.name LIKE ? \
                OR applications.address LIKE ? \
                OR applications.tech_stack LIKE ? \
                OR applications.git_repo LIKE ? \
                OR applications.owner LIKE ?)"
                .to_string(),
        );
        for _ in 0..5 {
            sql_params.push(Box::new(like_value.clone()));
        }
    }

    if let Some(filters) = &params.filters {
        for (column, key) in [
            ("applications.type", "type"),
            ("applications.env", "env"),
            ("applications.status", "status"),
        ] {
            if let Some(value) = filters.get(key) {
                let values = parse_filter_values(value);
                if values.is_empty() {
                    continue;
                }
                if values.len() == 1 {
                    conditions.push(format!("{} = ?", column));
                    sql_params.push(Box::new(values[0].clone()));
                } else {
                    let placeholders = vec!["?"; values.len()].join(", ");
                    conditions.push(format!("{} IN ({})", column, placeholders));
                    for item in values {
                        sql_params.push(Box::new(item));
                    }
                }
            }
        }
    }

    (format!("WHERE {}", conditions.join(" AND ")), sql_params)
}

fn save_business_application_inner(
    command: &str,
    conn: &rusqlite::Connection,
    data: BusinessApplication,
) -> AppResult<String> {
    validate_business_application(&data).map_err(|e| AppError::validation(command, e))?;

    let now = chrono::Utc::now().to_rfc3339();
    let is_new = data.id.is_empty() || {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM business_applications WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![data.id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        count == 0
    };

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<String> = (|| {
        if is_new {
            let id = if data.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                data.id.clone()
            };
            conn.execute(
                "INSERT INTO business_applications (id, name, code, owner, description, env, status, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, NULL, ?8, ?8)",
                rusqlite::params![
                    id,
                    data.name,
                    data.code,
                    data.owner,
                    data.description,
                    data.env,
                    data.status,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建业务应用", e))?;
            insert_audit_log(
                conn,
                "create",
                "business_application",
                &id,
                Some(&data.name),
                None,
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
            Ok(id)
        } else {
            conn.execute(
                "UPDATE business_applications
                 SET name = ?1, code = ?2, owner = ?3, description = ?4, env = ?5, status = ?6, updated_at = ?7
                 WHERE id = ?8 AND is_deleted = 0",
                rusqlite::params![
                    data.name,
                    data.code,
                    data.owner,
                    data.description,
                    data.env,
                    data.status,
                    now,
                    data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新业务应用", e))?;
            insert_audit_log(
                conn,
                "update",
                "business_application",
                &data.id,
                Some(&data.name),
                None,
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
            Ok(data.id.clone())
        }
    })();

    match result {
        Ok(id) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(id)
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn attach_services_to_business_application_inner(
    command: &str,
    conn: &rusqlite::Connection,
    business_application_id: &str,
    application_ids: &[String],
) -> AppResult<AttachServicesResult> {
    validate_required(business_application_id, "business_application_id")
        .map_err(|e| AppError::validation(command, e))?;
    if application_ids.is_empty() {
        return Err(AppError::validation(command, "application_ids is required"));
    }

    let ba_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM business_applications WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![business_application_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::from_db_error(command, "查询业务应用", e))?;
    if ba_exists == 0 {
        return Err(AppError::not_found(
            command,
            "业务应用不存在或已删除。",
            None,
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<AttachServicesResult> = (|| {
        let mut attached_count = 0u64;
        let mut skipped_count = 0u64;

        for application_id in application_ids {
            let info = conn
                .query_row(
                    "SELECT name, business_application_id
                     FROM applications
                     WHERE id = ?1 AND is_deleted = 0",
                    rusqlite::params![application_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
                )
                .map_err(|e| {
                    AppError::not_found(command, "应用服务不存在或已删除。", Some(e.to_string()))
                })?;

            if let Some(current_ba_id) = info
                .1
                .as_deref()
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
            {
                if current_ba_id == business_application_id {
                    skipped_count += 1;
                    continue;
                }
                return Err(AppError::conflict(
                    command,
                    "服务已归属其他业务应用，请先解绑后再挂载。",
                    Some(format!(
                        "application_id={}, application_name={}, current_business_application_id={}",
                        application_id, info.0, current_ba_id
                    )),
                ));
            }

            conn.execute(
                "UPDATE applications
                 SET business_application_id = ?1, updated_at = ?2
                 WHERE id = ?3 AND is_deleted = 0",
                rusqlite::params![business_application_id, now, application_id],
            )
            .map_err(|e| AppError::from_db_error(command, "绑定应用服务", e))?;
            insert_audit_log(
                conn,
                "update",
                "application",
                application_id,
                Some(&info.0),
                Some(&format!(
                    "bind business_application_id={}",
                    business_application_id
                )),
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
            attached_count += 1;
        }

        Ok(AttachServicesResult {
            attached_count,
            skipped_count,
        })
    })();

    match result {
        Ok(summary) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(summary)
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn detach_service_from_business_application_inner(
    command: &str,
    conn: &rusqlite::Connection,
    business_application_id: &str,
    application_id: &str,
) -> AppResult<()> {
    validate_required(business_application_id, "business_application_id")
        .map_err(|e| AppError::validation(command, e))?;
    validate_required(application_id, "application_id")
        .map_err(|e| AppError::validation(command, e))?;

    let info = conn
        .query_row(
            "SELECT name, business_application_id
             FROM applications
             WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![application_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .map_err(|e| {
            AppError::not_found(command, "应用服务不存在或已删除。", Some(e.to_string()))
        })?;

    let current_ba_id = info.1.as_deref().unwrap_or("").trim().to_string();
    if current_ba_id.is_empty() || current_ba_id != business_application_id {
        return Err(AppError::not_found(
            command,
            "该应用服务未归属当前业务应用。",
            None,
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<()> = (|| {
        conn.execute(
            "UPDATE applications
             SET business_application_id = NULL, updated_at = ?1
             WHERE id = ?2 AND is_deleted = 0",
            rusqlite::params![now, application_id],
        )
        .map_err(|e| AppError::from_db_error(command, "解绑应用服务", e))?;
        insert_audit_log(
            conn,
            "update",
            "application",
            application_id,
            Some(&info.0),
            Some(&format!(
                "unbind business_application_id={}",
                business_application_id
            )),
        )
        .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
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
pub fn list_business_applications(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<BusinessApplication>> {
    let command = "list_business_applications";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let search_columns = &["name", "code", "owner", "description"];
    let filter_columns = &["status", "env"];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);
    let total = count_query(&conn, "business_applications", &where_clause, &sql_params)
        .map_err(|e| AppError::from_db_error(command, "查询业务应用数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let sql = format!(
        "SELECT {} FROM business_applications {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_BUSINESS_APPLICATION_COLUMNS, where_clause
    );
    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|param| param.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询业务应用列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_business_application)
        .map_err(|e| AppError::from_db_error(command, "读取业务应用列表", e))?;
    let data: Vec<BusinessApplication> = rows.filter_map(|row| row.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_business_application(pool: State<DbPool>, id: String) -> AppResult<BusinessApplication> {
    let command = "get_business_application";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let sql = format!(
        "SELECT {} FROM business_applications WHERE id = ?1 AND is_deleted = 0",
        SELECT_BUSINESS_APPLICATION_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_business_application)
        .map_err(|e| AppError::not_found(command, "业务应用不存在或已删除。", Some(e.to_string())))
}

#[tauri::command]
pub fn save_business_application(
    pool: State<DbPool>,
    data: BusinessApplication,
) -> AppResult<String> {
    let command = "save_business_application";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    save_business_application_inner(command, &conn, data)
}

#[tauri::command]
pub fn soft_delete_business_application(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "soft_delete_business_application";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    let now = chrono::Utc::now().to_rfc3339();
    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM business_applications WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<()> = (|| {
        soft_delete(&conn, "business_applications", &id)
            .map_err(|e| AppError::from_db_error(command, "删除业务应用", e))?;
        conn.execute(
            "UPDATE applications
             SET business_application_id = NULL, updated_at = ?1
             WHERE business_application_id = ?2 AND is_deleted = 0",
            rusqlite::params![now, id],
        )
        .map_err(|e| AppError::from_db_error(command, "解除应用归属", e))?;
        insert_audit_log(
            &conn,
            "delete",
            "business_application",
            &id,
            name.as_deref(),
            None,
        )
        .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
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
pub fn list_unassigned_application_services(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Application>> {
    let command = "list_unassigned_application_services";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let (where_clause, sql_params) = build_unassigned_applications_where_clause(&params);
    let count_sql = format!("SELECT COUNT(*) FROM applications {}", where_clause);
    let count_refs: Vec<&dyn rusqlite::types::ToSql> =
        sql_params.iter().map(|p| p.as_ref()).collect();
    let total: u64 = conn
        .query_row(&count_sql, count_refs.as_slice(), |row| {
            row.get::<_, i64>(0)
        })
        .map(|count| count as u64)
        .map_err(|e| AppError::from_db_error(command, "查询未归属应用数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let sql = format!(
        "SELECT {} FROM applications {} ORDER BY applications.created_at DESC LIMIT ? OFFSET ?",
        SELECT_APPLICATION_COLUMNS, where_clause
    );
    let mut query_params = sql_params;
    query_params.push(Box::new(page_size as i64));
    query_params.push(Box::new(offset as i64));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        query_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询未归属应用列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_application_with_business)
        .map_err(|e| AppError::from_db_error(command, "读取未归属应用列表", e))?;
    let data: Vec<Application> = rows.filter_map(|row| row.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn attach_services_to_business_application(
    pool: State<DbPool>,
    business_application_id: String,
    application_ids: Vec<String>,
) -> AppResult<AttachServicesResult> {
    let command = "attach_services_to_business_application";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    attach_services_to_business_application_inner(
        command,
        &conn,
        &business_application_id,
        &application_ids,
    )
}

#[tauri::command]
pub fn detach_service_from_business_application(
    pool: State<DbPool>,
    business_application_id: String,
    application_id: String,
) -> AppResult<()> {
    let command = "detach_service_from_business_application";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    detach_service_from_business_application_inner(
        command,
        &conn,
        &business_application_id,
        &application_id,
    )
}

#[tauri::command]
pub fn list_services_by_business_application(
    pool: State<DbPool>,
    business_application_id: String,
) -> AppResult<BusinessApplicationServices> {
    let command = "list_services_by_business_application";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    list_services_by_business_application_inner(command, &conn, &business_application_id)
}

fn list_services_by_business_application_inner(
    command: &str,
    conn: &rusqlite::Connection,
    business_application_id: &str,
) -> AppResult<BusinessApplicationServices> {
    validate_required(business_application_id, "business_application_id")
        .map_err(|e| AppError::validation(command, e))?;
    let sql = format!(
        "SELECT {} FROM applications
         WHERE is_deleted = 0 AND business_application_id = ?1
         ORDER BY created_at DESC",
        SELECT_APPLICATION_COLUMNS
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询业务应用服务", e))?;
    let rows = stmt
        .query_map(
            rusqlite::params![business_application_id],
            row_to_application_with_business,
        )
        .map_err(|e| AppError::from_db_error(command, "读取业务应用服务", e))?;
    let services: Vec<Application> = rows.filter_map(|row| row.ok()).collect();

    let mut frontend = Vec::new();
    let mut backend = Vec::new();
    for service in services {
        if service.app_type == "frontend" {
            frontend.push(service);
        } else {
            backend.push(service);
        }
    }

    Ok(BusinessApplicationServices { frontend, backend })
}

#[cfg(test)]
mod tests {
    use super::{
        attach_services_to_business_application_inner,
        detach_service_from_business_application_inner,
        list_services_by_business_application_inner, save_business_application_inner,
        AttachServicesResult,
    };
    use crate::error::AppErrorCode;
    use crate::models::business_application::BusinessApplication;
    use crate::test_helpers::{insert_test_application, setup_test_db};

    fn make_business_application(name: &str) -> BusinessApplication {
        BusinessApplication {
            id: "".into(),
            name: name.into(),
            code: Some("PAY".into()),
            owner: Some("alice".into()),
            description: None,
            env: Some("prod".into()),
            status: "active".into(),
            is_deleted: 0,
            deleted_at: None,
            created_at: "".into(),
            updated_at: "".into(),
        }
    }

    fn insert_business_application(conn: &rusqlite::Connection, id: &str, name: &str) {
        conn.execute(
            "INSERT INTO business_applications (id, name, code, owner, description, env, status, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, 'PAY', 'alice', NULL, 'prod', 'active', 0, NULL, datetime('now'), datetime('now'))",
            rusqlite::params![id, name],
        )
        .expect("insert business application");
    }

    #[test]
    fn save_business_application_inner_should_create_and_update() {
        let conn = setup_test_db();
        let mut payload = make_business_application("支付中心");
        let created_id =
            save_business_application_inner("test", &conn, payload.clone()).expect("create");
        assert!(!created_id.is_empty());

        payload.id = created_id.clone();
        payload.name = "支付中心-新".into();
        let updated_id = save_business_application_inner("test", &conn, payload).expect("update");
        assert_eq!(updated_id, created_id);
    }

    #[test]
    fn attach_services_should_bind_unassigned_services() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        insert_test_application(&conn, "app-b", "api-b", "prod");

        let result = attach_services_to_business_application_inner(
            "test",
            &conn,
            "ba-1",
            &["app-a".into(), "app-b".into()],
        )
        .expect("attach should pass");
        assert_eq!(
            result,
            AttachServicesResult {
                attached_count: 2,
                skipped_count: 0
            }
        );
    }

    #[test]
    fn attach_services_should_reject_conflict_service() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_business_application(&conn, "ba-2", "订单中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        conn.execute(
            "UPDATE applications SET business_application_id = 'ba-2' WHERE id = 'app-a'",
            [],
        )
        .expect("seed bind");

        let err =
            attach_services_to_business_application_inner("test", &conn, "ba-1", &["app-a".into()])
                .expect_err("should reject");
        assert_eq!(err.code, AppErrorCode::Conflict);
    }

    #[test]
    fn detach_service_should_clear_business_application_binding() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        conn.execute(
            "UPDATE applications SET business_application_id = 'ba-1' WHERE id = 'app-a'",
            [],
        )
        .expect("seed bind");

        detach_service_from_business_application_inner("test", &conn, "ba-1", "app-a")
            .expect("detach");
        let current: Option<String> = conn
            .query_row(
                "SELECT business_application_id FROM applications WHERE id = 'app-a'",
                [],
                |row| row.get(0),
            )
            .expect("query current binding");
        assert!(current.is_none());
    }

    #[test]
    fn services_grouping_should_split_frontend_and_backend() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        insert_test_application(&conn, "app-b", "api-b", "prod");
        conn.execute(
            "UPDATE applications SET type = 'frontend' WHERE id = 'app-a'",
            [],
        )
        .expect("seed frontend");
        conn.execute(
            "UPDATE applications SET business_application_id = 'ba-1' WHERE id IN ('app-a','app-b')",
            [],
        )
        .expect("seed bind");

        let services = list_services_by_business_application_inner("test", &conn, "ba-1")
            .expect("list services");

        assert_eq!(services.frontend.len(), 1);
        assert_eq!(services.backend.len(), 1);
    }
}
