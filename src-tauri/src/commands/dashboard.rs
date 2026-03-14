use std::collections::BTreeMap;
use tauri::State;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::dashboard::{
    DashboardCoverage, DashboardHealth, DashboardOverview, DashboardRecentChange,
    DashboardRiskItem, DashboardTotals, EnvCount,
};

fn count_active(conn: &rusqlite::Connection, table: &str) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {} WHERE is_deleted = 0", table),
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count as u64)
}

fn count_abnormal(conn: &rusqlite::Connection, table: &str) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM {} WHERE is_deleted = 0 AND status IN ('stopped', 'maintenance')",
            table
        ),
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count as u64)
}

fn count_deployed_by_type(
    conn: &rusqlite::Connection,
    resource_type: &str,
) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(DISTINCT resource_id)
         FROM deployments
         WHERE is_deleted = 0 AND resource_type = ?1",
        rusqlite::params![resource_type],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count as u64)
}

fn count_undeployed_by_type(
    conn: &rusqlite::Connection,
    table: &str,
    resource_type: &str,
) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*)
             FROM {table} r
             WHERE r.is_deleted = 0
               AND NOT EXISTS (
                 SELECT 1
                 FROM deployments d
                 WHERE d.is_deleted = 0
                   AND d.resource_type = ?1
                   AND d.resource_id = r.id
               )"
        ),
        rusqlite::params![resource_type],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count as u64)
}

fn count_related_resources(conn: &rusqlite::Connection) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*)
         FROM (
           SELECT DISTINCT owner_type, owner_id
           FROM call_relations
           WHERE is_deleted = 0
             AND owner_type IN ('service', 'middleware', 'nginx')
         )",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count as u64)
}

fn count_dependency_total(conn: &rusqlite::Connection) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM call_relations WHERE is_deleted = 0 AND direction = 'upstream'",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count as u64)
}

fn query_env_distribution(conn: &rusqlite::Connection) -> Result<Vec<EnvCount>, rusqlite::Error> {
    let sql = "SELECT env, COUNT(*) AS cnt
               FROM (
                 SELECT env FROM hosts WHERE is_deleted = 0
                 UNION ALL
                 SELECT env FROM services WHERE is_deleted = 0
                 UNION ALL
                 SELECT env FROM middlewares WHERE is_deleted = 0
                 UNION ALL
                 SELECT env FROM nginx_configs WHERE is_deleted = 0
               )
               GROUP BY env
               ORDER BY cnt DESC";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(EnvCount {
            env: row.get(0)?,
            count: row.get::<_, i64>(1)? as u64,
        })
    })?;

    Ok(rows.filter_map(|item| item.ok()).collect())
}

fn query_recent_changes(
    conn: &rusqlite::Connection,
    limit: u64,
) -> Result<Vec<DashboardRecentChange>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, action, resource_type, resource_id, resource_name, created_at
         FROM audit_logs
         ORDER BY created_at DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
        Ok(DashboardRecentChange {
            id: row.get(0)?,
            action: row.get(1)?,
            resource_type: row.get(2)?,
            resource_id: row.get(3)?,
            resource_name: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    Ok(rows.filter_map(|item| item.ok()).collect())
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64
}

fn round_1_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn percent(numerator: u64, denominator: u64) -> f64 {
    round_1_decimal(ratio(numerator, denominator) * 100.0)
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "critical" => 0,
        "warning" => 1,
        _ => 2,
    }
}

fn make_filter_map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

