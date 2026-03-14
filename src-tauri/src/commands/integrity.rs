use std::collections::{BTreeMap, HashSet};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use tauri::State;

use crate::backup;
use crate::commands::system_jobs::{
    get_latest_system_job_finished_at, get_latest_system_job_result_json, record_system_job,
};
use crate::db::audit::insert_audit_log;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::integrity::{
    IntegrityFinding, IntegrityRepairInput, IntegrityRepairResult, IntegrityReport,
    IntegritySummary,
};
use crate::storage::StoragePaths;

fn route_for_resource(resource_type: &str) -> &'static str {
    match resource_type {
        "host" => "Hosts",
        "ip_address" => "IpAddresses",
        "service" => "Services",
        "middleware" => "Middlewares",
        "nginx" => "NginxConfigs",
        "system" => "Systems",
        _ => "Topology",
    }
}

fn make_filters(pairs: &[(&str, String)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), value.clone()))
        .collect()
}

fn resource_display_name(
    conn: &Connection,
    resource_type: &str,
    resource_id: &str,
) -> Result<Option<String>, String> {
    match resource_type {
        "host" => conn
            .query_row(
                "SELECT hostname FROM hosts WHERE id = ?1 AND is_deleted = 0",
                params![resource_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| format!("query host display name failed: {}", err)),
        "ip_address" => conn
            .query_row(
                "SELECT ip_address FROM ip_addresses WHERE id = ?1 AND is_deleted = 0",
                params![resource_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| format!("query ip display name failed: {}", err)),
        "service" => conn
            .query_row(
                "SELECT name FROM services WHERE id = ?1 AND is_deleted = 0",
                params![resource_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| format!("query service display name failed: {}", err)),
        "middleware" => conn
            .query_row(
                "SELECT name FROM middlewares WHERE id = ?1 AND is_deleted = 0",
                params![resource_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| format!("query middleware display name failed: {}", err)),
        "nginx" => conn
            .query_row(
                "SELECT name FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
                params![resource_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| format!("query nginx display name failed: {}", err)),
        "system" => conn
            .query_row(
                "SELECT name FROM systems WHERE id = ?1 AND is_deleted = 0",
                params![resource_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| format!("query system display name failed: {}", err)),
        _ => Ok(None),
    }
}

fn resource_label(resource_type: &str) -> &'static str {
    match resource_type {
        "host" => "服务器",
        "ip_address" => "IP 资源",
        "service" => "服务",
        "middleware" => "中间件",
        "nginx" => "负载均衡",
        "system" => "系统",
        _ => "资源",
    }
}

fn push_finding(findings: &mut Vec<IntegrityFinding>, finding: IntegrityFinding) {
    findings.push(finding);
}

fn collect_orphan_deployments(
    conn: &Connection,
    findings: &mut Vec<IntegrityFinding>,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, resource_type, resource_id, host_id
             FROM deployments WHERE is_deleted = 0 ORDER BY created_at DESC",
        )
        .map_err(|err| format!("prepare deployments failed: {}", err))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|err| format!("query deployments failed: {}", err))?;

    for row in rows {
        let (id, resource_type, resource_id, host_id) = row.map_err(|err| err.to_string())?;
        let resource_name = resource_display_name(conn, &resource_type, &resource_id)?;
        let host_name = resource_display_name(conn, "host", &host_id)?;
        if resource_name.is_some() && host_name.is_some() {
            continue;
        }

        let mut missing_parts = Vec::new();
        if resource_name.is_none() {
            missing_parts.push(format!(
                "{} {}",
                resource_label(&resource_type),
                resource_id
            ));
        }
        if host_name.is_none() {
            missing_parts.push(format!("服务器 {}", host_id));
        }

        push_finding(
            findings,
            IntegrityFinding {
                id: format!("deployment:{}", id),
                key: "orphan_deployment".to_string(),
                category: "integrity".to_string(),
                severity: "critical".to_string(),
                title: "部署关系引用缺失".to_string(),
                description: format!(
                    "部署关系 {} 引用了不存在的 {}。",
                    id,
                    missing_parts.join("、")
                ),
                resource_type: Some(resource_type.clone()),
                resource_id: Some(resource_id.clone()),
                resource_name,
                topology_node_id: Some(resource_id.clone()),
                target_route: route_for_resource(&resource_type).to_string(),
                target_filters: make_filters(&[("resourceId", resource_id)]),
                repair_supported: true,
            },
        );
    }

    Ok(())
}

fn collect_orphan_call_relations(
    conn: &Connection,
    findings: &mut Vec<IntegrityFinding>,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT pair_key, owner_id, owner_type, peer_id, peer_type, relation_type
             FROM call_relations WHERE is_deleted = 0 ORDER BY created_at DESC",
        )
        .map_err(|err| format!("prepare call_relations failed: {}", err))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|err| format!("query call_relations failed: {}", err))?;

    let mut seen_pairs = HashSet::new();
    for row in rows {
        let (pair_key, owner_id, owner_type, peer_id, peer_type, relation_type) =
            row.map_err(|err| err.to_string())?;
        if !seen_pairs.insert(pair_key.clone()) {
            continue;
        }

        let owner_name = resource_display_name(conn, &owner_type, &owner_id)?;
        let peer_name = resource_display_name(conn, &peer_type, &peer_id)?;
        if owner_name.is_some() && peer_name.is_some() {
            continue;
        }

        let mut missing_parts = Vec::new();
        if owner_name.is_none() {
            missing_parts.push(format!("上游 {} {}", resource_label(&owner_type), owner_id));
        }
        if peer_name.is_none() {
            missing_parts.push(format!("下游 {} {}", resource_label(&peer_type), peer_id));
        }

        push_finding(
            findings,
            IntegrityFinding {
                id: format!("call_relation:{}", pair_key),
                key: "orphan_call_relation".to_string(),
                category: "integrity".to_string(),
                severity: "critical".to_string(),
                title: "调用关系引用缺失".to_string(),
                description: format!(
                    "{} 调用关系缺少有效资源：{}。",
                    relation_type,
                    missing_parts.join("、")
                ),
                resource_type: Some(owner_type.clone()),
                resource_id: Some(owner_id.clone()),
                resource_name: owner_name,
                topology_node_id: Some(owner_id.clone()),
                target_route: "Topology".to_string(),
                target_filters: make_filters(&[("nodeId", owner_id)]),
                repair_supported: true,
            },
        );
    }

    Ok(())
}

fn collect_orphan_host_ip_bindings(
    conn: &Connection,
    findings: &mut Vec<IntegrityFinding>,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, host_id, ip_id FROM host_ip_bindings WHERE is_deleted = 0 ORDER BY created_at DESC",
        )
        .map_err(|err| format!("prepare host_ip_bindings failed: {}", err))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|err| format!("query host_ip_bindings failed: {}", err))?;

    for row in rows {
        let (id, host_id, ip_id) = row.map_err(|err| err.to_string())?;
        let host_name = resource_display_name(conn, "host", &host_id)?;
        let ip_name = resource_display_name(conn, "ip_address", &ip_id)?;
        if host_name.is_some() && ip_name.is_some() {
            continue;
        }

        push_finding(
            findings,
            IntegrityFinding {
                id: format!("host_ip_binding:{}", id),
                key: "orphan_host_ip_binding".to_string(),
                category: "integrity".to_string(),
                severity: "critical".to_string(),
                title: "主机与 IP 绑定失效".to_string(),
                description: format!(
                    "主机与 IP 绑定 {} 缺少 {}。",
                    id,
                    if host_name.is_none() && ip_name.is_none() {
                        "主机和 IP 资源"
                    } else if host_name.is_none() {
                        "主机资源"
                    } else {
                        "IP 资源"
                    }
                ),
                resource_type: Some("host".to_string()),
                resource_id: Some(host_id.clone()),
                resource_name: host_name,
                topology_node_id: None,
                target_route: "IpAddresses".to_string(),
                target_filters: make_filters(&[("hostId", host_id)]),
                repair_supported: true,
            },
        );
    }

    Ok(())
}

