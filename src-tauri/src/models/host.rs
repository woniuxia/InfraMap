use serde::{Deserialize, Serialize};

fn default_env() -> String {
    "prod".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    #[serde(default)]
    pub id: String,
    pub hostname: String,
    pub ip_display: Option<String>,
    #[serde(default = "default_env")]
    pub env: String,
    pub os_type: Option<String>,
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<i64>,
    pub cpu_threads: Option<i64>,
    pub cpu_freq: Option<String>,
    pub ram_gb: Option<i64>,
    pub disk_gb: Option<i64>,
    pub status: String,
    pub tags: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