fn build_risk_items(
    totals: &DashboardTotals,
    coverage: &DashboardCoverage,
) -> Vec<DashboardRiskItem> {
    let mut items: Vec<DashboardRiskItem> = Vec::new();

    if totals.host_abnormal > 0 {
        items.push(DashboardRiskItem {
            key: "host_abnormal".to_string(),
            label: "异常服务器".to_string(),
            count: totals.host_abnormal,
            severity: "critical".to_string(),
            target_route: "Hosts".to_string(),
            target_filters: make_filter_map(&[("status", "stopped,maintenance")]),
        });
    }

    if totals.service_abnormal > 0 {
        items.push(DashboardRiskItem {
            key: "service_abnormal".to_string(),
            label: "异常服务".to_string(),
            count: totals.service_abnormal,
            severity: "critical".to_string(),
            target_route: "Services".to_string(),
            target_filters: make_filter_map(&[("status", "stopped,maintenance")]),
        });
    }

    if totals.nginx_abnormal > 0 {
        items.push(DashboardRiskItem {
            key: "nginx_abnormal".to_string(),
            label: "异常负载均衡".to_string(),
            count: totals.nginx_abnormal,
            severity: "critical".to_string(),
            target_route: "NginxConfigs".to_string(),
            target_filters: make_filter_map(&[("status", "stopped,maintenance")]),
        });
    }

    if coverage.undeployed_total > 0 {
        items.push(DashboardRiskItem {
            key: "undeployed_resources".to_string(),
            label: "未部署资源".to_string(),
            count: coverage.undeployed_total,
            severity: "warning".to_string(),
            target_route: "Topology".to_string(),
            target_filters: BTreeMap::new(),
        });
    }

    if coverage.isolated_total > 0 {
        items.push(DashboardRiskItem {
            key: "isolated_resources".to_string(),
            label: "孤立关系资源".to_string(),
            count: coverage.isolated_total,
            severity: "warning".to_string(),
            target_route: "Topology".to_string(),
            target_filters: BTreeMap::new(),
        });
    }

    items.sort_by(|left, right| {
        severity_rank(&left.severity)
            .cmp(&severity_rank(&right.severity))
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.key.cmp(&right.key))
    });

    items
}

fn get_dashboard_overview_inner(
    command: &str,
    conn: &rusqlite::Connection,
) -> AppResult<DashboardOverview> {
    let host_total = count_active(conn, "hosts")
        .map_err(|e| AppError::from_db_error(command, "统计主机总数", e))?;
    let host_abnormal = count_abnormal(conn, "hosts")
        .map_err(|e| AppError::from_db_error(command, "统计主机异常数量", e))?;
    let service_total = count_active(conn, "services")
        .map_err(|e| AppError::from_db_error(command, "统计服务总数", e))?;
    let service_abnormal = count_abnormal(conn, "services")
        .map_err(|e| AppError::from_db_error(command, "统计服务异常数量", e))?;
    let middleware_total = count_active(conn, "middlewares")
        .map_err(|e| AppError::from_db_error(command, "统计中间件总数", e))?;
    let nginx_total = count_active(conn, "nginx_configs")
        .map_err(|e| AppError::from_db_error(command, "统计网关总数", e))?;
    let nginx_abnormal = count_abnormal(conn, "nginx_configs")
        .map_err(|e| AppError::from_db_error(command, "统计网关异常数量", e))?;
    let deployment_total = count_active(conn, "deployments")
        .map_err(|e| AppError::from_db_error(command, "统计部署总数", e))?;
    let dependency_total = count_dependency_total(conn)
        .map_err(|e| AppError::from_db_error(command, "统计依赖总数", e))?;

    let deployed_service_total = count_deployed_by_type(conn, "service")
        .map_err(|e| AppError::from_db_error(command, "统计已部署服务数量", e))?;
    let deployed_middleware_total = count_deployed_by_type(conn, "middleware")
        .map_err(|e| AppError::from_db_error(command, "统计已部署中间件数量", e))?;
    let deployed_nginx_total = count_deployed_by_type(conn, "nginx")
        .map_err(|e| AppError::from_db_error(command, "统计已部署网关数量", e))?;

    let undeployed_service_total =
        count_undeployed_by_type(conn, "services", "service")
            .map_err(|e| AppError::from_db_error(command, "统计未部署服务数量", e))?;
    let undeployed_middleware_total =
        count_undeployed_by_type(conn, "middlewares", "middleware")
            .map_err(|e| AppError::from_db_error(command, "统计未部署中间件数量", e))?;
    let undeployed_nginx_total = count_undeployed_by_type(conn, "nginx_configs", "nginx")
        .map_err(|e| AppError::from_db_error(command, "统计未部署网关数量", e))?;

    let env_distribution = query_env_distribution(conn)
        .map_err(|e| AppError::from_db_error(command, "统计环境分布", e))?;
    let recent_changes = query_recent_changes(conn, 20)
        .map_err(|e| AppError::from_db_error(command, "读取最近变更", e))?;
    let related_total = count_related_resources(conn)
        .map_err(|e| AppError::from_db_error(command, "统计关联资源数量", e))?;

    let totals = DashboardTotals {
        host_total,
        host_abnormal,
        service_total,
        service_abnormal,
        middleware_total,
        nginx_total,
        nginx_abnormal,
        deployment_total,
        dependency_total,
    };

    let abnormal_total = host_abnormal + service_abnormal + nginx_abnormal;
    let status_trackable_total = host_total + service_total + nginx_total;
    let deployable_total = service_total + middleware_total + nginx_total;
    let deployed_total =
        deployed_service_total + deployed_middleware_total + deployed_nginx_total;
    let undeployed_total =
        undeployed_service_total + undeployed_middleware_total + undeployed_nginx_total;
    let relatable_total = deployable_total;
    let isolated_total = relatable_total.saturating_sub(related_total);

    let abnormal_ratio = ratio(abnormal_total, status_trackable_total);
    let undeployed_ratio = ratio(undeployed_total, deployable_total);
    let isolated_ratio = ratio(isolated_total, relatable_total);
    let penalty = abnormal_ratio * 60.0 + undeployed_ratio * 25.0 + isolated_ratio * 15.0;
    let score = round_1_decimal((100.0 - penalty).clamp(0.0, 100.0));

    let health = DashboardHealth {
        abnormal_total,
        abnormal_rate: percent(abnormal_total, status_trackable_total),
        score,
    };

    let coverage = DashboardCoverage {
        deployable_total,
        deployed_total,
        undeployed_total,
        deployment_coverage: percent(deployed_total, deployable_total),
        relatable_total,
        related_total,
        isolated_total,
        relation_coverage: percent(related_total, relatable_total),
        undeployed_service_total,
        undeployed_middleware_total,
        undeployed_nginx_total,
    };

    let risk_items = build_risk_items(&totals, &coverage);

    Ok(DashboardOverview {
        totals,
        health,
        coverage,
        env_distribution,
        risk_items,
        recent_changes,
    })
}

