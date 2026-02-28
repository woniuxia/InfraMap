use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyCombo {
    pub id: String,
    pub label: String,
    pub ip: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyGraph {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
    pub combos: Vec<TopologyCombo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResult {
    pub paths: Vec<Vec<String>>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub affected_nodes: Vec<AffectedNode>,
    pub total_count: u32,
    pub max_depth: u32,
}
