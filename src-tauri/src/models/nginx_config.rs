use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NginxConfig {
    pub id: String,
    pub name: String,
    pub listen_port: Option<i64>,
    pub strategy: Option<String>,
    pub upstream_servers: Option<String>,
    pub env: String,
    pub status: String,
    pub description: Option<String>,
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
