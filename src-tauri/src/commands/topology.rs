use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::nginx_config::NginxEndpoint;
use crate::models::topology::{
    AffectedNode, ImpactResult, PathResult, TopologyEdgeV2, TopologyEnvCount, TopologyGraphV2,
    TopologyKindCount, TopologyLane, TopologyLayoutHints, TopologyLegendStats, TopologyNodeV2,
};
use rusqlite::Connection;
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

pub fn get_topology_graph_inner(conn: &Connection) -> Result<TopologyGraphV2, String> {
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

    let mut nodes: Vec<TopologyNodeV2> = Vec::new();
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

                Ok(TopologyNodeV2 {
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

                Ok(TopologyNodeV2 {
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

                Ok(TopologyNodeV2 {
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

    let mut edge_map: HashMap<(String, String, String), TopologyEdgeV2> = HashMap::new();
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
                .or_insert_with(|| TopologyEdgeV2 {
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

    let mut edges: Vec<TopologyEdgeV2> = edge_map.into_values().collect();
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

    Ok(TopologyGraphV2 {
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
) -> Result<PathResult, String> {
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

    Ok(PathResult { paths, truncated })
}

pub fn analyze_impact_inner(conn: &Connection, node_id: &str) -> Result<ImpactResult, String> {
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
    let mut affected: Vec<AffectedNode> = Vec::new();
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

                affected.push(AffectedNode {
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

    Ok(ImpactResult {
        affected_nodes: affected,
        total_count,
        max_depth,
    })
}

#[tauri::command]
pub fn get_topology_graph(pool: State<DbPool>) -> AppResult<TopologyGraphV2> {
    let command = "get_topology_graph";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    get_topology_graph_inner(&conn)
        .map_err(|e| AppError::from_db_error(command, "build topology graph", e))
}

#[tauri::command]
pub fn find_paths(
    pool: State<DbPool>,
    source_id: String,
    target_id: String,
    max_results: Option<usize>,
) -> AppResult<PathResult> {
    let command = "find_paths";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    find_paths_inner(&conn, &source_id, &target_id, max_results.unwrap_or(10))
        .map_err(|e| AppError::from_db_error(command, "find dependency paths", e))
}

#[tauri::command]
pub fn analyze_impact(pool: State<DbPool>, node_id: String) -> AppResult<ImpactResult> {
    let command = "analyze_impact";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    analyze_impact_inner(&conn, &node_id)
        .map_err(|e| AppError::from_db_error(command, "analyze impact", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    /// Set up test graph: A -> B -> C -> D, A -> C (shortcut)
    fn setup_graph(conn: &rusqlite::Connection) {
        insert_test_application(conn, "A", "App-A", "prod");
        insert_test_application(conn, "B", "App-B", "prod");
        insert_test_application(conn, "C", "App-C", "prod");
        insert_test_application(conn, "D", "App-D", "prod");

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
    }

    // --- find_paths tests ---
    #[test]
    fn test_find_paths_direct() {
        let conn = setup_test_db();
        setup_graph(&conn);
        let result = find_paths_inner(&conn, "A", "B", 10).unwrap();
        assert_eq!(result.paths.len(), 1);
        assert_eq!(result.paths[0], vec!["A", "B"]);
        assert!(!result.truncated);
    }

    #[test]
    fn test_find_paths_multiple() {
        let conn = setup_test_db();
        setup_graph(&conn);
        let result = find_paths_inner(&conn, "A", "C", 10).unwrap();
        // Two paths: A->B->C and A->C
        assert_eq!(result.paths.len(), 2);
    }

    #[test]
    fn test_find_paths_none_reverse() {
        let conn = setup_test_db();
        setup_graph(&conn);
        // No path from D to A (reverse direction)
        let result = find_paths_inner(&conn, "D", "A", 10).unwrap();
        assert_eq!(result.paths.len(), 0);
    }

    #[test]
    fn test_find_paths_max_results_truncation() {
        let conn = setup_test_db();
        setup_graph(&conn);
        let result = find_paths_inner(&conn, "A", "D", 1).unwrap();
        assert_eq!(result.paths.len(), 1);
        // There are more paths available, so truncated should be true
        assert!(result.truncated);
    }

    #[test]
    fn test_find_paths_same_node() {
        let conn = setup_test_db();
        setup_graph(&conn);
        let result = find_paths_inner(&conn, "A", "A", 10).unwrap();
        // No path from A to itself (path.len() must be > 1)
        assert_eq!(result.paths.len(), 0);
    }

    // --- analyze_impact tests ---
    #[test]
    fn test_analyze_impact_leaf_node() {
        let conn = setup_test_db();
        setup_graph(&conn);
        // A is a root (nothing depends on A via reverse adjacency from source->target)
        // Actually: D is a leaf (no one depends on D as target except C->D, so reverse: D -> [C])
        // Wait: reverse adj is target -> sources. So reverse_adj["B"] = ["A"], reverse_adj["C"] = ["B", "A"], reverse_adj["D"] = ["C"]
        // Impact of A: who depends on A? reverse_adj["A"] = nothing. So leaf/root impact = 0
        let result = analyze_impact_inner(&conn, "A").unwrap();
        assert_eq!(result.total_count, 0);
        assert_eq!(result.max_depth, 0);
    }

    #[test]
    fn test_analyze_impact_middle_node() {
        let conn = setup_test_db();
        setup_graph(&conn);
        // If C goes down: reverse_adj["C"] = ["B", "A"] (B->C and A->C edges).
        // From B: reverse_adj["B"] = ["A"], but A already visited.
        let result = analyze_impact_inner(&conn, "C").unwrap();
        assert_eq!(result.total_count, 2); // B and A affected
    }

    #[test]
    fn test_analyze_impact_target_node() {
        let conn = setup_test_db();
        setup_graph(&conn);
        // If D goes down: reverse_adj["D"] = ["C"]
        // From C: reverse_adj["C"] = ["B", "A"]
        // From B: reverse_adj["B"] = ["A"], A already visited
        let result = analyze_impact_inner(&conn, "D").unwrap();
        assert_eq!(result.total_count, 3); // C, B, A all affected
        assert!(result.max_depth >= 2);
    }

    // --- get_topology_graph tests ---
    #[test]
    fn test_get_topology_graph_node_count() {
        let conn = setup_test_db();
        setup_graph(&conn);
        insert_test_middleware(&conn, "M1", "Redis", "cache");
        insert_test_nginx_config(&conn, "N1", "nginx-lb");

        let graph = get_topology_graph_inner(&conn).unwrap();
        // 4 applications + 1 middleware + 1 nginx = 6 nodes
        assert_eq!(graph.nodes.len(), 6);
    }

    #[test]
    fn test_get_topology_graph_edge_count() {
        let conn = setup_test_db();
        setup_graph(&conn);
        let graph = get_topology_graph_inner(&conn).unwrap();
        assert_eq!(graph.edges.len(), 4); // e1, e2, e3, e4
    }

    #[test]
    fn test_get_topology_graph_should_always_return_three_lanes() {
        let conn = setup_test_db();
        let graph = get_topology_graph_inner(&conn).unwrap();

        assert_eq!(graph.lanes.len(), 3);
        assert_eq!(graph.lanes[0].id, "prod");
        assert_eq!(graph.lanes[1].id, "test");
        assert_eq!(graph.lanes[2].id, "dev");
    }

    #[test]
    fn test_get_topology_graph_should_assign_host_and_inherit_host_env() {
        let conn = setup_test_db();
        setup_graph(&conn);
        insert_test_host(&conn, "H1", "server1", "10.0.0.1");
        conn.execute("UPDATE hosts SET env = 'test' WHERE id = 'H1'", [])
            .expect("update host env");
        insert_test_deployment(&conn, "dep1", "A", "application", "H1");

        let graph = get_topology_graph_inner(&conn).unwrap();
        let node_a = graph
            .nodes
            .iter()
            .find(|node| node.id == "A")
            .expect("node A should exist");
        assert_eq!(node_a.host_id.as_deref(), Some("H1"));
        assert_eq!(node_a.host_name.as_deref(), Some("server1"));
        assert_eq!(node_a.host_ip_display.as_deref(), Some("10.0.0.1"));
        assert_eq!(node_a.env, "test");

        let lane_test = graph
            .lanes
            .iter()
            .find(|lane| lane.id == "test")
            .expect("test lane should exist");
        assert!(lane_test.node_count >= 1);
    }

    #[test]
    fn test_get_topology_graph_should_set_host_name_without_ip_when_no_binding() {
        let conn = setup_test_db();
        setup_graph(&conn);
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO hosts (id, hostname, env, status, is_deleted, created_at, updated_at)
             VALUES ('H2', 'server2', 'prod', 'running', 0, ?1, ?2)",
            rusqlite::params![now, now],
        )
        .expect("insert host without ip binding");
        insert_test_deployment(&conn, "dep2", "B", "application", "H2");

        let graph = get_topology_graph_inner(&conn).unwrap();
        let node_b = graph
            .nodes
            .iter()
            .find(|node| node.id == "B")
            .expect("node B should exist");
        assert_eq!(node_b.host_id.as_deref(), Some("H2"));
        assert_eq!(node_b.host_name.as_deref(), Some("server2"));
        assert_eq!(node_b.host_ip_display, None);
    }

    #[test]
    fn test_get_topology_graph_should_keep_host_meta_empty_when_host_missing() {
        let conn = setup_test_db();
        setup_graph(&conn);
        insert_test_deployment(&conn, "dep3", "C", "application", "H-MISSING");

        let graph = get_topology_graph_inner(&conn).unwrap();
        let node_c = graph
            .nodes
            .iter()
            .find(|node| node.id == "C")
            .expect("node C should exist");
        assert_eq!(node_c.host_id.as_deref(), Some("H-MISSING"));
        assert_eq!(node_c.host_name, None);
        assert_eq!(node_c.host_ip_display, None);
        assert_eq!(node_c.env, "prod");
    }

    #[test]
    fn test_get_topology_graph_should_compute_cross_env_and_legend_stats() {
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

        let graph = get_topology_graph_inner(&conn).unwrap();
        assert_eq!(graph.edges.len(), 1);
        assert!(graph.edges[0].cross_env);
        assert_eq!(graph.edges[0].strength, 1);
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
    fn test_get_topology_graph_should_enable_high_density_mode() {
        let conn = setup_test_db();
        for index in 0..810 {
            insert_test_application(
                &conn,
                &format!("APP-{index}"),
                &format!("app-{index}"),
                "prod",
            );
        }

        let graph = get_topology_graph_inner(&conn).unwrap();
        assert!(graph.layout_hints.high_density_mode);
    }
}
