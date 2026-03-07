use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSettings {
    pub id: String,
    pub auto_backup_enabled: bool,
    pub backup_interval_hours: i64,
    pub max_backups: i64,
    pub last_backup_time: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsInput {
    pub auto_backup_enabled: bool,
    pub backup_interval_hours: i64,
    pub max_backups: i64,
}

#[derive(Debug, Serialize)]
pub struct StorageProfile {
    pub active_root_path: String,
    pub db_path: String,
    pub backup_dir: String,
    pub is_default_path: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStoragePathInput {
    pub root_path: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateStoragePathResult {
    pub restart_required: bool,
    pub migrated: bool,
}

#[derive(Debug, Serialize)]
pub struct BackupEntry {
    pub filename: String,
    pub file_size: u64,
    pub created_at: String,
    pub is_auto: bool,
}

#[derive(Debug, Serialize)]
pub struct DbPreviewSummary {
    pub hosts: u64,
    pub applications: u64,
    pub middlewares: u64,
    pub nginx_configs: u64,
    pub deployments: u64,
    pub call_relations: u64,
    pub schema_version: i32,
    pub is_compatible: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub metadata: ExportMetadata,
    pub data: ExportPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub export_time: String,
    pub app_version: String,
    pub schema_version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportPayload {
    pub hosts: Vec<serde_json::Value>,
    pub applications: Vec<serde_json::Value>,
    pub middlewares: Vec<serde_json::Value>,
    pub nginx_configs: Vec<serde_json::Value>,
    pub deployments: Vec<serde_json::Value>,
    pub call_relations: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub hosts_imported: u64,
    pub applications_imported: u64,
    pub middlewares_imported: u64,
    pub nginx_configs_imported: u64,
    pub deployments_imported: u64,
    pub call_relations_imported: u64,
}
