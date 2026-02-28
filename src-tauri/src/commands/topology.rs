use std::collections::{HashMap, HashSet, VecDeque};
use rusqlite::Connection;
use tauri::State;
use crate::db::DbPool;
use crate::models::topology::{
    TopologyNode, TopologyEdge, TopologyCombo, TopologyGraph,
    PathResult, AffectedNode, ImpactResult,
};

pub fn get_topology_graph_inner(conn: &Connection) -> Result<TopologyGraph, String> {
    // 1. Query all non-deleted hosts -> TopologyCombo
    let mut combos: Vec<TopologyCombo> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, hostname, ip_address, status FROM hosts WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(TopologyCombo {
                id: row.get(0)?,
                label: row.get(1)?,
                ip: row.get(2)?,
                status: row.get(3)?,
            })
        }).map_err(|e| e.to_string())?;
        for row in rows {
            combos.push(row.map_err(|e| e.to_string())?);
        }
    }

    // 2. Query applications -> TopologyNode
    let mut nodes: Vec<TopologyNode> = Vec::new();
    let mut node_index: HashMap<String, usize> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, name, type, address, port, tech_stack, env, status \
             FROM applications WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let app_type: Option<String> = row.get(2)?;
            let address: Option<String> = row.get(3)?;
            let port: Option<i64> = row.get(4)?;
            let tech_stack: Option<String> = row.get(5)?;
            let env: Option<String> = row.get(6)?;
            let status: Option<String> = row.get(7)?;

            let mut extra = serde_json::Map::new();
            if let Some(ref t) = app_type {
                extra.insert("type".to_string(), serde_json::Value::String(t.clone()));
            }
            if let Some(ref ts) = tech_stack {
                extra.insert("tech_stack".to_string(), serde_json::Value::String(ts.clone()));
            }
            if let Some(ref addr) = address {
                extra.insert("address".to_string(), serde_json::Value::String(addr.clone()));
            }
            if let Some(p) = port {
                extra.insert("port".to_string(), serde_json::json!(p));
            }

            Ok(TopologyNode {
                id,
                name,
                node_type: "application".to_string(),
                status,
                env,
                parent_id: None,
                extra: Some(serde_json::Value::Object(extra)),
            })
        }).map_err(|e| e.to_string())?;
        for row in rows {
            let node = row.map_err(|e| e.to_string())?;
            node_index.insert(node.id.clone(), nodes.len());
            nodes.push(node);
        }
    }

    // 3. Query middlewares -> TopologyNode
    {
        let mut stmt = conn.prepare(
            "SELECT id, name, category, type, address, port, version, env \
             FROM middlewares WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let category: Option<String> = row.get(2)?;
            let mw_type: Option<String> = row.get(3)?;
            let address: Option<String> = row.get(4)?;
            let port: Option<i64> = row.get(5)?;
            let version: Option<String> = row.get(6)?;
            let env: Option<String> = row.get(7)?;

            let mut extra = serde_json::Map::new();
            if let Some(ref c) = category {
                extra.insert("category".to_string(), serde_json::Value::String(c.clone()));
            }
            if let Some(ref t) = mw_type {
                extra.insert("type".to_string(), serde_json::Value::String(t.clone()));
            }
            if let Some(ref addr) = address {
                extra.insert("address".to_string(), serde_json::Value::String(addr.clone()));
            }
            if let Some(p) = port {
                extra.insert("port".to_string(), serde_json::json!(p));
            }
            if let Some(ref v) = version {
                extra.insert("version".to_string(), serde_json::Value::String(v.clone()));
            }

            Ok(TopologyNode {
                id,
                name,
                node_type: "middleware".to_string(),
                status: None,
                env,
                parent_id: None,
                extra: Some(serde_json::Value::Object(extra)),
            })
        }).map_err(|e| e.to_string())?;
        for row in rows {
            let node = row.map_err(|e| e.to_string())?;
            node_index.insert(node.id.clone(), nodes.len());
            nodes.push(node);
        }
    }

    // 4. Query nginx_configs -> TopologyNode
    {
        let mut stmt = conn.prepare(
            "SELECT id, name, listen_port, strategy, env, status \
             FROM nginx_configs WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let listen_port: Option<i64> = row.get(2)?;
            let strategy: Option<String> = row.get(3)?;
            let env: Option<String> = row.get(4)?;
            let status: Option<String> = row.get(5)?;

            let mut extra = serde_json::Map::new();
            if let Some(p) = listen_port {
                extra.insert("listen_port".to_string(), serde_json::json!(p));
            }
            if let Some(ref s) = strategy {
                extra.insert("strategy".to_string(), serde_json::Value::String(s.clone()));
            }

            Ok(TopologyNode {
                id,
                name,
                node_type: "nginx".to_string(),
                status,
                env,
                parent_id: None,
                extra: Some(serde_json::Value::Object(extra)),
            })
        }).map_err(|e| e.to_string())?;
        for row in rows {
            let node = row.map_err(|e| e.to_string())?;
            node_index.insert(node.id.clone(), nodes.len());
            nodes.push(node);
        }
    }

    // 5. Query deployments -> set parent_id (combo membership)
    {
        let mut stmt = conn.prepare(
            "SELECT resource_id, host_id FROM deployments WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            let (resource_id, host_id) = row.map_err(|e| e.to_string())?;
            if let Some(&idx) = node_index.get(&resource_id) {
                nodes[idx].parent_id = Some(host_id);
            }
        }
    }

    // 6. Query dependencies -> TopologyEdge
    let mut edges: Vec<TopologyEdge> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, source_id, target_id, relation_type, description \
             FROM dependencies WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(TopologyEdge {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                edge_type: row.get(3)?,
                label: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;
        for row in rows {
            edges.push(row.map_err(|e| e.to_string())?);
        }
    }

    Ok(TopologyGraph { nodes, edges, combos })
}

