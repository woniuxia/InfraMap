use std::collections::{HashMap, HashSet};
use tauri::State;

use crate::backup;
use crate::commands::taxonomy::{
    parse_tech_stack_terms, rebuild_taxonomy_term_stats, save_resource_terms, FIELD_OWNER,
    FIELD_TECH_STACK,
};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::settings::{
    BackupEntry, DbPreviewSummary, ExportData, ExportMetadata, ExportPayload, ImportResult,
};
use crate::storage::StoragePaths;

#[tauri::command]
pub fn create_backup(pool: State<DbPool>, storage_paths: State<StoragePaths>) -> AppResult<String> {
    let command = "create_backup";

    let backup_dir = backup::get_backup_dir(&storage_paths.active_root_path)
        .map_err(|e| AppError::from_backup_error(command, "create backup directory", e))?;

    let now = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("backup_manual_{}.db", now);
    let dest = backup_dir.join(&filename);

    backup::perform_backup(&pool, &dest)
        .map_err(|e| AppError::from_backup_error(command, "perform backup", e))?;

    let now_rfc = chrono::Utc::now().to_rfc3339();
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    conn.execute(
        "UPDATE system_settings SET last_backup_time = ?1, updated_at = ?2 WHERE id = 'default'",
        rusqlite::params![now_rfc, now_rfc],
    )
    .map_err(|e| AppError::from_db_error(command, "update last backup time", e))?;

    let (_, _, max_backups, _) = backup::read_backup_settings(&pool)
        .map_err(|e| AppError::from_backup_error(command, "read backup settings", e))?;
    backup::enforce_max_backups(&backup_dir, max_backups as usize)
        .map_err(|e| AppError::from_backup_error(command, "cleanup expired backups", e))?;

    Ok(filename)
}

#[tauri::command]
pub fn list_backups(storage_paths: State<StoragePaths>) -> AppResult<Vec<BackupEntry>> {
    let command = "list_backups";

    let backup_dir = backup::get_backup_dir(&storage_paths.active_root_path)
        .map_err(|e| AppError::from_backup_error(command, "read backup directory", e))?;

    let mut entries: Vec<BackupEntry> = std::fs::read_dir(&backup_dir)
        .map_err(|e| AppError::from_io_error(command, "read backup directory", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("backup_") && name.ends_with(".db")
        })
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let meta = e.metadata().ok()?;
            let modified = meta.modified().ok()?;
            let datetime: chrono::DateTime<chrono::Utc> = modified.into();
            Some(BackupEntry {
                filename: name.clone(),
                file_size: meta.len(),
                created_at: datetime.to_rfc3339(),
                is_auto: name.starts_with("backup_auto_"),
            })
        })
        .collect();

    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(entries)
}

#[tauri::command]
pub fn delete_backup(storage_paths: State<StoragePaths>, filename: String) -> AppResult<()> {
    let command = "delete_backup";

    validate_filename(&filename).map_err(|e| AppError::validation(command, e))?;

    let backup_dir = backup::get_backup_dir(&storage_paths.active_root_path)
        .map_err(|e| AppError::from_backup_error(command, "read backup directory", e))?;
    let path = backup_dir.join(&filename);

    if !path.exists() {
        return Err(AppError::not_found(
            command,
            "backup file does not exist",
            Some(filename),
        ));
    }

    std::fs::remove_file(&path)
        .map_err(|e| AppError::from_io_error(command, "delete backup file", e))?;
    Ok(())
}

#[tauri::command]
pub fn preview_restore(
    storage_paths: State<StoragePaths>,
    filename: String,
) -> AppResult<DbPreviewSummary> {
    let command = "preview_restore";

    validate_filename(&filename).map_err(|e| AppError::validation(command, e))?;

    let backup_dir = backup::get_backup_dir(&storage_paths.active_root_path)
        .map_err(|e| AppError::from_backup_error(command, "read backup directory", e))?;
    let path = backup_dir.join(&filename);

    if !path.exists() {
        return Err(AppError::not_found(
            command,
            "backup file does not exist",
            Some(filename),
        ));
    }

    let conn =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| AppError::from_backup_error(command, "open backup file", e))?;

    let count = |table: &str| -> u64 {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM {} WHERE is_deleted = 0", table),
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64
    };

    let schema_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let current_max_version = crate::db::schema::MIGRATIONS
        .last()
        .map(|(v, _)| *v)
        .unwrap_or(0);

    Ok(DbPreviewSummary {
        hosts: count("hosts"),
        applications: count("applications"),
        middlewares: count("middlewares"),
        nginx_configs: count("nginx_configs"),
        deployments: count("deployments"),
        call_relations: count("call_relations"),
        schema_version,
        is_compatible: schema_version <= current_max_version,
    })
}

