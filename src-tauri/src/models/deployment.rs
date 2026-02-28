use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    #[serde(default)]
    pub id: String,
    pub resource_id: String,
    pub resource_type: String,
    pub host_id: String,
    pub port: Option<i64>,
    #[serde(default)]
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::Deployment;

    #[test]
    fn deployment_deserialize_should_allow_missing_audit_fields() {
        let payload = r#"{
            "id": "",
            "resource_id": "app-1",
            "resource_type": "application",
            "host_id": "host-1",
            "port": 8080
        }"#;

        let parsed: Result<Deployment, _> = serde_json::from_str(payload);
        assert!(parsed.is_ok());
    }
}
