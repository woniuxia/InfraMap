use std::collections::{HashMap, HashSet};
use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{
    build_exists_like_clause, count_query, delete_by_id, parse_filter_values, resolve_upsert_state,
    write_audit_log_entry,
};
use crate::db::transaction::with_transaction;
use crate::db::{get_conn, DbPool};
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ReplaceServicesResult {
    pub attached_count: u64,
    pub detached_count: u64,
    pub unchanged_count: u64,
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
        owner_contact_ids: Some(Vec::new()),
        owners: Some(Vec::new()),
        description: row.get(3)?,
        env: row.get(4)?,
        status: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
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
        owner_contact_ids: Some(Vec::new()),
        owners: Some(Vec::new()),
        business_application_id: row.get(9)?,
        business_application_name: row.get(10)?,
        status: row.get(11)?,
        description: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

const SELECT_BUSINESS_APPLICATION_COLUMNS: &str =
    "id, name, code, description, env, status, created_at, updated_at";
const SELECT_APPLICATION_COLUMNS: &str = "applications.id, applications.name, applications.type, applications.address, \
     applications.port, applications.tech_stack, applications.deploy_mode, applications.env, applications.git_repo, \
     applications.business_application_id, \
     (SELECT ba.name FROM business_applications ba WHERE ba.id = applications.business_application_id AND ba.is_deleted = 0) AS business_application_name, \
     applications.status, applications.description, applications.created_at, applications.updated_at";

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
                OR applications.git_repo LIKE ?)"
                .to_string(),
        );
        for _ in 0..4 {
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

fn build_business_applications_where_clause(
    params: &QueryParams,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions: Vec<String> = vec!["business_applications.is_deleted = 0".to_string()];
    let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(search) = params
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let like_value = format!("%{}%", search);
        let owner_name_search_clause = build_exists_like_clause(
            "business_application_owner_contacts baoc JOIN contacts c ON c.id = baoc.contact_id",
            &[
                "baoc.business_application_id = business_applications.id",
                "baoc.is_deleted = 0",
                "c.is_deleted = 0",
            ],
            "c.name",
        );
        let owner_phone_search_clause = build_exists_like_clause(
            "business_application_owner_contacts baoc JOIN contacts c ON c.id = baoc.contact_id",
            &[
                "baoc.business_application_id = business_applications.id",
                "baoc.is_deleted = 0",
                "c.is_deleted = 0",
            ],
            "c.phone",
        );
        let owner_email_search_clause = build_exists_like_clause(
            "business_application_owner_contacts baoc JOIN contacts c ON c.id = baoc.contact_id",
            &[
                "baoc.business_application_id = business_applications.id",
                "baoc.is_deleted = 0",
                "c.is_deleted = 0",
            ],
            "c.email",
        );

        conditions.push(
            format!(
                "(business_applications.name LIKE ? \
                  OR business_applications.code LIKE ? \
                  OR business_applications.description LIKE ? \
                  OR {owner_name_search_clause} \
                  OR {owner_phone_search_clause} \
                  OR {owner_email_search_clause})"
            ),
        );
        for _ in 0..6 {
            sql_params.push(Box::new(like_value.clone()));
        }
    }

    if let Some(filters) = &params.filters {
        for (column, key) in [
            ("business_applications.status", "status"),
            ("business_applications.env", "env"),
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

        if let Some(owner_filter) = filters.get("owner") {
            let owner_contact_ids = parse_filter_values(owner_filter);
            if !owner_contact_ids.is_empty() {
                let placeholders = vec!["?"; owner_contact_ids.len()].join(", ");
                conditions.push(format!(
                    "EXISTS (
                        SELECT 1
                        FROM business_application_owner_contacts baoc
                        JOIN contacts c ON c.id = baoc.contact_id
                        WHERE baoc.business_application_id = business_applications.id
                          AND baoc.is_deleted = 0
                          AND c.is_deleted = 0
                          AND baoc.contact_id IN ({})
                    )",
                    placeholders
                ));
                for contact_id in owner_contact_ids {
                    sql_params.push(Box::new(contact_id));
                }
            }
        }
    }

    (format!("WHERE {}", conditions.join(" AND ")), sql_params)
}