#[tauri::command]
pub fn restore_backup(
    pool: State<DbPool>,
    storage_paths: State<StoragePaths>,
    filename: String,
) -> AppResult<()> {
    let command = "restore_backup";

    validate_filename(&filename).map_err(|e| AppError::validation(command, e))?;

    let backup_dir = backup::get_backup_dir(&storage_paths.active_root_path)
        .map_err(|e| AppError::from_backup_error(command, "read backup directory", e))?;
    let source_path = backup_dir.join(&filename);

    if !source_path.exists() {
        return Err(AppError::not_found(
            command,
            "backup file does not exist",
            Some(filename),
        ));
    }

    let now = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let safety_filename = format!("backup_pre_restore_{}.db", now);
    let safety_path = backup_dir.join(&safety_filename);
    backup::perform_backup(&pool, &safety_path)
        .map_err(|e| AppError::from_backup_error(command, "create pre-restore backup", e))?;

    let db_path = storage_paths.db_path.clone();
    let src_conn = rusqlite::Connection::open_with_flags(
        &source_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| AppError::from_backup_error(command, "open backup source database", e))?;

    let mut dst_conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::from_backup_error(command, "open current database", e))?;

    let restore = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn)
        .map_err(|e| AppError::from_backup_error(command, "initialize restore flow", e))?;

    restore
        .run_to_completion(100, std::time::Duration::from_millis(50), None)
        .map_err(|e| AppError::from_backup_error(command, "restore database", e))?;

    Ok(())
}

#[tauri::command]
pub fn export_json(pool: State<DbPool>, filepath: String) -> AppResult<()> {
    let command = "export_json";

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let schema_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .map_err(|e| AppError::from_db_error(command, "read schema version", e))?;

    let tables = [
        "hosts",
        "applications",
        "middlewares",
        "nginx_configs",
        "deployments",
        "call_relations",
    ];
    let mut table_data: Vec<Vec<serde_json::Value>> = Vec::new();

    for table in &tables {
        let rows = read_table_rows(&conn, table)
            .map_err(|e| AppError::from_backup_error(command, "read export rows", e))?;
        table_data.push(rows);
    }

    let hosts = table_data.remove(0);
    let mut applications = table_data.remove(0);
    let owner_map = load_application_owner_map(&conn)
        .map_err(|e| AppError::from_backup_error(command, "load application owner terms", e))?;
    attach_application_owners(&mut applications, &owner_map);

    let export = ExportData {
        metadata: ExportMetadata {
            export_time: chrono::Utc::now().to_rfc3339(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version,
        },
        data: ExportPayload {
            hosts,
            applications,
            middlewares: table_data.remove(0),
            nginx_configs: table_data.remove(0),
            deployments: table_data.remove(0),
            call_relations: table_data.remove(0),
        },
    };

    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| AppError::from_backup_error(command, "serialize export payload", e))?;
    std::fs::write(&filepath, json)
        .map_err(|e| AppError::from_io_error(command, "write export file", e))?;

    Ok(())
}

