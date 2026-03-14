use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraSystem {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    #[serde(default)]
    pub owner_contact_ids: Option<Vec<String>>,
    #[serde(default)]
    pub owners: Option<Vec<String>>,
    pub description: Option<String>,
    pub env: String,
    pub status: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
