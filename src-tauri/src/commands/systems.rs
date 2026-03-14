use std::collections::{HashMap, HashSet};
use tauri::State;

use crate::db::crud::{
    build_exists_like_clause, count_query, delete_by_id, parse_filter_values, resolve_upsert_state,
    write_audit_log_entry,
};
use crate::db::transaction::with_transaction;
use crate::db::{get_conn, DbPool};
use crate::error::{AppError, AppResult};
use crate::models::service::Service;
use crate::models::system::InfraSystem;
use crate::models::common::{PagedResult, QueryParams};
use crate::validation::{validate_system, validate_required};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemServices {
    pub frontend: Vec<Service>,
    pub backend: Vec<Service>,
}

fn row_to_system(row: &rusqlite::Row) -> rusqlite::Result<InfraSystem> {
    Ok(InfraSystem {
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

fn row_to_service_with_system(row: &rusqlite::Row) -> rusqlite::Result<Service> {
    Ok(Service {
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
        system_id: row.get(9)?,
        system_name: row.get(10)?,
        status: row.get(11)?,
        description: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

const SELECT_SYSTEM_COLUMNS: &str =
    "id, name, code, description, env, status, created_at, updated_at";
const SELECT_SERVICE_COLUMNS: &str = "services.id, services.name, services.type, services.address, \
     services.port, services.tech_stack, services.deploy_mode, services.env, services.git_repo, \
     services.system_id, \
     (SELECT s.name FROM systems s WHERE s.id = services.system_id AND s.is_deleted = 0) AS system_name, \
     services.status, services.description, services.created_at, services.updated_at";

fn build_systems_where_clause(
    params: &QueryParams,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions: Vec<String> = vec!["systems.is_deleted = 0".to_string()];
    let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(search) = params
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let like_value = format!("%{}%", search);
        let owner_name_search_clause = build_exists_like_clause(
            "system_owner_contacts soc JOIN contacts c ON c.id = soc.contact_id",
            &[
                "soc.system_id = systems.id",
                "soc.is_deleted = 0",
                "c.is_deleted = 0",
            ],
            "c.name",
        );
        let owner_phone_search_clause = build_exists_like_clause(
            "system_owner_contacts soc JOIN contacts c ON c.id = soc.contact_id",
            &[
                "soc.system_id = systems.id",
                "soc.is_deleted = 0",
                "c.is_deleted = 0",
            ],
            "c.phone",
        );
        let owner_email_search_clause = build_exists_like_clause(
            "system_owner_contacts soc JOIN contacts c ON c.id = soc.contact_id",
            &[
                "soc.system_id = systems.id",
                "soc.is_deleted = 0",
                "c.is_deleted = 0",
            ],
            "c.email",
        );

        conditions.push(
            format!(
                "(systems.name LIKE ? \
                  OR systems.code LIKE ? \
                  OR systems.description LIKE ? \
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
            ("systems.status", "status"),
            ("systems.env", "env"),
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
                        FROM system_owner_contacts soc
                        JOIN contacts c ON c.id = soc.contact_id
                        WHERE soc.system_id = systems.id
                          AND soc.is_deleted = 0
                          AND c.is_deleted = 0
                          AND soc.contact_id IN ({})
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

fn load_system_owner_contacts_map(
    conn: &rusqlite::Connection,
    system_ids: &[String],
) -> Result<HashMap<String, Vec<(String, String)>>, rusqlite::Error> {
    if system_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = vec!["?"; system_ids.len()].join(", ");
    let sql = format!(
        "SELECT soc.system_id, c.id, c.name
         FROM system_owner_contacts soc
         JOIN contacts c ON c.id = soc.contact_id
         WHERE soc.is_deleted = 0
           AND c.is_deleted = 0
           AND soc.system_id IN ({})
         ORDER BY c.name COLLATE NOCASE ASC, c.id ASC",
        placeholders
    );
    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&str> = system_ids.iter().map(String::as_str).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(param_refs), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for row in rows {
        let (system_id, contact_id, contact_name) = row?;
        map.entry(system_id)
            .or_default()
            .push((contact_id, contact_name));
    }
    Ok(map)
}

fn merge_system_owners(
    system: &mut InfraSystem,
    owner_map: &HashMap<String, Vec<(String, String)>>,
) {
    if let Some(items) = owner_map.get(&system.id) {
        system.owner_contact_ids = Some(items.iter().map(|(id, _)| id.clone()).collect());
        system.owners = Some(items.iter().map(|(_, name)| name.clone()).collect());
    } else {
        system.owner_contact_ids = Some(Vec::new());
        system.owners = Some(Vec::new());
    }
}

fn sync_system_owner_contacts(
    command: &str,
    conn: &rusqlite::Connection,
    system_id: &str,
    owner_contact_ids: &[String],
    now: &str,
) -> AppResult<()> {
    conn.execute(
        "DELETE FROM system_owner_contacts WHERE system_id = ?1",
        rusqlite::params![system_id],
    )
    .map_err(|e| AppError::from_db_error(command, "清理系统负责人绑定", e))?;

    if owner_contact_ids.is_empty() {
        return Ok(());
    }

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
            "INSERT INTO system_owner_contacts (id, system_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, NULL, ?4, ?4)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                system_id,
                contact_id,
                now
            ],
        )
        .map_err(|e| AppError::from_db_error(command, "创建系统负责人绑定", e))?;
    }

    Ok(())
}

fn save_system_inner(
    command: &str,
    conn: &rusqlite::Connection,
    data: InfraSystem,
) -> AppResult<String> {
    let normalized_owner_contact_ids = normalize_owner_contact_ids(data.owner_contact_ids.clone());
    let mut normalized_data = data.clone();
    normalized_data.owner_contact_ids = Some(normalized_owner_contact_ids.clone());

    validate_system(&normalized_data).map_err(|e| AppError::validation(command, e))?;

    let now = chrono::Utc::now().to_rfc3339();
    let (persisted_id, is_new) =
        resolve_upsert_state(command, conn, "systems", &normalized_data.id)?;

    with_transaction(conn, command, |conn| {
        if is_new {
            conn.execute(
                "INSERT INTO systems (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
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
            .map_err(|e| AppError::from_db_error(command, "创建系统", e))?;
            sync_system_owner_contacts(
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
                "system",
                &persisted_id,
                Some(&normalized_data.name),
            )?;
            Ok(persisted_id.clone())
        } else {
            // Check if name or env changed, cascade update to child services
            let (old_name, old_env): (String, String) = conn
                .query_row(
                    "SELECT name, env FROM systems WHERE id = ?1 AND is_deleted = 0",
                    rusqlite::params![normalized_data.id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|e| AppError::from_db_error(command, "查询系统旧数据", e))?;

            conn.execute(
                "UPDATE systems
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
            .map_err(|e| AppError::from_db_error(command, "更新系统", e))?;

            // Cascade update child services' name/env if changed
            if old_name != normalized_data.name || old_env != normalized_data.env {
                conn.execute(
                    "UPDATE services SET name = ?1, env = ?2, updated_at = ?3
                     WHERE system_id = ?4 AND is_deleted = 0",
                    rusqlite::params![normalized_data.name, normalized_data.env, now, normalized_data.id],
                )
                .map_err(|e| AppError::from_db_error(command, "级联更新子服务", e))?;
            }

            sync_system_owner_contacts(
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
                "system",
                &normalized_data.id,
                Some(&normalized_data.name),
            )?;
            Ok(normalized_data.id.clone())
        }
    })
}

#[tauri::command]
pub fn list_systems(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<InfraSystem>> {
    let command = "list_systems";
    let conn = get_conn(pool.inner(), command)?;

    let (where_clause, sql_params) = build_systems_where_clause(&params);
    let total = count_query(
        command,
        "查询系统数量",
        &conn,
        "systems",
        &where_clause,
        &sql_params,
    )?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let sql = format!(
        "SELECT {} FROM systems {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_SYSTEM_COLUMNS, where_clause
    );
    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|param| param.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询系统列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_system)
        .map_err(|e| AppError::from_db_error(command, "读取系统列表", e))?;
    let mut data: Vec<InfraSystem> = rows.filter_map(|row| row.ok()).collect();
    let system_ids: Vec<String> = data.iter().map(|item| item.id.clone()).collect();
    let owner_map = load_system_owner_contacts_map(&conn, &system_ids)
        .map_err(|e| AppError::from_db_error(command, "读取系统负责人标签", e))?;
    for item in &mut data {
        merge_system_owners(item, &owner_map);
    }

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_system(pool: State<DbPool>, id: String) -> AppResult<InfraSystem> {
    let command = "get_system";
    let conn = get_conn(pool.inner(), command)?;
    let sql = format!(
        "SELECT {} FROM systems WHERE id = ?1 AND is_deleted = 0",
        SELECT_SYSTEM_COLUMNS
    );
    let mut system = conn
        .query_row(&sql, rusqlite::params![id], row_to_system)
        .map_err(|e| {
            AppError::not_found(command, "系统不存在或已删除。", Some(e.to_string()))
        })?;
    let owner_map = load_system_owner_contacts_map(&conn, &[system.id.clone()])
        .map_err(|e| AppError::from_db_error(command, "读取系统负责人标签", e))?;
    merge_system_owners(&mut system, &owner_map);
    Ok(system)
}

#[tauri::command]
pub fn save_system(
    pool: State<DbPool>,
    data: InfraSystem,
) -> AppResult<String> {
    let command = "save_system";
    let conn = get_conn(pool.inner(), command)?;
    save_system_inner(command, &conn, data)
}

#[tauri::command]
pub fn delete_system(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "delete_system";
    let conn = get_conn(pool.inner(), command)?;
    let now = chrono::Utc::now().to_rfc3339();
    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM systems WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    with_transaction(&conn, command, |conn| {
        delete_by_id(command, "删除系统", &conn, "systems", &id)?;
        conn.execute(
            "DELETE FROM system_owner_contacts WHERE system_id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| AppError::from_db_error(command, "删除系统负责人绑定", e))?;
        // Cascade soft-delete child services
        conn.execute(
            "UPDATE services
             SET is_deleted = 1, deleted_at = ?1, updated_at = ?1
             WHERE system_id = ?2 AND is_deleted = 0",
            rusqlite::params![now, id],
        )
        .map_err(|e| AppError::from_db_error(command, "级联删除子服务", e))?;
        write_audit_log_entry(
            command,
            conn,
            "delete",
            "system",
            &id,
            name.as_deref(),
        )?;
        Ok(())
    })
}

#[tauri::command]
pub fn list_services_by_system(
    pool: State<DbPool>,
    system_id: String,
) -> AppResult<SystemServices> {
    let command = "list_services_by_system";
    let conn = get_conn(pool.inner(), command)?;
    list_services_by_system_inner(command, &conn, &system_id)
}

fn list_services_by_system_inner(
    command: &str,
    conn: &rusqlite::Connection,
    system_id: &str,
) -> AppResult<SystemServices> {
    validate_required(system_id, "system_id")
        .map_err(|e| AppError::validation(command, e))?;
    let sql = format!(
        "SELECT {} FROM services
         WHERE is_deleted = 0 AND system_id = ?1
         ORDER BY created_at DESC",
        SELECT_SERVICE_COLUMNS
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询系统服务", e))?;
    let rows = stmt
        .query_map(
            rusqlite::params![system_id],
            row_to_service_with_system,
        )
        .map_err(|e| AppError::from_db_error(command, "读取系统服务", e))?;
    let services: Vec<Service> = rows.filter_map(|row| row.ok()).collect();

    let mut frontend = Vec::new();
    let mut backend = Vec::new();
    for service in services {
        if service.app_type == "frontend" {
            frontend.push(service);
        } else {
            backend.push(service);
        }
    }

    Ok(SystemServices { frontend, backend })
}

#[cfg(test)]
mod tests {
    use super::{
        build_systems_where_clause, list_services_by_system_inner,
        save_system_inner, SystemServices,
    };
    use crate::error::AppErrorCode;
    use crate::models::system::InfraSystem;
    use crate::models::common::QueryParams;
    use crate::test_helpers::{insert_test_service, setup_test_db};
    use std::collections::HashMap;

    fn make_system(name: &str) -> InfraSystem {
        InfraSystem {
            id: "".into(),
            name: name.into(),
            code: Some("PAY".into()),
            owner_contact_ids: None,
            owners: None,
            description: None,
            env: "prod".into(),
            status: "active".into(),
            created_at: "".into(),
            updated_at: "".into(),
        }
    }

    fn insert_system(conn: &rusqlite::Connection, id: &str, name: &str) {
        conn.execute(
            "INSERT INTO systems (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, 'PAY', NULL, 'prod', 'active', 0, NULL, datetime('now'), datetime('now'))",
            rusqlite::params![id, name],
        )
        .expect("insert system");
    }

    #[test]
    fn build_systems_where_clause_should_support_owner_contact_filter_and_search() {
        let mut filters = HashMap::new();
        filters.insert("owner".to_string(), r#"["c-alice","c-bob"]"#.to_string());
        let params = QueryParams {
            search: Some("alice".into()),
            filters: Some(filters),
            ..Default::default()
        };

        let (where_clause, sql_params) = build_systems_where_clause(&params);
        assert!(where_clause.contains("system_owner_contacts"));
        assert!(where_clause.contains("contacts"));
        assert!(!where_clause.contains("taxonomy_bindings"));
        assert!(sql_params.len() >= 8);
    }

    #[test]
    fn save_system_inner_should_create_update_and_sync_owner_contacts() {
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

        let mut payload = make_system("支付中心");
        payload.owner_contact_ids = Some(vec!["c-alice".into()]);
        let created_id = save_system_inner("test", &conn, payload.clone())
            .expect("create");
        assert!(!created_id.is_empty());

        payload.id = created_id.clone();
        payload.name = "支付中心-新".into();
        payload.owner_contact_ids = Some(vec!["c-bob".into()]);
        let updated_id = save_system_inner("test", &conn, payload).expect("update");
        assert_eq!(updated_id, created_id);

        let owner_binding_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM system_owner_contacts WHERE system_id = ?1 AND is_deleted = 0",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("count owner bindings");
        assert_eq!(owner_binding_count, 1);

        let current_owner: String = conn
            .query_row(
                "SELECT contact_id FROM system_owner_contacts WHERE system_id = ?1 AND is_deleted = 0 LIMIT 1",
                rusqlite::params![created_id],
                |row| row.get(0),
            )
            .expect("query current owner contact id");
        assert_eq!(current_owner, "c-bob");
    }

    #[test]
    fn save_system_inner_should_cascade_update_child_services() {
        let conn = setup_test_db();
        let sys_id = save_system_inner("test", &conn, make_system("旧名称")).expect("create system");

        // Insert a child service manually
        conn.execute(
            "INSERT INTO services (id, name, type, env, system_id, status, is_deleted, created_at, updated_at)
             VALUES ('svc-1', '旧名称', 'backend', 'prod', ?1, 'running', 0, datetime('now'), datetime('now'))",
            rusqlite::params![sys_id],
        )
        .expect("insert child service");

        let mut updated = make_system("新名称");
        updated.id = sys_id.clone();
        updated.env = "test".into();
        save_system_inner("test", &conn, updated).expect("update system");

        let (svc_name, svc_env): (String, String) = conn
            .query_row(
                "SELECT name, env FROM services WHERE id = 'svc-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("query child service");
        assert_eq!(svc_name, "新名称");
        assert_eq!(svc_env, "test");
    }

    #[test]
    fn services_grouping_should_split_frontend_and_backend() {
        let conn = setup_test_db();
        insert_system(&conn, "sys-1", "支付中心");
        insert_test_service(&conn, "svc-a", "web-a", "prod", "sys-1");
        insert_test_service(&conn, "svc-b", "api-b", "prod", "sys-1");
        conn.execute(
            "UPDATE services SET type = 'frontend' WHERE id = 'svc-a'",
            [],
        )
        .expect("seed frontend");

        let services = list_services_by_system_inner("test", &conn, "sys-1")
            .expect("list services");

        assert_eq!(services.frontend.len(), 1);
        assert_eq!(services.backend.len(), 1);
    }
}
