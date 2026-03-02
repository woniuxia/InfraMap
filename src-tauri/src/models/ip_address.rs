use serde::{Deserialize, Serialize};

fn default_env() -> String {
    "prod".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddress {
    #[serde(default)]
    pub id: String,
    pub ip_address: String,
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default)]
    pub is_vip: bool,
    pub real_ips: Option<String>,
    pub tags: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
