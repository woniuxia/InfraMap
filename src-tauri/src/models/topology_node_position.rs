use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNodePosition {
    pub node_id: String,
    pub layout_type: String,
    pub focus_target: Option<String>,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveTopologyPositionsInput {
    pub layout_type: String,
    pub focus_target: Option<String>,
    pub positions: Vec<TopologyNodePositionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNodePositionEntry {
    pub node_id: String,
    pub x: f64,
    pub y: f64,
}
