use crate::models::topology::{
    AffectedNode, TopologyEdgeV2, TopologyLane, TopologyLayoutHints, TopologyLegendStats,
    TopologyNodeV2,
};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_evidence: Option<bool>,
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
pub struct TopologySnapshotV3 {
    pub meta: TopologySnapshotMetaV3,
    pub lanes: Vec<TopologyLane>,
    pub nodes: Vec<TopologyNodeV2>,
    pub edges: Vec<TopologyEdgeV2>,
    pub legend_stats: TopologyLegendStats,
    pub layout_hints: TopologyLayoutHints,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_node: Option<TopologyNodeV2>,
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
    pub node: TopologyNodeV2,
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
    pub affected_nodes: Vec<AffectedNode>,
    pub total_count: u32,
    pub max_depth: u32,
    pub severity: String,
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