fn normalize_owner_contact_ids(owner_contact_ids: Option<Vec<String>>) -> Vec<String> {
    let mut normalized: Vec<String> = Vec::new();
    let mut dedupe: HashSet<String> = HashSet::new();

    if let Some(items) = owner_contact_ids {
        for item in items {
            let contact_id = item.trim().to_string();
            if contact_id.is_empty() {
                continue;
            }
            if dedupe.insert(contact_id.clone()) {
                normalized.push(contact_id);
            }
        }
    }

    normalized
}

fn load_business_application_owner_contacts_map(
    conn: &rusqlite::Connection,
    business_app_ids: &[String],
) -> Result<HashMap<String, Vec<(String, String)>>, rusqlite::Error> {
    if business_app_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = vec!["?"; business_app_ids.len()].join(", ");
    let sql = format!(
        "SELECT baoc.business_application_id, c.id, c.name
         FROM business_application_owner_contacts baoc
         JOIN contacts c ON c.id = baoc.contact_id
         WHERE baoc.is_deleted = 0
           AND c.is_deleted = 0
           AND baoc.business_application_id IN ({})
         ORDER BY c.name COLLATE NOCASE ASC, c.id ASC",
        placeholders
    );
    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&str> = business_app_ids.iter().map(String::as_str).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(param_refs), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for row in rows {
        let (business_app_id, contact_id, contact_name) = row?;
        map.entry(business_app_id)
            .or_default()
            .push((contact_id, contact_name));
    }
    Ok(map)
}

fn merge_business_application_owners(
    business: &mut BusinessApplication,
    owner_map: &HashMap<String, Vec<(String, String)>>,
) {
    if let Some(items) = owner_map.get(&business.id) {
        business.owner_contact_ids = Some(items.iter().map(|(id, _)| id.clone()).collect());
        business.owners = Some(items.iter().map(|(_, name)| name.clone()).collect());
    } else {
        business.owner_contact_ids = Some(Vec::new());
        business.owners = Some(Vec::new());
    }
}

fn sync_business_application_owner_contacts(
    command: &str,
    conn: &rusqlite::Connection,
    business_application_id: &str,
    owner_contact_ids: &[String],
    now: &str,
) -> AppResult<()> {
    // 物理删除 + 重建关系：简单可靠，避免残留重复数据。
    conn.execute(
        "DELETE FROM business_application_owner_contacts WHERE business_application_id = ?1",
        rusqlite::params![business_application_id],
    )
    .map_err(|e| AppError::from_db_error(command, "清理业务应用负责人绑定", e))?;

    if owner_contact_ids.is_empty() {
        return Ok(());
    }

    // 防御式校验 contact_id：必须存在且未删除。
    let placeholders = vec!["?"; owner_contact_ids.len()].join(", ");
    let sql = format!(
        "SELECT id FROM contacts WHERE is_deleted = 0 AND id IN ({})",
        placeholders
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询联系人", e))?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(owner_contact_ids.iter()),
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| AppError::from_db_error(command, "读取联系人", e))?;

    let mut existing: HashSet<String> = HashSet::new();
    for row in rows {
        existing.insert(row.map_err(|e| AppError::from_db_error(command, "读取联系人", e))?);
    }

    let missing: Vec<String> = owner_contact_ids
        .iter()
        .filter(|id| !existing.contains(*id))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(AppError::validation(
            command,
            format!("负责人联系人不存在或已删除: {}", missing.join(", ")),
        ));
    }

    for contact_id in owner_contact_ids {
        conn.execute(
            "INSERT INTO business_application_owner_contacts (id, business_application_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, NULL, ?4, ?4)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                business_application_id,
                contact_id,
                now
            ],
        )
        .map_err(|e| AppError::from_db_error(command, "创建业务应用负责人绑定", e))?;
    }

    Ok(())
}

