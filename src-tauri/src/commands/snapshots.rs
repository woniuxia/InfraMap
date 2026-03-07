use rusqlite::{types::Value as SqlValue, Connection};
use serde_json::Value;
use std::path::Path;
use tauri::State;

use crate::backup;
use crate::commands::system_jobs::record_system_job;
use crate::commands::taxonomy::rebuild_taxonomy_term_stats;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::snapshot::{
    SnapshotExportResult, SnapshotImportInput, SnapshotImportResult, SnapshotManifest,
    SnapshotPayload, SnapshotPreview, SnapshotTableCount, SnapshotTableData,
};
use crate::storage::StoragePaths;

const SNAPSHOT_FORMAT_VERSION: u32 = 2;

// 仅导出业务快照表，`taxonomy_term_stats` 在导入后重建。
const SNAPSHOT_TABLES: [(&str, bool); 13] = [
    ("system_settings", false),
    ("hosts", true),
    ("ip_addresses", true),
    ("host_ip_bindings", true),
    ("business_applications", true),
    ("applications", true),
    ("application_owners", true),
    ("middlewares", true),
    ("nginx_configs", true),
    ("deployments", true),
    ("call_relations", true),
    ("taxonomy_terms", true),
    ("taxonomy_bindings", true),
];

const SNAPSHOT_CLEAR_TABLES: [&str; 14] = [
    "call_relations",
    "deployments",
    "host_ip_bindings",
    "application_owners",
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

fn read_schema_version(conn: &Connection) -> Result<i32, String> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
    .map_err(|err| format!("read schema version failed: {err}"))
}

fn count_table_rows(conn: &Connection, table: &str, soft_delete: bool) -> Result<u64, String> {
    let sql = if soft_delete {
        format!("SELECT COUNT(*) FROM {table} WHERE is_deleted = 0")
    } else {
        format!("SELECT COUNT(*) FROM {table}")
    };

    conn.query_row(&sql, [], |row| row.get::<_, u64>(0))
        .map_err(|err| format!("count {table} failed: {err}"))
}

fn read_table_rows(
    conn: &Connection,
    table: &str,
    soft_delete: bool,
) -> Result<Vec<Value>, String> {
    let sql = if soft_delete {
        format!("SELECT * FROM {table} WHERE is_deleted = 0")
    } else {
        format!("SELECT * FROM {table}")
    };
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("prepare {table} export failed: {err}"))?;

    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|index| stmt.column_name(index).unwrap_or("").to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let mut object = serde_json::Map::new();
            for (index, name) in column_names.iter().enumerate() {
                let value = match row.get_ref(index) {
                    Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(inner)) => Value::Number(inner.into()),
                    Ok(rusqlite::types::ValueRef::Real(inner)) => {
                        serde_json::Number::from_f64(inner)
                            .map(Value::Number)
                            .unwrap_or(Value::Null)
                    }
                    Ok(rusqlite::types::ValueRef::Text(inner)) => {
                        Value::String(String::from_utf8_lossy(inner).to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(inner)) => Value::String(
                        inner
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>(),
                    ),
                    Err(_) => Value::Null,
                };
                object.insert(name.clone(), value);
            }
            Ok(Value::Object(object))
        })
        .map_err(|err| format!("query {table} export failed: {err}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("read {table} export rows failed: {err}"))
}

fn build_snapshot_payload(
    conn: &Connection,
) -> Result<(SnapshotPayload, Vec<SnapshotTableCount>, u64), String> {
    let manifest = SnapshotManifest {
        format_version: SNAPSHOT_FORMAT_VERSION,
        export_time: chrono::Utc::now().to_rfc3339(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version: read_schema_version(conn)?,
    };

    let mut tables = Vec::new();
    let mut table_counts = Vec::new();
    let mut total_rows = 0u64;

    for (table, soft_delete) in SNAPSHOT_TABLES {
        let rows = read_table_rows(conn, table, soft_delete)?;
        let count = rows.len() as u64;
        total_rows += count;
        table_counts.push(SnapshotTableCount {
            table: table.to_string(),
            count,
        });
        tables.push(SnapshotTableData {
            table: table.to_string(),
            rows,
        });
    }

    Ok((
        SnapshotPayload { manifest, tables },
        table_counts,
        total_rows,
    ))
}

fn load_snapshot_payload(filepath: &str) -> Result<SnapshotPayload, String> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|err| format!("read snapshot file failed: {err}"))?;
    serde_json::from_str(&content).map_err(|err| format!("parse snapshot json failed: {err}"))
}

