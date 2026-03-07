use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::nginx_config::NginxEndpoint;
use crate::models::topology::{
    RuntimeTopologyEdge as TopologyEdge, RuntimeTopologyEnvCount as TopologyEnvCount,
    RuntimeTopologyGraph as TopologyGraph, RuntimeTopologyImpact as LegacyTopologyImpact,
    RuntimeTopologyImpactNode as LegacyTopologyImpactNode,
    RuntimeTopologyKindCount as TopologyKindCount, RuntimeTopologyLane as TopologyLane,
    RuntimeTopologyLayoutHints as TopologyLayoutHints,
    RuntimeTopologyLegendStats as TopologyLegendStats, RuntimeTopologyNode as TopologyNode,
    RuntimeTopologyPaths as LegacyTopologyPaths, TopologyAffectedNodeV3, TopologyDrilldown,
    TopologyDrilldownQuery, TopologyEdgeV3, TopologyEnvCountV3, TopologyEvidence,
    TopologyEvidenceItem, TopologyEvidenceQuery, TopologyImpact, TopologyImpactQuery,
    TopologyKindCountV3, TopologyLaneV3, TopologyLayoutHintsV3, TopologyLegendStatsV3,
    TopologyNeighborV3, TopologyNodeV3, TopologyPathItem, TopologyPaths, TopologyPathsQuery,
    TopologySnapshot, TopologySnapshotMetaV3, TopologySnapshotQuery, TopologyTaskInsight,
    TopologyTaskView, TopologyTaskViewQuery, TopologyTroubleshootReportSummaryV3,
    TopologyTroubleshootReportV3, TopologyTroubleshootReportV3Query,
};
use rusqlite::{Connection, OptionalExtension};
use std::collections::{HashMap, HashSet, VecDeque};
use tauri::State;

const ENV_ORDER: [&str; 3] = ["prod", "test", "dev"];

fn normalize_env(raw: Option<String>) -> String {
    match raw.as_deref().map(str::trim).unwrap_or("prod") {
        "prod" => "prod".to_string(),
        "test" => "test".to_string(),
        "dev" => "dev".to_string(),
        _ => "prod".to_string(),
    }
}

fn env_label(env: &str) -> &'static str {
    match env {
        "prod" => "生产",
        "test" => "测试",
        "dev" => "开发",
        _ => "生产",
    }
}

fn vectorize_kind_count(map: HashMap<String, u32>) -> Vec<TopologyKindCount> {
    let mut items: Vec<TopologyKindCount> = map
        .into_iter()
        .map(|(kind, count)| TopologyKindCount { kind, count })
        .collect();
    items.sort_by(|a, b| a.kind.cmp(&b.kind));
    items
}

