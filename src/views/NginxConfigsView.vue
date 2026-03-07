<script setup lang="ts">
import { ref, onMounted } from "vue";
import type { NginxConfig, NginxEndpoint } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listNginxConfigs, deleteNginxConfig } from "@/api/nginx-configs";
import { useResourceList } from "@/composables/useResourceList";
import { buildNginxCopyDraft } from "@/utils/resourceCopy";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import NginxConfigEditorDialog from "@/components/resource-editors/NginxConfigEditorDialog.vue";

type NginxEditorMode = "create" | "edit" | "copy";

const {
  loading,
  data,
  total,
  queryParams,
  fetchData,
  handleQuery,
  handlePageChange,
  handlePageSizeChange,
  handleDelete,
} = useResourceList<NginxConfig>({
  listFn: listNginxConfigs,
  deleteFn: deleteNginxConfig,
  entityLabel: "负载均衡",
});

const searchText = ref("");
const editorVisible = ref(false);
const editorMode = ref<NginxEditorMode>("create");
const editorInitialDraft = ref<Partial<NginxConfig>>({});

interface NginxListFilters {
  env: string[];
  status: string[];
  strategy: string[];
}

function createDefaultFilters(): NginxListFilters {
  return {
    env: [],
    status: [],
    strategy: [],
  };
}

const listFilters = ref<NginxListFilters>(createDefaultFilters());
const envOptions = [
  { label: "生产", value: "prod" },
  { label: "开发", value: "dev" },
  { label: "测试", value: "test" },
];

const statusOptions = [
  { label: "运行中", value: "running" },
  { label: "已停止", value: "stopped" },
  { label: "维护中", value: "maintenance" },
];

const strategyOptions = [
  { label: "轮询", value: "roundrobin" },
  { label: "IP 哈希", value: "ip_hash" },
];

const toolbarFields: SearchFieldConfig[] = [
  {
    key: "env",
    queryKey: "env",
    label: "环境",
    type: "multi-select",
    width: "sm",
    options: envOptions,
  },
  {
    key: "status",
    queryKey: "status",
    label: "状态",
    type: "multi-select",
    width: "md",
    maxCollapseTags: 2,
    options: statusOptions,
  },
  {
    key: "strategy",
    queryKey: "strategy",
    label: "策略",
    type: "multi-select",
    width: "sm",
    options: strategyOptions,
  },
];

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

function createEmptyEndpoint(): NginxEndpoint {
  return {
    host: "",
    port: 80,
  };
}

function normalizeEndpointDraft(endpoints?: NginxEndpoint[]): NginxEndpoint[] {
  if (!Array.isArray(endpoints)) {
    return [createEmptyEndpoint()];
  }
  const normalized = endpoints.map((item) => ({
    host: (item?.host ?? "").trim(),
    port: Number(item?.port ?? 0),
  }));
  return normalized.length > 0 ? normalized : [createEmptyEndpoint()];
}

function openAdd() {
  editorInitialDraft.value = {
    id: "",
    endpoints: [createEmptyEndpoint()],
    status: "running",
    env: "prod",
    strategy: "roundrobin",
    created_at: "",
    updated_at: "",
  };
  editorMode.value = "create";
  editorVisible.value = true;
}

function openEdit(row: NginxConfig) {
  editorInitialDraft.value = {
    ...row,
    endpoints: normalizeEndpointDraft(row.endpoints),
  };
  editorMode.value = "edit";
  editorVisible.value = true;
}

function openCopy(row: NginxConfig) {
  editorInitialDraft.value = {
    ...buildNginxCopyDraft(row),
    endpoints: normalizeEndpointDraft(row.endpoints),
  };
  editorMode.value = "copy";
  editorVisible.value = true;
}

function handleEditorSaved() {
  fetchData();
}

function statusTagType(status: string): "primary" | "success" | "warning" | "info" | "danger" {
  const map: Record<string, "primary" | "success" | "warning" | "info" | "danger"> = {
    running: "success",
    stopped: "danger",
    maintenance: "warning",
  };
  return map[status] || "info";
}

function statusLabel(status: string) {
  return ({ running: "运行中", stopped: "已停止", maintenance: "维护中" } as Record<string, string>)[status] || status;
}

function strategyLabel(strategy: string) {
  return ({ roundrobin: "轮询", ip_hash: "IP 哈希" } as Record<string, string>)[strategy] || strategy || "-";
}

function formatEndpoint(item: NginxEndpoint): string {
  const host = (item.host || "").trim();
  if (!host) return "-";
  return `${host}:${item.port}`;
}

function endpointSummary(endpoints?: NginxEndpoint[]): string {
  if (!Array.isArray(endpoints) || endpoints.length === 0) {
    return "-";
  }
  const first = formatEndpoint(endpoints[0]);
  if (endpoints.length === 1) {
    return first;
  }
  return `${first} +${endpoints.length - 1}`;
}

