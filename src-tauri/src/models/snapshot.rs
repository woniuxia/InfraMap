use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub format_version: u32,
    pub export_time: String,
    pub app_version: String,
    pub schema_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotTableCount {
    pub table: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotPayload {
    pub manifest: SnapshotManifest,
    pub tables: Vec<SnapshotTableData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotTableData {
    pub table: String,
    pub rows: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotPreview {
    pub manifest: SnapshotManifest,
    pub snapshot_counts: Vec<SnapshotTableCount>,
    pub current_counts: Vec<SnapshotTableCount>,
    pub compatible: bool,
    pub warnings: Vec<String>,
    pub total_rows: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotExportResult {
    pub job_id: String,
    pub filepath: String,
    pub total_rows: u64,
    pub table_counts: Vec<SnapshotTableCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotImportInput {
    pub filepath: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotImportResult {
    pub job_id: String,
    pub backup_filename: String,
    pub total_rows: u64,
    pub table_counts: Vec<SnapshotTableCount>,
}