#[derive(Debug, Clone)]
struct HostTopologyMeta {
    env: String,
    host_name: Option<String>,
    host_ip_display: Option<String>,
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub fn get_topology_graph_inner(conn: &Connection) -> Result<TopologyGraph, String> {
    let mut host_meta_map: HashMap<String, HostTopologyMeta> = HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT
                    h.id,
                    h.hostname,
                    h.env,
                    NULLIF((
                      SELECT GROUP_CONCAT(ia.ip_address, ', ')
                      FROM host_ip_bindings hb
                      JOIN ip_addresses ia ON ia.id = hb.ip_id
                      WHERE hb.host_id = h.id AND hb.is_deleted = 0 AND ia.is_deleted = 0
                    ), '') AS host_ip_display
                FROM hosts h
                WHERE h.is_deleted = 0",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                let host_id: String = row.get(0)?;
                let host_name: Option<String> = row.get(1)?;
                let env: Option<String> = row.get(2)?;
                let host_ip_display: Option<String> = row.get(3)?;
                Ok((
                    host_id,
                    HostTopologyMeta {
                        env: normalize_env(env),
                        host_name: normalize_optional_text(host_name),
                        host_ip_display: normalize_optional_text(host_ip_display),
                    },
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (host_id, host_meta) = row.map_err(|e| e.to_string())?;
            host_meta_map.insert(host_id, host_meta);
        }
    }

    let mut deployment_host_map: HashMap<String, String> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT resource_id, host_id FROM deployments WHERE is_deleted = 0")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (resource_id, host_id) = row.map_err(|e| e.to_string())?;
            deployment_host_map.insert(resource_id, host_id);
        }
    }

    let mut nodes: Vec<TopologyNode> = Vec::new();
    let mut node_env_map: HashMap<String, String> = HashMap::new();

    {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, type, address, port, tech_stack, env, status \
                 FROM applications WHERE is_deleted = 0",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let app_type: Option<String> = row.get(2)?;
                let address: Option<String> = row.get(3)?;
                let port: Option<i64> = row.get(4)?;
                let tech_stack: Option<String> = row.get(5)?;
                let env = normalize_env(row.get(6)?);
                let status: Option<String> = row.get(7)?;

                let mut extra = serde_json::Map::new();
                if let Some(ref value) = app_type {
                    extra.insert("type".to_string(), serde_json::Value::String(value.clone()));
                }
                if let Some(ref value) = tech_stack {
                    extra.insert(
                        "tech_stack".to_string(),
                        serde_json::Value::String(value.clone()),
                    );
                }
                if let Some(ref value) = address {
                    extra.insert(
                        "address".to_string(),
                        serde_json::Value::String(value.clone()),
                    );
                }
                if let Some(value) = port {
                    extra.insert("port".to_string(), serde_json::json!(value));
                }

                Ok(TopologyNode {
                    id,
                    name,
                    node_type: "application".to_string(),
                    env,
                    group_kind: "application_service".to_string(),
                    host_id: None,
                    host_name: None,
                    host_ip_display: None,
                    status,
                    importance: 1.0,
                    extra: Some(serde_json::Value::Object(extra)),
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let mut node = row.map_err(|e| e.to_string())?;
            node.host_id = deployment_host_map.get(&node.id).cloned();
            if let Some(host_id) = node.host_id.as_ref() {
                if let Some(host_meta) = host_meta_map.get(host_id) {
                    node.env = host_meta.env.clone();
                    node.host_name = host_meta.host_name.clone();
                    node.host_ip_display = host_meta.host_ip_display.clone();
                }
            }
            node_env_map.insert(node.id.clone(), node.env.clone());
            nodes.push(node);
        }
    }

    {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, type, address, port, version, env \
                 FROM middlewares WHERE is_deleted = 0",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let category: Option<String> = row.get(2)?;
                let middleware_type: Option<String> = row.get(3)?;
                let address: Option<String> = row.get(4)?;
                let port: Option<i64> = row.get(5)?;
                let version: Option<String> = row.get(6)?;
                let env = normalize_env(row.get(7)?);

                let mut extra = serde_json::Map::new();
                if let Some(ref value) = category {
                    extra.insert(
                        "category".to_string(),
                        serde_json::Value::String(value.clone()),
                    );
                }
                if let Some(ref value) = middleware_type {
                    extra.insert("type".to_string(), serde_json::Value::String(value.clone()));
                }
                if let Some(ref value) = address {
                    extra.insert(
                        "address".to_string(),
                        serde_json::Value::String(value.clone()),
                    );
                }
                if let Some(value) = port {
                    extra.insert("port".to_string(), serde_json::json!(value));
                }
                if let Some(ref value) = version {
                    extra.insert(
                        "version".to_string(),
                        serde_json::Value::String(value.clone()),
                    );
                }

                Ok(TopologyNode {
                    id,
                    name,
                    node_type: "middleware".to_string(),
                    env,
                    group_kind: "middleware".to_string(),
                    host_id: None,
                    host_name: None,
                    host_ip_display: None,
                    status: None,
                    importance: 0.82,
                    extra: Some(serde_json::Value::Object(extra)),
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let mut node = row.map_err(|e| e.to_string())?;
            node.host_id = deployment_host_map.get(&node.id).cloned();
            if let Some(host_id) = node.host_id.as_ref() {
                if let Some(host_meta) = host_meta_map.get(host_id) {
                    node.env = host_meta.env.clone();
                    node.host_name = host_meta.host_name.clone();
                    node.host_ip_display = host_meta.host_ip_display.clone();
                }
            }
            node_env_map.insert(node.id.clone(), node.env.clone());
            nodes.push(node);
        }
    }

    {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, endpoints, strategy, env, status \
                 FROM nginx_configs WHERE is_deleted = 0",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let endpoints_raw: String = row.get(2)?;
                let endpoints: Vec<NginxEndpoint> =
                    serde_json::from_str(&endpoints_raw).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;
                let strategy: Option<String> = row.get(3)?;
                let env = normalize_env(row.get(4)?);
                let status: Option<String> = row.get(5)?;

                let mut extra = serde_json::Map::new();
                extra.insert(
                    "endpoint_count".to_string(),
                    serde_json::json!(endpoints.len()),
                );
                if let Some(first_endpoint) = endpoints.first() {
                    let endpoint_text =
                        format!("{}:{}", first_endpoint.host.trim(), first_endpoint.port);
                    extra.insert(
                        "first_endpoint".to_string(),
                        serde_json::Value::String(endpoint_text.clone()),
                    );
                    extra.insert(
                        "address".to_string(),
                        serde_json::Value::String(endpoint_text),
                    );
                }
                if let Some(ref value) = strategy {
                    extra.insert(
                        "strategy".to_string(),
                        serde_json::Value::String(value.clone()),
                    );
                }

                Ok(TopologyNode {
                    id,
                    name,
                    node_type: "nginx".to_string(),
                    env,
                    group_kind: "nginx".to_string(),
                    host_id: None,
                    host_name: None,
                    host_ip_display: None,
                    status,
                    importance: 0.9,
                    extra: Some(serde_json::Value::Object(extra)),
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let mut node = row.map_err(|e| e.to_string())?;
            node.host_id = deployment_host_map.get(&node.id).cloned();
            if let Some(host_id) = node.host_id.as_ref() {
                if let Some(host_meta) = host_meta_map.get(host_id) {
                    node.env = host_meta.env.clone();
                    node.host_name = host_meta.host_name.clone();
                    node.host_ip_display = host_meta.host_ip_display.clone();
                }
            }
            node_env_map.insert(node.id.clone(), node.env.clone());
            nodes.push(node);
        }
    }

    let mut edge_map: HashMap<(String, String, String), TopologyEdge> = HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, owner_id, peer_id, relation_type, description
                 FROM call_relations
                 WHERE is_deleted = 0 AND direction = 'upstream'",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (id, source, target, edge_type, label) = row.map_err(|e| e.to_string())?;
            let key = (source.clone(), target.clone(), edge_type.clone());
            let source_env = node_env_map
                .get(&source)
                .map(String::as_str)
                .unwrap_or("prod");
            let target_env = node_env_map
                .get(&target)
                .map(String::as_str)
                .unwrap_or("prod");
            let cross_env = source_env != target_env;

            edge_map
                .entry(key)
                .and_modify(|edge| {
                    edge.strength += 1;
                    if edge.label.is_none() && label.is_some() {
                        edge.label = label.clone();
                    }
                })
                .or_insert_with(|| TopologyEdge {
                    id,
                    source,
                    target,
                    edge_type,
                    label,
                    strength: 1,
                    cross_env,
                });
        }
    }

    let mut edges: Vec<TopologyEdge> = edge_map.into_values().collect();
    edges.sort_by(|a, b| a.id.cmp(&b.id));

    let mut env_node_counts: HashMap<String, u32> = ENV_ORDER
        .iter()
        .map(|env| (env.to_string(), 0_u32))
        .collect();
    let mut env_app_counts: HashMap<String, u32> = ENV_ORDER
        .iter()
        .map(|env| (env.to_string(), 0_u32))
        .collect();
    let mut node_type_counts: HashMap<String, u32> = HashMap::new();

    for node in &nodes {
        *env_node_counts.entry(node.env.clone()).or_insert(0) += 1;
        if node.node_type == "application" {
            *env_app_counts.entry(node.env.clone()).or_insert(0) += 1;
        }
        *node_type_counts.entry(node.node_type.clone()).or_insert(0) += 1;
    }

    let lanes: Vec<TopologyLane> = ENV_ORDER
        .iter()
        .enumerate()
        .map(|(index, env)| TopologyLane {
            id: (*env).to_string(),
            label: env_label(env).to_string(),
            order: index as u8,
            node_count: *env_node_counts.get(*env).unwrap_or(&0),
            app_count: *env_app_counts.get(*env).unwrap_or(&0),
        })
        .collect();

    let env_counts: Vec<TopologyEnvCount> = ENV_ORDER
        .iter()
        .map(|env| TopologyEnvCount {
            env: (*env).to_string(),
            count: *env_node_counts.get(*env).unwrap_or(&0),
            app_count: *env_app_counts.get(*env).unwrap_or(&0),
        })
        .collect();

    let mut edge_type_counts: HashMap<String, u32> = HashMap::new();
    for edge in &edges {
        *edge_type_counts.entry(edge.edge_type.clone()).or_insert(0) += edge.strength;
    }

    let application_service_count = nodes
        .iter()
        .filter(|node| node.group_kind == "application_service")
        .count() as u32;

    let legend_stats = TopologyLegendStats {
        env_counts,
        node_type_counts: vectorize_kind_count(node_type_counts),
        edge_type_counts: vectorize_kind_count(edge_type_counts),
        application_service_count,
    };

    let layout_hints = TopologyLayoutHints {
        lane_order: ENV_ORDER.iter().map(|env| (*env).to_string()).collect(),
        default_collapsed_groups: vec!["middleware".to_string(), "nginx".to_string()],
        high_density_mode: nodes.len() > 800,
    };

    Ok(TopologyGraph {
        lanes,
        nodes,
        edges,
        legend_stats,
        layout_hints,
    })
}

