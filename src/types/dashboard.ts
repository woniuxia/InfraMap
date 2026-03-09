export interface DashboardTotals {
  host_total: number;
  host_abnormal: number;
  application_total: number;
  application_abnormal: number;
  middleware_total: number;
  nginx_total: number;
  nginx_abnormal: number;
  deployment_total: number;
  dependency_total: number;
}

export interface DashboardHealth {
  abnormal_total: number;
  abnormal_rate: number;
  score: number;
}

export interface DashboardCoverage {
  deployable_total: number;
  deployed_total: number;
  undeployed_total: number;
  deployment_coverage: number;
  relatable_total: number;
  related_total: number;
  isolated_total: number;
  relation_coverage: number;
  undeployed_application_total: number;
  undeployed_middleware_total: number;
  undeployed_nginx_total: number;
}

export type DashboardRiskSeverity = "critical" | "warning" | "info";

export interface DashboardRiskItem {
  key: string;
  label: string;
  count: number;
  severity: DashboardRiskSeverity;
  target_route: string;
  target_filters: Record<string, string>;
}

export interface DashboardRecentChange {
  id: string;
  action: "create" | "update" | "delete";
  resource_type: string;
  resource_id: string;
  resource_name?: string;
  created_at: string;
}

export interface DashboardOverview {
  totals: DashboardTotals;
  health: DashboardHealth;
  coverage: DashboardCoverage;
  env_distribution: { env: string; count: number }[];
  risk_items: DashboardRiskItem[];
  recent_changes: DashboardRecentChange[];
}
