use crate::commands::topology::{analyze_impact_inner, find_paths_inner, get_topology_graph_inner};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::topology::{TopologyKindCount, TopologyNodeV2};
use crate::models::topology_v3::{
    TopologyDrilldownV3, TopologyDrilldownV3Query, TopologyEvidenceItemV3, TopologyEvidenceV3,
    TopologyEvidenceV3Query, TopologyImpactV3, TopologyImpactV3Query, TopologyNeighborV3,
    TopologyPathItemV3, TopologyPathsV3, TopologyPathsV3Query, TopologySnapshotMetaV3,
    TopologySnapshotV3, TopologySnapshotV3Query, TopologyTaskInsightV3, TopologyTaskViewV3,
    TopologyTaskViewV3Query,
};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use tauri::State;

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
    query: &TopologySnapshotV3Query,
) -> Result<TopologySnapshotV3, String> {
    let mut graph = get_topology_graph_inner(conn)?;
    if let Some(env_filter) = query
        .env
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let filtered_nodes: Vec<TopologyNodeV2> = graph
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
            .cloned()
    });
    let node_count = graph.nodes.len() as u32;
    let edge_count = graph.edges.len() as u32;
    let task_view = normalize_task_view(query.task_view.as_deref());
    let meta = TopologySnapshotMetaV3 {
        version: "3.0".to_string(),
        task_view,
        generated_at: chrono::Utc::now().to_rfc3339(),
        env: query.env.clone(),
        max_depth: query.max_depth,
        node_count,
        edge_count,
    };

    Ok(TopologySnapshotV3 {
        meta,
        lanes: graph.lanes,
        nodes: graph.nodes,
        edges: graph.edges,
        legend_stats: graph.legend_stats,
        layout_hints: graph.layout_hints,
        focus_node,
        node_count,
        edge_count,
    })
}

pub fn get_topology_drilldown_v3_inner(
    conn: &Connection,
    query: &TopologyDrilldownV3Query,
) -> Result<TopologyDrilldownV3, String> {
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

    Ok(TopologyDrilldownV3 {
        node,
        upstream,
        downstream,
        inbound_edge_count,
        outbound_edge_count,
    })
}