#[tauri::command]
pub fn import_json(
    pool: State<DbPool>,
    storage_paths: State<StoragePaths>,
    filepath: String,
) -> AppResult<ImportResult> {
    let command = "import_json";

    let backup_dir = backup::get_backup_dir(&storage_paths.active_root_path)
        .map_err(|e| AppError::from_backup_error(command, "read backup directory", e))?;

    let now = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let safety_filename = format!("backup_pre_import_{}.db", now);
    let safety_path = backup_dir.join(&safety_filename);
    backup::perform_backup(&pool, &safety_path)
        .map_err(|e| AppError::from_backup_error(command, "create pre-import backup", e))?;

    let content = std::fs::read_to_string(&filepath)
        .map_err(|e| AppError::from_io_error(command, "read import file", e))?;

    let export: ExportData = serde_json::from_str(&content)
        .map_err(|e| AppError::validation(command, format!("invalid import json: {}", e)))?;

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "begin transaction", e))?;

    let result: AppResult<ImportResult> = (|| {
        let clear_tables = [
            "call_relations",
            "deployments",
            "nginx_configs",
            "middlewares",
            "applications",
            "hosts",
            "audit_logs",
        ];

        for table in &clear_tables {
            conn.execute(&format!("DELETE FROM {}", table), [])
                .map_err(|e| AppError::from_db_error(command, "clear existing data", e))?;
        }

        let hosts_count = import_table_rows(&conn, "hosts", &export.data.hosts)
            .map_err(|e| AppError::from_backup_error(command, "import hosts", e))?;
        conn.execute("DELETE FROM taxonomy_bindings WHERE resource_type = 'application'", [])
            .map_err(|e| AppError::from_db_error(command, "clear application taxonomy bindings", e))?;
        let (application_rows, application_terms) =
            split_application_rows_for_import(&export.data.applications)
                .map_err(|e| AppError::from_backup_error(command, "parse applications import payload", e))?;
        let apps_count = import_table_rows(&conn, "applications", &application_rows)
            .map_err(|e| AppError::from_backup_error(command, "import applications", e))?;
        sync_imported_application_terms(&conn, &application_terms)
            .map_err(|e| AppError::from_backup_error(command, "sync application taxonomy terms", e))?;
        let mw_count = import_table_rows(&conn, "middlewares", &export.data.middlewares)
            .map_err(|e| AppError::from_backup_error(command, "import middlewares", e))?;
        let nginx_count = import_table_rows(&conn, "nginx_configs", &export.data.nginx_configs)
            .map_err(|e| AppError::from_backup_error(command, "import nginx_configs", e))?;
        let dep_count = import_table_rows(&conn, "deployments", &export.data.deployments)
            .map_err(|e| AppError::from_backup_error(command, "import deployments", e))?;
        let deps_count = import_table_rows(&conn, "call_relations", &export.data.call_relations)
            .map_err(|e| AppError::from_backup_error(command, "import call_relations", e))?;
        let now = chrono::Utc::now().to_rfc3339();
        rebuild_taxonomy_term_stats(&conn, &now)
            .map_err(|e| AppError::from_db_error(command, "rebuild taxonomy stats", e))?;

        Ok(ImportResult {
            hosts_imported: hosts_count,
            applications_imported: apps_count,
            middlewares_imported: mw_count,
            nginx_configs_imported: nginx_count,
            deployments_imported: dep_count,
            call_relations_imported: deps_count,
        })
    })();

    match result {
        Ok(result) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "commit transaction", e))?;
            Ok(result)
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Invalid filename".into());
    }
    if !filename.ends_with(".db") {
        return Err("Invalid backup file extension".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_filename_valid() {
        assert!(validate_filename("backup_manual_20240101_120000.db").is_ok());
        assert!(validate_filename("backup_auto_20240101.db").is_ok());
    }

    #[test]
    fn test_validate_filename_path_traversal() {
        assert!(validate_filename("../../../etc/passwd.db").is_err());
        assert!(validate_filename("..\\windows\\system32.db").is_err());
        assert!(validate_filename("backup/../evil.db").is_err());
    }

    #[test]
    fn test_validate_filename_wrong_extension() {
        assert!(validate_filename("backup.txt").is_err());
        assert!(validate_filename("backup.exe").is_err());
        assert!(validate_filename("backup").is_err());
    }
}

fn read_table_rows(
    conn: &rusqlite::Connection,
    table: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let sql = format!("SELECT * FROM {} WHERE is_deleted = 0", table);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Prepare failed for {}: {}", table, e))?;

    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let mut map = serde_json::Map::new();
            for (i, name) in col_names.iter().enumerate() {
                let val = match row.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(v)) => {
                        serde_json::Value::Number(v.into())
                    }
                    Ok(rusqlite::types::ValueRef::Real(v)) => serde_json::Number::from_f64(v)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    Ok(rusqlite::types::ValueRef::Text(v)) => {
                        serde_json::Value::String(String::from_utf8_lossy(v).to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(v)) => {
                        serde_json::Value::String(base64_encode(v))
                    }
                    Err(_) => serde_json::Value::Null,
                };
                map.insert(name.clone(), val);
            }
            Ok(serde_json::Value::Object(map))
        })
        .map_err(|e| format!("Query failed for {}: {}", table, e))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("Row read error in {}: {}", table, e))?);
    }
    Ok(result)
}

fn load_application_owner_map(
    conn: &rusqlite::Connection,
) -> Result<HashMap<String, Vec<String>>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT tb.resource_id, tt.display_name
             FROM taxonomy_bindings tb
             JOIN taxonomy_terms tt ON tt.id = tb.term_id
             WHERE tb.is_deleted = 0
               AND tt.is_deleted = 0
               AND tb.resource_type = 'application'
               AND tt.field_key = 'owner'
             ORDER BY tb.resource_id ASC, tt.display_name ASC",
        )
        .map_err(|e| format!("Prepare failed for application owners: {}", e))?;

    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| format!("Query failed for application owners: {}", e))?;

    let mut owner_map: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (application_id, owner_name) =
            row.map_err(|e| format!("Row read error in application owners: {}", e))?;
        owner_map.entry(application_id).or_default().push(owner_name);
    }
    Ok(owner_map)
}

