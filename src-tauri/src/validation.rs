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

pub fn validate_string_length(value: &str, min: usize, max: usize, field_name: &str) -> Result<(), String> {
    let len = value.chars().count();
    if len < min || len > max {
        Err(format!("{} length must be {}-{}, got {}", field_name, min, max, len))
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
        Err(format!("{} must be one of {:?}, got '{}'", field_name, allowed, value))
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
    validate_enum(&host.status, &["running", "stopped", "maintenance"], "status")?;
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
    Ok(())
}

pub fn validate_application(app: &Application) -> Result<(), String> {
    validate_required(&app.name, "name")?;
    validate_string_length(&app.name, 1, 200, "name")?;
    validate_enum(
        &app.app_type,
        &["frontend", "backend", "gateway", "batch_job", "microservice", "other"],
        "type",
    )?;
    validate_enum(&app.env, &["prod", "dev", "test"], "env")?;
    validate_enum(&app.status, &["running", "stopped", "maintenance"], "status")?;
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
        &["database", "message_queue", "cache", "search_engine", "config_center", "other"],
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
        &["http_call", "tcp", "mq_produce", "mq_consume"],
        "relation_type",
    )?;
    Ok(())
}
