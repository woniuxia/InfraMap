use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegritySummary {
    pub total: u64,
    pub critical: u64,
    pub warning: u64,
    pub info: u64,
    pub repairable: u64,
    pub generated_at: String,
    pub last_scanned_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityFinding {
    pub id: String,
    pub key: String,
    pub category: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub topology_node_id: Option<String>,
    pub target_route: String,
    pub target_filters: BTreeMap<String, String>,
    pub repair_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub job_id: Option<String>,
    pub summary: IntegritySummary,
    pub findings: Vec<IntegrityFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityRepairInput {
    pub finding_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityRepairResult {
    pub job_id: String,
    pub backup_filename: String,
    pub repaired_count: u64,
    pub skipped_count: u64,
    pub report: IntegrityReport,
}
