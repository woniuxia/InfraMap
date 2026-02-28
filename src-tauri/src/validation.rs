use std::net::Ipv4Addr;

use crate::models::application::Application;
use crate::models::dependency::Dependency;
use crate::models::deployment::Deployment;
use crate::models::host::Host;
use crate::models::middleware::Middleware;
use crate::models::nginx_config::NginxConfig;

pub fn validate_required(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{} is required", field_name))
    } else {
        Ok(())
    }
}

pub fn validate_string_length(
    value: &str,
    min: usize,
    max: usize,
    field_name: &str,
) -> Result<(), String> {
    let len = value.chars().count();
    if len < min || len > max {
        Err(format!(
            "{} length must be {}-{}, got {}",
            field_name, min, max, len
        ))
    } else {
        Ok(())
    }
}

pub fn validate_ipv4(ip: &str) -> Result<(), String> {
    ip.parse::<Ipv4Addr>()
        .map(|_| ())
        .map_err(|_| format!("Invalid IPv4 address: {}", ip))
}

pub fn validate_port(port: i64) -> Result<(), String> {
    if (1..=65535).contains(&port) {
        Ok(())
    } else {
        Err(format!("Port must be 1-65535, got {}", port))
    }
}

pub fn validate_enum(value: &str, allowed: &[&str], field_name: &str) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "{} must be one of {:?}, got '{}'",
            field_name, allowed, value
        ))
    }
}

pub fn validate_json_array(value: &str, field_name: &str) -> Result<(), String> {
    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(value);
    parsed
        .map(|_| ())
        .map_err(|_| format!("{} must be a valid JSON array", field_name))
}

pub fn validate_positive_int(value: i64, field_name: &str) -> Result<(), String> {
    if value > 0 {
        Ok(())
    } else {
        Err(format!("{} must be a positive integer", field_name))
    }
}

// --- Entity-specific validation ---

pub fn validate_host(host: &Host) -> Result<(), String> {
    validate_required(&host.hostname, "hostname")?;
    validate_string_length(&host.hostname, 1, 200, "hostname")?;
    validate_required(&host.ip_address, "ip_address")?;
    validate_ipv4(&host.ip_address)?;
    validate_enum(&host.env, &["prod", "dev", "test"], "env")?;
    validate_enum(
        &host.status,
        &["running", "stopped", "maintenance"],
        "status",
    )?;
    if let Some(ref tags) = host.tags {
        if !tags.is_empty() {
            validate_json_array(tags, "tags")?;
        }
    }
    if let Some(ram) = host.ram_gb {
        validate_positive_int(ram, "ram_gb")?;
    }
    if let Some(disk) = host.disk_gb {
        validate_positive_int(disk, "disk_gb")?;
    }
    if let Some(cores) = host.cpu_cores {
        validate_positive_int(cores, "cpu_cores")?;
    }
    if let Some(threads) = host.cpu_threads {
        validate_positive_int(threads, "cpu_threads")?;
    }
    Ok(())
}

pub fn validate_application(app: &Application) -> Result<(), String> {
    validate_required(&app.name, "name")?;
    validate_string_length(&app.name, 1, 200, "name")?;
    validate_enum(
        &app.app_type,
        &[
            "frontend",
            "backend",
            "gateway",
            "batch_job",
            "microservice",
            "other",
        ],
        "type",
    )?;
    validate_enum(&app.env, &["prod", "dev", "test"], "env")?;
    validate_enum(
        &app.status,
        &["running", "stopped", "maintenance"],
        "status",
    )?;
    if let Some(port) = app.port {
        validate_port(port)?;
    }
    Ok(())
}

pub fn validate_middleware(mw: &Middleware) -> Result<(), String> {
    validate_required(&mw.name, "name")?;
    validate_string_length(&mw.name, 1, 200, "name")?;
    validate_enum(
        &mw.category,
        &[
            "database",
            "message_queue",
            "cache",
            "search_engine",
            "config_center",
            "other",
        ],
        "category",
    )?;
    validate_required(&mw.mw_type, "type")?;
    validate_required(&mw.address, "address")?;
    validate_enum(&mw.env, &["prod", "dev", "test"], "env")?;
    if let Some(port) = mw.port {
        validate_port(port)?;
    }
    Ok(())
}

pub fn validate_nginx_config(nc: &NginxConfig) -> Result<(), String> {
    validate_required(&nc.name, "name")?;
    validate_string_length(&nc.name, 1, 200, "name")?;
    validate_enum(&nc.env, &["prod", "dev", "test"], "env")?;
    validate_enum(&nc.status, &["running", "stopped", "maintenance"], "status")?;
    if let Some(ref strategy) = nc.strategy {
        if !strategy.is_empty() {
            validate_enum(strategy, &["roundrobin", "ip_hash"], "strategy")?;
        }
    }
    if let Some(port) = nc.listen_port {
        validate_port(port)?;
    }
    if let Some(ref upstream) = nc.upstream_servers {
        if !upstream.is_empty() {
            validate_json_array(upstream, "upstream_servers")?;
        }
    }
    Ok(())
}