pub fn find_paths_inner(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    max_results: usize,
) -> Result<LegacyTopologyPaths, String> {
    let max_depth: usize = 10;

    // Build adjacency list from call_relations (upstream rows only)
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT owner_id, peer_id
                 FROM call_relations
                 WHERE is_deleted = 0 AND direction = 'upstream'",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (src, tgt) = row.map_err(|e| e.to_string())?;
            adj.entry(src).or_default().push(tgt);
        }
    }

    // BFS to find all paths
    let mut paths: Vec<Vec<String>> = Vec::new();
    let mut truncated = false;

    let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();
    queue.push_back((source_id.to_string(), vec![source_id.to_string()]));

    while let Some((current, path)) = queue.pop_front() {
        if path.len() > max_depth {
            continue;
        }

        if current == target_id && path.len() > 1 {
            if paths.len() >= max_results {
                truncated = true;
                break;
            }
            paths.push(path);
            continue;
        }

        if let Some(neighbors) = adj.get(&current) {
            for next in neighbors {
                if !path.contains(next) {
                    let mut new_path = path.clone();
                    new_path.push(next.clone());
                    queue.push_back((next.clone(), new_path));
                }
            }
        }
    }

    Ok(LegacyTopologyPaths { paths, truncated })
}

pub fn analyze_impact_inner(
    conn: &Connection,
    node_id: &str,
) -> Result<LegacyTopologyImpact, String> {
    // Build reverse adjacency list (target -> sources, i.e. who depends on target)
    let mut reverse_adj: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT owner_id, peer_id
                 FROM call_relations
                 WHERE is_deleted = 0 AND direction = 'upstream'",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (src, tgt) = row.map_err(|e| e.to_string())?;
            reverse_adj.entry(tgt).or_default().push(src);
        }
    }

    // Build name lookup table from all resource types with UNION ALL
    let mut name_map: HashMap<String, (String, String)> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, name, 'application' AS node_type FROM applications WHERE is_deleted = 0 \
             UNION ALL \
             SELECT id, name, 'middleware' AS node_type FROM middlewares WHERE is_deleted = 0 \
             UNION ALL \
             SELECT id, name, 'nginx' AS node_type FROM nginx_configs WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, name, node_type) = row.map_err(|e| e.to_string())?;
            name_map.insert(id, (name, node_type));
        }
    }

    // Reverse BFS from node_id to find all upstream dependents
    let mut affected: Vec<LegacyTopologyImpactNode> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(node_id.to_string());

    let mut queue: VecDeque<(String, u32)> = VecDeque::new();
    queue.push_back((node_id.to_string(), 0));

    let mut max_depth: u32 = 0;

    while let Some((current, depth)) = queue.pop_front() {
        if let Some(dependents) = reverse_adj.get(&current) {
            for dep_id in dependents {
                if visited.contains(dep_id) {
                    continue;
                }
                visited.insert(dep_id.clone());
                let next_depth = depth + 1;
                if next_depth > max_depth {
                    max_depth = next_depth;
                }

                let (name, node_type) = name_map
                    .get(dep_id)
                    .cloned()
                    .unwrap_or_else(|| (dep_id.clone(), "unknown".to_string()));

                affected.push(LegacyTopologyImpactNode {
                    id: dep_id.clone(),
                    name,
                    node_type,
                    depth: next_depth,
                });

                queue.push_back((dep_id.clone(), next_depth));
            }
        }
    }

    let total_count = affected.len() as u32;

    Ok(LegacyTopologyImpact {
        affected_nodes: affected,
        total_count,
        max_depth,
    })
}

fn normalize_task_view(value: Option<&str>) -> String {
    let raw = value.unwrap_or("explore").trim();
    match raw {
        "troubleshoot" => "troubleshoot".to_string(),
        "impact" => "impact".to_string(),
        "change" => "change".to_string(),
        _ => "explore".to_string(),
    }
}

pub fn get_topology_snapshot_v3_inner(
    conn: &Connection,
    query: &TopologySnapshotQuery,
) -> Result<TopologySnapshot, String> {
    let mut graph = get_topology_graph_inner(conn)?;
    if let Some(env_filter) = query
        .env
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let filtered_nodes: Vec<TopologyNode> = graph
            .nodes
            .iter()
            .filter(|node| node.env == env_filter)
            .cloned()
            .collect();
        let node_ids: HashSet<String> = filtered_nodes.iter().map(|node| node.id.clone()).collect();
        let filtered_edges = graph
            .edges
            .iter()
            .filter(|edge| node_ids.contains(&edge.source) && node_ids.contains(&edge.target))
            .cloned()
            .collect::<Vec<_>>();

        graph.nodes = filtered_nodes;
        graph.edges = filtered_edges;

        for lane in &mut graph.lanes {
            lane.node_count = graph
                .nodes
                .iter()
                .filter(|node| node.env == lane.id)
                .count() as u32;
            lane.app_count = graph
                .nodes
                .iter()
                .filter(|node| node.env == lane.id && node.node_type == "application")
                .count() as u32;
        }

        for env_count in &mut graph.legend_stats.env_counts {
            env_count.count = graph
                .nodes
                .iter()
                .filter(|node| node.env == env_count.env)
                .count() as u32;
            env_count.app_count = graph
                .nodes
                .iter()
                .filter(|node| node.env == env_count.env && node.node_type == "application")
                .count() as u32;
        }

        let mut node_type_map: HashMap<String, u32> = HashMap::new();
        for node in &graph.nodes {
            *node_type_map.entry(node.node_type.clone()).or_insert(0) += 1;
        }
        graph.legend_stats.node_type_counts = map_kind_counts(node_type_map);

        let mut edge_type_map: HashMap<String, u32> = HashMap::new();
        for edge in &graph.edges {
            *edge_type_map.entry(edge.edge_type.clone()).or_insert(0) += edge.strength;
        }
        graph.legend_stats.edge_type_counts = map_kind_counts(edge_type_map);
        graph.legend_stats.application_service_count = graph
            .nodes
            .iter()
            .filter(|node| node.group_kind == "application_service")
            .count() as u32;
        graph.layout_hints.high_density_mode = graph.nodes.len() > 800;
    }

    let focus_node = query.focus_node_id.as_ref().and_then(|focus_id| {
        graph
            .nodes
            .iter()
            .find(|node| &node.id == focus_id)
            .map(to_node_v3)
    });
    let node_count = graph.nodes.len() as u32;
    let edge_count = graph.edges.len() as u32;
    let task_view = normalize_task_view(query.task_view.as_deref());
    let legend_stats = to_legend_stats_v3(&graph.legend_stats, &graph.edges, query.env.clone());
    let layout_hints = to_layout_hints_v3(&graph.layout_hints);
    let lanes = graph.lanes.iter().map(to_lane_v3).collect();
    let nodes = graph.nodes.iter().map(to_node_v3).collect();
    let edges = graph.edges.iter().map(to_edge_v3).collect();
    let meta = TopologySnapshotMetaV3 {
        version: "3.0".to_string(),
        task_view,
        generated_at: chrono::Utc::now().to_rfc3339(),
        env: query.env.clone(),
        max_depth: query.max_depth,
        node_count,
        edge_count,
    };

    Ok(TopologySnapshot {
        meta,
        lanes,
        nodes,
        edges,
        legend_stats,
        layout_hints,
        focus_node,
        node_count,
        edge_count,
    })
}

