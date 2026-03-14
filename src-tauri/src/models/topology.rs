use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TopologySnapshotV3Query {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologySnapshotMetaV3 {
    pub version: String,
    pub task_view: String,
    pub generated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
    pub node_count: u32,
    pub edge_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyLaneV3 {
    pub id: String,
    pub label: String,
    pub order: u8,
    pub node_count: u32,
    pub app_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyNodeV3 {
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
#[serde(rename_all = "camelCase")]
pub struct TopologyEdgeV3 {
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
#[serde(rename_all = "camelCase")]
pub struct TopologyEnvCountV3 {
    pub env: String,
    pub count: u32,
    pub app_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyKindCountV3 {
    pub kind: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyLegendStatsV3 {
    pub env_counts: Vec<TopologyEnvCountV3>,
    pub node_type_counts: Vec<TopologyKindCountV3>,
    pub edge_type_counts: Vec<TopologyKindCountV3>,
    pub service_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_node_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_env_edge_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyLayoutHintsV3 {
    pub lane_order: Vec<String>,
    pub default_collapsed_groups: Vec<String>,
    pub high_density_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologySnapshotV3 {
    pub meta: TopologySnapshotMetaV3,
    pub lanes: Vec<TopologyLaneV3>,
    pub nodes: Vec<TopologyNodeV3>,
    pub edges: Vec<TopologyEdgeV3>,
    pub legend_stats: TopologyLegendStatsV3,
    pub layout_hints: TopologyLayoutHintsV3,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_node: Option<TopologyNodeV3>,
    pub node_count: u32,
    pub edge_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyDrilldownV3Query {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyNeighborV3 {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyDrilldownV3 {
    pub node: TopologyNodeV3,
    pub upstream: Vec<TopologyNeighborV3>,
    pub downstream: Vec<TopologyNeighborV3>,
    pub inbound_edge_count: u32,
    pub outbound_edge_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TopologyTaskViewV3Query {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyTaskInsightV3 {
    pub kind: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edge_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyTaskViewV3 {
    pub task_view: String,
    pub snapshot: TopologySnapshotV3,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub insights: Vec<TopologyTaskInsightV3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<TopologySnapshotMetaV3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyTroubleshootReportV3Query {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TopologyTroubleshootReportSummaryV3 {
    pub abnormal_status_count: u32,
    pub deployment_count: u32,
    pub recent_audit_change_count: u32,
    pub inbound_dependency_count: u32,
    pub outbound_dependency_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyTroubleshootReportV3 {
    pub node_id: String,
    pub task_view: String,
    pub summary: TopologyTroubleshootReportSummaryV3,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub insights: Vec<TopologyTaskInsightV3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyPathsV3Query {
    pub source_id: String,
    pub target_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyPathItemV3 {
    pub node_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyPathsV3 {
    pub paths: Vec<TopologyPathItemV3>,
    pub truncated: bool,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyImpactV3Query {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyImpactV3 {
    pub affected_nodes: Vec<TopologyAffectedNodeV3>,
    pub total_count: u32,
    pub max_depth: u32,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyAffectedNodeV3 {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyEvidenceV3Query {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyEvidenceItemV3 {
    pub id: String,
    pub evidence_type: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyEvidenceV3 {
    pub items: Vec<TopologyEvidenceItemV3>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyLane {
    pub id: String,
    pub label: String,
    pub order: u8,
    pub node_count: u32,
    pub app_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyNode {
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
pub struct RuntimeTopologyEdge {
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
pub struct RuntimeTopologyEnvCount {
    pub env: String,
    pub count: u32,
    pub app_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyKindCount {
    pub kind: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyLegendStats {
    pub env_counts: Vec<RuntimeTopologyEnvCount>,
    pub node_type_counts: Vec<RuntimeTopologyKindCount>,
    pub edge_type_counts: Vec<RuntimeTopologyKindCount>,
    pub service_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyLayoutHints {
    pub lane_order: Vec<String>,
    pub default_collapsed_groups: Vec<String>,
    pub high_density_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyGraph {
    pub lanes: Vec<RuntimeTopologyLane>,
    pub nodes: Vec<RuntimeTopologyNode>,
    pub edges: Vec<RuntimeTopologyEdge>,
    pub legend_stats: RuntimeTopologyLegendStats,
    pub layout_hints: RuntimeTopologyLayoutHints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyPaths {
    pub paths: Vec<Vec<String>>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyImpactNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTopologyImpact {
    pub affected_nodes: Vec<RuntimeTopologyImpactNode>,
    pub total_count: u32,
    pub max_depth: u32,
}

pub type TopologySnapshotQuery = TopologySnapshotV3Query;
pub type TopologySnapshot = TopologySnapshotV3;
pub type TopologyDrilldownQuery = TopologyDrilldownV3Query;
pub type TopologyDrilldown = TopologyDrilldownV3;
pub type TopologyTaskViewQuery = TopologyTaskViewV3Query;
pub type TopologyTaskView = TopologyTaskViewV3;
pub type TopologyTaskInsight = TopologyTaskInsightV3;
pub type TopologyPathsQuery = TopologyPathsV3Query;
pub type TopologyPaths = TopologyPathsV3;
pub type TopologyPathItem = TopologyPathItemV3;
pub type TopologyImpactQuery = TopologyImpactV3Query;
pub type TopologyImpact = TopologyImpactV3;
pub type TopologyEvidenceQuery = TopologyEvidenceV3Query;
pub type TopologyEvidence = TopologyEvidenceV3;
pub type TopologyEvidenceItem = TopologyEvidenceItemV3;
