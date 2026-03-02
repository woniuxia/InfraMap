import { ref } from "vue";
import { listAuditLogs } from "@/api/audit-logs";
import { getDashboardOverview, getDashboardStats } from "@/api/dashboard";
import type {
  AuditLog,
  DashboardCoverage,
  DashboardOverview,
  DashboardRecentChange,
  DashboardRiskItem,
  DashboardStats,
  DashboardTotals,
} from "@/types";

function round1(value: number): number {
  return Math.round(value * 10) / 10;
}

function ratio(numerator: number, denominator: number): number {
  if (denominator <= 0) {
    return 0;
  }
  return numerator / denominator;
}

function percent(numerator: number, denominator: number): number {
  return round1(ratio(numerator, denominator) * 100);
}

function buildRiskItems(totals: DashboardTotals, coverage: DashboardCoverage): DashboardRiskItem[] {
  const items: DashboardRiskItem[] = [];

  if (totals.host_abnormal > 0) {
    items.push({
      key: "host_abnormal",
      label: "异常服务器",
      count: totals.host_abnormal,
      severity: "critical",
      target_route: "Hosts",
      target_filters: { status: "stopped,maintenance" },
    });
  }

  if (totals.application_abnormal > 0) {
    items.push({
      key: "application_abnormal",
      label: "异常应用服务",
      count: totals.application_abnormal,
      severity: "critical",
      target_route: "Applications",
      target_filters: { status: "stopped,maintenance" },
    });
  }

  if (totals.nginx_abnormal > 0) {
    items.push({
      key: "nginx_abnormal",
      label: "异常负载均衡",
      count: totals.nginx_abnormal,
      severity: "critical",
      target_route: "NginxConfigs",
      target_filters: { status: "stopped,maintenance" },
    });
  }

  if (coverage.undeployed_total > 0) {
    items.push({
      key: "undeployed_resources",
      label: "未部署资源",
      count: coverage.undeployed_total,
      severity: "warning",
      target_route: "Topology",
      target_filters: {},
    });
  }

  if (coverage.isolated_total > 0) {
    items.push({
      key: "isolated_resources",
      label: "孤立关系资源",
      count: coverage.isolated_total,
      severity: "warning",
      target_route: "Topology",
      target_filters: {},
    });
  }

  const severityRank: Record<string, number> = {
    critical: 0,
    warning: 1,
    info: 2,
  };

  return items.sort((left, right) => {
    const severityDiff = (severityRank[left.severity] ?? 9) - (severityRank[right.severity] ?? 9);
    if (severityDiff !== 0) {
      return severityDiff;
    }
    if (right.count !== left.count) {
      return right.count - left.count;
    }
    return left.key.localeCompare(right.key);
  });
}

function toRecentChanges(logs: AuditLog[]): DashboardRecentChange[] {
  return logs.map((log) => ({
    id: log.id,
    action: log.action,
    resource_type: log.resource_type,
    resource_id: log.resource_id,
    resource_name: log.resource_name,
    created_at: log.created_at,
  }));
}

function buildOverviewFromLegacy(stats: DashboardStats, logs: AuditLog[]): DashboardOverview {
  const totals: DashboardTotals = {
    host_total: stats.host_total,
    host_abnormal: stats.host_abnormal,
    application_total: stats.application_total,
    application_abnormal: stats.application_abnormal,
    middleware_total: stats.middleware_total,
    nginx_total: stats.nginx_total,
    nginx_abnormal: stats.nginx_abnormal,
    deployment_total: stats.deployment_total,
    dependency_total: stats.dependency_total,
  };

  const deployableTotal = totals.application_total + totals.middleware_total + totals.nginx_total;
  const deployedTotal = Math.min(totals.deployment_total, deployableTotal);
  const undeployedTotal = Math.max(0, deployableTotal - deployedTotal);
  const relatedTotal = Math.min(totals.dependency_total, deployableTotal);
  const isolatedTotal = Math.max(0, deployableTotal - relatedTotal);

  const coverage: DashboardCoverage = {
    deployable_total: deployableTotal,
    deployed_total: deployedTotal,
    undeployed_total: undeployedTotal,
    deployment_coverage: percent(deployedTotal, deployableTotal),
    relatable_total: deployableTotal,
    related_total: relatedTotal,
    isolated_total: isolatedTotal,
    relation_coverage: percent(relatedTotal, deployableTotal),
    undeployed_application_total: 0,
    undeployed_middleware_total: 0,
    undeployed_nginx_total: 0,
  };

  const abnormalTotal = totals.host_abnormal + totals.application_abnormal + totals.nginx_abnormal;
  const statusTrackableTotal = totals.host_total + totals.application_total + totals.nginx_total;
  const abnormalRatio = ratio(abnormalTotal, statusTrackableTotal);
  const undeployedRatio = ratio(undeployedTotal, deployableTotal);
  const isolatedRatio = ratio(isolatedTotal, deployableTotal);
  const penalty = abnormalRatio * 60 + undeployedRatio * 25 + isolatedRatio * 15;
  const score = round1(Math.max(0, Math.min(100, 100 - penalty)));

  return {
    totals,
    health: {
      abnormal_total: abnormalTotal,
      abnormal_rate: percent(abnormalTotal, statusTrackableTotal),
      score,
    },
    coverage,
    env_distribution: stats.env_distribution,
    risk_items: buildRiskItems(totals, coverage),
    recent_changes: toRecentChanges(logs),
  };
}

export function useDashboardOverview() {
  const overview = ref<DashboardOverview | null>(null);
  const loading = ref(false);
  const usedFallback = ref(false);

  async function loadDashboardOverview() {
    loading.value = true;
    try {
      overview.value = await getDashboardOverview();
      usedFallback.value = false;
    } catch {
      try {
        const [stats, logsResult] = await Promise.all([
          getDashboardStats(),
          listAuditLogs({ page: 1, page_size: 20 }),
        ]);
        overview.value = buildOverviewFromLegacy(stats, logsResult.data);
        usedFallback.value = true;
      } catch {
        overview.value = null;
      }
    } finally {
      loading.value = false;
    }
  }

  return {
    overview,
    loading,
    usedFallback,
    loadDashboardOverview,
  };
}

export { buildOverviewFromLegacy };