#[tauri::command]
pub fn get_dashboard_overview(pool: State<DbPool>) -> AppResult<DashboardOverview> {
    let command = "get_dashboard_overview";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    get_dashboard_overview_inner(command, &conn)
}

#[cfg(test)]
mod tests {
    use super::get_dashboard_overview_inner;
    use crate::test_helpers::{
        insert_test_service, insert_test_dependency, insert_test_deployment, insert_test_host,
        insert_test_middleware, insert_test_nginx_config, setup_test_db,
    };

    fn insert_audit_log(
        conn: &rusqlite::Connection,
        id: &str,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        resource_name: &str,
        created_at: &str,
    ) {
        conn.execute(
            "INSERT INTO audit_logs (id, action, resource_type, resource_id, resource_name, changes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6)",
            rusqlite::params![id, action, resource_type, resource_id, resource_name, created_at],
        )
        .expect("insert audit log should succeed");
    }

    #[test]
    fn get_dashboard_overview_inner_should_return_zeroed_summary_for_empty_database() {
        let conn = setup_test_db();
        let overview = get_dashboard_overview_inner("test", &conn).expect("query overview");

        assert_eq!(overview.totals.host_total, 0);
        assert_eq!(overview.totals.service_total, 0);
        assert_eq!(overview.coverage.deployable_total, 0);
        assert_eq!(overview.coverage.undeployed_total, 0);
        assert_eq!(overview.health.abnormal_total, 0);
        assert_eq!(overview.health.abnormal_rate, 0.0);
        assert_eq!(overview.health.score, 100.0);
        assert!(overview.risk_items.is_empty());
        assert!(overview.recent_changes.is_empty());
    }