fn find_snapshot_table<'a>(
    payload: &'a SnapshotPayload,
    table: &str,
) -> Option<&'a SnapshotTableData> {
    payload.tables.iter().find(|item| item.table == table)
}

fn build_snapshot_preview(conn: &Connection, filepath: &str) -> Result<SnapshotPreview, String> {
    let payload = load_snapshot_payload(filepath)?;
    let current_schema_version = read_schema_version(conn)?;

    let mut warnings = Vec::new();
    let mut compatible = true;

    if payload.manifest.format_version != SNAPSHOT_FORMAT_VERSION {
        compatible = false;
        warnings.push(format!(
            "快照格式版本 {} 与当前支持版本 {} 不一致。",
            payload.manifest.format_version, SNAPSHOT_FORMAT_VERSION
        ));
    }
    if payload.manifest.schema_version > current_schema_version {
        compatible = false;
        warnings.push(format!(
            "快照 schema 版本 {} 高于当前数据库版本 {}。",
            payload.manifest.schema_version, current_schema_version
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
    let mut snapshot_counts = Vec::new();
    let mut total_rows = 0u64;

    for (table, soft_delete) in SNAPSHOT_TABLES {
        current_counts.push(SnapshotTableCount {
            table: table.to_string(),
            count: count_table_rows(conn, table, soft_delete)?,
        });

        let count = find_snapshot_table(&payload, table)
            .map(|item| item.rows.len() as u64)
            .unwrap_or(0);
        if find_snapshot_table(&payload, table).is_none() {
            compatible = false;
            warnings.push(format!("快照缺少必需表 {table}。"));
        }

        snapshot_counts.push(SnapshotTableCount {
            table: table.to_string(),
            count,
        });
        total_rows += count;
    }

    for extra in &payload.tables {
        if !SNAPSHOT_TABLES
            .iter()
            .any(|(table, _)| *table == extra.table)
        {
            warnings.push(format!(
                "检测到未识别的快照表 {}，导入时将忽略。",
                extra.table
            ));
        }
    }

    Ok(SnapshotPreview {
        manifest: payload.manifest,
        snapshot_counts,
        current_counts,
        compatible,
        warnings,
        total_rows,
    })
}

fn import_table_rows(conn: &Connection, table: &str, rows: &[Value]) -> Result<u64, String> {
    if rows.is_empty() {
        return Ok(0);
    }

    let first = rows
        .first()
        .and_then(|value| value.as_object())
        .ok_or_else(|| format!("{table} rows must be objects"))?;
    let columns: Vec<String> = first.keys().cloned().collect();
    let placeholders = (0..columns.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT INTO {table} ({}) VALUES ({placeholders})",
        columns.join(", ")
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("prepare import for {table} failed: {err}"))?;

    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| format!("{table} row must be object"))?;
        let values = columns
            .iter()
            .map(|column| object.get(column).cloned().unwrap_or(Value::Null))
            .map(|value| match value {
                Value::Null => SqlValue::Null,
                Value::Bool(inner) => SqlValue::Integer(if inner { 1 } else { 0 }),
                Value::Number(inner) => {
                    if let Some(value) = inner.as_i64() {
                        SqlValue::Integer(value)
                    } else if let Some(value) = inner.as_u64() {
                        SqlValue::Integer(value as i64)
                    } else if let Some(value) = inner.as_f64() {
                        SqlValue::Real(value)
                    } else {
                        SqlValue::Null
                    }
                }
                Value::String(inner) => SqlValue::Text(inner),
                other => SqlValue::Text(other.to_string()),
            })
            .collect::<Vec<_>>();

        stmt.execute(rusqlite::params_from_iter(values))
            .map_err(|err| format!("insert into {table} failed: {err}"))?;
    }

    Ok(rows.len() as u64)
}

fn import_snapshot_payload(
    conn: &Connection,
    payload: &SnapshotPayload,
) -> Result<(Vec<SnapshotTableCount>, u64), String> {
    for table in SNAPSHOT_CLEAR_TABLES {
        conn.execute(&format!("DELETE FROM {table}"), [])
            .map_err(|err| format!("clear {table} failed: {err}"))?;
    }

    let import_order = [
        "system_settings",
        "hosts",
        "ip_addresses",
        "business_applications",
        "applications",
        "application_owners",
        "middlewares",
        "nginx_configs",
        "host_ip_bindings",
        "deployments",
        "call_relations",
        "taxonomy_terms",
        "taxonomy_bindings",
    ];

    let mut table_counts = Vec::new();
    let mut total_rows = 0u64;

    for table in import_order {
        let rows = find_snapshot_table(payload, table)
            .map(|item| item.rows.as_slice())
            .unwrap_or(&[]);
        let imported = import_table_rows(conn, table, rows)?;
        total_rows += imported;
        table_counts.push(SnapshotTableCount {
            table: table.to_string(),
            count: imported,
        });
    }

    rebuild_taxonomy_term_stats(conn, &chrono::Utc::now().to_rfc3339())
        .map_err(|err| format!("rebuild taxonomy stats failed: {err}"))?;

    Ok((table_counts, total_rows))
}

pub(crate) fn preview_snapshot_v2_inner(
    command: &str,
    pool: &DbPool,
    filepath: &str,
) -> AppResult<SnapshotPreview> {
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {err}")))?;
    build_snapshot_preview(&conn, filepath)
        .map_err(|err| AppError::validation(command, format!("快照预检失败: {err}")))
}

pub(crate) fn export_snapshot_v2_inner(
    command: &str,
    pool: &DbPool,
    filepath: &str,
) -> AppResult<SnapshotExportResult> {
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {err}")))?;
    let (payload, table_counts, total_rows) = build_snapshot_payload(&conn)
        .map_err(|err| AppError::from_db_error(command, "读取快照数据", err))?;

    if let Some(parent) = Path::new(filepath).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| AppError::from_io_error(command, "创建快照目录", err))?;
    }

    let json = serde_json::to_string_pretty(&payload)
        .map_err(|err| AppError::from_backup_error(command, "序列化快照数据", err))?;
    std::fs::write(filepath, json)
        .map_err(|err| AppError::from_io_error(command, "写入快照文件", err))?;

    let mut result = SnapshotExportResult {
        job_id: String::new(),
        filepath: filepath.to_string(),
        total_rows,
        table_counts,
    };
    let payload_json = serde_json::json!({ "filepath": filepath });
    let result_json = serde_json::to_value(&result)
        .map_err(|err| AppError::from_backup_error(command, "序列化导出结果", err))?;
    let raw_job_id = record_system_job(
        &conn,
        "snapshot_export_v2",
        "导出快照 V2",
        "completed",
        Some(&format!("导出 {} 行业务数据", total_rows)),
        Some(&payload_json),
        Some(&result_json),
        None,
        false,
        false,
    )
    .map_err(|err| AppError::from_db_error(command, "记录快照导出任务", err))?;
    result.job_id = format!("system:{raw_job_id}");

    Ok(result)
}