fn collect_orphan_taxonomy_bindings(
    conn: &Connection,
    findings: &mut Vec<IntegrityFinding>,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, term_id, resource_type, resource_id
             FROM taxonomy_bindings WHERE is_deleted = 0 ORDER BY created_at DESC",
        )
        .map_err(|err| format!("prepare taxonomy_bindings failed: {}", err))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|err| format!("query taxonomy_bindings failed: {}", err))?;

    for row in rows {
        let (id, term_id, resource_type, resource_id) = row.map_err(|err| err.to_string())?;
        let term_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM taxonomy_terms WHERE id = ?1 AND is_deleted = 0",
                params![term_id],
                |row| row.get(0),
            )
            .map_err(|err| format!("query taxonomy term failed: {}", err))?;
        let resource_name = resource_display_name(conn, &resource_type, &resource_id)?;
        if term_exists && resource_name.is_some() {
            continue;
        }

        push_finding(
            findings,
            IntegrityFinding {
                id: format!("taxonomy_binding:{}", id),
                key: "orphan_taxonomy_binding".to_string(),
                category: "integrity".to_string(),
                severity: "critical".to_string(),
                title: "标签绑定失效".to_string(),
                description: format!(
                    "标签绑定记录 {}（term_id={}）指向了无效的标签或资源。",
                    id, term_id
                ),
                resource_type: Some(resource_type.clone()),
                resource_id: Some(resource_id.clone()),
                resource_name,
                topology_node_id: None,
                target_route: route_for_resource(&resource_type).to_string(),
                target_filters: make_filters(&[("resourceId", resource_id)]),
                repair_supported: true,
            },
        );
    }

    Ok(())
}

