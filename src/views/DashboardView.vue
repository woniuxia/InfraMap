<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import type { Component } from "vue";
import type { DashboardRecentChange, DashboardRiskItem } from "@/types";
import { useDashboardOverview } from "@/composables/useDashboardOverview";
import {
  Connection,
  DataAnalysis,
  Files,
  Monitor,
  Refresh,
  SetUp,
  Warning,
  Menu,
  Share,
} from "@element-plus/icons-vue";

const router = useRouter();
const { overview, loading, usedFallback, loadDashboardOverview } = useDashboardOverview();

interface KpiCard {
  key: string;
  title: string;
  value: number;
  subtitle: string;
  routeName: string;
  icon: Component;
  tone: "primary" | "success" | "warning" | "danger" | "info";
}

interface QuickAction {
  key: string;
  title: string;
  desc: string;
  routeName?: string;
  icon: Component;
}

const totalAssets = computed(() => {
  const totals = overview.value?.totals;
  if (!totals) {
    return 0;
  }
  return (
    totals.host_total + totals.application_total + totals.middleware_total + totals.nginx_total
  );
});

const kpiCards = computed<KpiCard[]>(() => {
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

const quickActions = computed<QuickAction[]>(() => [
  {
    key: "hosts",
    title: "资产服务器",
    desc: "查看并维护主机资产",
    routeName: "Hosts",
    icon: Monitor,
  },
  {
    key: "applications",
    title: "应用服务",
    desc: "查看应用及状态",
    routeName: "Applications",
    icon: Menu,
  },
  {
    key: "middlewares",
    title: "中间件",
    desc: "查看中间件实例",
    routeName: "Middlewares",
    icon: Connection,
  },
  {
    key: "nginx",
    title: "负载均衡",
    desc: "查看网关配置",
    routeName: "NginxConfigs",
    icon: SetUp,
  },
  {
    key: "topology",
    title: "拓扑分析",
    desc: "进入链路拓扑图",
    routeName: "Topology",
    icon: Share,
  },
  {
    key: "refresh",
    title: "刷新统计",
    desc: "重新拉取首页数据",
    icon: Refresh,
  },
]);

function formatInt(value: number) {
  return value.toLocaleString("zh-CN");
}

function formatPercent(value: number) {
  return `${value.toFixed(value % 1 === 0 ? 0 : 1)}%`;
}

function envLabel(env: string) {
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env;
}

function actionLabel(action: string) {
  return ({ create: "创建", update: "更新", delete: "删除" } as Record<string, string>)[action] || action;
}

function actionTagType(action: string): "primary" | "success" | "warning" | "info" | "danger" {
  return (
    ({ create: "success", update: "warning", delete: "danger" } as Record<string, "success" | "warning" | "danger">)[
      action
    ] || "info"
  );
}

function resourceTypeLabel(type: string) {
  return (
    ({
      host: "服务器",
      application: "应用",
      middleware: "中间件",
      nginx: "负载均衡",
      deployment: "部署关系",
      dependency: "依赖关系",
    } as Record<string, string>)[type] || type
  );
}

function resourceTypeTagType(type: string): "primary" | "success" | "warning" | "info" | "danger" {
  return (
    ({
      host: "primary",
      application: "success",
      middleware: "warning",
      nginx: "danger",
      deployment: "info",
      dependency: "info",
    } as Record<string, "primary" | "success" | "warning" | "info" | "danger">)[type] || "info"
  );
}

function severityLabel(severity: string) {
  return ({ critical: "高", warning: "中", info: "低" } as Record<string, string>)[severity] || "中";
}

function severityTagType(severity: string): "primary" | "success" | "warning" | "info" | "danger" {
  return (
    ({ critical: "danger", warning: "warning", info: "info" } as Record<string, "danger" | "warning" | "info">)[
      severity
    ] || "info"
  );
}

function formatTime(iso: string) {
  if (!iso) return "-";
  const time = new Date(iso);
  return time.toLocaleString("zh-CN", { hour12: false });
}

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

function onQuickAction(action: QuickAction) {
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
    <el-card class="hero-card">
      <div class="hero-header">
        <div>
          <h3 class="hero-title">资产健康驾驶舱</h3>
          <p class="hero-subtitle">聚焦资产规模、健康状态和高风险入口</p>
        </div>
        <div class="hero-actions">
          <el-tag v-if="usedFallback" type="warning" effect="plain">统计回退模式</el-tag>
          <el-button class="refresh-button" type="primary" @click="loadDashboardOverview">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </div>

      <div class="hero-metrics">
        <div class="hero-metric">
          <div class="metric-label">总资产</div>
          <div class="metric-value">{{ formatInt(totalAssets) }}</div>
          <div class="metric-sub">主机 + 应用 + 中间件 + 负载均衡</div>
        </div>
        <div class="hero-metric">
          <div class="metric-label">健康评分</div>
          <div class="metric-value">{{ overview ? formatInt(Math.round(overview.health.score)) : "0" }}</div>
          <div class="metric-sub">异常、未部署、孤立关系综合计算</div>
        </div>
        <div class="hero-metric">
          <div class="metric-label">异常率</div>
          <div class="metric-value">
            {{ overview ? formatPercent(overview.health.abnormal_rate) : "0%" }}
          </div>
          <div class="metric-sub">异常 {{ formatInt(overview?.health.abnormal_total ?? 0) }}</div>
        </div>
        <div class="hero-metric">
          <div class="metric-label">部署覆盖率</div>
          <div class="metric-value">
            {{ overview ? formatPercent(overview.coverage.deployment_coverage) : "0%" }}
          </div>
          <div class="metric-sub">
            未部署 {{ formatInt(overview?.coverage.undeployed_total ?? 0) }}
          </div>
        </div>
      </div>
    </el-card>

    <el-row :gutter="16" class="section-row">
      <el-col
        v-for="item in kpiCards"
        :key="item.key"
        :xs="24"
        :sm="12"
        :lg="8"
      >
        <el-card class="kpi-card" shadow="hover">
          <button class="kpi-button" type="button" @click="onKpiClick(item.routeName)">
            <div class="kpi-icon" :class="`kpi-icon-${item.tone}`">
              <el-icon :size="20">
                <component :is="item.icon" />
              </el-icon>
            </div>
            <div class="kpi-content">
              <div class="kpi-title">{{ item.title }}</div>
              <div class="kpi-value">
                <span>{{ formatInt(item.value) }}</span>
                <span v-if="item.key === 'deployment' || item.key === 'relation'" class="kpi-unit">%</span>
              </div>
              <div class="kpi-subtitle">{{ item.subtitle }}</div>
            </div>
          </button>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="16" class="section-row">
      <el-col :xs="24" :lg="10">
        <el-card class="panel-card">
          <template #header>
            <div class="panel-header">
              <span class="panel-title">环境分布</span>
              <el-icon :size="16"><DataAnalysis /></el-icon>
            </div>
          </template>
          <div v-if="overview?.env_distribution?.length" class="env-list">
            <div
              v-for="item in overview.env_distribution"
              :key="item.env"
              class="env-item"
            >
              <div class="env-top">
                <span class="env-label">{{ envLabel(item.env) }}</span>
                <span class="env-count">{{ formatInt(item.count) }}</span>
              </div>
              <el-progress
                :stroke-width="10"
                :percentage="
                  Math.round(
                    (item.count /
                      Math.max(
                        overview.env_distribution.reduce((sum, current) => sum + current.count, 0),
                        1
                      )) *
                      100
                  )
                "
                :show-text="false"
              />
            </div>
          </div>
          <el-empty v-else description="暂无环境分布数据" :image-size="64" />
        </el-card>
      </el-col>

      <el-col :xs="24" :lg="14">
        <el-card class="panel-card">
          <template #header>
            <div class="panel-header">
              <span class="panel-title">风险清单</span>
              <span class="panel-hint">按严重级别和数量排序</span>
            </div>
          </template>

          <div v-if="overview?.risk_items?.length" class="risk-list">
            <button
              v-for="item in overview.risk_items"
              :key="item.key"
              type="button"
              class="risk-item-button"
              @click="onRiskClick(item)"
            >
              <div class="risk-item-main">
                <div class="risk-item-title">{{ item.label }}</div>
                <div class="risk-item-sub">点击查看处理页</div>
              </div>
              <div class="risk-item-side">
                <el-tag :type="severityTagType(item.severity)" size="small" effect="plain">
                  风险{{ severityLabel(item.severity) }}
                </el-tag>
                <span class="risk-count">{{ formatInt(item.count) }}</span>
              </div>
            </button>
          </div>
          <el-empty v-else description="当前无高优先风险项" :image-size="64" />
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="16" class="section-row">
      <el-col :xs="24" :lg="8">
        <el-card class="panel-card">
          <template #header>
            <div class="panel-header">
              <span class="panel-title">快捷操作</span>
              <el-icon :size="16"><Warning /></el-icon>
            </div>
          </template>

          <div class="quick-actions-grid">
            <button
              v-for="action in quickActions"
              :key="action.key"
              type="button"
              class="quick-action-button"
              @click="onQuickAction(action)"
            >
              <el-icon :size="18">
                <component :is="action.icon" />
              </el-icon>
              <div class="quick-action-text">
                <div class="quick-action-title">{{ action.title }}</div>
                <div class="quick-action-desc">{{ action.desc }}</div>
              </div>
            </button>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :lg="16">
        <el-card class="panel-card">
          <template #header>
            <div class="panel-header">
              <span class="panel-title">最近变更</span>
              <span class="panel-hint">最近 20 条审计记录</span>
            </div>
          </template>
          <el-table :data="overview?.recent_changes ?? []" max-height="360" stripe size="small">
            <el-table-column prop="action" label="操作" width="90" align="center">
              <template #default="{ row }">
                <el-tag :type="actionTagType(row.action)" size="small">
                  {{ actionLabel(row.action) }}
                </el-tag>
              </template>
            </el-table-column>

            <el-table-column prop="resource_type" label="资源类型" width="110" align="center">
              <template #default="{ row }">
                <el-tag :type="resourceTypeTagType(row.resource_type)" size="small" effect="plain">
                  {{ resourceTypeLabel(row.resource_type) }}
                </el-tag>
              </template>
            </el-table-column>

            <el-table-column prop="resource_name" label="资源" min-width="180" show-overflow-tooltip>
              <template #default="{ row }">
                {{ row.resource_name || row.resource_id }}
              </template>
            </el-table-column>

            <el-table-column prop="created_at" label="时间" width="180">
              <template #default="{ row }">
                {{ formatTime(row.created_at) }}
              </template>
            </el-table-column>

            <el-table-column label="操作" width="90" align="center">
              <template #default="{ row }">
                <el-button text type="primary" size="small" @click="onRecentChangeClick(row)">
                  查看
                </el-button>
              </template>
            </el-table-column>
          </el-table>
          <el-empty
            v-if="!(overview?.recent_changes?.length) && !loading"
            description="暂无变更记录"
            :image-size="64"
          />
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped lang="scss">
.dashboard-view {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-row {
  margin: 0;
}

.hero-card {
  border: 1px solid var(--im-border-light);
  background:
    linear-gradient(140deg, color-mix(in srgb, var(--im-surface-1) 82%, transparent) 0%, var(--im-surface-0) 68%),
    radial-gradient(circle at 0% 0%, var(--im-accent-soft) 0%, transparent 52%);

  :deep(.el-card__body) {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
}

.hero-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.hero-title {
  margin: 0;
  font-size: 22px;
  font-family: var(--im-font-display);
  color: var(--im-text-primary);
}

.hero-subtitle {
  margin-top: 6px;
  font-size: 13px;
  color: var(--im-text-secondary);
}

.hero-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.refresh-button {
  min-height: 32px;
}

.hero-metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.hero-metric {
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-md);
  background: var(--im-surface-1);
  padding: 12px 14px;
}

.metric-label {
  color: var(--im-text-secondary);
  font-size: 12px;
  margin-bottom: 4px;
}

.metric-value {
  font-size: 28px;
  line-height: 1.1;
  font-weight: 700;
  color: var(--im-text-primary);
}

.metric-sub {
  margin-top: 6px;
  font-size: 12px;
  color: var(--im-text-muted);
}

.kpi-card {
  border: 1px solid var(--im-border-light);
  background: var(--im-surface-0);
  transition:
    transform var(--im-duration-base) var(--im-ease-standard),
    box-shadow var(--im-duration-base) var(--im-ease-standard),
    border-color var(--im-duration-base) var(--im-ease-standard);

  :deep(.el-card__body) {
    padding: 0;
  }

  &:hover {
    border-color: var(--im-border-active);
    box-shadow: var(--im-shadow-md);
    transform: translateY(-2px);
  }
}

.kpi-button {
  border: 0;
  width: 100%;
  min-height: 106px;
  cursor: pointer;
  background: transparent;
  color: inherit;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px;
  text-align: left;
}

.kpi-button:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: -2px;
}

.kpi-button:active {
  transform: translateY(1px);
}

.kpi-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.kpi-icon-primary {
  color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.kpi-icon-success {
  color: var(--el-color-success);
  background: color-mix(in srgb, var(--el-color-success) 20%, transparent);
}

.kpi-icon-warning {
  color: var(--el-color-warning);
  background: color-mix(in srgb, var(--el-color-warning) 22%, transparent);
}

.kpi-icon-danger {
  color: var(--el-color-danger);
  background: color-mix(in srgb, var(--el-color-danger) 20%, transparent);
}

.kpi-icon-info {
  color: var(--el-color-info);
  background: color-mix(in srgb, var(--el-color-info) 20%, transparent);
}

.kpi-content {
  min-width: 0;
  flex: 1;
}

.kpi-title {
  font-size: 13px;
  color: var(--im-text-secondary);
}

.kpi-value {
  margin-top: 2px;
  font-size: 24px;
  line-height: 1.2;
  font-weight: 700;
  color: var(--im-text-primary);
}

.kpi-unit {
  margin-left: 2px;
  font-size: 15px;
  font-weight: 600;
  color: var(--im-text-secondary);
}

.kpi-subtitle {
  margin-top: 6px;
  font-size: 12px;
  color: var(--im-text-muted);
}

.panel-card {
  border: 1px solid var(--im-border-light);
  background: var(--im-surface-0);

  :deep(.el-card__body) {
    padding: 14px 16px 16px;
  }
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.panel-hint {
  font-size: 12px;
  color: var(--im-text-muted);
}

.env-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.env-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.env-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.env-label {
  font-size: 13px;
  color: var(--im-text-regular);
}

.env-count {
  font-size: 13px;
  color: var(--im-text-secondary);
}

.risk-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.risk-item-button {
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-sm);
  background: var(--im-surface-1);
  width: 100%;
  min-height: 52px;
  padding: 10px 12px;
  cursor: pointer;
  color: inherit;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  text-align: left;
  transition:
    border-color var(--im-duration-base) var(--im-ease-standard),
    background-color var(--im-duration-base) var(--im-ease-standard),
    transform var(--im-duration-base) var(--im-ease-standard);
}

.risk-item-button:hover {
  border-color: var(--im-border-active);
  background: var(--im-surface-2);
  transform: translateY(-1px);
}

.risk-item-button:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}

.risk-item-button:active {
  transform: translateY(0);
}

.risk-item-main {
  min-width: 0;
}

.risk-item-title {
  font-size: 13px;
  color: var(--im-text-primary);
}

.risk-item-sub {
  margin-top: 2px;
  font-size: 12px;
  color: var(--im-text-muted);
}

.risk-item-side {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.risk-count {
  min-width: 30px;
  text-align: right;
  font-size: 16px;
  font-weight: 700;
  color: var(--im-text-primary);
}

.quick-actions-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 10px;
}

.quick-action-button {
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-sm);
  background: var(--im-surface-1);
  color: inherit;
  cursor: pointer;
  min-height: 52px;
  padding: 10px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  text-align: left;
  transition:
    border-color var(--im-duration-base) var(--im-ease-standard),
    background-color var(--im-duration-base) var(--im-ease-standard),
    transform var(--im-duration-base) var(--im-ease-standard);
}

.quick-action-button:hover {
  border-color: var(--im-border-active);
  background: var(--im-surface-2);
  transform: translateY(-1px);
}

.quick-action-button:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}

.quick-action-button:active {
  transform: translateY(0);
}

.quick-action-text {
  min-width: 0;
}

.quick-action-title {
  font-size: 13px;
  color: var(--im-text-primary);
}

.quick-action-desc {
  margin-top: 2px;
  font-size: 12px;
  color: var(--im-text-muted);
}

@media (max-width: 1280px) {
  .hero-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 768px) {
  .hero-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .hero-actions {
    width: 100%;
    justify-content: flex-start;
  }

  .hero-metrics {
    grid-template-columns: 1fr;
  }
}
</style>
