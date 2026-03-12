use rusqlite::Connection;
use tauri::State;

use crate::backup;
use crate::commands::system_jobs::{insert_completed_system_job, CompletedSystemJob};
use crate::commands::taxonomy::rebuild_taxonomy_term_stats;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::snapshot::{
    SnapshotExportResult, SnapshotImportInput, SnapshotImportResult, SnapshotManifest, SnapshotPayload,
    SnapshotPreview, SnapshotTableCount, SnapshotTableData,
};
use crate::storage::StoragePaths;

const SNAPSHOT_FORMAT_VERSION: u32 = 2;
const SNAPSHOT_TABLES: [(&str, bool); 16] = [
    ("system_settings", false),
    ("hosts", true),
    ("ip_addresses", true),
    ("host_ip_bindings", true),
    ("business_applications", true),
    ("applications", true),
    ("contacts", true),
    ("application_owner_contacts", true),
    ("business_application_owner_contacts", true),
    ("middlewares", true),
    ("nginx_configs", true),
    ("deployments", true),
    ("call_relations", true),
    ("taxonomy_terms", true),
    ("taxonomy_bindings", true),
    ("taxonomy_term_stats", false),
];

fn current_schema_version(conn: &Connection) -> Result<i32, String> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
    .map_err(|err| format!("query schema version failed: {}", err))
}

fn read_table_rows(conn: &Connection, table: &str, soft_delete: bool) -> Result<Vec<serde_json::Value>, String> {
    let sql = if soft_delete {
        format!("SELECT * FROM {} WHERE is_deleted = 0", table)
    } else {
        format!("SELECT * FROM {}", table)
    };
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("prepare {} failed: {}", table, err))?;

    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|index| stmt.column_name(index).unwrap_or("").to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let mut map = serde_json::Map::new();
            for (index, name) in col_names.iter().enumerate() {
                let value = match row.get_ref(index) {
                    Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(inner)) => serde_json::Value::Number(inner.into()),
                    Ok(rusqlite::types::ValueRef::Real(inner)) => serde_json::Number::from_f64(inner)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    Ok(rusqlite::types::ValueRef::Text(inner)) => {
                        serde_json::Value::String(String::from_utf8_lossy(inner).to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(inner)) => serde_json::Value::String(
                        inner.iter().map(|byte| format!("{:02x}", byte)).collect::<String>(),
                    ),
                    Err(_) => serde_json::Value::Null,
                };
                map.insert(name.clone(), value);
            }
            Ok(serde_json::Value::Object(map))
        })
        .map_err(|err| format!("query {} failed: {}", table, err))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("read {} rows failed: {}", table, err))
}

fn count_table_rows(conn: &Connection, table: &str, soft_delete: bool) -> Result<u64, String> {
    let sql = if soft_delete {
        format!("SELECT COUNT(*) FROM {} WHERE is_deleted = 0", table)
    } else {
        format!("SELECT COUNT(*) FROM {}", table)
    };
    conn.query_row(&sql, [], |row| row.get::<_, u64>(0))
        .map_err(|err| format!("count {} failed: {}", table, err))
}

fn build_snapshot_payload(conn: &Connection) -> Result<(SnapshotPayload, Vec<SnapshotTableCount>, u64), String> {
    let schema_version = current_schema_version(conn)?;
    let manifest = SnapshotManifest {
        format_version: SNAPSHOT_FORMAT_VERSION,
        export_time: chrono::Utc::now().to_rfc3339(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
    };

    let mut tables = Vec::new();
    let mut counts = Vec::new();
    let mut total_rows = 0u64;
    for (table, soft_delete) in SNAPSHOT_TABLES {
        let rows = read_table_rows(conn, table, soft_delete)?;
        total_rows += rows.len() as u64;
        counts.push(SnapshotTableCount {
            table: table.to_string(),
            count: rows.len() as u64,
        });
        tables.push(SnapshotTableData {
            table: table.to_string(),
            rows,
        });
    }

    Ok((
        SnapshotPayload {
            manifest,
            tables,
        },
        counts,
        total_rows,
    ))
}

fn load_snapshot_payload(filepath: &str) -> Result<SnapshotPayload, String> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|err| format!("read snapshot file failed: {}", err))?;
    serde_json::from_str(&content).map_err(|err| format!("parse snapshot file failed: {}", err))
}

