use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Clone)]
pub struct DashboardStats {
    pub host_total: u64,
    pub host_abnormal: u64,
    pub application_total: u64,
    pub application_abnormal: u64,
    pub middleware_total: u64,
    pub nginx_total: u64,
    pub nginx_abnormal: u64,
    pub deployment_total: u64,
    pub dependency_total: u64,
    pub env_distribution: Vec<EnvCount>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnvCount {
    pub env: String,
    pub count: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardOverview {
    pub totals: DashboardTotals,
    pub health: DashboardHealth,
    pub coverage: DashboardCoverage,
    pub env_distribution: Vec<EnvCount>,
    pub risk_items: Vec<DashboardRiskItem>,
    pub recent_changes: Vec<DashboardRecentChange>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardTotals {
    pub host_total: u64,
    pub host_abnormal: u64,
    pub application_total: u64,
    pub application_abnormal: u64,
    pub middleware_total: u64,
    pub nginx_total: u64,
    pub nginx_abnormal: u64,
    pub deployment_total: u64,
    pub dependency_total: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardHealth {
    pub abnormal_total: u64,
    pub abnormal_rate: f64,
    pub score: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardCoverage {
    pub deployable_total: u64,
    pub deployed_total: u64,
    pub undeployed_total: u64,
    pub deployment_coverage: f64,
    pub relatable_total: u64,
    pub related_total: u64,
    pub isolated_total: u64,
    pub relation_coverage: f64,
    pub undeployed_application_total: u64,
    pub undeployed_middleware_total: u64,
    pub undeployed_nginx_total: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardRiskItem {
    pub key: String,
    pub label: String,
    pub count: u64,
    pub severity: String,
    pub target_route: String,
    pub target_filters: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardRecentChange {
    pub id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub created_at: String,
}
