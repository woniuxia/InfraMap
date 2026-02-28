use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
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
    pub owner: Option<String>,
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