fn collect_dangling_system_refs(
    conn: &Connection,
    findings: &mut Vec<IntegrityFinding>,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, system_id
             FROM services
             WHERE is_deleted = 0 AND system_id IS NOT NULL AND TRIM(system_id) <> ''",
        )
        .map_err(|err| format!("prepare dangling system refs failed: {}", err))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|err| format!("query dangling system refs failed: {}", err))?;

    for row in rows {
        let (id, name, system_id) = row.map_err(|err| err.to_string())?;
        if resource_display_name(conn, "system", &system_id)?.is_some()
        {
            continue;
        }

        push_finding(
            findings,
            IntegrityFinding {
                id: format!("service_system_binding:{}", id),
                key: "dangling_system_ref".to_string(),
                category: "integrity".to_string(),
                severity: "critical".to_string(),
                title: "服务挂载了不存在的系统".to_string(),
                description: format!(
                    "服务 {} 引用了不存在的系统 {}。",
                    name, system_id
                ),
                resource_type: Some("service".to_string()),
                resource_id: Some(id.clone()),
                resource_name: Some(name),
                topology_node_id: Some(id.clone()),
                target_route: "Services".to_string(),
                target_filters: make_filters(&[("resourceId", id)]),
                repair_supported: true,
            },
        );
    }

    Ok(())
}

