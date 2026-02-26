use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub source_id: String,
    pub source_type: String,
    pub target_id: String,
    pub target_type: String,
    pub relation_type: String,
    pub description: Option<String>,
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