fn attach_application_owners(
    rows: &mut [serde_json::Value],
    owner_map: &HashMap<String, Vec<String>>,
) {
    for row in rows {
        let Some(object) = row.as_object_mut() else {
            continue;
        };
        let Some(application_id) = object.get("id").and_then(|value| value.as_str()) else {
            continue;
        };
        let owners = owner_map
            .get(application_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(serde_json::Value::String)
            .collect::<Vec<_>>();
        object.insert("owners".to_string(), serde_json::Value::Array(owners));
    }
}

#[derive(Debug, Default)]
struct ImportedApplicationTermSync {
    id: String,
    owners: Vec<String>,
    tech_stack_terms: Vec<String>,
}

fn split_application_rows_for_import(
    rows: &[serde_json::Value],
) -> Result<(Vec<serde_json::Value>, Vec<ImportedApplicationTermSync>), String> {
    let mut stripped_rows = Vec::with_capacity(rows.len());
    let mut term_sync = Vec::with_capacity(rows.len());

    for row in rows {
        let object = row
            .as_object()
            .cloned()
            .ok_or_else(|| "Invalid row data in applications".to_string())?;
        let application_id = object
            .get("id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Missing id in applications import row".to_string())?
            .to_string();
        let owners = normalize_owner_names(object.get("owners").cloned());
        let tech_stack_terms = object
            .get("tech_stack")
            .and_then(|value| value.as_str())
            .map(|value| parse_tech_stack_terms(Some(value)))
            .unwrap_or_default();

        stripped_rows.push(serde_json::Value::Object(object));
        term_sync.push(ImportedApplicationTermSync {
            id: application_id,
            owners,
            tech_stack_terms,
        });
    }

    Ok((stripped_rows, term_sync))
}

fn normalize_owner_names(value: Option<serde_json::Value>) -> Vec<String> {
    let Some(serde_json::Value::Array(items)) = value else {
        return Vec::new();
    };

    let mut owners = Vec::new();
    let mut seen = HashSet::new();
    for item in items {
        let Some(owner) = item.as_str() else {
            continue;
        };
        let trimmed = owner.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            owners.push(trimmed.to_string());
        }
    }
    owners
}

fn sync_imported_application_terms(
    conn: &rusqlite::Connection,
    items: &[ImportedApplicationTermSync],
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    for item in items {
        save_resource_terms(conn, "application", &item.id, FIELD_OWNER, &item.owners, &now)
            .map_err(|e| format!("Sync application owners failed: {}", e))?;
        save_resource_terms(
            conn,
            "application",
            &item.id,
            FIELD_TECH_STACK,
            &item.tech_stack_terms,
            &now,
        )
        .map_err(|e| format!("Sync application tech stacks failed: {}", e))?;
    }
    Ok(())
}

fn import_table_rows(
    conn: &rusqlite::Connection,
    table: &str,
    rows: &[serde_json::Value],
) -> Result<u64, String> {
    if rows.is_empty() {
        return Ok(0);
    }

    let first = rows[0]
        .as_object()
        .ok_or_else(|| format!("Invalid row data in {}", table))?;
    let columns: Vec<&String> = first.keys().collect();
    let col_list = columns
        .iter()
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders = columns
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table, col_list, placeholders
    );

    let mut count = 0u64;
    for row in rows {
        let obj = row
            .as_object()
            .ok_or_else(|| format!("Invalid row data in {}", table))?;

        let params: Vec<Box<dyn rusqlite::types::ToSql>> = columns
            .iter()
            .map(|col| match obj.get(*col) {
                Some(serde_json::Value::Null) | None => {
                    Box::new(Option::<String>::None) as Box<dyn rusqlite::types::ToSql>
                }
                Some(serde_json::Value::Number(n)) => {
                    if let Some(i) = n.as_i64() {
                        Box::new(i) as Box<dyn rusqlite::types::ToSql>
                    } else if let Some(f) = n.as_f64() {
                        Box::new(f) as Box<dyn rusqlite::types::ToSql>
                    } else {
                        Box::new(Option::<String>::None) as Box<dyn rusqlite::types::ToSql>
                    }
                }
                Some(serde_json::Value::String(s)) => {
                    Box::new(s.clone()) as Box<dyn rusqlite::types::ToSql>
                }
                Some(serde_json::Value::Bool(b)) => {
                    Box::new(if *b { 1i64 } else { 0i64 }) as Box<dyn rusqlite::types::ToSql>
                }
                Some(other) => Box::new(other.to_string()) as Box<dyn rusqlite::types::ToSql>,
            })
            .collect();

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|p| &**p as &dyn rusqlite::types::ToSql)
            .collect();

        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| format!("Insert failed in {}: {}", table, e))?;
        count += 1;
    }

    Ok(count)
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}
