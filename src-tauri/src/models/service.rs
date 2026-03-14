use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub app_type: String,
    pub address: Option<String>,
    pub port: Option<i64>,
    pub tech_stack: Option<String>,
    pub deploy_mode: Option<String>,
    pub env: String,
    pub git_repo: Option<String>,
    #[serde(default)]
    pub owner_contact_ids: Option<Vec<String>>,
    #[serde(default)]
    pub owners: Option<Vec<String>>,
    pub system_id: String,
    pub system_name: Option<String>,
    pub status: String,
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