pub fn get_topology_drilldown_v3_inner(
    conn: &Connection,
    query: &TopologyDrilldownQuery,
) -> Result<TopologyDrilldown, String> {
    let graph = get_topology_graph_inner(conn)?;
    let node = graph
        .nodes
        .iter()
        .find(|item| item.id == query.node_id)
        .cloned()
        .ok_or_else(|| format!("Record not found: node_id={}", query.node_id))?;
    let node_map = graph
        .nodes
        .iter()
        .map(|item| (item.id.clone(), item.clone()))
        .collect::<HashMap<_, _>>();

    let mut inbound_edge_count: u32 = 0;
    let mut outbound_edge_count: u32 = 0;
    let mut upstream_ids: HashSet<String> = HashSet::new();
    let mut downstream_ids: HashSet<String> = HashSet::new();

    for edge in &graph.edges {
        if edge.target == query.node_id {
            inbound_edge_count += edge.strength;
            upstream_ids.insert(edge.source.clone());
        }
        if edge.source == query.node_id {
            outbound_edge_count += edge.strength;
            downstream_ids.insert(edge.target.clone());
        }
    }

    let mut upstream = upstream_ids
        .into_iter()
        .filter_map(|id| node_map.get(&id).map(to_neighbor))
        .collect::<Vec<_>>();
    let mut downstream = downstream_ids
        .into_iter()
        .filter_map(|id| node_map.get(&id).map(to_neighbor))
        .collect::<Vec<_>>();
    upstream.sort_by(|left, right| left.id.cmp(&right.id));
    downstream.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(TopologyDrilldown {
        node: to_node_v3(&node),
        upstream,
        downstream,
        inbound_edge_count,
        outbound_edge_count,
    })
}

pub fn get_topology_task_view_v3_inner(
    conn: &Connection,
    query: &TopologyTaskViewQuery,
) -> Result<TopologyTaskView, String> {
    let task_view = normalize_task_view(query.task_view.as_deref());
    let snapshot = get_topology_snapshot_v3_inner(
        conn,
        &TopologySnapshotQuery {
            env: query.env.clone(),
            task_view: Some(task_view.clone()),
            max_depth: query.max_depth,
            focus_node_id: query
                .focus_node_id
                .clone()
                .or_else(|| query.node_id.clone()),
        },
    )?;

    let focus_id = query
        .node_id
        .clone()
        .or_else(|| query.focus_node_id.clone())
        .or_else(|| snapshot.nodes.first().map(|node| node.id.clone()));

    let mut insights: Vec<TopologyTaskInsight> = Vec::new();

    if let Some(node_id) = focus_id {
        let drilldown = get_topology_drilldown_v3_inner(
            conn,
            &TopologyDrilldownQuery {
                node_id: node_id.clone(),
                task_view: Some(task_view.clone()),
                env: query.env.clone(),
                max_depth: query.max_depth,
                direction: None,
            },
        )?;

        insights.push(TopologyTaskInsight {
            kind: "profile".to_string(),
            title: "核查节点画像".to_string(),
            description: format!(
                "{} ({}) in {}",
                drilldown.node.name, drilldown.node.node_type, drilldown.node.env
            ),
            severity: if drilldown.node.status.as_deref() != Some("running") {
                "critical".to_string()
            } else {
                "info".to_string()
            },
            node_ids: vec![drilldown.node.id.clone()],
            edge_ids: Vec::new(),
        });

        insights.push(TopologyTaskInsight {
            kind: "dependency".to_string(),
            title: "依赖关系摘要".to_string(),
            description: format!(
                "inbound={}, outbound={}",
                drilldown.inbound_edge_count, drilldown.outbound_edge_count
            ),
            severity: if drilldown.inbound_edge_count >= 2 {
                "warning".to_string()
            } else {
                "info".to_string()
            },
            node_ids: vec![drilldown.node.id],
            edge_ids: Vec::new(),
        });
    }

    let title = Some(
        match task_view.as_str() {
            "troubleshoot" => "故障排查视图",
            "impact" => "影响面视图",
            "change" => "变更评估视图",
            _ => "拓扑探索视图",
        }
        .to_string(),
    );
    let summary = Some(format!(
        "聚焦节点数={}, 关系线数={}",
        snapshot.node_count, snapshot.edge_count
    ));
    let meta = Some(snapshot.meta.clone());

    Ok(TopologyTaskView {
        task_view,
        snapshot,
        title,
        summary,
        insights,
        meta,
    })
}

