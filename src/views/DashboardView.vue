<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import type { DashboardRecentChange, DashboardRiskItem } from "@/types";
import DashboardKpiGrid from "@/components/dashboard/DashboardKpiGrid.vue";
import DashboardOverviewPanels from "@/components/dashboard/DashboardOverviewPanels.vue";
import DashboardQuickHub from "@/components/dashboard/DashboardQuickHub.vue";
import DashboardRecentChanges from "@/components/dashboard/DashboardRecentChanges.vue";
import type { DashboardKpiCard, DashboardQuickAction } from "@/components/dashboard/types";
import { useDashboardOverview } from "@/composables/useDashboardOverview";
import {
  Connection,
  Files,
  Monitor,
  Refresh,
  SetUp,
  Menu,
  Share,
  Upload,
} from "@element-plus/icons-vue";

const router = useRouter();
const { overview, loading, loadDashboardOverview } = useDashboardOverview();

const totalAssets = computed(() => {
  const totals = overview.value?.totals;
  if (!totals) {
    return 0;
  }
  return (
    totals.host_total + totals.application_total + totals.middleware_total + totals.nginx_total
  );
});

const kpiCards = computed<DashboardKpiCard[]>(() => {
  const totals = overview.value?.totals;
  const coverage = overview.value?.coverage;
  if (!totals || !coverage) {
    return [];
  }

  return [
    {
      key: "host",
      title: "服务器",
      value: totals.host_total,
      subtitle: `异常 ${totals.host_abnormal}`,
      routeName: "Hosts",
      icon: Monitor,
      tone: "primary",
    },
    {
      key: "application",
      title: "应用服务",
      value: totals.application_total,
      subtitle: `异常 ${totals.application_abnormal} / 未部署 ${coverage.undeployed_application_total}`,
      routeName: "Applications",
      icon: Menu,
      tone: "success",
    },
    {
      key: "middleware",
      title: "中间件",
      value: totals.middleware_total,
      subtitle: `未部署 ${coverage.undeployed_middleware_total}`,
      routeName: "Middlewares",
      icon: Connection,
      tone: "warning",
    },
    {
      key: "nginx",
      title: "负载均衡",
      value: totals.nginx_total,
      subtitle: `异常 ${totals.nginx_abnormal} / 未部署 ${coverage.undeployed_nginx_total}`,
      routeName: "NginxConfigs",
      icon: SetUp,
      tone: "danger",
    },
    {
      key: "deployment",
      title: "部署覆盖",
      value: Math.round(coverage.deployment_coverage),
      subtitle: `${coverage.deployed_total}/${coverage.deployable_total} 已部署`,
      routeName: "Topology",
      icon: Share,
      tone: "info",
    },
    {
      key: "relation",
      title: "关系完备",
      value: Math.round(coverage.relation_coverage),
      subtitle: `孤立 ${coverage.isolated_total}`,
      routeName: "Topology",
      icon: Files,
      tone: "info",
    },
  ];
});

const quickActions = computed<DashboardQuickAction[]>(() => [
  {
    key: "hosts",
    title: "服务器",
    desc: "进入主机资产列表",
    routeName: "Hosts",
    icon: Monitor,
    priority: "primary",
    badge: "常用",
  },
  {
    key: "applications",
    title: "应用服务",
    desc: "进入应用服务列表",
    routeName: "Applications",
    icon: Menu,
    priority: "primary",
    badge: "常用",
  },
  {
    key: "middlewares",
    title: "中间件",
    desc: "进入中间件列表",
    routeName: "Middlewares",
    icon: Connection,
    priority: "primary",
  },
  {
    key: "nginx",
    title: "负载均衡",
    desc: "进入负载均衡列表",
    routeName: "NginxConfigs",
    icon: SetUp,
    priority: "primary",
  },
  {
    key: "import-workbench",
    title: "批量录入",
    desc: "打开导入工作台",
    routeName: "ImportWorkbench",
    icon: Upload,
    priority: "primary",
  },
  {
    key: "topology",
    title: "拓扑分析",
    desc: "进入链路拓扑图",
    routeName: "Topology",
    icon: Share,
    priority: "secondary",
  },
  {
    key: "jobs",
    title: "任务中心",
    desc: "查看批量录入与快照任务",
    routeName: "Jobs",
    icon: Files,
    priority: "secondary",
    badge: "新入口",
  },
  {
    key: "integrity-center",
    title: "数据健康",
    desc: "进入健康扫描与修复中心",
    routeName: "IntegrityCenter",
    icon: DataAnalysis,
    priority: "secondary",
    badge: "推荐",
  },
  {
    key: "refresh",
    title: "刷新首页",
    desc: "重新拉取最新统计",
    icon: Refresh,
    priority: "secondary",
  },
]);

const primaryQuickActions = computed(() =>
  quickActions.value.filter((action) => action.priority === "primary"),
);

const secondaryQuickActions = computed(() =>
  quickActions.value.filter((action) => action.priority === "secondary"),
);

function navigateToRoute(routeName: string, query?: Record<string, string>) {
  router.push({
    name: routeName,
    query: query && Object.keys(query).length > 0 ? query : undefined,
  });
}

function onKpiClick(routeName: string) {
  navigateToRoute(routeName);
}

function onRiskClick(item: DashboardRiskItem) {
  navigateToRoute(item.target_route, item.target_filters);
}

function onQuickAction(action: DashboardQuickAction) {
  if (action.key === "refresh") {
    loadDashboardOverview();
    return;
  }
  if (action.routeName) {
    navigateToRoute(action.routeName);
  }
}

function targetRouteByResourceType(type: string) {
  const map: Record<string, string> = {
    host: "Hosts",
    application: "Applications",
    middleware: "Middlewares",
    nginx: "NginxConfigs",
  };
  return map[type];
}

function onRecentChangeClick(change: DashboardRecentChange) {
  const routeName = targetRouteByResourceType(change.resource_type);
  if (!routeName) {
    return;
  }
  navigateToRoute(routeName);
}

onMounted(loadDashboardOverview);
</script>

<template>
  <div class="dashboard-view" v-loading="loading">
    <DashboardQuickHub
      :primary-actions="primaryQuickActions"
      :secondary-actions="secondaryQuickActions"
      @quick-action="onQuickAction"
      @refresh="loadDashboardOverview"
    />

    <DashboardKpiGrid :items="kpiCards" @select="onKpiClick" />

    <DashboardRecentChanges
      :items="overview?.recent_changes ?? []"
      :env-distribution="overview?.env_distribution ?? []"
      :loading="loading"
      @select="onRecentChangeClick"
    />

    <DashboardOverviewPanels
      :overview="overview"
      :total-assets="totalAssets"
      @risk-select="onRiskClick"
    />
  </div>
</template>

<style scoped lang="scss">
.dashboard-view {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
