use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub hostname: String,
    pub ip_address: String,
    pub os_type: Option<String>,
    pub cpu_info: Option<String>,
    pub ram_gb: Option<i64>,
    pub disk_gb: Option<i64>,
    pub status: String,
    pub tags: Option<String>,
    pub description: Option<String>,
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
