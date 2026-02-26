use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub resource_id: String,
    pub resource_type: String,
    pub host_id: String,
    pub port: Option<i64>,
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