pub fn get_topology_troubleshoot_report_v3_inner(
    conn: &Connection,
    query: &TopologyTroubleshootReportV3Query,
) -> Result<TopologyTroubleshootReportV3, String> {
    let task_view = match normalize_task_view(query.task_view.as_deref()).as_str() {
        "troubleshoot" => "troubleshoot".to_string(),
        _ => "troubleshoot".to_string(),
    };
    let profile = load_node_profile(conn, &query.node_id)?;

    if let Some(env) = query
        .env
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if profile.env != env {
            return Err(format!(
                "Record not found: node_id={}, env={}",
                query.node_id, env
            ));
        }
    }

    let drilldown = get_topology_drilldown_v3_inner(
        conn,
        &TopologyDrilldownQuery {
            node_id: query.node_id.clone(),
            task_view: Some(task_view.clone()),
            env: query.env.clone(),
            max_depth: None,
            direction: None,
        },
    )?;
    let max_items = query.max_items.unwrap_or(10).clamp(1, 20);

    let deployment_count: u32 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM deployments
             WHERE is_deleted = 0
               AND resource_id = ?1
               AND resource_type = ?2",
            rusqlite::params![query.node_id, profile.node_type],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count.max(0) as u32)
        .map_err(|err| err.to_string())?;

    let recent_audit_change_count: u32 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM audit_logs
             WHERE resource_id = ?1
               AND resource_type = ?2",
            rusqlite::params![query.node_id, profile.node_type],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count.max(0) as u32)
        .map_err(|err| err.to_string())?;

    let latest_audit = conn
        .query_row(
            "SELECT action, created_at
             FROM audit_logs
             WHERE resource_id = ?1
               AND resource_type = ?2
             ORDER BY created_at DESC
             LIMIT 1",
            rusqlite::params![query.node_id, profile.node_type],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(|err| err.to_string())?;

    let abnormal_status_count = if profile.status.as_deref() == Some("running") {
        0
    } else {
        1
    };
    let summary = TopologyTroubleshootReportSummaryV3 {
        abnormal_status_count,
        deployment_count,
        recent_audit_change_count,
        inbound_dependency_count: drilldown.inbound_edge_count,
        outbound_dependency_count: drilldown.outbound_edge_count,
    };

    let mut insights = vec![TopologyTaskInsight {
        kind: "status".to_string(),
        title: "节点状态".to_string(),
        description: profile
            .status
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        severity: if abnormal_status_count > 0 {
            "critical".to_string()
        } else {
            "info".to_string()
        },
        node_ids: vec![query.node_id.clone()],
        edge_ids: Vec::new(),
    }];

    insights.push(TopologyTaskInsight {
        kind: "deployment".to_string(),
        title: "部署摘要".to_string(),
        description: format!("deployment_count={}", deployment_count),
        severity: if deployment_count == 0 {
            "warning".to_string()
        } else {
            "info".to_string()
        },
        node_ids: vec![query.node_id.clone()],
        edge_ids: Vec::new(),
    });

    if let Some((action, created_at)) = latest_audit {
        insights.push(TopologyTaskInsight {
            kind: "audit".to_string(),
            title: "最近审计变更".to_string(),
            description: format!("{} @ {}", action, created_at),
            severity: "info".to_string(),
            node_ids: vec![query.node_id.clone()],
            edge_ids: Vec::new(),
        });
    }

    insights.push(TopologyTaskInsight {
        kind: "dependency".to_string(),
        title: "依赖摘要".to_string(),
        description: format!(
            "inbound={}, outbound={}",
            summary.inbound_dependency_count, summary.outbound_dependency_count
        ),
        severity: if summary.inbound_dependency_count + summary.outbound_dependency_count >= 3 {
            "warning".to_string()
        } else {
            "info".to_string()
        },
        node_ids: vec![query.node_id.clone()],
        edge_ids: Vec::new(),
    });

    if insights.len() > max_items {
        insights.truncate(max_items);
    }

    Ok(TopologyTroubleshootReportV3 {
        node_id: query.node_id.clone(),
        task_view,
        summary,
        insights,
    })
}

pub fn get_topology_paths_v3_inner(
    conn: &Connection,
    query: &TopologyPathsQuery,
) -> Result<TopologyPaths, String> {
    let max_results = query.max_results.unwrap_or(10).max(1);
    let result = find_paths_inner(conn, &query.source_id, &query.target_id, max_results)?;
    let paths = result
        .paths
        .into_iter()
        .map(|node_ids| TopologyPathItem {
            node_ids,
            score: None,
        })
        .collect::<Vec<_>>();
    Ok(TopologyPaths {
        total_count: paths.len() as u32,
        paths,
        truncated: result.truncated,
    })
}

pub fn get_topology_impact_v3_inner(
    conn: &Connection,
    query: &TopologyImpactQuery,
) -> Result<TopologyImpact, String> {
    let result = analyze_impact_inner(conn, &query.node_id)?;
    let severity = if result.total_count >= 8 || result.max_depth >= 4 {
        "critical"
    } else if result.total_count >= 2 || result.max_depth >= 2 {
        "warning"
    } else {
        "info"
    };

    Ok(TopologyImpact {
        affected_nodes: result
            .affected_nodes
            .iter()
            .map(to_affected_node_v3)
            .collect(),
        total_count: result.total_count,
        max_depth: result.max_depth,
        severity: severity.to_string(),
    })
}

