use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{Manager, Runtime};

pub const DATABASE_FILE_NAME: &str = "inframap.db";
pub const BACKUP_DIR_NAME: &str = "backups";
pub const BOOTSTRAP_CONFIG_FILE_NAME: &str = "storage-bootstrap.json";
const BOOTSTRAP_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct StoragePaths {
    pub active_root_path: PathBuf,
    pub db_path: PathBuf,
    pub backup_dir: PathBuf,
    pub bootstrap_config_path: PathBuf,
    pub default_root_path: PathBuf,
}

impl StoragePaths {
    pub fn is_default_path(&self) -> bool {
        self.active_root_path == self.default_root_path
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageBootstrapConfig {
    version: u32,
    custom_root_path: Option<String>,
}

pub fn normalize_path(path: &Path) -> Result<PathBuf, String> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to read current directory: {}", e))?
            .join(path)
    };

    if abs.exists() {
        abs.canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {}", e))
    } else {
        Ok(abs)
    }
}

pub fn read_bootstrap_config(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read bootstrap config: {}", e))?;
    let config: StorageBootstrapConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse bootstrap config: {}", e))?;

    if config.version != BOOTSTRAP_VERSION {
        return Err(format!(
            "Unsupported bootstrap config version: {}",
            config.version
        ));
    }

    Ok(config.custom_root_path.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }))
}

pub fn write_bootstrap_config(path: &Path, custom_root: Option<&Path>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config = StorageBootstrapConfig {
        version: BOOTSTRAP_VERSION,
        custom_root_path: custom_root.map(|p| p.to_string_lossy().to_string()),
    };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize bootstrap config: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write bootstrap config: {}", e))
}

pub fn ensure_target_is_clean(target_root: &Path) -> Result<(), String> {
    if target_root.exists() && target_root.is_file() {
        return Err("Target path points to a file".to_string());
    }

    let target_db = target_root.join(DATABASE_FILE_NAME);
    if target_db.exists() {
        return Err(format!(
            "Target path already contains {}",
            DATABASE_FILE_NAME
        ));
    }

    let target_backup_dir = target_root.join(BACKUP_DIR_NAME);
    if target_backup_dir.exists() {
        return Err(format!("Target path already contains {}", BACKUP_DIR_NAME));
    }

    Ok(())
}