fn build_snapshot_preview(conn: &Connection, filepath: &str) -> Result<SnapshotPreview, String> {
    let payload = load_snapshot_payload(filepath)?;
    let schema_version = current_schema_version(conn)?;
    let mut warnings = Vec::new();
    if payload.manifest.format_version != SNAPSHOT_FORMAT_VERSION {
        warnings.push(format!(
            "快照格式版本 {} 与当前期望版本 {} 不一致。",
            payload.manifest.format_version, SNAPSHOT_FORMAT_VERSION
        ));
    }
    if payload.manifest.schema_version > schema_version {
        warnings.push(format!(
            "快照 schema 版本 {} 高于当前数据库版本 {}。",
            payload.manifest.schema_version, schema_version
        ));
    }
    if payload.manifest.app_version != env!("CARGO_PKG_VERSION") {
        warnings.push(format!(
            "快照来自应用版本 {}，当前应用版本为 {}。",
            payload.manifest.app_version,
            env!("CARGO_PKG_VERSION")
        ));
    }

    let mut current_counts = Vec::new();
    let mut total_rows = 0u64;
    for (table, soft_delete) in SNAPSHOT_TABLES {
        current_counts.push(SnapshotTableCount {
            table: table.to_string(),
            count: count_table_rows(conn, table, soft_delete)?,
        });
    }

    let snapshot_counts: Vec<SnapshotTableCount> = payload
        .tables
        .iter()
        .map(|table| SnapshotTableCount {
            table: table.table.clone(),
            count: table.rows.len() as u64,
        })
        .collect();
    for item in &snapshot_counts {
        total_rows += item.count;
    }

    Ok(SnapshotPreview {
        manifest: payload.manifest,
        snapshot_counts,
        current_counts,
        compatible: warnings.is_empty(),
        warnings,
        total_rows,
    })
}

fn import_table_rows(conn: &Connection, table: &str, rows: &[serde_json::Value]) -> Result<u64, String> {
    if rows.is_empty() {
        return Ok(0);
    }

    let first = rows
        .first()
        .and_then(|value| value.as_object())
        .ok_or_else(|| format!("{} rows must be objects", table))?;
    let columns: Vec<String> = first.keys().cloned().collect();
    let placeholders = (0..columns.len()).map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        columns.join(", "),
        placeholders
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("prepare import for {} failed: {}", table, err))?;

    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| format!("{} row must be object", table))?;
        let values = columns
            .iter()
            .map(|column| object.get(column).cloned().unwrap_or(serde_json::Value::Null))
            .collect::<Vec<_>>();
        let params = values
            .iter()
            .map(|value| match value {
                serde_json::Value::Null => rusqlite::types::Value::Null,
                serde_json::Value::Bool(inner) => rusqlite::types::Value::Integer(if *inner { 1 } else { 0 }),
                serde_json::Value::Number(inner) => {
                    if let Some(value) = inner.as_i64() {
                        rusqlite::types::Value::Integer(value)
                    } else if let Some(value) = inner.as_u64() {
                        rusqlite::types::Value::Integer(value as i64)
                    } else if let Some(value) = inner.as_f64() {
                        rusqlite::types::Value::Real(value)
                    } else {
                        rusqlite::types::Value::Null
                    }
                }
                serde_json::Value::String(inner) => rusqlite::types::Value::Text(inner.clone()),
                _ => rusqlite::types::Value::Text(value.to_string()),
            })
            .collect::<Vec<_>>();
        stmt.execute(rusqlite::params_from_iter(params))
            .map_err(|err| format!("insert into {} failed: {}", table, err))?;
    }

    Ok(rows.len() as u64)
}