fn collect_undeployed_resources(
    conn: &Connection,
    findings: &mut Vec<IntegrityFinding>,
) -> Result<(), String> {
    let definitions = [
        (
            "service",
            "Services",
            "服务",
            "SELECT id, name FROM services WHERE is_deleted = 0
             AND NOT EXISTS (
                SELECT 1 FROM deployments d
                WHERE d.resource_id = services.id AND d.resource_type = 'service' AND d.is_deleted = 0
             )",
        ),
        (
            "middleware",
            "Middlewares",
            "中间件",
            "SELECT id, name FROM middlewares WHERE is_deleted = 0
             AND NOT EXISTS (
                SELECT 1 FROM deployments d
                WHERE d.resource_id = middlewares.id AND d.resource_type = 'middleware' AND d.is_deleted = 0
             )",
        ),
        (
            "nginx",
            "NginxConfigs",
            "负载均衡",
            "SELECT id, name FROM nginx_configs WHERE is_deleted = 0
             AND NOT EXISTS (
                SELECT 1 FROM deployments d
                WHERE d.resource_id = nginx_configs.id AND d.resource_type = 'nginx' AND d.is_deleted = 0
             )",
        ),
    ];

    for (resource_type, target_route, label, sql) in definitions {
        let mut stmt = conn
            .prepare(sql)
            .map_err(|err| format!("prepare undeployed query failed: {}", err))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| format!("query undeployed resources failed: {}", err))?;
        for row in rows {
            let (id, name) = row.map_err(|err| err.to_string())?;
            push_finding(
                findings,
                IntegrityFinding {
                    id: format!("undeployed:{}:{}", resource_type, id),
                    key: "undeployed_resource".to_string(),
                    category: "coverage".to_string(),
                    severity: "warning".to_string(),
                    title: "资源尚未建立部署关系".to_string(),
                    description: format!("{} {} 尚未建立部署关系。", label, name),
                    resource_type: Some(resource_type.to_string()),
                    resource_id: Some(id.clone()),
                    resource_name: Some(name),
                    topology_node_id: Some(id.clone()),
                    target_route: target_route.to_string(),
                    target_filters: make_filters(&[("resourceId", id)]),
                    repair_supported: false,
                },
            );
        }
    }

    Ok(())
}

fn collect_isolated_resources(
    conn: &Connection,
    findings: &mut Vec<IntegrityFinding>,
) -> Result<(), String> {
    let definitions = [
        (
            "service",
            "Services",
            "服务",
            "SELECT id, name FROM services WHERE is_deleted = 0
             AND NOT EXISTS (
                SELECT 1 FROM call_relations cr
                WHERE cr.is_deleted = 0
                  AND ((cr.owner_type = 'service' AND cr.owner_id = services.id)
                    OR (cr.peer_type = 'service' AND cr.peer_id = services.id))
             )",
        ),
        (
            "middleware",
            "Middlewares",
            "中间件",
            "SELECT id, name FROM middlewares WHERE is_deleted = 0
             AND NOT EXISTS (
                SELECT 1 FROM call_relations cr
                WHERE cr.is_deleted = 0
                  AND ((cr.owner_type = 'middleware' AND cr.owner_id = middlewares.id)
                    OR (cr.peer_type = 'middleware' AND cr.peer_id = middlewares.id))
             )",
        ),
        (
            "nginx",
            "NginxConfigs",
            "负载均衡",
            "SELECT id, name FROM nginx_configs WHERE is_deleted = 0
             AND NOT EXISTS (
                SELECT 1 FROM call_relations cr
                WHERE cr.is_deleted = 0
                  AND ((cr.owner_type = 'nginx' AND cr.owner_id = nginx_configs.id)
                    OR (cr.peer_type = 'nginx' AND cr.peer_id = nginx_configs.id))
             )",
        ),
    ];

    for (resource_type, _target_route, label, sql) in definitions {
        let mut stmt = conn
            .prepare(sql)
            .map_err(|err| format!("prepare isolated query failed: {}", err))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| format!("query isolated resources failed: {}", err))?;
        for row in rows {
            let (id, name) = row.map_err(|err| err.to_string())?;
            push_finding(
                findings,
                IntegrityFinding {
                    id: format!("isolated:{}:{}", resource_type, id),
                    key: "isolated_resource".to_string(),
                    category: "coverage".to_string(),
                    severity: "warning".to_string(),
                    title: "资源缺少上下游关系".to_string(),
                    description: format!("{} {} 当前没有任何调用关系。", label, name),
                    resource_type: Some(resource_type.to_string()),
                    resource_id: Some(id.clone()),
                    resource_name: Some(name),
                    topology_node_id: Some(id.clone()),
                    target_route: "Topology".to_string(),
                    target_filters: make_filters(&[("nodeId", id)]),
                    repair_supported: false,
                },
            );
        }
    }

    Ok(())
}

fn sort_findings(findings: &mut [IntegrityFinding]) {
    let severity_rank = |severity: &str| match severity {
        "critical" => 0,
        "warning" => 1,
        _ => 2,
    };

    findings.sort_by(|left, right| {
        severity_rank(&left.severity)
            .cmp(&severity_rank(&right.severity))
            .then(right.description.cmp(&left.description))
            .then(left.id.cmp(&right.id))
    });
}

