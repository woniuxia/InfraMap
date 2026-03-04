use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NginxEndpoint {
    pub host: String,
    pub port: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NginxConfig {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub endpoints: Vec<NginxEndpoint>,
    pub strategy: Option<String>,
    pub env: String,
    pub status: String,
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
