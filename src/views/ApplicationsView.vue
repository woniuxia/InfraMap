<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import type { Application } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import {
  listApplicationOwnerCandidates,
  listApplications,
  listTopApplicationTechStacks,
  saveApplication,
  softDeleteApplication,
} from "@/api/applications";
import type { ApplicationTechStackSide } from "@/api/applications";
import { useResourceList } from "@/composables/useResourceList";
import { buildTechStackSuggestions, parseTechStack, techStackToText } from "@/utils/techStack";
import { buildApplicationCopyDraft } from "@/utils/resourceCopy";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import DeploymentPanel from "@/components/DeploymentPanel.vue";
import DependencyPanel from "@/components/DependencyPanel.vue";

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
} = useResourceList<Application>({
  listFn: listApplications,
  deleteFn: softDeleteApplication,
  entityLabel: "应用服务",
});

const searchText = ref("");
const drawerVisible = ref(false);
const editingApp = ref<Partial<Application>>({});
const isEditing = ref(false);
const saveLoading = ref(false);

const techStackList = ref<string[]>([]);
const topTechStackOptions = ref<string[]>([]);
const ownerList = ref<string[]>([]);
const ownerOptions = ref<string[]>([]);
const techStackSuggestions = computed(() =>
  buildTechStackSuggestions(
    topTechStackOptions.value.map((item) => ({ tech_stack: item })),
    techStackList.value
  )
);
const ownerSuggestions = computed(() => normalizeOwners([...ownerOptions.value, ...ownerList.value], ""));

interface ApplicationListFilters {
  type: string[];
  env: string[];
  status: string[];
  deploy_mode: string[];
}

function createDefaultFilters(): ApplicationListFilters {
  return {
    type: [],
    env: [],
    status: [],
    deploy_mode: [],
  };
}

const listFilters = ref<ApplicationListFilters>(createDefaultFilters());
const typeOptions = [
  { label: "前端", value: "frontend" },
  { label: "后端", value: "backend" },
  { label: "网关", value: "gateway" },
  { label: "批处理", value: "batch_job" },
  { label: "微服务", value: "microservice" },
  { label: "其他", value: "other" },
];

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

const deployModeOptions = [
  { label: "物理机", value: "physical" },
  { label: "虚拟机", value: "vm" },
  { label: "Docker", value: "docker" },
  { label: "Kubernetes", value: "k8s" },
  { label: "Serverless", value: "serverless" },
  { label: "其他", value: "other" },
];

const toolbarFields: SearchFieldConfig[] = [
  {
    key: "type",
    queryKey: "type",
    label: "类型",
    type: "multi-select",
    width: "md",
    options: typeOptions,
  },
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
    key: "deploy_mode",
    queryKey: "deploy_mode",
    label: "部署方式",
    section: "advanced",
    type: "multi-select",
    width: "md",
    options: deployModeOptions,
  },
];

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

function handleTechStackChange(values: string[]) {
  techStackList.value = buildTechStackSuggestions([], values);
}

function normalizeOwners(owners?: string[], owner?: string) {
  const values = [...(owners ?? []), owner ?? ""]
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
  return Array.from(new Set(values));
}

function handleOwnerChange(values: string[]) {
  ownerList.value = normalizeOwners(values);
}

function ownersForRow(row: Application) {
  return normalizeOwners(row.owners, row.owner);
}

function resolveTechStackSide(type: Application["type"] | undefined): ApplicationTechStackSide {
  return type === "frontend" ? "frontend" : "backend";
}

function applyDefaultPortByType(type: Application["type"] | undefined) {
  if (type === "frontend") {
    editingApp.value.port = 80;
    return;
  }
  if (type === "backend") {
    editingApp.value.port = 8080;
  }
}

function openAdd() {
  editingApp.value = {
    id: "",
    status: "running",
    env: "prod",
    type: "backend",
    port: 8080,
    is_deleted: 0,
    created_at: "",
    updated_at: "",
  };
  techStackList.value = [];
  ownerList.value = [];
  topTechStackOptions.value = [];
  ownerOptions.value = [];
  isEditing.value = false;
  drawerVisible.value = true;
  fetchTopTechStackOptions(editingApp.value.type);
  fetchOwnerOptions();
}

function openEdit(row: Application) {
  editingApp.value = { ...row };
  techStackList.value = parseTechStack(row.tech_stack);
  ownerList.value = normalizeOwners(row.owners, row.owner);
  isEditing.value = true;
  drawerVisible.value = true;
  fetchTopTechStackOptions(editingApp.value.type);
  fetchOwnerOptions();
}