pub fn get_topology_task_view_v3_inner(
    conn: &Connection,
    query: &TopologyTaskViewV3Query,
) -> Result<TopologyTaskViewV3, String> {
    let task_view = normalize_task_view(query.task_view.as_deref());
    let snapshot = get_topology_snapshot_v3_inner(
        conn,
        &TopologySnapshotV3Query {
            env: query.env.clone(),
            task_view: Some(task_view.clone()),
            max_depth: query.max_depth,
            focus_node_id: query.focus_node_id.clone().or_else(|| query.node_id.clone()),
            include_evidence: None,
        },
    )?;

    let focus_id = query
        .node_id
        .clone()
        .or_else(|| query.focus_node_id.clone())
        .or_else(|| snapshot.nodes.first().map(|node| node.id.clone()));

    let mut insights: Vec<TopologyTaskInsightV3> = Vec::new();

    if let Some(node_id) = focus_id {
        let drilldown = get_topology_drilldown_v3_inner(
            conn,
            &TopologyDrilldownV3Query {
                node_id: node_id.clone(),
                task_view: Some(task_view.clone()),
                env: query.env.clone(),
                max_depth: query.max_depth,
                direction: None,
            },
        )?;

        insights.push(TopologyTaskInsightV3 {
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

        insights.push(TopologyTaskInsightV3 {
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

    let title = Some(match task_view.as_str() {
        "troubleshoot" => "故障排查视图",
        "impact" => "影响面视图",
        "change" => "变更评估视图",
        _ => "拓扑探索视图",
    }
    .to_string());
    let summary = Some(format!(
        "聚焦节点数={}, 关系线数={}",
        snapshot.node_count, snapshot.edge_count
    ));
    let meta = Some(snapshot.meta.clone());

    Ok(TopologyTaskViewV3 {
        task_view,
        snapshot,
        title,
        summary,
        insights,
        meta,
    })
}

pub fn get_topology_paths_v3_inner(
    conn: &Connection,
    query: &TopologyPathsV3Query,
) -> Result<TopologyPathsV3, String> {
    let max_results = query.max_results.unwrap_or(10).max(1);
    let result = find_paths_inner(conn, &query.source_id, &query.target_id, max_results)?;
    let paths = result
        .paths
        .into_iter()
        .map(|node_ids| TopologyPathItemV3 {
            node_ids,
            score: None,
        })
        .collect::<Vec<_>>();
    Ok(TopologyPathsV3 {
        total_count: paths.len() as u32,
        paths,
        truncated: result.truncated,
    })
}

pub fn get_topology_impact_v3_inner(
    conn: &Connection,
    query: &TopologyImpactV3Query,
) -> Result<TopologyImpactV3, String> {
    let result = analyze_impact_inner(conn, &query.node_id)?;
    let severity = if result.total_count >= 8 || result.max_depth >= 4 {
        "high"
    } else if result.total_count >= 2 || result.max_depth >= 2 {
        "medium"
    } else {
        "low"
    };

    Ok(TopologyImpactV3 {
        affected_nodes: result.affected_nodes,
        total_count: result.total_count,
        max_depth: result.max_depth,
        severity: severity.to_string(),
    })
}

pub fn get_topology_evidence_v3_inner(
    conn: &Connection,
    query: &TopologyEvidenceV3Query,
) -> Result<TopologyEvidenceV3, String> {
    let profile = load_node_profile(conn, &query.node_id)?;
    let max_items = query.max_items.unwrap_or(20).clamp(1, 50);
    let mut items: Vec<TopologyEvidenceItemV3> = Vec::new();

    push_evidence(
        &mut items,
        TopologyEvidenceItemV3 {
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
            TopologyEvidenceItemV3 {
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
            TopologyEvidenceItemV3 {
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
            TopologyEvidenceItemV3 {
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

    Ok(TopologyEvidenceV3 {
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

fn to_neighbor(node: &TopologyNodeV2) -> TopologyNeighborV3 {
    TopologyNeighborV3 {
        id: node.id.clone(),
        name: node.name.clone(),
        node_type: node.node_type.clone(),
        env: node.env.clone(),
    }
}

fn push_evidence(
    evidence: &mut Vec<TopologyEvidenceItemV3>,
    item: TopologyEvidenceItemV3,
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
    query: Option<TopologySnapshotV3Query>,
) -> AppResult<TopologySnapshotV3> {
    let command = "get_topology_snapshot_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_snapshot_v3_inner(&conn, &query.unwrap_or_default())
        .map_err(|e| AppError::from_db_error(command, "build topology snapshot v3", e))
}

#[tauri::command]
pub fn get_topology_drilldown_v3(
    pool: State<DbPool>,
    query: TopologyDrilldownV3Query,
) -> AppResult<TopologyDrilldownV3> {
    let command = "get_topology_drilldown_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_drilldown_v3_inner(&conn, &query)
        .map_err(|e| AppError::from_db_error(command, "build topology drilldown v3", e))
}

#[tauri::command]
pub fn get_topology_task_view_v3(
    pool: State<DbPool>,
    query: TopologyTaskViewV3Query,
) -> AppResult<TopologyTaskViewV3> {
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
    query: TopologyPathsV3Query,
) -> AppResult<TopologyPathsV3> {
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
    query: TopologyImpactV3Query,
) -> AppResult<TopologyImpactV3> {
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
    query: TopologyEvidenceV3Query,
) -> AppResult<TopologyEvidenceV3> {
    let command = "get_topology_evidence_v3";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {e}")))?;
    get_topology_evidence_v3_inner(&conn, &query)
        .map_err(|e| AppError::from_db_error(command, "collect topology evidence v3", e))
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn topology_v3_snapshot_should_filter_env_and_resolve_focus_node() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologySnapshotV3Query {
            env: Some("prod".to_string()),
            task_view: Some("explore".to_string()),
            max_depth: Some(3),
            focus_node_id: Some("A".to_string()),
            include_evidence: None,
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
    fn topology_v3_drilldown_should_include_upstream_and_downstream_neighbors() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyDrilldownV3Query {
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

        let query = TopologyTaskViewV3Query {
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
        assert!(result
            .insights
            .iter()
            .any(|item| item.kind == "dependency"));
    }

    #[test]
    fn topology_v3_paths_should_reuse_path_finder_and_report_truncation() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyPathsV3Query {
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
        assert_eq!(result.paths[0].node_ids.first().map(String::as_str), Some("A"));
    }

    #[test]
    fn topology_v3_impact_should_reuse_impact_analysis_and_add_severity() {
        let conn = setup_test_db();
        setup_graph(&conn);

        let query = TopologyImpactV3Query {
            node_id: "D".to_string(),
            task_view: Some("impact".to_string()),
            env: None,
            max_depth: Some(3),
        };
        let result = get_topology_impact_v3_inner(&conn, &query).expect("impact query");

        assert_eq!(result.total_count, 3);
        assert_eq!(result.max_depth, 2);
        assert_eq!(result.severity, "medium");
    }

    #[test]
    fn topology_v3_evidence_should_return_non_empty_structured_items() {
        let conn = setup_test_db();
        setup_graph(&conn);
        insert_test_host(&conn, "H1", "host-1", "10.0.0.1");
        insert_test_deployment(&conn, "dep-1", "A", "application", "H1");

        let query = TopologyEvidenceV3Query {
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
}