pub fn get_topology_evidence_v3_inner(
    conn: &Connection,
    query: &TopologyEvidenceQuery,
) -> Result<TopologyEvidence, String> {
    let profile = load_node_profile(conn, &query.node_id)?;
    let max_items = query.max_items.unwrap_or(20).clamp(1, 50);
    let mut items: Vec<TopologyEvidenceItem> = Vec::new();

    push_evidence(
        &mut items,
        TopologyEvidenceItem {
            id: format!("profile:{}", profile.id),
            evidence_type: "profile".to_string(),
            title: "节点画像".to_string(),
            description: format!(
                "节点 {} ({})，环境={}，状态={}",
                profile.name,
                profile.node_type,
                profile.env,
                profile.status.as_deref().unwrap_or("unknown")
            ),
            source: Some("topology_v3".to_string()),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            node_id: Some(profile.id.clone()),
            edge_id: None,
        },
        max_items,
    );

    let outbound_count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM call_relations
             WHERE is_deleted = 0
               AND direction = 'upstream'
               AND owner_id = ?1",
            rusqlite::params![query.node_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    let inbound_count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM call_relations
             WHERE is_deleted = 0
               AND direction = 'upstream'
               AND peer_id = ?1",
            rusqlite::params![query.node_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    if outbound_count > 0 || inbound_count > 0 {
        push_evidence(
            &mut items,
            TopologyEvidenceItem {
                id: format!("dependency:{}", query.node_id),
                evidence_type: "dependency_summary".to_string(),
                title: "依赖关系摘要".to_string(),
                description: format!("outbound={}, inbound={}", outbound_count, inbound_count),
                source: Some("call_relations".to_string()),
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
                node_id: Some(query.node_id.clone()),
                edge_id: None,
            },
            max_items,
        );
    }

    let mut deployment_stmt = conn
        .prepare(
            "SELECT d.id, COALESCE(h.hostname, d.host_id), COALESCE(h.env, 'prod')
             FROM deployments d
             LEFT JOIN hosts h ON h.id = d.host_id AND h.is_deleted = 0
             WHERE d.is_deleted = 0
               AND d.resource_id = ?1
             ORDER BY d.created_at DESC
             LIMIT ?2",
        )
        .map_err(|err| err.to_string())?;
    let deployment_rows = deployment_stmt
        .query_map(rusqlite::params![query.node_id, max_items as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|err| err.to_string())?;

    for row in deployment_rows {
        let (deployment_id, host_name, host_env) = row.map_err(|err| err.to_string())?;
        push_evidence(
            &mut items,
            TopologyEvidenceItem {
                id: format!("deployment:{}", deployment_id),
                evidence_type: "deployment".to_string(),
                title: format!("部署记录 {}", deployment_id),
                description: format!("host={}, env={}", host_name, host_env),
                source: Some("deployments".to_string()),
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
                node_id: Some(query.node_id.clone()),
                edge_id: None,
            },
            max_items,
        );
        if items.len() >= max_items {
            break;
        }
    }

    let mut audit_stmt = conn
        .prepare(
            "SELECT action, created_at
             FROM audit_logs
             WHERE resource_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .map_err(|err| err.to_string())?;
    let audit_rows = audit_stmt
        .query_map(rusqlite::params![query.node_id, max_items as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|err| err.to_string())?;

    for row in audit_rows {
        let (action, created_at) = row.map_err(|err| err.to_string())?;
        push_evidence(
            &mut items,
            TopologyEvidenceItem {
                id: format!("audit:{}:{}", query.node_id, created_at),
                evidence_type: "audit".to_string(),
                title: format!("审计事件 {}", action),
                description: created_at.clone(),
                source: Some("audit_logs".to_string()),
                timestamp: Some(created_at),
                node_id: Some(query.node_id.clone()),
                edge_id: None,
            },
            max_items,
        );
        if items.len() >= max_items {
            break;
        }
    }

    Ok(TopologyEvidence {
        total: items.len() as u32,
        items,
    })
}

fn map_kind_counts(map: HashMap<String, u32>) -> Vec<TopologyKindCount> {
    let mut items: Vec<TopologyKindCount> = map
        .into_iter()
        .map(|(kind, count)| TopologyKindCount { kind, count })
        .collect();
    items.sort_by(|left, right| left.kind.cmp(&right.kind));
    items
}

fn to_lane_v3(lane: &TopologyLane) -> TopologyLaneV3 {
    TopologyLaneV3 {
        id: lane.id.clone(),
        label: lane.label.clone(),
        order: lane.order,
        node_count: lane.node_count,
        app_count: lane.app_count,
    }
}

fn to_node_v3(node: &TopologyNode) -> TopologyNodeV3 {
    TopologyNodeV3 {
        id: node.id.clone(),
        name: node.name.clone(),
        node_type: node.node_type.clone(),
        env: node.env.clone(),
        group_kind: node.group_kind.clone(),
        host_id: node.host_id.clone(),
        host_name: node.host_name.clone(),
        host_ip_display: node.host_ip_display.clone(),
        status: node.status.clone(),
        importance: node.importance,
        extra: node.extra.clone(),
    }
}

fn to_edge_v3(edge: &TopologyEdge) -> TopologyEdgeV3 {
    TopologyEdgeV3 {
        id: edge.id.clone(),
        source: edge.source.clone(),
        target: edge.target.clone(),
        edge_type: edge.edge_type.clone(),
        label: edge.label.clone(),
        strength: edge.strength,
        cross_env: edge.cross_env,
    }
}

fn to_env_count_v3(env_count: &TopologyEnvCount) -> TopologyEnvCountV3 {
    TopologyEnvCountV3 {
        env: env_count.env.clone(),
        count: env_count.count,
        app_count: env_count.app_count,
    }
}

fn to_kind_count_v3(kind_count: &TopologyKindCount) -> TopologyKindCountV3 {
    TopologyKindCountV3 {
        kind: kind_count.kind.clone(),
        count: kind_count.count,
    }
}

fn to_legend_stats_v3(
    stats: &TopologyLegendStats,
    edges: &[TopologyEdge],
    current_env: Option<String>,
) -> TopologyLegendStatsV3 {
    TopologyLegendStatsV3 {
        env_counts: stats.env_counts.iter().map(to_env_count_v3).collect(),
        node_type_counts: stats
            .node_type_counts
            .iter()
            .map(to_kind_count_v3)
            .collect(),
        edge_type_counts: stats
            .edge_type_counts
            .iter()
            .map(to_kind_count_v3)
            .collect(),
        application_service_count: stats.application_service_count,
        current_env,
        external_node_count: None,
        cross_env_edge_count: Some(edges.iter().filter(|edge| edge.cross_env).count() as u32),
    }
}

fn to_layout_hints_v3(layout_hints: &TopologyLayoutHints) -> TopologyLayoutHintsV3 {
    TopologyLayoutHintsV3 {
        lane_order: layout_hints.lane_order.clone(),
        default_collapsed_groups: layout_hints.default_collapsed_groups.clone(),
        high_density_mode: layout_hints.high_density_mode,
    }
}

fn to_affected_node_v3(node: &LegacyTopologyImpactNode) -> TopologyAffectedNodeV3 {
    TopologyAffectedNodeV3 {
        id: node.id.clone(),
        name: node.name.clone(),
        node_type: node.node_type.clone(),
        depth: node.depth,
    }
}

fn to_neighbor(node: &TopologyNode) -> TopologyNeighborV3 {
    TopologyNeighborV3 {
        id: node.id.clone(),
        name: node.name.clone(),
        node_type: node.node_type.clone(),
        env: node.env.clone(),
    }
}

fn push_evidence(
    evidence: &mut Vec<TopologyEvidenceItem>,
    item: TopologyEvidenceItem,
    max_items: usize,
) {
    if evidence.len() < max_items {
        evidence.push(item);
    }
}

#[derive(Debug, Clone)]
struct NodeProfile {
    id: String,
    name: String,
    node_type: String,
    env: String,
    status: Option<String>,
}

fn load_node_profile(conn: &Connection, node_id: &str) -> Result<NodeProfile, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, node_type, env, status
             FROM (
               SELECT id, name, 'application' AS node_type, env, status
               FROM applications
               WHERE is_deleted = 0
               UNION ALL
               SELECT id, name, 'middleware' AS node_type, env, NULL AS status
               FROM middlewares
               WHERE is_deleted = 0
               UNION ALL
               SELECT id, name, 'nginx' AS node_type, env, status
               FROM nginx_configs
               WHERE is_deleted = 0
             )
             WHERE id = ?1
             LIMIT 1",
        )
        .map_err(|err| err.to_string())?;

    stmt.query_row(rusqlite::params![node_id], |row| {
        Ok(NodeProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            node_type: row.get(2)?,
            env: row.get(3)?,
            status: row.get(4)?,
        })
    })
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => format!("Record not found: node_id={node_id}"),
        other => other.to_string(),
    })
}

#[tauri::command]
pub fn get_topology_snapshot_v3(
    pool: State<DbPool>,
    query: Option<TopologySnapshotQuery>,
) -> AppResult<TopologySnapshot> {
    let command = "get_topology_snapshot_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_snapshot_v3_inner(&conn, &query.unwrap_or_default())
        .map_err(|e| AppError::from_db_error(command, "build topology snapshot v3", e))
}

#[tauri::command]
pub fn get_topology_task_view_v3(
    pool: State<DbPool>,
    query: TopologyTaskViewQuery,
) -> AppResult<TopologyTaskView> {
    let command = "get_topology_task_view_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_task_view_v3_inner(&conn, &query)
        .map_err(|e| AppError::from_db_error(command, "build topology task view v3", e))
}

#[tauri::command]
pub fn get_topology_paths_v3(
    pool: State<DbPool>,
    query: TopologyPathsQuery,
) -> AppResult<TopologyPaths> {
    let command = "get_topology_paths_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_paths_v3_inner(&conn, &query)
        .map_err(|e| AppError::from_db_error(command, "find topology paths v3", e))
}

#[tauri::command]
pub fn get_topology_impact_v3(
    pool: State<DbPool>,
    query: TopologyImpactQuery,
) -> AppResult<TopologyImpact> {
    let command = "get_topology_impact_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_impact_v3_inner(&conn, &query)
        .map_err(|e| AppError::from_db_error(command, "analyze topology impact v3", e))
}

