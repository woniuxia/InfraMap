use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemJobSummary {
    pub id: String,
    pub source: String,
    pub job_type: String,
    pub title: String,
    pub status: String,
    pub summary: Option<String>,
    pub progress_percent: f64,
    pub retryable: bool,
    pub cancellable: bool,
    pub created_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemJobDetail {
    pub summary: SystemJobSummary,
    pub payload: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub import_rows: Option<Vec<SystemJobImportRow>>,
    pub import_issues: Option<Vec<SystemJobImportIssue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemJobImportRow {
    pub row_no: u32,
    pub resource_type: String,
    pub name: String,
    pub env: String,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemJobImportIssue {
    pub row_no: u32,
    pub field_key: Option<String>,
    pub issue_type: String,
    pub code: String,
    pub message: String,
}
