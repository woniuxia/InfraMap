<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import type { Application, BusinessApplication } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import {
  attachServicesToBusinessApplication,
  getBusinessApplication,
  listBusinessApplications,
  listServicesByBusinessApplication,
  listUnassignedApplicationServices,
  saveBusinessApplication,
  softDeleteBusinessApplication,
  detachServiceFromBusinessApplication,
} from "@/api/business-applications";
import { useResourceList } from "@/composables/useResourceList";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";

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
} = useResourceList<BusinessApplication>({
  listFn: listBusinessApplications,
  deleteFn: softDeleteBusinessApplication,
  entityLabel: "业务应用",
});

const searchText = ref("");
const listFilters = ref<{ status: string[]; env: string[] }>({ status: [], env: [] });
const selectedBusinessId = ref("");
const selectedBusiness = ref<BusinessApplication | null>(null);
const detailLoading = ref(false);
const frontendServices = ref<Application[]>([]);
const backendServices = ref<Application[]>([]);

const drawerVisible = ref(false);
const isEditing = ref(false);
const saveLoading = ref(false);
const editingBusiness = ref<Partial<BusinessApplication>>({});

const attachVisible = ref(false);
const attachLoading = ref(false);
const unassignedLoading = ref(false);
const unassignedServices = ref<Application[]>([]);
const selectedAttachIds = ref<string[]>([]);
const unassignedSearch = ref("");

const envOptions = [
  { label: "生产", value: "prod" },
  { label: "开发", value: "dev" },
  { label: "测试", value: "test" },
];
const statusOptions = [
  { label: "激活", value: "active" },
  { label: "停用", value: "inactive" },
];
const toolbarFields: SearchFieldConfig[] = [
  {
    key: "status",
    queryKey: "status",
    label: "状态",
    type: "multi-select",
    width: "md",
    options: statusOptions,
  },
  {
    key: "env",
    queryKey: "env",
    label: "环境",
    type: "multi-select",
    width: "sm",
    options: envOptions,
  },
];

const attachCandidates = computed(() => {
  if (!unassignedSearch.value.trim()) return unassignedServices.value;
  const keyword = unassignedSearch.value.trim().toLowerCase();
  return unassignedServices.value.filter((item) =>
    [item.name, item.address, item.tech_stack]
      .filter(Boolean)
      .some((text) => String(text).toLowerCase().includes(keyword))
  );
});

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

function envLabel(env?: string) {
  if (!env) return "-";
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env;
}

function statusLabel(status?: string) {
  if (!status) return "-";
  return ({ active: "激活", inactive: "停用" } as Record<string, string>)[status] || status;
}