    #[test]
    fn get_dashboard_overview_inner_should_aggregate_health_coverage_and_risks() {
        let conn = setup_test_db();

        insert_test_host(&conn, "host-1", "host-1", "10.0.0.1");
        insert_test_host(&conn, "host-2", "host-2", "10.0.0.2");
        conn.execute(
            "UPDATE hosts SET status = 'stopped' WHERE id = 'host-2'",
            [],
        )
        .expect("update host status");

        insert_test_service(&conn, "app-1", "app-1", "prod", "");
        insert_test_service(&conn, "app-2", "app-2", "prod", "");
        conn.execute(
            "UPDATE services SET status = 'maintenance' WHERE id = 'app-2'",
            [],
        )
        .expect("update application status");

        insert_test_middleware(&conn, "mw-1", "mw-1", "cache");
        insert_test_middleware(&conn, "mw-2", "mw-2", "database");

        insert_test_nginx_config(&conn, "nginx-1", "nginx-1");
        insert_test_nginx_config(&conn, "nginx-2", "nginx-2");
        conn.execute(
            "UPDATE nginx_configs SET status = 'stopped' WHERE id = 'nginx-2'",
            [],
        )
        .expect("update nginx status");

        insert_test_deployment(&conn, "dep-1", "app-1", "service", "host-1");
        insert_test_deployment(&conn, "dep-2", "mw-1", "middleware", "host-1");
        insert_test_deployment(&conn, "dep-3", "nginx-1", "nginx", "host-1");

        insert_test_dependency(
            &conn,
            "rel-1",
            "app-1",
            "service",
            "mw-1",
            "middleware",
            "http_call",
        );

        insert_audit_log(
            &conn,
            "log-old",
            "create",
            "service",
            "app-1",
            "app-1",
            "2026-03-01T09:00:00.000Z",
        );
        insert_audit_log(
            &conn,
            "log-new",
            "update",
            "middleware",
            "mw-1",
            "mw-1",
            "2026-03-01T10:00:00.000Z",
        );

        let overview = get_dashboard_overview_inner("test", &conn).expect("query overview");

        assert_eq!(overview.totals.host_total, 2);
        assert_eq!(overview.totals.host_abnormal, 1);
        assert_eq!(overview.totals.service_total, 2);
        assert_eq!(overview.totals.service_abnormal, 1);
        assert_eq!(overview.totals.middleware_total, 2);
        assert_eq!(overview.totals.nginx_total, 2);
        assert_eq!(overview.totals.nginx_abnormal, 1);
        assert_eq!(overview.totals.deployment_total, 3);
        assert_eq!(overview.totals.dependency_total, 1);

        assert_eq!(overview.health.abnormal_total, 3);
        assert!((overview.health.abnormal_rate - 50.0).abs() < 0.001);
        assert!((overview.health.score - 47.5).abs() < 0.001);

        assert_eq!(overview.coverage.deployable_total, 6);
        assert_eq!(overview.coverage.deployed_total, 3);
        assert_eq!(overview.coverage.undeployed_total, 3);
        assert!((overview.coverage.deployment_coverage - 50.0).abs() < 0.001);
        assert_eq!(overview.coverage.related_total, 2);
        assert_eq!(overview.coverage.isolated_total, 4);
        assert!((overview.coverage.relation_coverage - 33.3).abs() < 0.001);
        assert_eq!(overview.coverage.undeployed_service_total, 1);
        assert_eq!(overview.coverage.undeployed_middleware_total, 1);
        assert_eq!(overview.coverage.undeployed_nginx_total, 1);

        assert_eq!(overview.recent_changes.len(), 2);
        assert_eq!(overview.recent_changes[0].id, "log-new");
        assert_eq!(overview.recent_changes[1].id, "log-old");

        assert!(overview
            .risk_items
            .iter()
            .any(|item| item.key == "undeployed_resources" && item.count == 3));
        assert!(overview
            .risk_items
            .iter()
            .any(|item| item.key == "isolated_resources" && item.count == 4));
        assert_eq!(overview.risk_items[0].severity, "critical");
    }
}
