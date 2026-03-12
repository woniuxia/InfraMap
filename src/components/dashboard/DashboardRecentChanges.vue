<script setup lang="ts">
import { computed, type PropType } from "vue";
import { DataAnalysis } from "@element-plus/icons-vue";
import type { DashboardOverview, DashboardRecentChange } from "@/types";

const props = defineProps({
  items: {
    type: Array as PropType<DashboardRecentChange[]>,
    default: () => [],
  },
  envDistribution: {
    type: Array as PropType<DashboardOverview["env_distribution"]>,
    default: () => [],
  },
  loading: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits<{
  select: [change: DashboardRecentChange];
}>();

const envTotal = computed(() => props.envDistribution.reduce((sum, item) => sum + item.count, 0));

function actionLabel(action: string) {
  return (
    ({ create: "创建", update: "更新", delete: "删除" } as Record<string, string>)[action] || action
  );
}

function actionTagType(action: string): "primary" | "success" | "warning" | "info" | "danger" {
  return (
    (
      { create: "success", update: "warning", delete: "danger" } as Record<
        string,
        "success" | "warning" | "danger"
      >
    )[action] || "info"
  );
}

function resourceTypeLabel(type: string) {
  return (
    (
      {
        host: "服务器",
        application: "应用",
        middleware: "中间件",
        nginx: "网关",
        deployment: "部署关系",
        dependency: "依赖关系",
      } as Record<string, string>
    )[type] || type
  );
}

function resourceTypeTagType(type: string): "primary" | "success" | "warning" | "info" | "danger" {
  return (
    (
      {
        host: "primary",
        application: "success",
        middleware: "warning",
        nginx: "danger",
        deployment: "info",
        dependency: "info",
      } as Record<string, "primary" | "success" | "warning" | "info" | "danger">
    )[type] || "info"
  );
}

function formatTime(iso: string) {
  if (!iso) return "-";
  const time = new Date(iso);
  return time.toLocaleString("zh-CN", { hour12: false });
}

function envLabel(env: string) {
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env;
}

function envPercentage(count: number) {
  return Math.round((count / Math.max(envTotal.value, 1)) * 100);
}
</script>

<template>
  <el-row :gutter="16" class="section-row">
    <el-col :xs="24" :lg="16">
      <el-card class="panel-card">
        <template #header>
          <div class="panel-header">
            <span class="panel-title">最近变更</span>
            <span class="panel-hint">最近 20 条审计记录</span>
          </div>
        </template>

        <el-table :data="items" max-height="360" stripe size="small">
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
              <el-button text type="primary" size="small" @click="emit('select', row)">
                查看
              </el-button>
            </template>
          </el-table-column>
        </el-table>

        <el-empty v-if="!items.length && !loading" description="暂无变更记录" :image-size="64" />
      </el-card>
    </el-col>

    <el-col :xs="24" :lg="8">
      <el-card class="panel-card">
        <template #header>
          <div class="panel-header">
            <span class="panel-title">环境分布</span>
            <el-icon :size="16"><DataAnalysis /></el-icon>
          </div>
        </template>

        <div v-if="envDistribution.length" class="env-list">
          <div v-for="item in envDistribution" :key="item.env" class="env-item">
            <div class="env-top">
              <span class="env-label">{{ envLabel(item.env) }}</span>
              <span class="env-count">{{ item.count.toLocaleString("zh-CN") }}</span>
            </div>
            <el-progress
              :stroke-width="10"
              :percentage="envPercentage(item.count)"
              :show-text="false"
            />
          </div>
        </div>

        <el-empty v-else description="暂无环境分布数据" :image-size="64" />
      </el-card>
    </el-col>
  </el-row>
</template>

<style scoped lang="scss">
.section-row {
  margin: 0;
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
</style>
