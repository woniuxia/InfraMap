use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRelation {
    #[serde(default)]
    pub id: String,
    pub pair_key: String,
    pub owner_id: String,
    pub owner_type: String,
    pub peer_id: String,
    pub peer_type: String,
    pub direction: String,
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
    use super::CallRelation;

    #[test]
    fn call_relation_deserialize_should_allow_missing_audit_fields() {
        let payload = r#"{
            "id": "",
            "pair_key": "application:a|application:b|http_call",
            "owner_id": "app-a",
            "owner_type": "application",
            "peer_id": "app-b",
            "peer_type": "application",
            "direction": "upstream",
            "relation_type": "http_call"
        }"#;

        let parsed: Result<CallRelation, _> = serde_json::from_str(payload);
        assert!(parsed.is_ok());
    }
}