#[tauri::command]
pub fn get_topology_evidence_v3(
    pool: State<DbPool>,
    query: TopologyEvidenceQuery,
) -> AppResult<TopologyEvidence> {
    let command = "get_topology_evidence_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_evidence_v3_inner(&conn, &query)
        .map_err(|e| AppError::from_db_error(command, "collect topology evidence v3", e))
}

#[tauri::command]
pub fn get_topology_drilldown_v3(
    pool: State<DbPool>,
    query: TopologyDrilldownQuery,
) -> AppResult<TopologyDrilldown> {
    let command = "get_topology_drilldown_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_drilldown_v3_inner(&conn, &query)
        .map_err(|e| AppError::from_db_error(command, "build topology drilldown v3", e))
}

#[tauri::command]
pub fn get_topology_troubleshoot_report_v3(
    pool: State<DbPool>,
    query: TopologyTroubleshootReportV3Query,
) -> AppResult<TopologyTroubleshootReportV3> {
    let command = "get_topology_troubleshoot_report_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_troubleshoot_report_v3_inner(&conn, &query)
        .map_err(|e| AppError::from_db_error(command, "build topology troubleshoot report v3", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::audit::insert_audit_log;
    use crate::test_helpers::{
        insert_test_application, insert_test_dependency, insert_test_deployment, insert_test_host,
        setup_test_db,
    };

    fn setup_graph(conn: &Connection) {
        insert_test_application(conn, "A", "App-A", "prod");
        insert_test_application(conn, "B", "App-B", "prod");
        insert_test_application(conn, "C", "App-C", "prod");
        insert_test_application(conn, "D", "App-D", "prod");
        insert_test_application(conn, "E", "App-E", "test");

        insert_test_dependency(
            conn,
            "e1",
            "A",
            "application",
            "B",
            "application",
            "http_call",
        );
        insert_test_dependency(
            conn,
            "e2",
            "B",
            "application",
            "C",
            "application",
            "http_call",
        );
        insert_test_dependency(
            conn,
            "e3",
            "C",
            "application",
            "D",
            "application",
            "http_call",
        );
        insert_test_dependency(conn, "e4", "A", "application", "C", "application", "tcp");
        insert_test_dependency(
            conn,
            "e5",
            "B",
            "application",
            "E",
            "application",
            "http_call",
        );
    }

    fn update_application_status(conn: &Connection, id: &str, status: &str) {
        conn.execute(
            "UPDATE applications SET status = ?1 WHERE id = ?2",
            rusqlite::params![status, id],
        )
        .expect("update application status");
    }

    #[test]
    fn topology_v3_snapshot_should_filter_env_and_resolve_focus_node() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologySnapshotQuery {
            env: Some("prod".to_string()),
            task_view: Some("explore".to_string()),
            max_depth: Some(3),
            focus_node_id: Some("A".to_string()),
        };
        let snapshot = get_topology_snapshot_v3_inner(&conn, &query).expect("snapshot query");

        assert_eq!(snapshot.node_count, 4);
        assert_eq!(snapshot.edge_count, 4);
        assert!(snapshot.nodes.iter().all(|node| node.env == "prod"));
        assert_eq!(
            snapshot.focus_node.as_ref().map(|node| node.id.as_str()),
            Some("A")
        );
        assert_eq!(snapshot.meta.task_view, "explore");
    }

    #[test]
    fn topology_v3_snapshot_should_serialize_nested_contract_in_camel_case() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologySnapshotQuery {
            env: Some("prod".to_string()),
            task_view: Some("explore".to_string()),
            max_depth: Some(3),
            focus_node_id: Some("A".to_string()),
        };
        let snapshot = get_topology_snapshot_v3_inner(&conn, &query).expect("snapshot query");
        let value = serde_json::to_value(&snapshot).expect("serialize snapshot");

        let lane = &value["lanes"][0];
        assert!(lane.get("nodeCount").is_some());
        assert!(lane.get("node_count").is_none());
        assert!(lane.get("appCount").is_some());
        assert!(lane.get("app_count").is_none());

        let node = &value["nodes"][0];
        assert!(node.get("nodeType").is_some());
        assert!(node.get("node_type").is_none());
        assert!(node.get("groupKind").is_some());
        assert!(node.get("group_kind").is_none());

        let edge = &value["edges"][0];
        assert!(edge.get("edgeType").is_some());
        assert!(edge.get("edge_type").is_none());
        assert!(edge.get("crossEnv").is_some());
        assert!(edge.get("cross_env").is_none());

        let legend_stats = &value["legendStats"];
        assert!(legend_stats.get("envCounts").is_some());
        assert!(legend_stats.get("env_counts").is_none());
        assert!(legend_stats.get("nodeTypeCounts").is_some());
        assert!(legend_stats.get("node_type_counts").is_none());
        assert!(legend_stats.get("applicationServiceCount").is_some());
        assert!(legend_stats.get("application_service_count").is_none());

        let env_count = &legend_stats["envCounts"][0];
        assert!(env_count.get("appCount").is_some());
        assert!(env_count.get("app_count").is_none());

        let layout_hints = &value["layoutHints"];
        assert!(layout_hints.get("laneOrder").is_some());
        assert!(layout_hints.get("lane_order").is_none());
        assert!(layout_hints.get("defaultCollapsedGroups").is_some());
        assert!(layout_hints.get("default_collapsed_groups").is_none());
        assert!(layout_hints.get("highDensityMode").is_some());
        assert!(layout_hints.get("high_density_mode").is_none());

        let focus_node = &value["focusNode"];
        assert!(focus_node.get("nodeType").is_some());
        assert!(focus_node.get("node_type").is_none());
    }

    #[test]
    fn topology_v3_drilldown_should_include_upstream_and_downstream_neighbors() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyDrilldownQuery {
            node_id: "C".to_string(),
            task_view: Some("explore".to_string()),
            env: None,
            max_depth: None,
            direction: None,
        };
        let result = get_topology_drilldown_v3_inner(&conn, &query).expect("drilldown query");

        assert_eq!(result.node.id, "C");
        assert_eq!(result.inbound_edge_count, 2);
        assert_eq!(result.outbound_edge_count, 1);
        assert_eq!(result.upstream.len(), 2);
        assert_eq!(result.downstream.len(), 1);
    }

    #[test]
    fn topology_v3_task_view_should_return_non_empty_tasks() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyTaskViewQuery {
            task_view: Some("troubleshoot".to_string()),
            env: Some("prod".to_string()),
            max_depth: Some(3),
            focus_node_id: Some("C".to_string()),
            node_id: Some("C".to_string()),
        };
        let result = get_topology_task_view_v3_inner(&conn, &query).expect("task view query");

        assert_eq!(result.task_view, "troubleshoot");
        assert_eq!(result.snapshot.meta.task_view, "troubleshoot");
        assert!(!result.insights.is_empty());
        assert!(result.insights.iter().any(|item| item.kind == "dependency"));
    }

    #[test]
    fn topology_v3_paths_should_reuse_path_finder_and_report_truncation() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyPathsQuery {
            source_id: "A".to_string(),
            target_id: "D".to_string(),
            task_view: Some("explore".to_string()),
            env: None,
            max_depth: None,
            max_results: Some(1),
        };
        let result = get_topology_paths_v3_inner(&conn, &query).expect("paths query");

        assert_eq!(result.total_count, 1);
        assert!(result.truncated);
        assert_eq!(
            result.paths[0].node_ids.first().map(String::as_str),
            Some("A")
        );
    }

    #[test]
    fn topology_v3_impact_should_reuse_impact_analysis_and_add_severity() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyImpactQuery {
            node_id: "D".to_string(),
            task_view: Some("impact".to_string()),
            env: None,
            max_depth: Some(3),
        };
        let result = get_topology_impact_v3_inner(&conn, &query).expect("impact query");

        assert_eq!(result.total_count, 3);
        assert_eq!(result.max_depth, 2);
        assert_eq!(result.severity, "warning");
    }

    #[test]
    fn topology_v3_impact_should_serialize_affected_nodes_in_camel_case() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyImpactQuery {
            node_id: "D".to_string(),
            task_view: Some("impact".to_string()),
            env: None,
            max_depth: Some(3),
        };
        let result = get_topology_impact_v3_inner(&conn, &query).expect("impact query");
        let value = serde_json::to_value(&result).expect("serialize impact");

        let affected_node = &value["affectedNodes"][0];
        assert!(affected_node.get("nodeType").is_some());
        assert!(affected_node.get("node_type").is_none());
    }

    #[test]
    fn topology_v3_graph_should_assign_host_and_inherit_host_env() {
        let conn = setup_test_db();
        setup_graph(&conn);
        insert_test_host(&conn, "H1", "server1", "10.0.0.1");
        conn.execute("UPDATE hosts SET env = 'test' WHERE id = 'H1'", [])
            .expect("update host env");
        insert_test_deployment(&conn, "dep1", "A", "application", "H1");

        let graph = get_topology_graph_inner(&conn).expect("graph query");
        let node_a = graph
            .nodes
            .iter()
            .find(|node| node.id == "A")
            .expect("node A should exist");

        assert_eq!(node_a.host_id.as_deref(), Some("H1"));
        assert_eq!(node_a.host_name.as_deref(), Some("server1"));
        assert_eq!(node_a.host_ip_display.as_deref(), Some("10.0.0.1"));
        assert_eq!(node_a.env, "test");
    }

    #[test]
    fn topology_v3_graph_should_compute_cross_env_and_legend_stats() {
        let conn = setup_test_db();
        insert_test_application(&conn, "A", "App-A", "prod");
        insert_test_application(&conn, "B", "App-B", "test");
        insert_test_dependency(
            &conn,
            "e1",
            "A",
            "application",
            "B",
            "application",
            "http_call",
        );

        let graph = get_topology_graph_inner(&conn).expect("graph query");
        assert_eq!(graph.edges.len(), 1);
        assert!(graph.edges[0].cross_env);
        assert_eq!(graph.legend_stats.application_service_count, 2);

        let env_prod = graph
            .legend_stats
            .env_counts
            .iter()
            .find(|item| item.env == "prod")
            .expect("prod env should exist");
        let env_test = graph
            .legend_stats
            .env_counts
            .iter()
            .find(|item| item.env == "test")
            .expect("test env should exist");
        assert_eq!(env_prod.count, 1);
        assert_eq!(env_test.count, 1);
    }

    #[test]
    fn topology_v3_evidence_should_return_non_empty_structured_items() {
        let conn = setup_test_db();
        setup_graph(&conn);
        insert_test_host(&conn, "H1", "host-1", "10.0.0.1");
        insert_test_deployment(&conn, "dep-1", "A", "application", "H1");

        let query = TopologyEvidenceQuery {
            node_id: "A".to_string(),
            edge_id: None,
            related_node_id: None,
            task_view: Some("troubleshoot".to_string()),
            max_items: Some(10),
        };
        let result = get_topology_evidence_v3_inner(&conn, &query).expect("evidence query");

        assert!(result.total >= 1);
        assert!(!result.items.is_empty());
        assert!(result
            .items
            .iter()
            .any(|item| item.evidence_type == "profile"));
    }

    #[test]
    fn topology_v3_troubleshoot_report_should_flag_abnormal_status() {
        let conn = setup_test_db();
        setup_graph(&conn);
        update_application_status(&conn, "A", "stopped");

        let query = TopologyTroubleshootReportV3Query {
            node_id: "A".to_string(),
            task_view: None,
            env: Some("prod".to_string()),
            max_items: Some(10),
        };
        let result =
            get_topology_troubleshoot_report_v3_inner(&conn, &query).expect("report query");

        assert_eq!(result.task_view, "troubleshoot");
        assert_eq!(result.summary.abnormal_status_count, 1);
        assert!(result
            .insights
            .iter()
            .any(|item| item.kind == "status" && item.severity == "critical"));
    }

    #[test]
    fn topology_v3_troubleshoot_report_should_warn_when_no_deployments() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyTroubleshootReportV3Query {
            node_id: "A".to_string(),
            task_view: None,
            env: Some("prod".to_string()),
            max_items: Some(10),
        };
        let result =
            get_topology_troubleshoot_report_v3_inner(&conn, &query).expect("report query");

        assert_eq!(result.summary.deployment_count, 0);
        assert!(result
            .insights
            .iter()
            .any(|item| item.kind == "deployment" && item.severity == "warning"));
    }

    #[test]
    fn topology_v3_troubleshoot_report_should_include_recent_audit_change() {
        let conn = setup_test_db();
        setup_graph(&conn);
        insert_audit_log(
            &conn,
            "update_application",
            "application",
            "A",
            Some("App-A"),
            Some("{\"status\":\"running\"}"),
        )
        .expect("insert audit log");

        let query = TopologyTroubleshootReportV3Query {
            node_id: "A".to_string(),
            task_view: None,
            env: Some("prod".to_string()),
            max_items: Some(10),
        };
        let result =
            get_topology_troubleshoot_report_v3_inner(&conn, &query).expect("report query");

        assert_eq!(result.summary.recent_audit_change_count, 1);
        assert!(result.insights.iter().any(|item| item.kind == "audit"));
    }

    #[test]
    fn topology_v3_troubleshoot_report_should_summarize_dependency_counts() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyTroubleshootReportV3Query {
            node_id: "C".to_string(),
            task_view: None,
            env: Some("prod".to_string()),
            max_items: Some(10),
        };
        let result =
            get_topology_troubleshoot_report_v3_inner(&conn, &query).expect("report query");

        assert_eq!(result.summary.inbound_dependency_count, 2);
        assert_eq!(result.summary.outbound_dependency_count, 1);
        assert!(result
            .insights
            .iter()
            .any(|item| item.kind == "dependency" && item.description.contains("inbound=2")));
    }
}
