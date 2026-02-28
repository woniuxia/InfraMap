use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Middleware {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(rename = "type")]
    pub mw_type: String,
    pub address: String,
    pub port: Option<i64>,
    pub version: Option<String>,
    pub env: String,
    pub description: Option<String>,
    #[serde(default)]
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