function envLabel(env: string) {
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env;
}

function envTagType(env: string): "primary" | "success" | "warning" | "info" | "danger" {
  const map: Record<string, "primary" | "success" | "warning" | "info" | "danger"> = {
    prod: "danger",
    dev: "info",
    test: "warning",
  };
  return map[env] || "info";
}

onMounted(() => fetchData());
</script>

<template>
  <div class="resource-view">
    <SearchToolbar
      v-model:search-text="searchText"
      v-model:filters="listFilters"
      search-placeholder="搜索配置名称..."
      :fields="toolbarFields"
      @query="handleToolbarQuery"
    >
      <template #actions="{ hasActiveFilters, reset }">
        <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
        <el-button type="primary" @click="openAdd">新增配置</el-button>
      </template>
    </SearchToolbar>
    <el-table :data="data" v-loading="loading" border stripe class="w-full im-table-fixed-ops">
      <el-table-column prop="name" label="配置名称" min-width="180" align="center" />
      <el-table-column label="连接端点" min-width="260" align="center">
        <template #default="{ row }">{{ endpointSummary(row.endpoints) }}</template>
      </el-table-column>
      <el-table-column prop="strategy" label="负载策略" width="100" align="center">
        <template #default="{ row }">{{ strategyLabel(row.strategy) }}</template>
      </el-table-column>
      <el-table-column prop="env" label="环境" width="80" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="100" align="center">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)" size="small">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="210" fixed="right" align="center">
        <template #default="{ row }">
          <el-button text type="primary" size="small" @click="openEdit(row)">编辑</el-button>
          <el-button text type="primary" size="small" @click="openCopy(row)">复制</el-button>
          <el-button text type="danger" size="small" @click="handleDelete(row.id, row.name)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-wrapper">
      <el-pagination
        v-model:current-page="queryParams.page"
        v-model:page-size="queryParams.page_size"
        :total="total"
        :page-sizes="[10, 20, 50]"
        layout="total, sizes, prev, pager, next"
        @current-change="handlePageChange"
        @size-change="handlePageSizeChange"
      />
    </div>

    <NginxConfigEditorDialog
      v-model="editorVisible"
      :mode="editorMode"
      :initial-draft="editorInitialDraft"
      @saved="handleEditorSaved"
    />
  </div>
</template>

<style scoped lang="scss">
.resource-view {
  padding: 0;
}
.filter-bar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px;
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-md);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--im-surface-1) 82%, transparent), var(--im-surface-0));
}
.filter-row {
  display: grid;
  gap: 12px;
  align-items: center;
}
.filter-row-primary {
  grid-template-columns: minmax(0, 2.4fr) minmax(0, 1.2fr) minmax(0, 1.5fr) minmax(0, 1.5fr);
}
.filter-row-secondary {
  grid-template-columns: minmax(0, 1fr) auto;
}
.filter-spacer {
  min-width: 0;
}
.filter-field {
  display: grid;
  grid-template-columns: 52px minmax(0, 1fr);
  align-items: center;
  gap: 8px;
  width: 100%;
  min-width: 0;
}
.filter-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  justify-self: end;
  white-space: nowrap;
}
.filter-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
  text-align: right;
}
.search-filter,
.env-filter,
.status-filter,
.strategy-filter {
  width: 100%;
  min-width: 0;
}
:deep(.env-filter .el-select__selection),
:deep(.status-filter .el-select__selection),
:deep(.strategy-filter .el-select__selection) {
  flex-wrap: nowrap;
  overflow: hidden;
}
:deep(.env-filter .el-select__selected-item),
:deep(.status-filter .el-select__selected-item),
:deep(.strategy-filter .el-select__selected-item) {
  max-width: 100%;
}
@media (max-width: 1080px) {
  .filter-row-primary {
    grid-template-columns: minmax(0, 2.2fr) minmax(0, 1.1fr) minmax(0, 1.3fr) minmax(0, 1.3fr);
  }
}
@media (max-width: 920px) {
  .filter-row-primary {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .search-field {
    grid-column: 1 / -1;
  }
}
@media (max-width: 768px) {
  .filter-row-primary,
  .filter-row-secondary {
    grid-template-columns: 1fr;
  }
  .filter-actions {
    width: 100%;
    justify-self: start;
    justify-content: flex-start;
  }
  .search-field {
    grid-column: auto;
  }
}
.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
.endpoint-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}
.endpoint-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 140px auto;
  gap: 8px;
  align-items: center;
}
.endpoint-host,
.endpoint-port {
  width: 100%;
}
@media (max-width: 768px) {
  .endpoint-row {
    grid-template-columns: 1fr;
  }
}
</style>
