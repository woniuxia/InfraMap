use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessApplication {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    #[serde(default)]
    pub owners: Option<Vec<String>>,
    pub description: Option<String>,
    pub env: Option<String>,
    pub status: String,
    #[serde(default)]
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