fn build_integrity_summary(
    generated_at: String,
    last_scanned_at: Option<String>,
    findings: &[IntegrityFinding],
) -> IntegritySummary {
    let mut critical = 0;
    let mut warning = 0;
    let mut info = 0;
    let mut repairable = 0;
    for finding in findings {
        match finding.severity.as_str() {
            "critical" => critical += 1,
            "warning" => warning += 1,
            _ => info += 1,
        }
        if finding.repair_supported {
            repairable += 1;
        }
    }

    IntegritySummary {
        total: findings.len() as u64,
        critical,
        warning,
        info,
        repairable,
        generated_at,
        last_scanned_at,
    }
}

pub(crate) fn build_integrity_report(conn: &Connection) -> Result<IntegrityReport, String> {
    let generated_at = chrono::Utc::now().to_rfc3339();
    let last_scanned_at = get_latest_system_job_finished_at(conn, "integrity_scan")?;
    let mut findings = Vec::new();

    collect_orphan_deployments(conn, &mut findings)?;
    collect_orphan_call_relations(conn, &mut findings)?;
    collect_orphan_host_ip_bindings(conn, &mut findings)?;
    collect_orphan_taxonomy_bindings(conn, &mut findings)?;
    collect_dangling_system_refs(conn, &mut findings)?;
    collect_undeployed_resources(conn, &mut findings)?;
    collect_isolated_resources(conn, &mut findings)?;
    sort_findings(&mut findings);

    Ok(IntegrityReport {
        job_id: None,
        summary: build_integrity_summary(generated_at, last_scanned_at, &findings),
        findings,
    })
}

fn parse_recorded_integrity_report(
    command: &str,
    job_id: Option<String>,
    value: serde_json::Value,
) -> AppResult<IntegrityReport> {
    let mut report = serde_json::from_value::<IntegrityReport>(value)
        .map_err(|err| AppError::internal(command, format!("解析完整性报告失败: {}", err)))?;
    if report.job_id.is_none() {
        report.job_id = job_id;
    }
    Ok(report)
}

