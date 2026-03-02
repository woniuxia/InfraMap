use std::path::Path;

use tauri::{AppHandle, State};

use crate::backup;
use crate::db::audit::insert_audit_log;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::settings::{
    StorageProfile, SystemSettings, UpdateSettingsInput, UpdateStoragePathInput,
    UpdateStoragePathResult,
};
use crate::storage::{self, StoragePaths};

fn to_display_path(path: &Path) -> String {
    let raw = path.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{}", rest);
        }
        if let Some(rest) = raw.strip_prefix(r"\\?\") {
            return rest.to_string();
        }
    }

    raw
}

#[tauri::command]
pub fn get_settings(pool: State<DbPool>) -> AppResult<SystemSettings> {
    let command = "get_settings";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    conn.query_row(
        "SELECT id, auto_backup_enabled, backup_interval_hours, max_backups, last_backup_time, created_at, updated_at
         FROM system_settings WHERE id = 'default'",
        [],
        |row| {
            Ok(SystemSettings {
                id: row.get(0)?,
                auto_backup_enabled: row.get::<_, i64>(1)? != 0,
                backup_interval_hours: row.get(2)?,
                max_backups: row.get(3)?,
                last_backup_time: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| AppError::from_db_error(command, "读取系统设置", e))
}

#[tauri::command]
pub fn update_settings(pool: State<DbPool>, data: UpdateSettingsInput) -> AppResult<()> {
    let command = "update_settings";

    if data.backup_interval_hours < 1 || data.backup_interval_hours > 720 {
        return Err(AppError::validation(
            command,
            "backup_interval_hours must be within 1..=720",
        ));
    }
    if data.max_backups < 1 || data.max_backups > 100 {
        return Err(AppError::validation(
            command,
            "max_backups must be within 1..=100",
        ));
    }

    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let now = chrono::Utc::now().to_rfc3339();
    let enabled_int: i64 = if data.auto_backup_enabled { 1 } else { 0 };

    conn.execute(
        "UPDATE system_settings SET auto_backup_enabled = ?1, backup_interval_hours = ?2, max_backups = ?3, updated_at = ?4
         WHERE id = 'default'",
        rusqlite::params![enabled_int, data.backup_interval_hours, data.max_backups, now],
    )
    .map_err(|e| AppError::from_db_error(command, "更新系统设置", e))?;

    insert_audit_log(
        &conn,
        "update",
        "system_settings",
        "default",
        Some("系统设置"),
        None,
    )
    .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_storage_profile(storage_paths: State<StoragePaths>) -> AppResult<StorageProfile> {
    Ok(StorageProfile {
        active_root_path: to_display_path(&storage_paths.active_root_path),
        db_path: to_display_path(&storage_paths.db_path),
        backup_dir: to_display_path(&storage_paths.backup_dir),
        is_default_path: storage_paths.is_default_path(),
    })
}

#[tauri::command]
pub fn update_storage_path(
    pool: State<DbPool>,
    storage_paths: State<StoragePaths>,
    data: UpdateStoragePathInput,
) -> AppResult<UpdateStoragePathResult> {
    let command = "update_storage_path";
    let trimmed = data.root_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(command, "root_path is required"));
    }

    let target_root = storage::normalize_path(Path::new(trimmed))
        .map_err(|e| AppError::validation(command, e))?;
    let current_root = storage::normalize_path(&storage_paths.active_root_path)
        .map_err(|e| AppError::from_io_error(command, "读取当前存储路径", e))?;

    if target_root == current_root {
        return Ok(UpdateStoragePathResult {
            restart_required: false,
            migrated: false,
        });
    }

    storage::ensure_target_is_clean(&target_root).map_err(|e| AppError::validation(command, e))?;

    if let Err(e) = std::fs::create_dir_all(&target_root) {
        return Err(AppError::from_io_error(command, "创建目标目录", e));
    }

    let target_db = target_root.join(storage::DATABASE_FILE_NAME);
    if let Err(e) = backup::perform_backup(&pool, &target_db) {
        storage::cleanup_partial_migration(&target_root);
        return Err(AppError::from_backup_error(command, "迁移数据库", e));
    }

    if let Err(e) = storage::migrate_backup_files(&storage_paths, &target_root) {
        storage::cleanup_partial_migration(&target_root);
        return Err(AppError::from_io_error(command, "迁移数据目录", e));
    }

    if let Err(e) =
        storage::write_bootstrap_config(&storage_paths.bootstrap_config_path, Some(&target_root))
    {
        storage::cleanup_partial_migration(&target_root);
        return Err(AppError::from_io_error(command, "写入存储路径配置", e));
    }

    Ok(UpdateStoragePathResult {
        restart_required: true,
        migrated: true,
    })
}

#[tauri::command]
pub fn restart_app(app: AppHandle) -> AppResult<()> {
    app.restart();
}