function openCopy(row: Application) {
  editingApp.value = buildApplicationCopyDraft(row);
  techStackList.value = parseTechStack(editingApp.value.tech_stack);
  ownerList.value = normalizeOwners(editingApp.value.owners, editingApp.value.owner);
  isEditing.value = false;
  drawerVisible.value = true;
  fetchTopTechStackOptions(editingApp.value.type);
  fetchOwnerOptions();
}

function handleTypeChange(type: Application["type"] | undefined) {
  applyDefaultPortByType(type);
  fetchTopTechStackOptions(type);
}

async function fetchTopTechStackOptions(type: Application["type"] | undefined) {
  try {
    topTechStackOptions.value = await listTopApplicationTechStacks(10, resolveTechStackSide(type));
  } catch {
    // error shown by tauriInvoke
  }
}

async function fetchOwnerOptions() {
  try {
    ownerOptions.value = await listApplicationOwnerCandidates(100);
  } catch {
    // error shown by tauriInvoke
  }
}

async function handleSave() {
  const owners = normalizeOwners(ownerList.value);
  const payload: Partial<Application> = {
    id: "",
    is_deleted: 0,
    created_at: "",
    updated_at: "",
    ...editingApp.value,
    owner: owners[0],
    owners,
    tech_stack: techStackToText(techStackList.value),
  };
  saveLoading.value = true;
  try {
    await saveApplication(payload);
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    drawerVisible.value = false;
    fetchData();
    fetchTopTechStackOptions((payload.type as Application["type"] | undefined) ?? editingApp.value.type);
    fetchOwnerOptions();
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

function typeLabel(type: string) {
  return (
    ({
      frontend: "前端",
      backend: "后端",
      gateway: "网关",
      batch_job: "批处理",
      microservice: "微服务",
      other: "其他",
    } as Record<string, string>)[type] || type
  );
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

onMounted(() => {
  fetchData();
  fetchTopTechStackOptions("backend");
  fetchOwnerOptions();
});
</script>

<template>
  <div class="resource-view">
    <SearchToolbar
      v-model:search-text="searchText"
      v-model:filters="listFilters"
      search-placeholder="搜索服务名/地址/负责人/技术栈..."
      :fields="toolbarFields"
      @query="handleToolbarQuery"
    >
      <template #actions="{ hasActiveFilters, reset }">
        <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
        <el-button type="primary" @click="openAdd">新增应用</el-button>
      </template>
    </SearchToolbar>

    <el-table :data="data" v-loading="loading" border stripe class="w-full im-table-fixed-ops">
      <el-table-column prop="name" label="服务名称" min-width="150" align="center" />
      <el-table-column prop="type" label="类型" width="100" align="center">
        <template #default="{ row }">{{ typeLabel(row.type) }}</template>
      </el-table-column>
      <el-table-column label="地址" min-width="180" align="center">
        <template #default="{ row }"> {{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }} </template>
      </el-table-column>
      <el-table-column prop="env" label="环境" width="80" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="tech_stack" label="技术栈" width="140" show-overflow-tooltip align="center" />
      <el-table-column label="负责人" min-width="160" align="center">
        <template #default="{ row }">
          <div v-if="ownersForRow(row).length > 0" class="owner-tags">
            <el-tag v-for="owner in ownersForRow(row)" :key="owner" size="small" effect="plain">{{ owner }}</el-tag>
          </div>
          <span v-else>-</span>
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
      :title="isEditing ? '编辑应用' : '新增应用'"
      width="700px"
      align-center
      destroy-on-close
    >
      <el-form :model="editingApp" label-width="96px">
        <el-divider content-position="left">基础信息</el-divider>
        <el-form-item label="服务名称" required>
          <el-input v-model="editingApp.name" placeholder="请输入服务名称" />
        </el-form-item>
        <el-form-item label="类型" required>
          <el-select v-model="editingApp.type" class="w-full" @change="(v) => handleTypeChange(v as Application['type'])">
            <el-option label="前端" value="frontend" />
            <el-option label="后端" value="backend" />
            <el-option label="网关" value="gateway" />
            <el-option label="批处理" value="batch_job" />
            <el-option label="微服务" value="microservice" />
            <el-option label="其他" value="other" />
          </el-select>
        </el-form-item>
        <el-form-item label="访问地址">
          <el-input v-model="editingApp.address" placeholder="如 api.example.com、https://app.example.com 或 192.168.1.100" />
        </el-form-item>
        <el-form-item label="端口">
          <el-input-number v-model="editingApp.port" :min="1" :max="65535" class="w-full" />
        </el-form-item>

        <el-divider content-position="left">部署信息</el-divider>
        <el-form-item label="技术栈">
          <el-select
            v-model="techStackList"
            class="w-full"
            multiple
            filterable
            allow-create
            default-first-option
            :reserve-keyword="false"
            placeholder="输入技术栈进行筛选，按回车可新增"
            @change="(values) => handleTechStackChange(values as string[])"
          >
            <el-option v-for="item in techStackSuggestions" :key="item" :label="item" :value="item" />
          </el-select>
        </el-form-item>
        <el-form-item label="部署方式">
          <el-select v-model="editingApp.deploy_mode" clearable placeholder="请选择部署方式" class="w-full">
            <el-option v-for="item in deployModeOptions" :key="item.value" :label="item.label" :value="item.value" />
          </el-select>
        </el-form-item>
        <el-form-item label="环境" required>
          <el-select v-model="editingApp.env" class="w-full">
            <el-option label="生产" value="prod" />
            <el-option label="开发" value="dev" />
            <el-option label="测试" value="test" />
          </el-select>
        </el-form-item>
        <el-form-item label="Git仓库">
          <el-input v-model="editingApp.git_repo" placeholder="Git 仓库地址" />
        </el-form-item>
        <el-form-item label="负责人">
          <el-select
            v-model="ownerList"
            class="w-full"
            multiple
            filterable
            allow-create
            default-first-option
            :reserve-keyword="false"
            placeholder="输入负责人姓名进行筛选，按回车可新增"
            @change="(values) => handleOwnerChange(values as string[])"
          >
            <el-option v-for="item in ownerSuggestions" :key="item" :label="item" :value="item" />
          </el-select>
        </el-form-item>

        <el-divider content-position="left">运维信息</el-divider>
        <el-form-item label="状态" required>
          <el-select v-model="editingApp.status" class="w-full">
            <el-option label="运行中" value="running" />
            <el-option label="已停止" value="stopped" />
            <el-option label="维护中" value="maintenance" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingApp.description" type="textarea" :rows="3" maxlength="300" show-word-limit />
        </el-form-item>
      </el-form>

      <DeploymentPanel
        v-if="isEditing && editingApp.id"
        :resource-id="editingApp.id!"
        resource-type="application"
        :default-port="editingApp.port"
      />
      <DependencyPanel v-if="isEditing && editingApp.id" :resource-id="editingApp.id!" resource-type="application" />

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
  grid-template-columns: minmax(0, 2.8fr) minmax(0, 1.7fr) minmax(0, 1.4fr) minmax(0, 1.9fr);
}
.filter-row-secondary {
  grid-template-columns: minmax(0, 1fr) auto;
}
.filter-field {
  display: grid;
  grid-template-columns: 52px minmax(0, 1fr);
  align-items: center;
  gap: 8px;
  width: 100%;
  min-width: 0;
}
.deploy-field {
  grid-template-columns: 64px minmax(0, 1fr);
  max-width: 32%;
  justify-self: start;
}
.search-field {
  grid-template-columns: 52px minmax(0, 1fr);
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
  flex-shrink: 0;
  text-align: right;
}
.search-filter {
  width: 100%;
}
.type-filter {
  width: 100%;
  min-width: 0;
}
.env-filter {
  width: 100%;
  min-width: 0;
}
.status-filter {
  width: 100%;
  min-width: 0;
}
.deploy-filter {
  width: 100%;
  min-width: 0;
}
:deep(.type-filter .el-select__selection),
:deep(.env-filter .el-select__selection),
:deep(.status-filter .el-select__selection),
:deep(.deploy-filter .el-select__selection) {
  flex-wrap: nowrap;
  overflow: hidden;
}
:deep(.type-filter .el-select__selected-item),
:deep(.env-filter .el-select__selected-item),
:deep(.status-filter .el-select__selected-item),
:deep(.deploy-filter .el-select__selected-item) {
  max-width: 100%;
}
@media (max-width: 1080px) {
  .filter-row-primary {
    grid-template-columns: minmax(0, 2.2fr) minmax(0, 1.3fr) minmax(0, 1.2fr) minmax(0, 1.5fr);
  }
  .deploy-field {
    max-width: 32%;
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
  .filter-row {
    width: 100%;
  }
  .filter-row-primary,
  .filter-row-secondary {
    grid-template-columns: 1fr;
  }
  .filter-field {
    width: 100%;
  }
  .filter-actions {
    width: 100%;
    justify-self: start;
    justify-content: flex-start;
  }
  .filter-field,
  .deploy-field {
    grid-template-columns: 56px minmax(0, 1fr);
  }
  .deploy-field {
    max-width: 100%;
  }
  .search-filter,
  .type-filter,
  .env-filter,
  .status-filter,
  .deploy-filter {
    width: 100%;
    min-width: 0;
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

.owner-tags {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 4px;
}
</style>


