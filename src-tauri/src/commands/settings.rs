use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::settings::{SystemSettings, UpdateSettingsInput};

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