pub(crate) fn import_snapshot_v2_inner(
    command: &str,
    pool: &DbPool,
    storage_paths: &StoragePaths,
    input: SnapshotImportInput,
) -> AppResult<SnapshotImportResult> {
    let preview = preview_snapshot_v2_inner(command, pool, &input.filepath)?;
    if !preview.compatible {
        return Err(AppError::validation(
            command,
            format!("快照预检未通过: {}", preview.warnings.join("；")),
        ));
    }

    let backup_dir = backup::get_backup_dir(&storage_paths.active_root_path)
        .map_err(|err| AppError::from_backup_error(command, "读取备份目录", err))?;
    let backup_filename = format!(
        "backup_pre_snapshot_import_{}.db",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let backup_path = backup_dir.join(&backup_filename);
    backup::perform_backup(pool, &backup_path)
        .map_err(|err| AppError::from_backup_error(command, "创建导入前安全备份", err))?;

    let payload = load_snapshot_payload(&input.filepath)
        .map_err(|err| AppError::validation(command, format!("读取快照失败: {err}")))?;
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {err}")))?;

    // 导入使用显式事务，任一表失败即整体回滚。
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

    let mut result = SnapshotImportResult {
        job_id: String::new(),
        backup_filename: backup_filename.clone(),
        total_rows,
        table_counts,
    };
    let payload_json = serde_json::json!({
        "filepath": input.filepath,
        "backup_filename": backup_filename,
    });
    let result_json = serde_json::to_value(&result)
        .map_err(|err| AppError::from_backup_error(command, "序列化导入结果", err))?;
    let raw_job_id = record_system_job(
        &conn,
        "snapshot_import_v2",
        "导入快照 V2",
        "completed",
        Some(&format!("导入 {} 行业务数据", total_rows)),
        Some(&payload_json),
        Some(&result_json),
        None,
        false,
        false,
    )
    .map_err(|err| AppError::from_db_error(command, "记录快照导入任务", err))?;
    result.job_id = format!("system:{raw_job_id}");

    Ok(result)
}

#[tauri::command]
pub fn preview_snapshot_v2(pool: State<DbPool>, filepath: String) -> AppResult<SnapshotPreview> {
    preview_snapshot_v2_inner("preview_snapshot_v2", &pool, &filepath)
}

#[tauri::command]
pub fn export_snapshot_v2(
    pool: State<DbPool>,
    filepath: String,
) -> AppResult<SnapshotExportResult> {
    export_snapshot_v2_inner("export_snapshot_v2", &pool, &filepath)
}

#[tauri::command]
pub fn import_snapshot_v2(
    pool: State<DbPool>,
    storage_paths: State<StoragePaths>,
    input: SnapshotImportInput,
) -> AppResult<SnapshotImportResult> {
    import_snapshot_v2_inner("import_snapshot_v2", &pool, &storage_paths, input)
}

#[cfg(test)]
mod tests {
    use super::{export_snapshot_v2_inner, import_snapshot_v2_inner, preview_snapshot_v2_inner};
    use crate::db::{init_db_pool, run_migrations};
    use crate::models::snapshot::{
        SnapshotImportInput, SnapshotManifest, SnapshotPayload, SnapshotTableData,
    };
    use crate::storage::StoragePaths;
    use crate::test_helpers::{insert_test_application, insert_test_host};

    fn make_storage_paths(root: &std::path::Path) -> StoragePaths {
        StoragePaths {
            active_root_path: root.to_path_buf(),
            db_path: root.join("inframap.db"),
            backup_dir: root.join("backups"),
            bootstrap_config_path: root.join("storage-bootstrap.json"),
            default_root_path: root.to_path_buf(),
        }
    }

    fn create_test_root(prefix: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("{prefix}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn export_and_preview_snapshot_v2_should_capture_selected_tables() {
        let root = create_test_root("inframap-snapshot-export");
        let db_path = root.join("inframap.db");
        let snapshot_path = root.join("snapshot-v2.json");
        let pool = init_db_pool(&db_path.to_string_lossy()).expect("init pool");
        run_migrations(&pool).expect("run migrations");

        let conn = pool.get().expect("get conn");
        insert_test_host(&conn, "host-1", "host-1", "10.0.0.11");
        insert_test_application(&conn, "app-1", "orders-api", "prod");
        drop(conn);

        let export =
            export_snapshot_v2_inner("test", &pool, snapshot_path.to_string_lossy().as_ref())
                .expect("export snapshot");
        let preview =
            preview_snapshot_v2_inner("test", &pool, snapshot_path.to_string_lossy().as_ref())
                .expect("preview snapshot");

        assert!(snapshot_path.exists());
        assert!(export.job_id.starts_with("system:"));
        assert!(export.total_rows >= 3);
        assert!(preview.compatible);
        assert!(preview.warnings.is_empty());
        assert!(preview
            .snapshot_counts
            .iter()
            .any(|item| item.table == "hosts" && item.count == 1));
        assert!(preview
            .snapshot_counts
            .iter()
            .any(|item| item.table == "applications" && item.count == 1));
        assert!(preview
            .snapshot_counts
            .iter()
            .any(|item| item.table == "host_ip_bindings" && item.count == 1));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn import_snapshot_v2_should_replace_existing_business_data() {
        let source_root = create_test_root("inframap-snapshot-source");
        let target_root = create_test_root("inframap-snapshot-target");

        let source_db = source_root.join("inframap.db");
        let source_file = source_root.join("snapshot-v2.json");
        let source_pool = init_db_pool(&source_db.to_string_lossy()).expect("init source pool");
        run_migrations(&source_pool).expect("run source migrations");

        let source_conn = source_pool.get().expect("get source conn");
        insert_test_host(&source_conn, "host-1", "host-source", "10.0.0.21");
        insert_test_application(&source_conn, "app-1", "orders-api", "prod");
        drop(source_conn);

        export_snapshot_v2_inner("test", &source_pool, source_file.to_string_lossy().as_ref())
            .expect("export source snapshot");

        let target_db = target_root.join("inframap.db");
        let target_pool = init_db_pool(&target_db.to_string_lossy()).expect("init target pool");
        run_migrations(&target_pool).expect("run target migrations");

        let target_conn = target_pool.get().expect("get target conn");
        insert_test_host(&target_conn, "host-old", "host-old", "10.0.0.31");
        insert_test_application(&target_conn, "app-old", "legacy-api", "prod");
        drop(target_conn);

        let result = import_snapshot_v2_inner(
            "test",
            &target_pool,
            &make_storage_paths(&target_root),
            SnapshotImportInput {
                filepath: source_file.to_string_lossy().to_string(),
            },
        )
        .expect("import snapshot");

        let target_conn = target_pool.get().expect("get target conn after import");
        let imported_count: i64 = target_conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE name = 'orders-api' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("query imported app");
        let legacy_count: i64 = target_conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE name = 'legacy-api' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("query legacy app");

        assert!(result.job_id.starts_with("system:"));
        assert!(result.total_rows >= 3);
        assert_eq!(imported_count, 1);
        assert_eq!(legacy_count, 0);
        assert!(target_root.join("backups").exists());

        let _ = std::fs::remove_dir_all(&source_root);
        let _ = std::fs::remove_dir_all(&target_root);
    }

    #[test]
    fn import_snapshot_v2_should_rollback_on_invalid_payload() {
        let target_root = create_test_root("inframap-snapshot-rollback");
        let target_db = target_root.join("inframap.db");
        let snapshot_file = target_root.join("invalid-snapshot-v2.json");
        let target_pool = init_db_pool(&target_db.to_string_lossy()).expect("init target pool");
        run_migrations(&target_pool).expect("run target migrations");

        let target_conn = target_pool.get().expect("get target conn");
        insert_test_host(&target_conn, "host-old", "host-old", "10.0.0.41");
        insert_test_application(&target_conn, "app-old", "legacy-api", "prod");
        let schema_version = super::read_schema_version(&target_conn).expect("read schema version");
        drop(target_conn);

        let now = chrono::Utc::now().to_rfc3339();
        let payload = SnapshotPayload {
            manifest: SnapshotManifest {
                format_version: 2,
                export_time: now.clone(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                schema_version,
            },
            tables: vec![
                SnapshotTableData {
                    table: "system_settings".to_string(),
                    rows: vec![serde_json::json!({
                        "id": "default",
                        "auto_backup_enabled": 0,
                        "backup_interval_hours": 24,
                        "max_backups": 10,
                        "last_backup_time": null,
                        "created_at": now,
                        "updated_at": chrono::Utc::now().to_rfc3339(),
                    })],
                },
                SnapshotTableData {
                    table: "hosts".to_string(),
                    rows: vec![serde_json::json!({
                        "id": "host-new",
                        "hostname": "host-new",
                        "env": "prod",
                        "status": "running",
                        "is_deleted": 0,
                        "deleted_at": null,
                        "created_at": chrono::Utc::now().to_rfc3339(),
                        "updated_at": chrono::Utc::now().to_rfc3339(),
                    })],
                },
                SnapshotTableData {
                    table: "ip_addresses".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "host_ip_bindings".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "business_applications".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "applications".to_string(),
                    rows: vec![serde_json::json!({
                        "bad_column": "boom"
                    })],
                },
                SnapshotTableData {
                    table: "application_owners".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "middlewares".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "nginx_configs".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "deployments".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "call_relations".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "taxonomy_terms".to_string(),
                    rows: vec![],
                },
                SnapshotTableData {
                    table: "taxonomy_bindings".to_string(),
                    rows: vec![],
                },
            ],
        };
        std::fs::write(
            &snapshot_file,
            serde_json::to_string_pretty(&payload).expect("serialize invalid snapshot"),
        )
        .expect("write invalid snapshot");

        let err = import_snapshot_v2_inner(
            "test",
            &target_pool,
            &make_storage_paths(&target_root),
            SnapshotImportInput {
                filepath: snapshot_file.to_string_lossy().to_string(),
            },
        )
        .expect_err("invalid import should fail");

        let target_conn = target_pool.get().expect("get target conn after rollback");
        let legacy_host_count: i64 = target_conn
            .query_row(
                "SELECT COUNT(*) FROM hosts WHERE hostname = 'host-old' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("query legacy host");
        let legacy_app_count: i64 = target_conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE name = 'legacy-api' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("query legacy app");
        let new_host_count: i64 = target_conn
            .query_row(
                "SELECT COUNT(*) FROM hosts WHERE hostname = 'host-new' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("query new host");

        assert_eq!(err.command, "test");
        assert_eq!(legacy_host_count, 1);
        assert_eq!(legacy_app_count, 1);
        assert_eq!(new_host_count, 0);

        let _ = std::fs::remove_dir_all(&target_root);
    }
}