function serviceTypeLabel(type: string) {
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

async function loadDetail(id: string) {
  if (!id) return;
  detailLoading.value = true;
  try {
    const [business, services] = await Promise.all([
      getBusinessApplication(id),
      listServicesByBusinessApplication(id),
    ]);
    selectedBusiness.value = business;
    frontendServices.value = services.frontend;
    backendServices.value = services.backend;
  } catch {
    // error shown by tauriInvoke
  } finally {
    detailLoading.value = false;
  }
}

function selectBusiness(row: BusinessApplication) {
  selectedBusinessId.value = row.id;
  loadDetail(row.id);
}

function openAdd() {
  editingBusiness.value = {
    id: "",
    status: "active",
    env: "prod",
    is_deleted: 0,
    created_at: "",
    updated_at: "",
  };
  isEditing.value = false;
  drawerVisible.value = true;
}

function openEdit(row: BusinessApplication) {
  editingBusiness.value = { ...row };
  isEditing.value = true;
  drawerVisible.value = true;
}

async function handleSave() {
  const payload: Partial<BusinessApplication> = {
    id: "",
    is_deleted: 0,
    created_at: "",
    updated_at: "",
    ...editingBusiness.value,
  };
  saveLoading.value = true;
  try {
    const id = await saveBusinessApplication(payload);
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    drawerVisible.value = false;
    await fetchData();
    selectedBusinessId.value = id;
    await loadDetail(id);
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

async function handleDeleteBusiness(row: BusinessApplication) {
  const deletingSelected = selectedBusinessId.value === row.id;
  await handleDelete(row.id, row.name);
  await fetchData();
  if (deletingSelected) {
    selectedBusinessId.value = "";
    selectedBusiness.value = null;
    frontendServices.value = [];
    backendServices.value = [];
  }
}

async function openAttach() {
  if (!selectedBusinessId.value) {
    ElMessage.warning("请先选择一个业务应用");
    return;
  }
  attachVisible.value = true;
  selectedAttachIds.value = [];
  unassignedSearch.value = "";
  unassignedLoading.value = true;
  try {
    const filters: Record<string, string> = {};
    if (selectedBusiness.value?.env) {
      filters.env = selectedBusiness.value.env;
    }
    const result = await listUnassignedApplicationServices({
      page: 1,
      page_size: 500,
      filters,
    });
    unassignedServices.value = result.data;
  } catch {
    // error shown by tauriInvoke
  } finally {
    unassignedLoading.value = false;
  }
}

async function handleAttach() {
  if (!selectedBusinessId.value || selectedAttachIds.value.length === 0) {
    ElMessage.warning("请选择至少一个服务");
    return;
  }
  attachLoading.value = true;
  try {
    const result = await attachServicesToBusinessApplication(
      selectedBusinessId.value,
      selectedAttachIds.value
    );
    ElMessage.success(`已挂载 ${result.attached_count} 个服务`);
    if (result.skipped_count > 0) {
      ElMessage.info(`已跳过 ${result.skipped_count} 个已归属服务`);
    }
    attachVisible.value = false;
    await loadDetail(selectedBusinessId.value);
  } catch {
    // error shown by tauriInvoke
  } finally {
    attachLoading.value = false;
  }
}

function handleAttachSelection(rows: Application[]) {
  selectedAttachIds.value = rows.map((row) => row.id);
}

async function handleDetach(service: Application) {
  if (!selectedBusinessId.value) return;
  try {
    await ElMessageBox.confirm(
      `确认将服务 "${service.name}" 从当前业务应用解绑？`,
      "确认解绑",
      {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning",
      }
    );
    await detachServiceFromBusinessApplication(selectedBusinessId.value, service.id);
    ElMessage.success("解绑成功");
    await loadDetail(selectedBusinessId.value);
  } catch {
    // cancelled or error already shown by tauriInvoke
  }
}

onMounted(fetchData);
</script>

<template>
  <div class="business-app-view">
    <div class="list-pane">
      <SearchToolbar
        v-model:search-text="searchText"
        v-model:filters="listFilters"
        search-placeholder="搜索业务应用名称/负责人..."
        :fields="toolbarFields"
        @query="handleToolbarQuery"
      >
        <template #actions="{ hasActiveFilters, reset }">
          <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
          <el-button type="primary" @click="openAdd">新增业务应用</el-button>
        </template>
      </SearchToolbar>

      <el-table
        :data="data"
        v-loading="loading"
        border
        stripe
        highlight-current-row
        row-key="id"
        :current-row-key="selectedBusinessId"
        @row-click="selectBusiness"
      >
        <el-table-column prop="name" label="业务应用" min-width="140" />
        <el-table-column prop="owner" label="负责人" min-width="100" />
        <el-table-column label="环境" width="80" align="center">
          <template #default="{ row }">{{ envLabel(row.env) }}</template>
        </el-table-column>
        <el-table-column label="状态" width="80" align="center">
          <template #default="{ row }">{{ statusLabel(row.status) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="150" align="center" fixed="right">
          <template #default="{ row }">
            <el-button text type="primary" size="small" @click.stop="openEdit(row)">编辑</el-button>
            <el-button text type="danger" size="small" @click.stop="handleDeleteBusiness(row)">删除</el-button>
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
    </div>

    <div class="detail-pane" v-loading="detailLoading">
      <template v-if="selectedBusiness">
        <div class="detail-header">
          <div>
            <h3>{{ selectedBusiness.name }}</h3>
            <p>
              负责人: {{ selectedBusiness.owner || "-" }} | 环境: {{ envLabel(selectedBusiness.env) }} | 状态:
              {{ statusLabel(selectedBusiness.status) }}
            </p>
          </div>
          <el-button type="primary" @click="openAttach">挂载服务</el-button>
        </div>

        <el-card shadow="never" class="service-card">
          <template #header>
            <div class="card-header">前端服务 ({{ frontendServices.length }})</div>
          </template>
          <el-table :data="frontendServices" size="small" stripe>
            <el-table-column prop="name" label="服务名称" min-width="140" />
            <el-table-column prop="type" label="类型" width="90" align="center">
              <template #default="{ row }">{{ serviceTypeLabel(row.type) }}</template>
            </el-table-column>
            <el-table-column label="地址" min-width="170">
              <template #default="{ row }">{{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }}</template>
            </el-table-column>
            <el-table-column label="操作" width="90" align="center">
              <template #default="{ row }">
                <el-button text type="danger" size="small" @click="handleDetach(row)">解绑</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>

        <el-card shadow="never" class="service-card">
          <template #header>
            <div class="card-header">后端服务 ({{ backendServices.length }})</div>
          </template>
          <el-table :data="backendServices" size="small" stripe>
            <el-table-column prop="name" label="服务名称" min-width="140" />
            <el-table-column prop="type" label="类型" width="90" align="center">
              <template #default="{ row }">{{ serviceTypeLabel(row.type) }}</template>
            </el-table-column>
            <el-table-column label="地址" min-width="170">
              <template #default="{ row }">{{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }}</template>
            </el-table-column>
            <el-table-column label="操作" width="90" align="center">
              <template #default="{ row }">
                <el-button text type="danger" size="small" @click="handleDetach(row)">解绑</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </template>
      <el-empty v-else description="请选择左侧业务应用进行维护" :image-size="72" />
    </div>

    <el-dialog v-model="drawerVisible" :title="isEditing ? '编辑业务应用' : '新增业务应用'" width="520px">
      <el-form :model="editingBusiness" label-width="96px">
        <el-form-item label="应用名称" required>
          <el-input v-model="editingBusiness.name" placeholder="请输入业务应用名称" />
        </el-form-item>
        <el-form-item label="编码">
          <el-input v-model="editingBusiness.code" placeholder="如 PAY、ORDER" />
        </el-form-item>
        <el-form-item label="负责人">
          <el-input v-model="editingBusiness.owner" placeholder="负责人姓名" />
        </el-form-item>
        <el-form-item label="环境">
          <el-select v-model="editingBusiness.env" class="w-full" clearable placeholder="可选">
            <el-option v-for="item in envOptions" :key="item.value" :label="item.label" :value="item.value" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态" required>
          <el-select v-model="editingBusiness.status" class="w-full">
            <el-option label="激活" value="active" />
            <el-option label="停用" value="inactive" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingBusiness.description" type="textarea" :rows="3" maxlength="300" show-word-limit />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="drawerVisible = false">取消</el-button>
        <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="attachVisible" title="挂载应用服务" width="680px">
      <div class="attach-toolbar">
        <el-input v-model="unassignedSearch" placeholder="搜索服务名称/地址/技术栈" clearable />
      </div>
      <el-table
        :data="attachCandidates"
        v-loading="unassignedLoading"
        row-key="id"
        size="small"
        @selection-change="handleAttachSelection"
      >
        <el-table-column type="selection" width="50" />
        <el-table-column prop="name" label="服务名称" min-width="160" />
        <el-table-column prop="type" label="类型" width="100" align="center">
          <template #default="{ row }">{{ serviceTypeLabel(row.type) }}</template>
        </el-table-column>
        <el-table-column label="环境" width="90" align="center">
          <template #default="{ row }">{{ envLabel(row.env) }}</template>
        </el-table-column>
        <el-table-column label="地址" min-width="180">
          <template #default="{ row }">{{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }}</template>
        </el-table-column>
      </el-table>
      <template #footer>
        <el-button @click="attachVisible = false">取消</el-button>
        <el-button type="primary" :loading="attachLoading" @click="handleAttach">挂载</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.business-app-view {
  display: grid;
  grid-template-columns: minmax(420px, 44%) minmax(0, 1fr);
  gap: 16px;
}

.list-pane,
.detail-pane {
  min-width: 0;
}

.detail-pane {
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-md);
  background: var(--im-surface-0);
  padding: 12px;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.detail-header h3 {
  margin: 0;
  font-size: 18px;
  color: var(--im-text-primary);
}

.detail-header p {
  margin: 6px 0 0;
  color: var(--im-text-secondary);
  font-size: 13px;
}

.service-card + .service-card {
  margin-top: 12px;
}

.card-header {
  font-weight: 600;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.attach-toolbar {
  margin-bottom: 12px;
}

@media (max-width: 1200px) {
  .business-app-view {
    grid-template-columns: 1fr;
  }
}
</style>