fn import_snapshot_payload(
    conn: &Connection,
    payload: &SnapshotPayload,
) -> Result<(Vec<SnapshotTableCount>, u64), String> {
    let clear_tables = [
        "call_relations",
        "deployments",
        "host_ip_bindings",
        "application_owner_contacts",
        "business_application_owner_contacts",
        "contacts",
        "taxonomy_bindings",
        "taxonomy_term_stats",
        "taxonomy_terms",
        "applications",
        "middlewares",
        "nginx_configs",
        "business_applications",
        "ip_addresses",
        "hosts",
        "system_settings",
    ];
    for table in clear_tables {
        conn.execute(&format!("DELETE FROM {}", table), [])
            .map_err(|err| format!("clear {} failed: {}", table, err))?;
    }

    let import_order = [
        "system_settings",
        "hosts",
        "ip_addresses",
        "business_applications",
        "applications",
        "contacts",
        "application_owner_contacts",
        "business_application_owner_contacts",
        "middlewares",
        "nginx_configs",
        "host_ip_bindings",
        "deployments",
        "call_relations",
        "taxonomy_terms",
        "taxonomy_bindings",
        "taxonomy_term_stats",
    ];

    let mut counts = Vec::new();
    let mut total_rows = 0u64;
    for table in import_order {
        let rows = payload
            .tables
            .iter()
            .find(|item| item.table == table)
            .map(|item| item.rows.as_slice())
            .unwrap_or(&[]);
        let imported = import_table_rows(conn, table, rows)?;
        total_rows += imported;
        counts.push(SnapshotTableCount {
            table: table.to_string(),
            count: imported,
        });
    }

    rebuild_taxonomy_term_stats(conn, &chrono::Utc::now().to_rfc3339())
        .map_err(|err| format!("rebuild taxonomy stats failed: {}", err))?;

    // 兼容旧快照：schema_version < 23 的数据没有 contacts/owner_contacts 表，需从 legacy owner taxonomy 回填。
    let has_contacts_tables = payload.tables.iter().any(|item| item.table == "contacts")
        && payload
            .tables
            .iter()
            .any(|item| item.table == "application_owner_contacts")
        && payload
            .tables
            .iter()
            .any(|item| item.table == "business_application_owner_contacts");
    if payload.manifest.schema_version < 23 && !has_contacts_tables {
        backfill_contacts_and_owner_contacts_from_taxonomy(conn)?;
    }

    Ok((counts, total_rows))
}

