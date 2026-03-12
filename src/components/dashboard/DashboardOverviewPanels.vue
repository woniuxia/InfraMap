<script setup lang="ts">
import type { PropType } from "vue";
import type { DashboardOverview, DashboardRiskItem } from "@/types";

const props = defineProps({
  overview: {
    type: Object as PropType<DashboardOverview | null>,
    default: null,
  },
  totalAssets: {
    type: Number,
    default: 0,
  },
});

const emit = defineEmits<{
  riskSelect: [item: DashboardRiskItem];
}>();

function formatInt(value: number) {
  return value.toLocaleString("zh-CN");
}

function formatPercent(value: number) {
  return `${value.toFixed(value % 1 === 0 ? 0 : 1)}%`;
}

function severityLabel(severity: string) {
  return (
    ({ critical: "高", warning: "中", info: "低" } as Record<string, string>)[severity] || "中"
  );
}

function severityTagType(severity: string): "primary" | "success" | "warning" | "info" | "danger" {
  return (
    (
      { critical: "danger", warning: "warning", info: "info" } as Record<
        string,
        "danger" | "warning" | "info"
      >
    )[severity] || "info"
  );
}
</script>

<template>
  <el-row :gutter="16" class="section-row section-row-secondary">
    <el-col :xs="24" :lg="10">
      <el-card class="panel-card secondary-panel-card">
        <template #header>
          <div class="panel-header">
            <span class="panel-title">运营概览（次要）</span>
          </div>
        </template>

        <div class="secondary-metrics">
          <div class="secondary-metric">
            <div class="secondary-metric-label">总资产</div>
            <div class="secondary-metric-value">
              {{ formatInt(totalAssets) }}
            </div>
            <div class="secondary-metric-sub">主机 + 应用 + 中间件 + 网关</div>
          </div>
          <div class="secondary-metric">
            <div class="secondary-metric-label">异常率</div>
            <div class="secondary-metric-value">
              {{ props.overview ? formatPercent(props.overview.health.abnormal_rate) : "0%" }}
            </div>
            <div class="secondary-metric-sub">
              异常 {{ formatInt(props.overview?.health.abnormal_total ?? 0) }}
            </div>
          </div>
          <div class="secondary-metric">
            <div class="secondary-metric-label">部署覆盖率</div>
            <div class="secondary-metric-value">
              {{
                props.overview ? formatPercent(props.overview.coverage.deployment_coverage) : "0%"
              }}
            </div>
            <div class="secondary-metric-sub">
              未部署
              {{ formatInt(props.overview?.coverage.undeployed_total ?? 0) }}
            </div>
          </div>
        </div>

        <p class="secondary-summary-note">
          健康评分
          {{
            props.overview ? formatInt(Math.round(props.overview.health.score)) : "0"
          }}，仅作排查参考
        </p>
      </el-card>
    </el-col>

    <el-col :xs="24" :lg="14">
      <el-card class="panel-card secondary-panel-card">
        <template #header>
          <div class="panel-header">
            <span class="panel-title">风险提示（次要）</span>
            <span class="panel-hint">按严重级别和数量排序</span>
          </div>
        </template>

        <div v-if="props.overview?.risk_items?.length" class="risk-list">
          <button
            v-for="item in props.overview.risk_items"
            :key="item.key"
            type="button"
            class="risk-item-button"
            @click="emit('riskSelect', item)"
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
</template>

<style scoped lang="scss">
.section-row {
  margin: 0;
}

.section-row-secondary {
  opacity: 0.92;
}

.panel-card {
  border: 1px solid var(--im-border-light);
  background: var(--im-surface-0);

  :deep(.el-card__body) {
    padding: 14px 16px 16px;
  }
}

.secondary-panel-card {
  background: color-mix(in srgb, var(--im-surface-0) 90%, var(--im-surface-1));
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

.secondary-metrics {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.secondary-metric {
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-sm);
  padding: 10px;
  background: var(--im-surface-1);
}

.secondary-metric-label {
  font-size: 12px;
  color: var(--im-text-secondary);
}

.secondary-metric-value {
  margin-top: 4px;
  font-size: 20px;
  line-height: 1.2;
  font-weight: 700;
  color: var(--im-text-primary);
}

.secondary-metric-sub {
  margin-top: 4px;
  font-size: 12px;
  color: var(--im-text-muted);
}

.secondary-summary-note {
  margin: 10px 0 0;
  font-size: 12px;
  color: var(--im-text-muted);
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
    transform var(--im-duration-base) var(--im-ease-standard),
    opacity var(--im-duration-base) var(--im-ease-standard);
}

.risk-item-button:hover {
  border-color: var(--im-border-active);
  background: color-mix(in srgb, var(--im-surface-1) 90%, transparent);
  opacity: 0.92;
  transform: translateY(0);
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

@media (max-width: 1280px) {
  .secondary-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 768px) {
  .secondary-metrics {
    grid-template-columns: 1fr;
  }
}
</style>