pub fn validate_deployment(dep: &Deployment) -> Result<(), String> {
    validate_required(&dep.resource_id, "resource_id")?;
    validate_enum(
        &dep.resource_type,
        &["application", "middleware", "nginx"],
        "resource_type",
    )?;
    validate_required(&dep.host_id, "host_id")?;
    if let Some(port) = dep.port {
        validate_port(port)?;
    }
    Ok(())
}

pub fn validate_dependency(dep: &Dependency) -> Result<(), String> {
    validate_required(&dep.source_id, "source_id")?;
    validate_required(&dep.source_type, "source_type")?;
    validate_required(&dep.target_id, "target_id")?;
    validate_required(&dep.target_type, "target_type")?;
    validate_enum(
        &dep.relation_type,
        &[
            "http_call",
            "tcp",
            "mq_produce",
            "mq_consume",
            "grpc_call",
            "db_query",
            "cache_access",
        ],
        "relation_type",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_required ---
    #[test]
    fn test_validate_required_empty_string() {
        assert!(validate_required("", "field").is_err());
    }

    #[test]
    fn test_validate_required_whitespace() {
        assert!(validate_required("   ", "field").is_err());
    }

    #[test]
    fn test_validate_required_valid() {
        assert!(validate_required("hello", "field").is_ok());
    }

    // --- validate_string_length ---
    #[test]
    fn test_validate_string_length_too_short() {
        assert!(validate_string_length("", 1, 10, "field").is_err());
    }

    #[test]
    fn test_validate_string_length_too_long() {
        let long = "a".repeat(201);
        assert!(validate_string_length(&long, 1, 200, "field").is_err());
    }

    #[test]
    fn test_validate_string_length_boundary() {
        assert!(validate_string_length("a", 1, 200, "field").is_ok());
        let exact = "a".repeat(200);
        assert!(validate_string_length(&exact, 1, 200, "field").is_ok());
    }

    // --- validate_ipv4 ---
    #[test]
    fn test_validate_ipv4_valid() {
        assert!(validate_ipv4("192.168.1.1").is_ok());
        assert!(validate_ipv4("0.0.0.0").is_ok());
        assert!(validate_ipv4("255.255.255.255").is_ok());
    }

    #[test]
    fn test_validate_ipv4_invalid() {
        assert!(validate_ipv4("999.999.999.999").is_err());
        assert!(validate_ipv4("abc").is_err());
        assert!(validate_ipv4("192.168.1").is_err());
        assert!(validate_ipv4("").is_err());
    }

    // --- validate_port ---
    #[test]
    fn test_validate_port_valid() {
        assert!(validate_port(1).is_ok());
        assert!(validate_port(80).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn test_validate_port_invalid() {
        assert!(validate_port(0).is_err());
        assert!(validate_port(65536).is_err());
        assert!(validate_port(-1).is_err());
    }

    // --- validate_enum ---
    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("running", &["running", "stopped"], "status").is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        assert!(validate_enum("unknown", &["running", "stopped"], "status").is_err());
    }

    // --- validate_json_array ---
    #[test]
    fn test_validate_json_array_valid() {
        assert!(validate_json_array(r#"["a","b"]"#, "tags").is_ok());
        assert!(validate_json_array("[]", "tags").is_ok());
    }

    #[test]
    fn test_validate_json_array_invalid() {
        assert!(validate_json_array(r#"{"key":"val"}"#, "tags").is_err());
        assert!(validate_json_array("not json", "tags").is_err());
    }

    // --- validate_positive_int ---
    #[test]
    fn test_validate_positive_int_valid() {
        assert!(validate_positive_int(1, "field").is_ok());
        assert!(validate_positive_int(100, "field").is_ok());
    }

    #[test]
    fn test_validate_positive_int_invalid() {
        assert!(validate_positive_int(0, "field").is_err());
        assert!(validate_positive_int(-5, "field").is_err());
    }

    // --- validate_host ---
    fn make_test_host() -> Host {
        Host {
            id: "h1".into(),
            hostname: "server1".into(),
            ip_address: "192.168.1.1".into(),
            env: "prod".into(),
            os_type: None,
            cpu_model: None,
            cpu_cores: None,
            cpu_threads: None,
            cpu_freq: None,
            ram_gb: None,
            disk_gb: None,
            status: "running".into(),
            tags: None,
            description: None,
            is_deleted: 0,
            deleted_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_validate_host_valid() {
        assert!(validate_host(&make_test_host()).is_ok());
    }

    #[test]
    fn test_validate_host_invalid_ip() {
        let mut host = make_test_host();
        host.ip_address = "invalid".into();
        assert!(validate_host(&host).is_err());
    }

    #[test]
    fn test_validate_host_invalid_status() {
        let mut host = make_test_host();
        host.status = "unknown".into();
        assert!(validate_host(&host).is_err());
    }

    #[test]
    fn test_validate_host_invalid_env() {
        let mut host = make_test_host();
        host.env = "staging".into();
        assert!(validate_host(&host).is_err());
    }

    #[test]
    fn test_validate_host_invalid_cpu_cores() {
        let mut host = make_test_host();
        host.cpu_cores = Some(0);
        assert!(validate_host(&host).is_err());
    }

    #[test]
    fn test_validate_host_invalid_cpu_threads() {
        let mut host = make_test_host();
        host.cpu_threads = Some(-1);
        assert!(validate_host(&host).is_err());
    }

    #[test]
    fn test_validate_host_valid_cpu_fields() {
        let mut host = make_test_host();
        host.cpu_model = Some("Intel Xeon E5-2680 v4".into());
        host.cpu_cores = Some(14);
        host.cpu_threads = Some(28);
        host.cpu_freq = Some("2.40 GHz".into());
        assert!(validate_host(&host).is_ok());
    }

    // --- validate_application ---
    fn make_test_app() -> Application {
        Application {
            id: "a1".into(),
            name: "app1".into(),
            app_type: "backend".into(),
            address: None,
            port: None,
            tech_stack: None,
            deploy_mode: None,
            env: "prod".into(),
            git_repo: None,
            owner: None,
            status: "running".into(),
            description: None,
            is_deleted: 0,
            deleted_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_validate_application_valid() {
        assert!(validate_application(&make_test_app()).is_ok());
    }

    #[test]
    fn test_validate_application_invalid_type() {
        let mut app = make_test_app();
        app.app_type = "invalid_type".into();
        assert!(validate_application(&app).is_err());
    }

    #[test]
    fn test_validate_application_invalid_env() {
        let mut app = make_test_app();
        app.env = "staging".into();
        assert!(validate_application(&app).is_err());
    }

    // --- validate_middleware ---
    fn make_test_middleware() -> Middleware {
        Middleware {
            id: "m1".into(),
            name: "redis-main".into(),
            category: "cache".into(),
            mw_type: "redis".into(),
            address: "127.0.0.1".into(),
            port: Some(6379),
            version: None,
            env: "prod".into(),
            description: None,
            is_deleted: 0,
            deleted_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_validate_middleware_valid() {
        assert!(validate_middleware(&make_test_middleware()).is_ok());
    }

    #[test]
    fn test_validate_middleware_invalid_category() {
        let mut mw = make_test_middleware();
        mw.category = "nosql".into();
        assert!(validate_middleware(&mw).is_err());
    }

    // --- validate_nginx_config ---
    fn make_test_nginx() -> NginxConfig {
        NginxConfig {
            id: "n1".into(),
            name: "nginx-main".into(),
            listen_port: Some(80),
            strategy: Some("roundrobin".into()),
            upstream_servers: Some(r#"["10.0.0.1:8080"]"#.into()),
            env: "prod".into(),
            status: "running".into(),
            description: None,
            is_deleted: 0,
            deleted_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_validate_nginx_config_valid() {
        assert!(validate_nginx_config(&make_test_nginx()).is_ok());
    }

    #[test]
    fn test_validate_nginx_config_invalid_strategy() {
        let mut nc = make_test_nginx();
        nc.strategy = Some("random".into());
        assert!(validate_nginx_config(&nc).is_err());
    }

    // --- validate_deployment ---
    fn make_test_deployment() -> Deployment {
        Deployment {
            id: "d1".into(),
            resource_id: "a1".into(),
            resource_type: "application".into(),
            host_id: "h1".into(),
            port: Some(8080),
            is_deleted: 0,
            deleted_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_validate_deployment_valid() {
        assert!(validate_deployment(&make_test_deployment()).is_ok());
    }

    #[test]
    fn test_validate_deployment_invalid_resource_type() {
        let mut dep = make_test_deployment();
        dep.resource_type = "unknown".into();
        assert!(validate_deployment(&dep).is_err());
    }

    // --- validate_dependency ---
    fn make_test_dependency() -> Dependency {
        Dependency {
            id: "dep1".into(),
            source_id: "a1".into(),
            source_type: "application".into(),
            target_id: "m1".into(),
            target_type: "middleware".into(),
            relation_type: "tcp".into(),
            description: None,
            is_deleted: 0,
            deleted_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_validate_dependency_valid() {
        assert!(validate_dependency(&make_test_dependency()).is_ok());
    }

    #[test]
    fn test_validate_dependency_new_relation_types() {
        let mut dep = make_test_dependency();
        dep.relation_type = "grpc_call".into();
        assert!(validate_dependency(&dep).is_ok());

        dep.relation_type = "db_query".into();
        assert!(validate_dependency(&dep).is_ok());

        dep.relation_type = "cache_access".into();
        assert!(validate_dependency(&dep).is_ok());
    }

    #[test]
    fn test_validate_dependency_invalid_relation_type() {
        let mut dep = make_test_dependency();
        dep.relation_type = "invalid_relation".into();
        assert!(validate_dependency(&dep).is_err());
    }
}
