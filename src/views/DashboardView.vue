<script setup lang="ts">
import { ref, onMounted } from "vue";
import type { DashboardStats, AuditLog } from "@/types";
import { getDashboardStats } from "@/api/dashboard";
import { listAuditLogs } from "@/api/audit-logs";
import { Monitor, Menu, Connection, SetUp } from "@element-plus/icons-vue";

const stats = ref<DashboardStats | null>(null);
const recentLogs = ref<AuditLog[]>([]);
const loading = ref(false);

async function loadDashboard() {
  loading.value = true;
  try {
    stats.value = await getDashboardStats();
    const logsResult = await listAuditLogs({ page: 1, page_size: 20 });
    recentLogs.value = logsResult.data;
  } catch {
    // error shown by tauriInvoke
  } finally {
    loading.value = false;
  }
}

function actionTagType(action: string) {
  return (
    ({ create: "success", update: "warning", delete: "danger" } as Record<string, string>)[
      action
    ] || "info"
  );
}

function actionLabel(action: string) {
  return (
    ({ create: "创建", update: "更新", delete: "删除" } as Record<string, string>)[action] ||
    action
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

function envLabel(env: string) {
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env;
}

function formatTime(iso: string) {
  if (!iso) return "-";
  const d = new Date(iso);
  return d.toLocaleString("zh-CN", { hour12: false });
}

onMounted(loadDashboard);
</script>

<template>
  <div class="dashboard-view" v-loading="loading">
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-icon" style="background: var(--el-color-primary-light-9)">
            <el-icon :size="28" color="var(--el-color-primary)"><Monitor /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-title">服务器</div>
            <div class="stat-number">{{ stats?.host_total ?? 0 }}</div>
            <div class="stat-sub" v-if="stats?.host_abnormal">
              <el-text type="danger" size="small">异常 {{ stats.host_abnormal }}</el-text>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-icon" style="background: var(--el-color-success-light-9)">
            <el-icon :size="28" color="var(--el-color-success)"><Menu /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-title">应用服务</div>
            <div class="stat-number">{{ stats?.application_total ?? 0 }}</div>
            <div class="stat-sub" v-if="stats?.application_abnormal">
              <el-text type="danger" size="small">异常 {{ stats.application_abnormal }}</el-text>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-icon" style="background: var(--el-color-warning-light-9)">
            <el-icon :size="28" color="var(--el-color-warning)"><Connection /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-title">中间件</div>
            <div class="stat-number">{{ stats?.middleware_total ?? 0 }}</div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-icon" style="background: var(--el-color-danger-light-9)">
            <el-icon :size="28" color="var(--el-color-danger)"><SetUp /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-title">负载均衡</div>
            <div class="stat-number">{{ stats?.nginx_total ?? 0 }}</div>
            <div class="stat-sub" v-if="stats?.nginx_abnormal">
              <el-text type="danger" size="small">异常 {{ stats.nginx_abnormal }}</el-text>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="16" style="margin-top: 16px">
      <el-col :span="8">
        <el-card>
          <template #header>环境分布</template>
          <div v-if="stats?.env_distribution?.length" class="env-list">
            <div v-for="item in stats.env_distribution" :key="item.env" class="env-item">
              <span class="env-label">{{ envLabel(item.env) }}</span>
              <el-progress
                :percentage="
                  Math.round(
                    (item.count /
                      Math.max(
                        stats.env_distribution.reduce((s, i) => s + i.count, 0),
                        1
                      )) *
                      100
                  )
                "
                :stroke-width="18"
                :format="() => String(item.count)"
              />
            </div>
          </div>
          <el-empty v-else description="暂无数据" :image-size="60" />
        </el-card>
      </el-col>
      <el-col :span="16">
        <el-card>
          <template #header>
            <div style="display: flex; justify-content: space-between; align-items: center">
              <span>最近变更</span>
              <el-button text type="primary" @click="loadDashboard">刷新</el-button>
            </div>
          </template>
          <el-table :data="recentLogs" max-height="360" stripe size="small">
            <el-table-column prop="action" label="操作" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="actionTagType(row.action)" size="small">{{
                  actionLabel(row.action)
                }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="resource_type" label="资源类型" width="100" align="center">
              <template #default="{ row }">{{
                resourceTypeLabel(row.resource_type)
              }}</template>
            </el-table-column>
            <el-table-column prop="resource_name" label="资源名称" min-width="150">
              <template #default="{ row }">{{ row.resource_name || row.resource_id }}</template>
            </el-table-column>
            <el-table-column prop="created_at" label="时间" width="170">
              <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
            </el-table-column>
          </el-table>
          <el-empty
            v-if="!recentLogs.length && !loading"
            description="暂无变更记录"
            :image-size="60"
          />
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped lang="scss">
.dashboard-view {
  padding: 0;
}
.stats-row {
  margin-bottom: 0;
}
.stat-card {
  :deep(.el-card__body) {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px;
  }
}
.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-info {
  flex: 1;
}
.stat-title {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
}
.stat-number {
  font-size: 28px;
  font-weight: 700;
  color: var(--el-text-color-primary);
  line-height: 1.2;
}
.stat-sub {
  margin-top: 4px;
}
.env-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.env-item {
  display: flex;
  align-items: center;
  gap: 12px;
}
.env-label {
  width: 40px;
  font-size: 13px;
  color: var(--el-text-color-regular);
  flex-shrink: 0;
}
.env-item .el-progress {
  flex: 1;
}
</style>
