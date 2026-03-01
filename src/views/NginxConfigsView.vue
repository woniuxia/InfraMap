<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import type { NginxConfig } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listNginxConfigs, saveNginxConfig, softDeleteNginxConfig } from "@/api/nginx-configs";
import { replaceResourceCallRelations } from "@/api/call-relations";
import { useResourceList } from "@/composables/useResourceList";
import { buildNginxCopyDraft } from "@/utils/resourceCopy";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import CallRelationsEditor from "@/components/CallRelationsEditor.vue";
import DeploymentPanel from "@/components/DeploymentPanel.vue";

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
  deleteFn: softDeleteNginxConfig,
  entityLabel: "负载均衡",
});

const searchText = ref("");
const drawerVisible = ref(false);
const editingNc = ref<Partial<NginxConfig>>({});
const isEditing = ref(false);
const saveLoading = ref(false);
const callRelationsEditorRef = ref<InstanceType<typeof CallRelationsEditor> | null>(null);

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

function openAdd() {
  editingNc.value = {
    id: "",
    address: "",
    status: "running",
    env: "prod",
    strategy: "roundrobin",
    is_deleted: 0,
    created_at: "",
    updated_at: "",
  };
  isEditing.value = false;
  drawerVisible.value = true;
}

function openEdit(row: NginxConfig) {
  editingNc.value = { ...row };
  isEditing.value = true;
  drawerVisible.value = true;
}

function openCopy(row: NginxConfig) {
  editingNc.value = buildNginxCopyDraft(row);
  isEditing.value = false;
  drawerVisible.value = true;
}

async function handleSave() {
  const draftItems = callRelationsEditorRef.value?.getDraftItems();
  if (draftItems === null) {
    return;
  }

  const payload: Partial<NginxConfig> = {
    id: "",
    is_deleted: 0,
    created_at: "",
    updated_at: "",
    ...editingNc.value,
  };
  saveLoading.value = true;
  try {
    const nginxId = await saveNginxConfig(payload);
    try {
      await replaceResourceCallRelations({
        resource_id: nginxId,
        resource_type: "nginx",
        items: draftItems ?? [],
      });
    } catch {
      ElMessage.warning("负载均衡已保存，调用关系保存失败，请重新编辑后重试。");
    }
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    drawerVisible.value = false;
    fetchData();
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
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
      search-placeholder="搜索配置名称/地址..."
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
      <el-table-column prop="address" label="连接地址" min-width="180" align="center" />
      <el-table-column prop="listen_port" label="监听端口" width="100" align="center">
        <template #default="{ row }">{{ row.listen_port || "-" }}</template>
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

    <el-dialog
      v-model="drawerVisible"
      :title="isEditing ? '编辑负载均衡' : '新增负载均衡'"
      width="700px"
      align-center
      destroy-on-close
    >
      <el-form :model="editingNc" label-width="96px">
        <el-divider content-position="left">基础信息</el-divider>
        <el-form-item label="配置名称" required>
          <el-input v-model="editingNc.name" placeholder="请输入配置名称" />
        </el-form-item>
        <el-form-item label="连接地址" required>
          <el-input v-model="editingNc.address" placeholder="如 192.168.1.200 或 https://edge.example.com" />
        </el-form-item>
        <el-form-item label="监听端口">
          <el-input-number v-model="editingNc.listen_port" :min="1" :max="65535" class="w-full" />
        </el-form-item>
        <el-form-item label="负载策略">
          <el-select v-model="editingNc.strategy" class="w-full">
            <el-option label="轮询 (roundrobin)" value="roundrobin" />
            <el-option label="IP 哈希 (ip_hash)" value="ip_hash" />
          </el-select>
        </el-form-item>

        <el-divider content-position="left">上游配置</el-divider>
        <el-form-item label="上游服务器">
          <el-input
            v-model="editingNc.upstream_servers"
            type="textarea"
            :rows="4"
            placeholder='JSON 数组，如 ["192.168.1.10:8080", "192.168.1.11:8080"]'
          />
        </el-form-item>

        <el-divider content-position="left">运维信息</el-divider>
        <el-form-item label="环境" required>
          <el-select v-model="editingNc.env" class="w-full">
            <el-option label="生产" value="prod" />
            <el-option label="开发" value="dev" />
            <el-option label="测试" value="test" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态" required>
          <el-select v-model="editingNc.status" class="w-full">
            <el-option label="运行中" value="running" />
            <el-option label="已停止" value="stopped" />
            <el-option label="维护中" value="maintenance" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingNc.description" type="textarea" :rows="3" maxlength="300" show-word-limit />
        </el-form-item>
      </el-form>

      <CallRelationsEditor
        ref="callRelationsEditorRef"
        :resource-id="isEditing ? editingNc.id : undefined"
        resource-type="nginx"
      />

      <DeploymentPanel v-if="isEditing && editingNc.id" :resource-id="editingNc.id!" resource-type="nginx" />

      <template #footer>
        <el-button @click="drawerVisible = false">取消</el-button>
        <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
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
</style>


