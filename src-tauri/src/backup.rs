use std::path::{Path, PathBuf};
use rusqlite::backup::Backup;
use crate::db::DbPool;

pub struct AppDataDir(pub PathBuf);

pub fn get_backup_dir(app_data_dir: &Path) -> Result<PathBuf, String> {
    let dir = app_data_dir.join("backups");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    Ok(dir)
}

pub fn perform_backup(pool: &DbPool, dest_path: &Path) -> Result<(), String> {
    let src_conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    let mut dst_conn = rusqlite::Connection::open(dest_path)
        .map_err(|e| format!("Failed to open backup destination: {}", e))?;
    let backup = Backup::new(&src_conn, &mut dst_conn)
        .map_err(|e| format!("Failed to initialize backup: {}", e))?;
    backup.run_to_completion(100, std::time::Duration::from_millis(50), None)
        .map_err(|e| format!("Backup failed: {}", e))?;
    Ok(())
}

pub fn enforce_max_backups(backup_dir: &Path, max_backups: usize) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(backup_dir)
        .map_err(|e| format!("Failed to read backup dir: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map(|ext| ext == "db").unwrap_or(false)
                && e.file_name().to_string_lossy().starts_with("backup_")
        })
        .collect();

    if entries.len() <= max_backups {
        return Ok(());
    }

    entries.sort_by_key(|e| {
        e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let to_remove = entries.len() - max_backups;
    for entry in entries.into_iter().take(to_remove) {
        let _ = std::fs::remove_file(entry.path());
    }

    Ok(())
}

pub fn read_backup_settings(pool: &DbPool) -> Result<(bool, i64, i64, Option<String>), String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    conn.query_row(
        "SELECT auto_backup_enabled, backup_interval_hours, max_backups, last_backup_time
         FROM system_settings WHERE id = 'default'",
        [],
        |row| {
            Ok((
                row.get::<_, i64>(0)? != 0,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        },
    )
    .map_err(|e| format!("Failed to read backup settings: {}", e))
}

pub fn start_auto_backup_thread(pool: DbPool, app_data_dir: PathBuf) {
    std::thread::spawn(move || {
        loop {
            let settings = match read_backup_settings(&pool) {
                Ok(s) => s,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                    continue;
                }
            };

            let (enabled, interval_hours, max_backups, last_backup_time) = settings;

            if enabled {
                let should_backup = match last_backup_time {
                    Some(ref t) => {
                        if let Ok(last) = chrono::DateTime::parse_from_rfc3339(t) {
                            let elapsed = chrono::Utc::now().signed_duration_since(last);
                            elapsed.num_hours() >= interval_hours
                        } else {
                            true
                        }
                    }
                    None => true,
                };

                if should_backup {
                    if let Ok(backup_dir) = get_backup_dir(&app_data_dir) {
                        let now = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                        let filename = format!("backup_auto_{}.db", now);
                        let dest = backup_dir.join(&filename);

                        if perform_backup(&pool, &dest).is_ok() {
                            let now_rfc = chrono::Utc::now().to_rfc3339();
                            if let Ok(conn) = pool.get() {
                                let _ = conn.execute(
                                    "UPDATE system_settings SET last_backup_time = ?1, updated_at = ?2 WHERE id = 'default'",
                                    rusqlite::params![now_rfc, now_rfc],
                                );
                            }
                            let _ = enforce_max_backups(&backup_dir, max_backups as usize);
                        }
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_enforce_max_backups_under_limit() {
        let dir = std::env::temp_dir().join("inframap_test_backup_under");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Create 3 backup files
        for i in 0..3 {
            fs::write(dir.join(format!("backup_{}.db", i)), "data").unwrap();
        }

        enforce_max_backups(&dir, 5).unwrap();

        // All 3 should remain
        let count = fs::read_dir(&dir).unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("backup_"))
            .count();
        assert_eq!(count, 3);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_enforce_max_backups_over_limit() {
        let dir = std::env::temp_dir().join("inframap_test_backup_over");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Create 5 backup files with slight time delay
        for i in 0..5 {
            fs::write(dir.join(format!("backup_{}.db", i)), format!("data{}", i)).unwrap();
            // Small sleep to ensure different modified times
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        enforce_max_backups(&dir, 3).unwrap();

        // Only 3 should remain
        let remaining: Vec<String> = fs::read_dir(&dir).unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("backup_"))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(remaining.len(), 3);

        let _ = fs::remove_dir_all(&dir);
    }
}
