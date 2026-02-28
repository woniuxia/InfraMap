use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    #[serde(default)]
    pub id: String,
    pub source_id: String,
    pub source_type: String,
    pub target_id: String,
    pub target_type: String,
    pub relation_type: String,
    pub description: Option<String>,
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
    use super::Dependency;

    #[test]
    fn dependency_deserialize_should_allow_missing_audit_fields() {
        let payload = r#"{
            "id": "",
            "source_id": "app-1",
            "source_type": "application",
            "target_id": "mw-1",
            "target_type": "middleware",
            "relation_type": "http_call"
        }"#;

        let parsed: Result<Dependency, _> = serde_json::from_str(payload);
        assert!(parsed.is_ok());
    }
}
