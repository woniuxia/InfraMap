use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NginxConfig {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub listen_port: Option<i64>,
    pub strategy: Option<String>,
    pub upstream_servers: Option<String>,
    pub env: String,
    pub status: String,
    pub description: Option<String>,
    #[serde(default)]
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