fn copy_dir_recursive(source_dir: &Path, target_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    let entries = std::fs::read_dir(source_dir)
        .map_err(|e| format!("Failed to read source directory: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let source_path = entry.path();
        let target_path = target_dir.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            std::fs::copy(&source_path, &target_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn migrate_storage(source: &StoragePaths, target_root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(target_root)
        .map_err(|e| format!("Failed to create target root directory: {}", e))?;

    if source.db_path.exists() {
        let target_db = target_root.join(DATABASE_FILE_NAME);
        std::fs::copy(&source.db_path, &target_db)
            .map_err(|e| format!("Failed to migrate database file: {}", e))?;
    }

    migrate_backup_files(source, target_root)
}

pub fn migrate_backup_files(source: &StoragePaths, target_root: &Path) -> Result<(), String> {
    if source.backup_dir.exists() {
        let target_backup_dir = target_root.join(BACKUP_DIR_NAME);
        copy_dir_recursive(&source.backup_dir, &target_backup_dir)?;
    }

    Ok(())
}

pub fn cleanup_partial_migration(target_root: &Path) {
    let _ = std::fs::remove_file(target_root.join(DATABASE_FILE_NAME));
    let _ = std::fs::remove_dir_all(target_root.join(BACKUP_DIR_NAME));
}

pub fn resolve_storage_paths<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<StoragePaths, String> {
    let default_root_raw = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;
    std::fs::create_dir_all(&default_root_raw)
        .map_err(|e| format!("Failed to create default app data directory: {}", e))?;
    let default_root_path = normalize_path(&default_root_raw)?;

    let app_config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to resolve app config directory: {}", e))?;
    std::fs::create_dir_all(&app_config_dir)
        .map_err(|e| format!("Failed to create app config directory: {}", e))?;

    let bootstrap_config_path = app_config_dir.join(BOOTSTRAP_CONFIG_FILE_NAME);
    let configured_root = read_bootstrap_config(&bootstrap_config_path)?;

    let active_root_path = match configured_root {
        Some(raw) => normalize_path(Path::new(&raw))?,
        None => default_root_path.clone(),
    };

    std::fs::create_dir_all(&active_root_path)
        .map_err(|e| format!("Failed to create active storage root: {}", e))?;
    let backup_dir = active_root_path.join(BACKUP_DIR_NAME);
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    let db_path = active_root_path.join(DATABASE_FILE_NAME);

    Ok(StoragePaths {
        active_root_path,
        db_path,
        backup_dir,
        bootstrap_config_path,
        default_root_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "inframap_storage_test_{}_{}",
            name,
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn bootstrap_config_should_roundtrip_custom_root_path() {
        let work_dir = unique_temp_dir("bootstrap_roundtrip");
        fs::create_dir_all(&work_dir).expect("create work dir");
        let bootstrap_path = work_dir.join(BOOTSTRAP_CONFIG_FILE_NAME);
        let custom_root = work_dir.join("custom_root");

        write_bootstrap_config(&bootstrap_path, Some(&custom_root)).expect("write bootstrap");
        let loaded = read_bootstrap_config(&bootstrap_path).expect("read bootstrap");

        assert_eq!(loaded, Some(custom_root.to_string_lossy().to_string()));
        let _ = fs::remove_dir_all(&work_dir);
    }

    #[test]
    fn ensure_target_is_clean_should_reject_existing_db_or_backups() {
        let work_dir = unique_temp_dir("target_conflict");
        fs::create_dir_all(&work_dir).expect("create work dir");

        let db_path = work_dir.join(DATABASE_FILE_NAME);
        fs::write(&db_path, "db").expect("write db");
        let db_err = ensure_target_is_clean(&work_dir).expect_err("should reject db conflict");
        assert!(db_err.contains(DATABASE_FILE_NAME));

        fs::remove_file(&db_path).expect("remove db");
        fs::create_dir_all(work_dir.join(BACKUP_DIR_NAME)).expect("create backup dir");
        let backup_err =
            ensure_target_is_clean(&work_dir).expect_err("should reject backup dir conflict");
        assert!(backup_err.contains(BACKUP_DIR_NAME));

        let _ = fs::remove_dir_all(&work_dir);
    }

    #[test]
    fn migrate_storage_should_copy_database_and_backups() {
        let source_root = unique_temp_dir("source");
        let target_root = unique_temp_dir("target");
        fs::create_dir_all(&source_root).expect("create source root");
        fs::create_dir_all(&target_root).expect("create target root");

        let source_db = source_root.join(DATABASE_FILE_NAME);
        fs::write(&source_db, "source-db").expect("write source db");
        let source_backup_dir = source_root.join(BACKUP_DIR_NAME);
        fs::create_dir_all(&source_backup_dir).expect("create source backups");
        fs::write(source_backup_dir.join("backup_manual_1.db"), "backup").expect("write backup");

        let source = StoragePaths {
            active_root_path: source_root.clone(),
            db_path: source_db,
            backup_dir: source_backup_dir,
            bootstrap_config_path: source_root.join(BOOTSTRAP_CONFIG_FILE_NAME),
            default_root_path: source_root.clone(),
        };

        migrate_storage(&source, &target_root).expect("migrate storage");

        assert!(target_root.join(DATABASE_FILE_NAME).exists());
        assert!(target_root
            .join(BACKUP_DIR_NAME)
            .join("backup_manual_1.db")
            .exists());

        let _ = fs::remove_dir_all(&source_root);
        let _ = fs::remove_dir_all(&target_root);
    }

    #[test]
    fn storage_paths_should_report_default_path_correctly() {
        let root = unique_temp_dir("default_flag");
        let paths = StoragePaths {
            active_root_path: root.clone(),
            db_path: root.join(DATABASE_FILE_NAME),
            backup_dir: root.join(BACKUP_DIR_NAME),
            bootstrap_config_path: root.join(BOOTSTRAP_CONFIG_FILE_NAME),
            default_root_path: root,
        };

        assert!(paths.is_default_path());
    }
}