pub fn find_paths_inner(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    max_results: usize,
) -> Result<PathResult, String> {
    let max_depth: usize = 10;

    // Build adjacency list from dependencies
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT source_id, target_id FROM dependencies WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| e.to_string())?;
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

pub fn analyze_impact_inner(
    conn: &Connection,
    node_id: &str,
) -> Result<ImpactResult, String> {
    // Build reverse adjacency list (target -> sources, i.e. who depends on target)
    let mut reverse_adj: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT source_id, target_id FROM dependencies WHERE is_deleted = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| e.to_string())?;
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
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        }).map_err(|e| e.to_string())?;
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

                let (name, node_type) = name_map.get(dep_id)
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
pub fn get_topology_graph(pool: State<DbPool>) -> Result<TopologyGraph, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    get_topology_graph_inner(&conn)
}

#[tauri::command]
pub fn find_paths(
    pool: State<DbPool>,
    source_id: String,
    target_id: String,
    max_results: Option<usize>,
) -> Result<PathResult, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    find_paths_inner(&conn, &source_id, &target_id, max_results.unwrap_or(10))
}

#[tauri::command]
pub fn analyze_impact(
    pool: State<DbPool>,
    node_id: String,
) -> Result<ImpactResult, String> {
    let conn = pool.get().map_err(|e| format!("Pool error: {}", e))?;
    analyze_impact_inner(&conn, &node_id)
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

        insert_test_dependency(conn, "e1", "A", "application", "B", "application", "http_call");
        insert_test_dependency(conn, "e2", "B", "application", "C", "application", "http_call");
        insert_test_dependency(conn, "e3", "C", "application", "D", "application", "http_call");
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
    fn test_get_topology_graph_combo_assignment() {
        let conn = setup_test_db();
        setup_graph(&conn);
        insert_test_host(&conn, "H1", "server1", "10.0.0.1");
        insert_test_deployment(&conn, "dep1", "A", "application", "H1");

        let graph = get_topology_graph_inner(&conn).unwrap();
        assert_eq!(graph.combos.len(), 1);
        let node_a = graph.nodes.iter().find(|n| n.id == "A").unwrap();
        assert_eq!(node_a.parent_id.as_deref(), Some("H1"));
    }
}