fn backfill_contacts_and_owner_contacts_from_taxonomy(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        INSERT OR IGNORE INTO contacts (id, name, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
        SELECT tt.id, tt.display_name, NULL, NULL, NULL, 0, NULL,
               COALESCE(tt.created_at, datetime('now')),
               COALESCE(tt.updated_at, datetime('now'))
          FROM taxonomy_terms tt
         WHERE tt.is_deleted = 0
           AND tt.field_key = 'owner'
           AND TRIM(COALESCE(tt.display_name, '')) <> '';

        INSERT OR IGNORE INTO application_owner_contacts (id, application_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tb.resource_id, tb.term_id, 0, NULL,
               COALESCE(tb.created_at, datetime('now')),
               COALESCE(tb.updated_at, datetime('now'))
          FROM taxonomy_bindings tb
          JOIN taxonomy_terms tt
            ON tt.id = tb.term_id
           AND tt.is_deleted = 0
           AND tt.field_key = 'owner'
          JOIN applications a
            ON a.id = tb.resource_id
           AND a.is_deleted = 0
         WHERE tb.resource_type = 'application'
           AND tb.is_deleted = 0;

        INSERT OR IGNORE INTO business_application_owner_contacts (id, business_application_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tb.resource_id, tb.term_id, 0, NULL,
               COALESCE(tb.created_at, datetime('now')),
               COALESCE(tb.updated_at, datetime('now'))
          FROM taxonomy_bindings tb
          JOIN taxonomy_terms tt
            ON tt.id = tb.term_id
           AND tt.is_deleted = 0
           AND tt.field_key = 'owner'
          JOIN business_applications ba
            ON ba.id = tb.resource_id
           AND ba.is_deleted = 0
         WHERE tb.resource_type = 'business_application'
           AND tb.is_deleted = 0;
        "#,
    )
    .map_err(|err| format!("backfill contacts from taxonomy failed: {}", err))?;

    Ok(())
}

pub(crate) fn preview_snapshot_v2_inner(
    command: &str,
    conn: &Connection,
    filepath: &str,
) -> AppResult<SnapshotPreview> {
    build_snapshot_preview(conn, filepath)
        .map_err(|err| AppError::validation(command, format!("快照预检失败: {}", err)))
}

#[tauri::command]
pub fn preview_snapshot_v2(pool: State<DbPool>, filepath: String) -> AppResult<SnapshotPreview> {
    let command = "preview_snapshot_v2";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    preview_snapshot_v2_inner(command, &conn, &filepath)
}

#[tauri::command]
pub fn export_snapshot_v2(pool: State<DbPool>, filepath: String) -> AppResult<SnapshotExportResult> {
    let command = "export_snapshot_v2";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    let (payload, table_counts, total_rows) = build_snapshot_payload(&conn)
        .map_err(|err| AppError::from_db_error(command, "读取快照数据", err))?;

    let json = serde_json::to_string_pretty(&payload)
        .map_err(|err| AppError::internal(command, err.to_string()))?;
    std::fs::write(&filepath, json)
        .map_err(|err| AppError::from_io_error(command, "写入快照文件", err))?;

    let job_id = insert_completed_system_job(
        &conn,
        CompletedSystemJob {
            job_type: "snapshot_export_v2".to_string(),
            title: "导出快照 V2".to_string(),
            status: "completed".to_string(),
            summary: Some(format!("导出 {} 行数据", total_rows)),
            progress_percent: 100.0,
            retryable: false,
            cancellable: false,
            payload: Some(serde_json::json!({ "filepath": filepath })),
            result: Some(serde_json::json!({
                "total_rows": total_rows,
                "table_counts": table_counts,
                "format_version": SNAPSHOT_FORMAT_VERSION,
            })),
            error_message: None,
        },
    )
    .map_err(|err| AppError::from_db_error(command, "记录快照导出任务", err))?;

    Ok(SnapshotExportResult {
        job_id: format!("system:{}", job_id),
        filepath,
        total_rows,
        table_counts,
    })
}

#[tauri::command]
pub fn import_snapshot_v2(
    pool: State<DbPool>,
    storage_paths: State<StoragePaths>,
    input: SnapshotImportInput,
) -> AppResult<SnapshotImportResult> {
    let command = "import_snapshot_v2";
    let preview = {
        let conn = pool
            .get()
            .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
        preview_snapshot_v2_inner(command, &conn, &input.filepath)?
    };
    if !preview.compatible {
        return Err(AppError::validation(
            command,
            format!("快照预检未通过: {}", preview.warnings.join("；")),
        ));
    }

    let backup_filename = format!(
        "backup_pre_snapshot_import_{}.db",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let backup_path = storage_paths.backup_dir.join(&backup_filename);
    backup::perform_backup(&pool, &backup_path)
        .map_err(|err| AppError::from_backup_error(command, "创建导入前安全快照", err))?;

    let payload = load_snapshot_payload(&input.filepath)
        .map_err(|err| AppError::validation(command, format!("读取快照失败: {}", err)))?;
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|err| AppError::from_db_error(command, "开启快照导入事务", err))?;
    let result = (|| -> AppResult<(Vec<SnapshotTableCount>, u64)> {
        import_snapshot_payload(&conn, &payload)
            .map_err(|err| AppError::from_db_error(command, "导入快照数据", err))
    })();

    let (table_counts, total_rows) = match result {
        Ok(value) => {
            conn.execute_batch("COMMIT;")
                .map_err(|err| AppError::from_db_error(command, "提交快照导入事务", err))?;
            value
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK;");
            return Err(err);
        }
    };

    let job_id = insert_completed_system_job(
        &conn,
        CompletedSystemJob {
            job_type: "snapshot_import_v2".to_string(),
            title: "导入快照 V2".to_string(),
            status: "completed".to_string(),
            summary: Some(format!("导入 {} 行数据", total_rows)),
            progress_percent: 100.0,
            retryable: false,
            cancellable: false,
            payload: Some(serde_json::json!({
                "filepath": input.filepath,
                "backup_filename": backup_filename,
            })),
            result: Some(serde_json::json!({
                "total_rows": total_rows,
                "table_counts": table_counts,
            })),
            error_message: None,
        },
    )
    .map_err(|err| AppError::from_db_error(command, "记录快照导入任务", err))?;

    Ok(SnapshotImportResult {
        job_id: format!("system:{}", job_id),
        backup_filename,
        total_rows,
        table_counts,
    })
}

#[cfg(test)]
mod tests {
    use super::{import_snapshot_payload, preview_snapshot_v2_inner};
    use crate::commands::taxonomy::save_resource_terms;
    use crate::test_helpers::{insert_test_application, insert_test_host, setup_test_db};

    #[test]
    fn preview_snapshot_v2_inner_should_read_manifest_and_counts() {
        let conn = setup_test_db();
        insert_test_host(&conn, "host-1", "host-1", "10.0.0.1");
        let temp = std::env::temp_dir().join(format!("inframap-snapshot-preview-{}.json", uuid::Uuid::new_v4()));
        let payload = super::build_snapshot_payload(&conn).expect("build snapshot").0;
        std::fs::write(&temp, serde_json::to_string_pretty(&payload).expect("serialize snapshot"))
            .expect("write snapshot");

        let preview = preview_snapshot_v2_inner("test", &conn, temp.to_string_lossy().as_ref())
            .expect("preview snapshot");
        assert_eq!(preview.manifest.format_version, 2);
        assert!(preview.snapshot_counts.iter().any(|item| item.table == "hosts" && item.count == 1));

        let _ = std::fs::remove_file(temp);
    }

    #[test]
    fn import_snapshot_payload_should_restore_hosts_and_applications() {
        let conn = setup_test_db();
        insert_test_host(&conn, "host-1", "host-1", "10.0.0.1");
        insert_test_application(&conn, "app-1", "orders-api", "prod");
        let payload = super::build_snapshot_payload(&conn).expect("build snapshot").0;

        conn.execute("DELETE FROM applications", []).expect("clear applications");
        conn.execute("DELETE FROM hosts", []).expect("clear hosts");
        conn.execute("DELETE FROM ip_addresses", []).expect("clear ips");
        conn.execute("DELETE FROM host_ip_bindings", []).expect("clear bindings");

        let (table_counts, total_rows) = import_snapshot_payload(&conn, &payload).expect("import payload");
        assert!(table_counts.iter().any(|item| item.table == "hosts" && item.count >= 1));
        assert!(total_rows >= 2);

        let host_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM hosts WHERE is_deleted = 0", [], |row| row.get(0))
            .expect("count hosts");
        let app_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM applications WHERE is_deleted = 0", [], |row| row.get(0))
            .expect("count applications");
        assert_eq!(host_count, 1);
        assert_eq!(app_count, 1);
    }

    #[test]
    fn import_snapshot_payload_should_backfill_contacts_when_missing() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-1", "orders-api", "prod");
        conn.execute(
            "INSERT INTO business_applications (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('ba-1', '支付中心', 'PAY', NULL, 'prod', 'active', 0, NULL, datetime('now'), datetime('now'))",
            [],
        )
        .expect("seed business application");

        let now = chrono::Utc::now().to_rfc3339();
        save_resource_terms(
            &conn,
            "application",
            "app-1",
            "owner",
            &vec!["alice".into()],
            &now,
        )
        .expect("seed application owner taxonomy");
        save_resource_terms(
            &conn,
            "business_application",
            "ba-1",
            "owner",
            &vec!["alice".into()],
            &now,
        )
        .expect("seed business application owner taxonomy");

        let mut payload = super::build_snapshot_payload(&conn).expect("build snapshot").0;
        payload.manifest.schema_version = 22;
        payload.tables.retain(|item| {
            item.table != "contacts"
                && item.table != "application_owner_contacts"
                && item.table != "business_application_owner_contacts"
        });

        import_snapshot_payload(&conn, &payload).expect("import payload");

        let contact_count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM contacts WHERE is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("count contacts");
        assert_eq!(contact_count, 1);

        let app_owner_count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM application_owner_contacts WHERE is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("count app owner contacts");
        assert_eq!(app_owner_count, 1);

        let business_owner_count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM business_application_owner_contacts WHERE is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("count business app owner contacts");
        assert_eq!(business_owner_count, 1);
    }
}