fn save_business_application_inner(
    command: &str,
    conn: &rusqlite::Connection,
    data: BusinessApplication,
) -> AppResult<String> {
    let normalized_owner_contact_ids = normalize_owner_contact_ids(data.owner_contact_ids.clone());
    let mut normalized_data = data.clone();
    normalized_data.owner_contact_ids = Some(normalized_owner_contact_ids.clone());

    validate_business_application(&normalized_data).map_err(|e| AppError::validation(command, e))?;

    let now = chrono::Utc::now().to_rfc3339();
    let (persisted_id, is_new) =
        resolve_upsert_state(command, conn, "business_applications", &normalized_data.id)?;

    with_transaction(conn, command, |conn| {
        if is_new {
            conn.execute(
                "INSERT INTO business_applications (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, NULL, ?7, ?7)",
                rusqlite::params![
                    &persisted_id,
                    normalized_data.name,
                    normalized_data.code,
                    normalized_data.description,
                    normalized_data.env,
                    normalized_data.status,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建业务应用", e))?;
            sync_business_application_owner_contacts(
                command,
                conn,
                &persisted_id,
                &normalized_owner_contact_ids,
                &now,
            )?;
            write_audit_log_entry(
                command,
                conn,
                "create",
                "business_application",
                &persisted_id,
                Some(&normalized_data.name),
            )?;
            Ok(persisted_id.clone())
        } else {
            conn.execute(
                "UPDATE business_applications
                 SET name = ?1, code = ?2, description = ?3, env = ?4, status = ?5, updated_at = ?6
                 WHERE id = ?7 AND is_deleted = 0",
                rusqlite::params![
                    normalized_data.name,
                    normalized_data.code,
                    normalized_data.description,
                    normalized_data.env,
                    normalized_data.status,
                    now,
                    normalized_data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新业务应用", e))?;
            sync_business_application_owner_contacts(
                command,
                conn,
                &normalized_data.id,
                &normalized_owner_contact_ids,
                &now,
            )?;
            write_audit_log_entry(
                command,
                conn,
                "update",
                "business_application",
                &normalized_data.id,
                Some(&normalized_data.name),
            )?;
            Ok(normalized_data.id.clone())
        }
    })
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
    with_transaction(conn, command, |conn| {
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
    })
}

fn normalize_application_ids(application_ids: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for raw_id in application_ids {
        let id = raw_id.trim();
        if id.is_empty() {
            continue;
        }
        if seen.insert(id.to_string()) {
            normalized.push(id.to_string());
        }
    }

    normalized
}

fn replace_services_by_business_application_inner(
    command: &str,
    conn: &rusqlite::Connection,
    business_application_id: &str,
    application_ids: &[String],
) -> AppResult<ReplaceServicesResult> {
    validate_required(business_application_id, "business_application_id")
        .map_err(|e| AppError::validation(command, e))?;
    let target_ids = normalize_application_ids(application_ids);

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

    let mut current_ids = HashSet::new();
    let mut current_order = Vec::new();
    let mut app_names: HashMap<String, String> = HashMap::new();
    let mut current_stmt = conn
        .prepare(
            "SELECT id, name
             FROM applications
             WHERE is_deleted = 0 AND business_application_id = ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| AppError::from_db_error(command, "查询当前挂载服务", e))?;
    let current_rows = current_stmt
        .query_map(rusqlite::params![business_application_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| AppError::from_db_error(command, "读取当前挂载服务", e))?;
    for row in current_rows {
        let (application_id, application_name) =
            row.map_err(|e| AppError::from_db_error(command, "读取当前挂载服务", e))?;
        current_ids.insert(application_id.clone());
        current_order.push(application_id.clone());
        app_names.insert(application_id, application_name);
    }

    for application_id in &target_ids {
        let (name, current_ba_id) = conn
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

        if let Some(current_id) = current_ba_id
            .as_deref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            if current_id != business_application_id {
                return Err(AppError::conflict(
                    command,
                    "服务已归属其他业务应用，请先解绑后再挂载。",
                    Some(format!(
                        "application_id={}, application_name={}, current_business_application_id={}",
                        application_id, name, current_id
                    )),
                ));
            }
        }

        app_names.insert(application_id.clone(), name);
    }

    let target_set: HashSet<String> = target_ids.iter().cloned().collect();
    let attach_ids: Vec<String> = target_ids
        .iter()
        .filter(|application_id| !current_ids.contains(application_id.as_str()))
        .cloned()
        .collect();
    let detach_ids: Vec<String> = current_order
        .into_iter()
        .filter(|application_id| !target_set.contains(application_id))
        .collect();
    let unchanged_count = current_ids.intersection(&target_set).count() as u64;

    let now = chrono::Utc::now().to_rfc3339();
    with_transaction(conn, command, |conn| {
        for application_id in &detach_ids {
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
                app_names.get(application_id).map(|value| value.as_str()),
                Some(&format!(
                    "unbind business_application_id={}",
                    business_application_id
                )),
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        }

        for application_id in &attach_ids {
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
                app_names.get(application_id).map(|value| value.as_str()),
                Some(&format!(
                    "bind business_application_id={}",
                    business_application_id
                )),
            )
            .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        }

        Ok(ReplaceServicesResult {
            attached_count: attach_ids.len() as u64,
            detached_count: detach_ids.len() as u64,
            unchanged_count,
        })
    })
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
    with_transaction(conn, command, |conn| {
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
    })
}

#[tauri::command]
pub fn list_business_applications(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<BusinessApplication>> {
    let command = "list_business_applications";
    let conn = get_conn(pool.inner(), command)?;

    let (where_clause, sql_params) = build_business_applications_where_clause(&params);
    let total = count_query(
        command,
        "查询业务应用数量",
        &conn,
        "business_applications",
        &where_clause,
        &sql_params,
    )?;

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
    let mut data: Vec<BusinessApplication> = rows.filter_map(|row| row.ok()).collect();
    let business_ids: Vec<String> = data.iter().map(|item| item.id.clone()).collect();
    let owner_map = load_business_application_owner_contacts_map(&conn, &business_ids)
        .map_err(|e| AppError::from_db_error(command, "读取业务应用负责人标签", e))?;
    for item in &mut data {
        merge_business_application_owners(item, &owner_map);
    }

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
    let conn = get_conn(pool.inner(), command)?;
    let sql = format!(
        "SELECT {} FROM business_applications WHERE id = ?1 AND is_deleted = 0",
        SELECT_BUSINESS_APPLICATION_COLUMNS
    );
    let mut business = conn
        .query_row(&sql, rusqlite::params![id], row_to_business_application)
        .map_err(|e| {
            AppError::not_found(command, "业务应用不存在或已删除。", Some(e.to_string()))
        })?;
    let owner_map = load_business_application_owner_contacts_map(&conn, &[business.id.clone()])
        .map_err(|e| AppError::from_db_error(command, "读取业务应用负责人标签", e))?;
    merge_business_application_owners(&mut business, &owner_map);
    Ok(business)
}

#[tauri::command]
pub fn save_business_application(
    pool: State<DbPool>,
    data: BusinessApplication,
) -> AppResult<String> {
    let command = "save_business_application";
    let conn = get_conn(pool.inner(), command)?;
    save_business_application_inner(command, &conn, data)
}

#[tauri::command]
pub fn delete_business_application(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "delete_business_application";
    let conn = get_conn(pool.inner(), command)?;
    let now = chrono::Utc::now().to_rfc3339();
    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM business_applications WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    with_transaction(&conn, command, |conn| {
        delete_by_id(command, "删除业务应用", &conn, "business_applications", &id)?;
        conn.execute(
            "DELETE FROM business_application_owner_contacts WHERE business_application_id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| AppError::from_db_error(command, "删除业务应用负责人绑定", e))?;
        conn.execute(
            "UPDATE applications
             SET business_application_id = NULL, updated_at = ?1
             WHERE business_application_id = ?2 AND is_deleted = 0",
            rusqlite::params![now, id],
        )
        .map_err(|e| AppError::from_db_error(command, "解除应用归属", e))?;
        write_audit_log_entry(
            command,
            conn,
            "delete",
            "business_application",
            &id,
            name.as_deref(),
        )?;
        Ok(())
    })
}

#[tauri::command]
pub fn list_unassigned_application_services(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Application>> {
    let command = "list_unassigned_application_services";
    let conn = get_conn(pool.inner(), command)?;

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
    let conn = get_conn(pool.inner(), command)?;
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
    let conn = get_conn(pool.inner(), command)?;
    detach_service_from_business_application_inner(
        command,
        &conn,
        &business_application_id,
        &application_id,
    )
}

#[tauri::command]
pub fn replace_services_by_business_application(
    pool: State<DbPool>,
    business_application_id: String,
    application_ids: Vec<String>,
) -> AppResult<ReplaceServicesResult> {
    let command = "replace_services_by_business_application";
    let conn = get_conn(pool.inner(), command)?;
    replace_services_by_business_application_inner(
        command,
        &conn,
        &business_application_id,
        &application_ids,
    )
}

#[tauri::command]
pub fn list_services_by_business_application(
    pool: State<DbPool>,
    business_application_id: String,
) -> AppResult<BusinessApplicationServices> {
    let command = "list_services_by_business_application";
    let conn = get_conn(pool.inner(), command)?;
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
        attach_services_to_business_application_inner, build_business_applications_where_clause,
        detach_service_from_business_application_inner,
        list_services_by_business_application_inner,
        replace_services_by_business_application_inner, save_business_application_inner,
        AttachServicesResult, ReplaceServicesResult,
    };
    use crate::error::AppErrorCode;
    use crate::models::business_application::BusinessApplication;
    use crate::models::common::QueryParams;
    use crate::test_helpers::{insert_test_application, setup_test_db};
    use std::collections::HashMap;

    fn make_business_application(name: &str) -> BusinessApplication {
        BusinessApplication {
            id: "".into(),
            name: name.into(),
            code: Some("PAY".into()),
            owner_contact_ids: None,
            owners: None,
            description: None,
            env: Some("prod".into()),
            status: "active".into(),
            created_at: "".into(),
            updated_at: "".into(),
        }
    }

    fn insert_business_application(conn: &rusqlite::Connection, id: &str, name: &str) {
        conn.execute(
            "INSERT INTO business_applications (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, 'PAY', NULL, 'prod', 'active', 0, NULL, datetime('now'), datetime('now'))",
            rusqlite::params![id, name],
        )
        .expect("insert business application");
    }

    #[test]
    fn build_business_applications_where_clause_should_support_owner_contact_filter_and_search() {
        let mut filters = HashMap::new();
        filters.insert("owner".to_string(), r#"["c-alice","c-bob"]"#.to_string());
        let params = QueryParams {
            search: Some("alice".into()),
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_business_applications_where_clause(&params);
        assert!(where_clause.contains("business_application_owner_contacts"));
        assert!(where_clause.contains("contacts"));
        assert!(!where_clause.contains("taxonomy_bindings"));
        assert!(sql_params.len() >= 8);
    }

    #[test]
    fn save_business_application_inner_should_create_update_and_sync_owner_contacts() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO contacts (id, name, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('c-alice', 'alice', NULL, NULL, NULL, 0, NULL, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("seed alice");
        conn.execute(
            "INSERT INTO contacts (id, name, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('c-bob', 'bob', NULL, NULL, NULL, 0, NULL, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("seed bob");

        let mut payload = make_business_application("支付中心");
        payload.owner_contact_ids = Some(vec!["c-alice".into()]);
        let created_id = save_business_application_inner("test", &conn, payload.clone())
            .expect("create");
        assert!(!created_id.is_empty());

        payload.id = created_id.clone();
        payload.name = "支付中心-新".into();
        payload.owner_contact_ids = Some(vec!["c-bob".into()]);
        let updated_id = save_business_application_inner("test", &conn, payload).expect("update");
        assert_eq!(updated_id, created_id);

        let owner_binding_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM business_application_owner_contacts WHERE business_application_id = ?1 AND is_deleted = 0",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("count owner bindings");
        assert_eq!(owner_binding_count, 1);

        let current_owner: String = conn
            .query_row(
                "SELECT contact_id FROM business_application_owner_contacts WHERE business_application_id = ?1 AND is_deleted = 0 LIMIT 1",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("query current owner contact id");
        assert_eq!(current_owner, "c-bob");
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

    #[test]
    fn replace_services_should_attach_new_services() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        insert_test_application(&conn, "app-b", "api-b", "prod");

        let result = replace_services_by_business_application_inner(
            "test",
            &conn,
            "ba-1",
            &["app-a".into(), "app-b".into()],
        )
        .expect("replace should pass");
        assert_eq!(
            result,
            ReplaceServicesResult {
                attached_count: 2,
                detached_count: 0,
                unchanged_count: 0
            }
        );
    }

    #[test]
    fn replace_services_should_detach_removed_services() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        insert_test_application(&conn, "app-b", "api-b", "prod");
        insert_test_application(&conn, "app-c", "api-c", "prod");
        conn.execute(
            "UPDATE applications SET business_application_id = 'ba-1' WHERE id IN ('app-a', 'app-b')",
            [],
        )
        .expect("seed bind");

        let result = replace_services_by_business_application_inner(
            "test",
            &conn,
            "ba-1",
            &["app-b".into(), "app-c".into()],
        )
        .expect("replace should pass");
        assert_eq!(
            result,
            ReplaceServicesResult {
                attached_count: 1,
                detached_count: 1,
                unchanged_count: 1
            }
        );

        let app_a_binding: Option<String> = conn
            .query_row(
                "SELECT business_application_id FROM applications WHERE id = 'app-a'",
                [],
                |row| row.get(0),
            )
            .expect("query app-a binding");
        let app_b_binding: Option<String> = conn
            .query_row(
                "SELECT business_application_id FROM applications WHERE id = 'app-b'",
                [],
                |row| row.get(0),
            )
            .expect("query app-b binding");
        let app_c_binding: Option<String> = conn
            .query_row(
                "SELECT business_application_id FROM applications WHERE id = 'app-c'",
                [],
                |row| row.get(0),
            )
            .expect("query app-c binding");

        assert!(app_a_binding.is_none());
        assert_eq!(app_b_binding.as_deref(), Some("ba-1"));
        assert_eq!(app_c_binding.as_deref(), Some("ba-1"));
    }

    #[test]
    fn replace_services_should_keep_unchanged_services() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        conn.execute(
            "UPDATE applications SET business_application_id = 'ba-1' WHERE id = 'app-a'",
            [],
        )
        .expect("seed bind");

        let result = replace_services_by_business_application_inner(
            "test",
            &conn,
            "ba-1",
            &["app-a".into(), "app-a".into()],
        )
        .expect("replace should pass");
        assert_eq!(
            result,
            ReplaceServicesResult {
                attached_count: 0,
                detached_count: 0,
                unchanged_count: 1
            }
        );
    }

    #[test]
    fn replace_services_should_reject_services_belonging_to_other_business_application() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_business_application(&conn, "ba-2", "订单中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        conn.execute(
            "UPDATE applications SET business_application_id = 'ba-2' WHERE id = 'app-a'",
            [],
        )
        .expect("seed bind");

        let err = replace_services_by_business_application_inner(
            "test",
            &conn,
            "ba-1",
            &["app-a".into()],
        )
        .expect_err("should reject");
        assert_eq!(err.code, AppErrorCode::Conflict);
    }

    #[test]
    fn replace_services_should_not_change_data_when_conflict_occurs() {
        let conn = setup_test_db();
        insert_business_application(&conn, "ba-1", "支付中心");
        insert_business_application(&conn, "ba-2", "订单中心");
        insert_test_application(&conn, "app-a", "web-a", "prod");
        insert_test_application(&conn, "app-b", "api-b", "prod");
        conn.execute(
            "UPDATE applications SET business_application_id = 'ba-2' WHERE id = 'app-b'",
            [],
        )
        .expect("seed bind");

        let err = replace_services_by_business_application_inner(
            "test",
            &conn,
            "ba-1",
            &["app-a".into(), "app-b".into()],
        )
        .expect_err("should reject");
        assert_eq!(err.code, AppErrorCode::Conflict);

        let app_a_binding: Option<String> = conn
            .query_row(
                "SELECT business_application_id FROM applications WHERE id = 'app-a'",
                [],
                |row| row.get(0),
            )
            .expect("query app-a binding");
        let app_b_binding: Option<String> = conn
            .query_row(
                "SELECT business_application_id FROM applications WHERE id = 'app-b'",
                [],
                |row| row.get(0),
            )
            .expect("query app-b binding");
        assert!(app_a_binding.is_none());
        assert_eq!(app_b_binding.as_deref(), Some("ba-2"));
    }
}