fn load_recorded_integrity_report_by_job_id(
    command: &str,
    conn: &Connection,
    job_id: &str,
) -> AppResult<Option<IntegrityReport>> {
    let raw_id = job_id.strip_prefix("system:").unwrap_or(job_id);
    let raw: Option<String> = conn
        .query_row(
            "SELECT result_json
             FROM system_jobs
             WHERE id = ?1
               AND job_type IN ('integrity_scan', 'integrity_repair')
               AND result_json IS NOT NULL",
            params![raw_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| AppError::from_db_error(command, "读取完整性任务结果", err))?;

    match raw {
        Some(content) => {
            let value = serde_json::from_str::<serde_json::Value>(&content)
                .map_err(|err| AppError::internal(command, format!("解析完整性任务结果失败: {}", err)))?;
            parse_recorded_integrity_report(command, Some(format!("system:{}", raw_id)), value)
                .map(Some)
        }
        None => Ok(None),
    }
}

fn load_latest_recorded_integrity_report(
    command: &str,
    conn: &Connection,
) -> AppResult<Option<IntegrityReport>> {
    let latest_job_type = match (
        get_latest_system_job_finished_at(conn, "integrity_scan")
            .map_err(|err| AppError::from_db_error(command, "读取扫描任务时间", err))?,
        get_latest_system_job_finished_at(conn, "integrity_repair")
            .map_err(|err| AppError::from_db_error(command, "读取修复任务时间", err))?,
    ) {
        (Some(scan), Some(repair)) => {
            if repair > scan {
                Some("integrity_repair")
            } else {
                Some("integrity_scan")
            }
        }
        (Some(_), None) => Some("integrity_scan"),
        (None, Some(_)) => Some("integrity_repair"),
        (None, None) => None,
    };

    let Some(job_type) = latest_job_type else {
        return Ok(None);
    };

    let value = get_latest_system_job_result_json(conn, job_type)
        .map_err(|err| AppError::from_db_error(command, "读取最新完整性报告", err))?;

    match value {
        Some(value) => parse_recorded_integrity_report(command, None, value).map(Some),
        None => Ok(None),
    }
}

pub(crate) fn get_integrity_report_inner(
    command: &str,
    conn: &Connection,
    job_id: Option<String>,
) -> AppResult<IntegrityReport> {
    if let Some(job_id) = job_id {
        return load_recorded_integrity_report_by_job_id(command, conn, &job_id)?.ok_or_else(|| {
            AppError::not_found(command, "完整性报告不存在", Some(job_id))
        });
    }

    if let Some(report) = load_latest_recorded_integrity_report(command, conn)? {
        return Ok(report);
    }

    build_integrity_report(conn)
        .map_err(|err| AppError::from_db_error(command, "生成完整性报告", err))
}

fn apply_repair(conn: &Connection, finding: &IntegrityFinding) -> Result<bool, String> {
    if let Some(id) = finding.id.strip_prefix("deployment:") {
        return conn
            .execute("DELETE FROM deployments WHERE id = ?1", params![id])
            .map(|affected| affected > 0)
            .map_err(|err| format!("delete deployment failed: {}", err));
    }

    if let Some(pair_key) = finding.id.strip_prefix("call_relation:") {
        return conn
            .execute(
                "DELETE FROM call_relations WHERE pair_key = ?1",
                params![pair_key],
            )
            .map(|affected| affected > 0)
            .map_err(|err| format!("delete call relations failed: {}", err));
    }

    if let Some(id) = finding.id.strip_prefix("host_ip_binding:") {
        return conn
            .execute("DELETE FROM host_ip_bindings WHERE id = ?1", params![id])
            .map(|affected| affected > 0)
            .map_err(|err| format!("delete host_ip_binding failed: {}", err));
    }

    if let Some(id) = finding.id.strip_prefix("taxonomy_binding:") {
        return conn
            .execute("DELETE FROM taxonomy_bindings WHERE id = ?1", params![id])
            .map(|affected| affected > 0)
            .map_err(|err| format!("delete taxonomy_binding failed: {}", err));
    }

    if let Some(id) = finding.id.strip_prefix("service_system_binding:") {
        return conn
            .execute(
                "UPDATE services SET system_id = '', updated_at = ?1 WHERE id = ?2 AND is_deleted = 0",
                params![chrono::Utc::now().to_rfc3339(), id],
            )
            .map(|affected| affected > 0)
            .map_err(|err| format!("clear system binding failed: {}", err));
    }

    Ok(false)
}

fn write_repair_audit_log(conn: &Connection, finding: &IntegrityFinding) -> Result<(), String> {
    let resource_type = finding.resource_type.as_deref().unwrap_or("integrity");
    let resource_id = finding.resource_id.as_deref().unwrap_or(&finding.id);
    let changes = json!({
        "finding_id": finding.id,
        "key": finding.key,
        "title": finding.title,
        "repair_supported": finding.repair_supported,
    })
    .to_string();

    insert_audit_log(
        conn,
        "update",
        resource_type,
        resource_id,
        finding.resource_name.as_deref().or(Some("完整性修复")),
        Some(&changes),
    )
}

pub(crate) fn repair_integrity_findings_inner(
    command: &str,
    conn: &Connection,
    input: IntegrityRepairInput,
) -> AppResult<(u64, u64, IntegrityReport)> {
    let current_report = build_integrity_report(conn)
        .map_err(|err| AppError::from_db_error(command, "读取完整性问题", err))?;
    let selected = if input.finding_ids.is_empty() {
        current_report
            .findings
            .iter()
            .filter(|finding| finding.repair_supported)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        current_report
            .findings
            .iter()
            .filter(|finding| {
                input.finding_ids.iter().any(|id| id == &finding.id) && finding.repair_supported
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|err| AppError::from_db_error(command, "开启修复事务", err))?;

    let result = (|| -> AppResult<(u64, u64, IntegrityReport)> {
        let mut repaired = 0u64;
        let mut skipped = 0u64;
        for finding in &selected {
            if apply_repair(conn, finding)
                .map_err(|err| AppError::from_db_error(command, "执行完整性修复", err))?
            {
                write_repair_audit_log(conn, finding)
                    .map_err(|err| AppError::from_db_error(command, "写入修复审计日志", err))?;
                repaired += 1;
            } else {
                skipped += 1;
            }
        }

        let report = build_integrity_report(conn)
            .map_err(|err| AppError::from_db_error(command, "生成修复后报告", err))?;
        Ok((repaired, skipped, report))
    })();

    match result {
        Ok(value) => {
            conn.execute_batch("COMMIT;")
                .map_err(|err| AppError::from_db_error(command, "提交修复事务", err))?;
            Ok(value)
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(err)
        }
    }
}

#[tauri::command]
pub fn get_integrity_report(pool: State<DbPool>, job_id: Option<String>) -> AppResult<IntegrityReport> {
    let command = "get_integrity_report";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    get_integrity_report_inner(command, &conn, job_id)
}

#[tauri::command]
pub fn scan_integrity(pool: State<DbPool>) -> AppResult<IntegrityReport> {
    let command = "scan_integrity";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    let mut report = build_integrity_report(&conn)
        .map_err(|err| AppError::from_db_error(command, "执行完整性扫描", err))?;

    let scan_payload = json!({ "scope": "all" });
    let scan_result = serde_json::to_value(&report)
        .map_err(|err| AppError::internal(command, err.to_string()))?;
    let job_id = record_system_job(
        &conn,
        "integrity_scan",
        "完整性扫描",
        "completed",
        Some(&format!("发现 {} 个问题", report.summary.total)),
        Some(&scan_payload),
        Some(&scan_result),
        None,
        false,
        false,
    )
    .map_err(|err| AppError::from_db_error(command, "记录扫描任务", err))?;
    report.job_id = Some(format!("system:{}", job_id));
    report.summary.last_scanned_at = Some(report.summary.generated_at.clone());

    Ok(report)
}

#[tauri::command]
pub fn repair_integrity_findings(
    pool: State<DbPool>,
    storage_paths: State<StoragePaths>,
    input: IntegrityRepairInput,
) -> AppResult<IntegrityRepairResult> {
    let command = "repair_integrity_findings";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;

    let backup_filename = format!(
        "backup_pre_integrity_repair_{}.db",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let backup_path = storage_paths.backup_dir.join(&backup_filename);
    backup::perform_backup(&pool, &backup_path)
        .map_err(|err| AppError::from_backup_error(command, "创建修复前安全快照", err))?;

    let (repaired_count, skipped_count, mut report) =
        repair_integrity_findings_inner(command, &conn, input)?;
    let repair_payload = json!({ "backup_filename": backup_filename, "mode": "safe_repair" });
    let repair_result = serde_json::to_value(&report)
        .map_err(|err| AppError::internal(command, err.to_string()))?;
    let job_id = record_system_job(
        &conn,
        "integrity_repair",
        "完整性修复",
        "completed",
        Some(&format!(
            "修复 {} 项，跳过 {} 项",
            repaired_count, skipped_count
        )),
        Some(&repair_payload),
        Some(&repair_result),
        None,
        false,
        false,
    )
    .map_err(|err| AppError::from_db_error(command, "记录修复任务", err))?;

    let changes = json!({
        "repaired_count": repaired_count,
        "skipped_count": skipped_count,
        "backup_filename": backup_filename,
    })
    .to_string();
    insert_audit_log(
        &conn,
        "update",
        "integrity",
        &job_id,
        Some("完整性修复"),
        Some(&changes),
    )
    .map_err(|err| AppError::from_db_error(command, "写入审计日志", err))?;

    report.job_id = Some(format!("system:{}", job_id.clone()));
    report.summary.last_scanned_at = Some(report.summary.generated_at.clone());

    Ok(IntegrityRepairResult {
        job_id: format!("system:{}", job_id),
        backup_filename,
        repaired_count,
        skipped_count,
        report,
    })
}

#[cfg(test)]
mod tests {
    use super::{get_integrity_report_inner, repair_integrity_findings_inner};
    use crate::commands::system_jobs::record_system_job;
    use crate::models::integrity::{IntegrityFinding, IntegrityReport, IntegritySummary};
    use crate::models::integrity::IntegrityRepairInput;
    use crate::test_helpers::{insert_test_service, insert_test_host, setup_test_db};
    use rusqlite::params;
    use serde_json::json;

    #[test]
    fn get_integrity_report_inner_should_detect_orphans_and_coverage_gaps() {
        let conn = setup_test_db();
        insert_test_host(&conn, "host-1", "host-1", "10.0.0.1");
        insert_test_service(&conn, "app-1", "orders-api", "prod", "");

        conn.execute(
            "INSERT INTO deployments (id, resource_id, resource_type, host_id, is_deleted, created_at, updated_at)
             VALUES ('dep-orphan', 'missing-app', 'service', 'host-1', 0, ?1, ?1)",
            params![chrono::Utc::now().to_rfc3339()],
        )
        .expect("insert orphan deployment");

        let report = get_integrity_report_inner("test", &conn, None).expect("integrity report");
        assert!(report
            .findings
            .iter()
            .any(|item| item.key == "orphan_deployment"));
        assert!(report
            .findings
            .iter()
            .any(|item| item.key == "undeployed_resource"));
        assert!(report.summary.total >= 2);
    }

    #[test]
    fn get_integrity_report_inner_should_return_recorded_job_report() {
        let conn = setup_test_db();
        let stored_report = IntegrityReport {
            job_id: None,
            summary: IntegritySummary {
                total: 1,
                critical: 1,
                warning: 0,
                info: 0,
                repairable: 1,
                generated_at: "2026-03-07T10:00:00.000Z".to_string(),
                last_scanned_at: Some("2026-03-07T10:00:00.000Z".to_string()),
            },
            findings: vec![IntegrityFinding {
                id: "deployment:dep-1".to_string(),
                key: "orphan_deployment".to_string(),
                category: "deployment".to_string(),
                severity: "critical".to_string(),
                title: "孤儿部署关系".to_string(),
                description: "部署记录指向了不存在的应用。".to_string(),
                resource_type: Some("service".to_string()),
                resource_id: Some("missing-app".to_string()),
                resource_name: Some("应用 A".to_string()),
                topology_node_id: Some("missing-app".to_string()),
                target_route: "Applications".to_string(),
                target_filters: std::collections::BTreeMap::new(),
                repair_supported: true,
            }],
        };

        let expected_job_id = format!(
            "system:{}",
            record_system_job(
                &conn,
                "integrity_scan",
                "完整性扫描",
                "completed",
                Some("发现 1 个问题"),
                Some(&json!({ "scope": "all" })),
                Some(&serde_json::to_value(&stored_report).expect("serialize report")),
                None,
                false,
                false,
            )
            .expect("record integrity job")
        );

        let report = get_integrity_report_inner("test", &conn, Some(expected_job_id.clone()))
            .expect("recorded integrity report");

        assert_eq!(report.job_id.as_deref(), Some(expected_job_id.as_str()));
        assert_eq!(report.summary.total, 1);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].title, "孤儿部署关系");
    }

    #[test]
    fn repair_integrity_findings_inner_should_remove_repairable_findings() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO deployments (id, resource_id, resource_type, host_id, is_deleted, created_at, updated_at)
             VALUES ('dep-orphan', 'missing-app', 'service', 'missing-host', 0, ?1, ?1)",
            params![chrono::Utc::now().to_rfc3339()],
        )
        .expect("insert orphan deployment");

        let (repaired, skipped, report) = repair_integrity_findings_inner(
            "test",
            &conn,
            IntegrityRepairInput {
                finding_ids: vec!["deployment:dep-orphan".to_string()],
            },
        )
        .expect("repair integrity findings");

        assert_eq!(repaired, 1);
        assert_eq!(skipped, 0);
        assert!(!report
            .findings
            .iter()
            .any(|item| item.id == "deployment:dep-orphan"));
    }
}
