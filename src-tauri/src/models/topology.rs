use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyLane {
    pub id: String,
    pub label: String,
    pub order: u8,
    pub node_count: u32,
    pub app_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNodeV2 {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub env: String,
    pub group_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_ip_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub importance: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdgeV2 {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub strength: u32,
    pub cross_env: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEnvCount {
    pub env: String,
    pub count: u32,
    pub app_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyKindCount {
    pub kind: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyLegendStats {
    pub env_counts: Vec<TopologyEnvCount>,
    pub node_type_counts: Vec<TopologyKindCount>,
    pub edge_type_counts: Vec<TopologyKindCount>,
    pub application_service_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyLayoutHints {
    pub lane_order: Vec<String>,
    pub default_collapsed_groups: Vec<String>,
    pub high_density_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyGraphV2 {
    pub lanes: Vec<TopologyLane>,
    pub nodes: Vec<TopologyNodeV2>,
    pub edges: Vec<TopologyEdgeV2>,
    pub legend_stats: TopologyLegendStats,
    pub layout_hints: TopologyLayoutHints,
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
