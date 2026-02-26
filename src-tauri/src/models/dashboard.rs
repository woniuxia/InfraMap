use serde::Serialize;

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct EnvCount {
    pub env: String,
    pub count: u64,
}
