<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import type { Application, BusinessApplication } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listApplications } from "@/api/applications";
import {
  listBusinessApplications,
  listServicesByBusinessApplication,
  replaceServicesByBusinessApplication,
  saveBusinessApplication,
  deleteBusinessApplication,
} from "@/api/business-applications";
import {
  listBusinessApplicationOwnerTerms,
  listResourceTerms,
  saveResourceTerms,
} from "@/api/taxonomy";
import { useResourceList } from "@/composables/useResourceList";
import {
  BUSINESS_APPLICATION_STATUS_OPTIONS,
  ENV_OPTIONS,
  getBusinessApplicationStatusLabel,
  getEnvLabel,
} from "@/constants/options";
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
  deleteFn: deleteBusinessApplication,
  entityLabel: "业务应用",
});

const searchText = ref("");
const listFilters = ref<{ status: string[]; env: string[]; owner: string[] }>({
  status: [],
  env: [],
  owner: [],
});
interface ServiceCellItem {
  id: string;
  name: string;
  addressText: string;
}

const serviceSummaryMap = ref<
  Record<string, { frontend: ServiceCellItem[]; backend: ServiceCellItem[] }>
>({});
let serviceSummaryToken = 0;

const drawerVisible = ref(false);
const isEditing = ref(false);
const saveLoading = ref(false);
const editingBusiness = ref<Partial<BusinessApplication>>({});

const serviceOptionsLoading = ref(false);
const serviceOptions = ref<Application[]>([]);
const selectedServiceIds = ref<string[]>([]);
const ownerList = ref<string[]>([]);
const ownerOptions = ref<string[]>([]);

const ownerFilterOptions = computed(() =>
  ownerOptions.value.map((item) => ({ label: item, value: item })),
);
const toolbarFields = computed<SearchFieldConfig[]>(() => [
  {
    key: "status",
    queryKey: "status",
    label: "状态",
    type: "multi-select",
    width: "md",
    options: [...BUSINESS_APPLICATION_STATUS_OPTIONS],
  },
  {
    key: "env",
    queryKey: "env",
    label: "环境",
    type: "multi-select",
    width: "sm",
    options: [...ENV_OPTIONS],
  },
  {
    key: "owner",
    queryKey: "owner",
    label: "负责人",
    type: "multi-select",
    width: "md",
    options: ownerFilterOptions.value,
  },
]);

const selectedServiceIdSet = computed(() => new Set(selectedServiceIds.value));
const editableServiceOptions = computed(() => {
  if (!editingBusiness.value.env) return serviceOptions.value;
  const businessEnv = editingBusiness.value.env;
  return serviceOptions.value.filter(
    (item) => item.env === businessEnv || selectedServiceIdSet.value.has(item.id),
  );
});
const frontendServiceOptions = computed(() =>
  editableServiceOptions.value.filter((item) => item.type === "frontend"),
);
const backendServiceOptions = computed(() =>
  editableServiceOptions.value.filter((item) => item.type !== "frontend"),
);
const selectedServiceSummary = computed(() => `已选择 ${selectedServiceIds.value.length} 个服务`);
const ownerSuggestions = computed(() =>
  normalizeOwners([...ownerOptions.value, ...ownerList.value]),
);

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

function serviceTypeLabel(type: string) {
  return (
    (
      {
        frontend: "前端",
        backend: "后端",
        gateway: "网关",
        batch_job: "批处理",
        microservice: "微服务",
        other: "其他",
      } as Record<string, string>
    )[type] || type
  );
}

function normalizeOwners(values: string[]) {
  return Array.from(new Set(values.map((item) => item.trim()).filter((item) => item.length > 0)));
}

function handleOwnerChange(values: string[]) {
  ownerList.value = normalizeOwners(values);
}

function ownersForRow(row: BusinessApplication) {
  return normalizeOwners(row.owners ?? []);
}

function serviceAddressLabel(service: Application) {
  if (!service.address) return "-";
  return `${service.address}${service.port ? ":" + service.port : ""}`;
}

function buildServiceCellItems(services: Application[]): ServiceCellItem[] {
  return services.map((service) => ({
    id: service.id,
    name: service.name?.trim() || "-",
    addressText: serviceAddressLabel(service),
  }));
}

async function loadServiceSummaries(rows: BusinessApplication[]) {
  const currentToken = ++serviceSummaryToken;

  if (rows.length === 0) {
    serviceSummaryMap.value = {};
    return;
  }

  const summaries = await Promise.all(
    rows.map(async (row) => {
      try {
        const services = await listServicesByBusinessApplication(row.id);
        return [
          row.id,
          {
            frontend: buildServiceCellItems(services.frontend),
            backend: buildServiceCellItems(services.backend),
          },
        ] as const;
      } catch {
        return [row.id, { frontend: [], backend: [] }] as const;
      }
    }),
  );

  if (currentToken !== serviceSummaryToken) return;
  serviceSummaryMap.value = Object.fromEntries(summaries);
}

function frontendItems(row: BusinessApplication) {
  return serviceSummaryMap.value[row.id]?.frontend ?? [];
}

function backendItems(row: BusinessApplication) {
  return serviceSummaryMap.value[row.id]?.backend ?? [];
}

function currentEditingBusinessId() {
  const raw = editingBusiness.value.id;
  return typeof raw === "string" ? raw.trim() : "";
}

function isServiceOptionLocked(service: Application) {
  const ownerBusinessId = service.business_application_id?.trim();
  if (!ownerBusinessId) return false;
  return ownerBusinessId !== currentEditingBusinessId();
}

function serviceOptionLabel(service: Application) {
  return `${service.name?.trim() || "-"}（${serviceTypeLabel(service.type)}）`;
}

async function fetchOwnerOptions() {
  try {
    ownerOptions.value = await listBusinessApplicationOwnerTerms(100);
  } catch {
    // error shown by tauriInvoke
  }
}

async function loadBusinessOwnerTerms(resourceId: string) {
  try {
    const owners = await listResourceTerms({
      resource_type: "business_application",
      resource_id: resourceId,
      field_key: "owner",
    });
    ownerList.value = normalizeOwners(owners);
  } catch {
    // error shown by tauriInvoke
  }
}

function normalizeSelectedServiceIds(ids: string[]) {
  const seen = new Set<string>();
  const serviceMap = new Map(serviceOptions.value.map((item) => [item.id, item]));
  const nextIds: string[] = [];

  for (const rawId of ids) {
    const id = rawId.trim();
    if (!id || seen.has(id)) continue;

    const service = serviceMap.get(id);
    if (!service) continue;
    if (editingBusiness.value.env && service.env !== editingBusiness.value.env) continue;
    if (isServiceOptionLocked(service)) continue;

    seen.add(id);
    nextIds.push(id);
  }

  return nextIds;
}

function syncSelectedServiceIds(ids = selectedServiceIds.value) {
  selectedServiceIds.value = normalizeSelectedServiceIds(ids);
}

async function loadServiceOptions() {
  serviceOptionsLoading.value = true;
  try {
    const result = await listApplications({
      page: 1,
      page_size: 500,
    });
    serviceOptions.value = result.data;
    syncSelectedServiceIds();
  } catch {
    // error shown by tauriInvoke
  } finally {
    serviceOptionsLoading.value = false;
  }
}

function handleBusinessEnvChange() {
  syncSelectedServiceIds();
}

async function openAdd() {
  editingBusiness.value = {
    id: "",
    status: "active",
    env: "prod",
    created_at: "",
    updated_at: "",
  };
  isEditing.value = false;
  selectedServiceIds.value = [];
  ownerList.value = [];
  drawerVisible.value = true;
  await Promise.all([loadServiceOptions(), fetchOwnerOptions()]);
}

async function openEdit(row: BusinessApplication) {
  editingBusiness.value = { ...row };
  isEditing.value = true;
  selectedServiceIds.value = [];
  ownerList.value = ownersForRow(row);
  drawerVisible.value = true;
  serviceOptionsLoading.value = true;
  try {
    const [applicationsResult, servicesResult] = await Promise.all([
      listApplications({
        page: 1,
        page_size: 500,
      }),
      listServicesByBusinessApplication(row.id),
    ]);
    const attachedServices = [...servicesResult.frontend, ...servicesResult.backend];
    const mergedOptions = [...applicationsResult.data];
    const existingServiceIds = new Set(mergedOptions.map((item) => item.id));
    for (const service of attachedServices) {
      if (!existingServiceIds.has(service.id)) {
        mergedOptions.push(service);
        existingServiceIds.add(service.id);
      }
    }
    serviceOptions.value = mergedOptions;
    selectedServiceIds.value = attachedServices.map((service) => service.id);
    syncSelectedServiceIds();
    await loadBusinessOwnerTerms(row.id);
  } catch {
    // error shown by tauriInvoke
  } finally {
    serviceOptionsLoading.value = false;
  }
  await fetchOwnerOptions();
}

async function handleSave() {
  const editingBeforeSave = isEditing.value;
  const normalizedOwners = normalizeOwners(ownerList.value);
  const { owners: _ignoredOwners, ...businessDraft } = editingBusiness.value;
  const payload: Partial<BusinessApplication> = {
    id: "",
    created_at: "",
    updated_at: "",
    ...businessDraft,
  };
  saveLoading.value = true;
  try {
    const id = await saveBusinessApplication(payload);
    try {
      await saveResourceTerms({
        resource_type: "business_application",
        resource_id: id,
        field_key: "owner",
        values: normalizedOwners,
      });
    } catch {
      ElMessage.warning("业务应用已保存，负责人标签保存失败，请重新编辑后重试。");
    }
    editingBusiness.value.id = id;
    isEditing.value = true;
    const normalizedIds = normalizeSelectedServiceIds(selectedServiceIds.value);
    selectedServiceIds.value = normalizedIds;
    const replaceResult = await replaceServicesByBusinessApplication(id, normalizedIds);
    ElMessage.success(
      `${editingBeforeSave ? "更新成功" : "创建成功"}，新增挂载 ${replaceResult.attached_count} 个，解绑 ${replaceResult.detached_count} 个`,
    );
    drawerVisible.value = false;
    await fetchData();
    await fetchOwnerOptions();
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

async function handleDeleteBusiness(row: BusinessApplication) {
  await handleDelete(row.id, row.name);
  await fetchData();
}

watch(
  data,
  (rows) => {
    loadServiceSummaries(rows);
  },
  { immediate: true },
);

onMounted(async () => {
  await Promise.all([fetchData(), fetchOwnerOptions()]);
});
</script>

<template>
  <div class="business-app-view">
    <div class="list-pane">
      <SearchToolbar
        v-model:search-text="searchText"
        v-model:filters="listFilters"
        search-placeholder="搜索业务应用名称/编码/描述..."
        :fields="toolbarFields"
        @query="handleToolbarQuery"
      >
        <template #actions="{ hasActiveFilters, reset }">
          <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
          <el-button type="primary" @click="openAdd">新增业务应用</el-button>
        </template>
      </SearchToolbar>

      <el-table :data="data" v-loading="loading" border stripe row-key="id">
        <el-table-column prop="name" label="业务应用" min-width="140" />
        <el-table-column label="负责人" min-width="160" align="center">
          <template #default="{ row }">
            <div v-if="ownersForRow(row).length > 0" class="owner-tags">
              <el-tag
                v-for="owner in ownersForRow(row)"
                :key="`${row.id}-${owner}`"
                size="small"
                effect="plain"
              >
                {{ owner }}
              </el-tag>
            </div>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column label="环境" width="80" align="center">
          <template #default="{ row }">{{ getEnvLabel(row.env) }}</template>
        </el-table-column>
        <el-table-column label="状态" width="80" align="center">
          <template #default="{ row }">
            {{ getBusinessApplicationStatusLabel(row.status) }}
          </template>
        </el-table-column>
        <el-table-column label="前端服务" min-width="260">
          <template #default="{ row }">
            <ul v-if="frontendItems(row).length > 0" class="service-cell-list">
              <li v-for="item in frontendItems(row)" :key="item.id" class="service-cell-item">
                <span class="service-cell-item__name">{{ item.name }}</span>
                <span class="service-cell-item__address">{{ item.addressText }}</span>
              </li>
            </ul>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column label="后端服务" min-width="260">
          <template #default="{ row }">
            <ul v-if="backendItems(row).length > 0" class="service-cell-list">
              <li v-for="item in backendItems(row)" :key="item.id" class="service-cell-item">
                <span class="service-cell-item__name">{{ item.name }}</span>
                <span class="service-cell-item__address">{{ item.addressText }}</span>
              </li>
            </ul>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150" align="center" fixed="right">
          <template #default="{ row }">
            <el-button text type="primary" size="small" @click.stop="openEdit(row)">编辑</el-button>
            <el-button text type="danger" size="small" @click.stop="handleDeleteBusiness(row)">
              删除
            </el-button>
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

    <el-dialog
      v-model="drawerVisible"
      :title="isEditing ? '编辑业务应用' : '新增业务应用'"
      width="520px"
    >
      <el-form :model="editingBusiness" label-width="96px">
        <el-form-item label="应用名称" required>
          <el-input v-model="editingBusiness.name" placeholder="请输入业务应用名称" />
        </el-form-item>
        <el-form-item label="编码">
          <el-input v-model="editingBusiness.code" placeholder="如 PAY、ORDER" />
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
        <el-form-item label="环境">
          <el-select
            v-model="editingBusiness.env"
            class="w-full"
            clearable
            placeholder="可选"
            @change="handleBusinessEnvChange"
          >
            <el-option
              v-for="item in ENV_OPTIONS"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="状态" required>
          <el-select v-model="editingBusiness.status" class="w-full">
            <el-option
              v-for="item in BUSINESS_APPLICATION_STATUS_OPTIONS"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="editingBusiness.description"
            type="textarea"
            :rows="3"
            maxlength="300"
            show-word-limit
          />
        </el-form-item>
        <el-divider content-position="left">挂载服务</el-divider>
        <el-form-item label="应用服务">
          <el-select
            v-model="selectedServiceIds"
            class="w-full"
            multiple
            filterable
            clearable
            :loading="serviceOptionsLoading"
            placeholder="请选择要挂载的应用服务"
          >
            <el-option-group v-if="frontendServiceOptions.length > 0" label="前端服务">
              <el-option
                v-for="item in frontendServiceOptions"
                :key="item.id"
                :value="item.id"
                :label="serviceOptionLabel(item)"
                :disabled="isServiceOptionLocked(item)"
              >
                <div class="service-option">
                  <span class="service-option__name">{{ item.name }}</span>
                  <span class="service-option__meta">
                    {{ serviceTypeLabel(item.type) }} | {{ getEnvLabel(item.env) }} |
                    {{ serviceAddressLabel(item) }}
                  </span>
                  <span v-if="isServiceOptionLocked(item)" class="service-option__lock">
                    已归属 {{ item.business_application_name || item.business_application_id }}
                  </span>
                </div>
              </el-option>
            </el-option-group>
            <el-option-group v-if="backendServiceOptions.length > 0" label="后端服务">
              <el-option
                v-for="item in backendServiceOptions"
                :key="item.id"
                :value="item.id"
                :label="serviceOptionLabel(item)"
                :disabled="isServiceOptionLocked(item)"
              >
                <div class="service-option">
                  <span class="service-option__name">{{ item.name }}</span>
                  <span class="service-option__meta">
                    {{ serviceTypeLabel(item.type) }} | {{ getEnvLabel(item.env) }} |
                    {{ serviceAddressLabel(item) }}
                  </span>
                  <span v-if="isServiceOptionLocked(item)" class="service-option__lock">
                    已归属 {{ item.business_application_name || item.business_application_id }}
                  </span>
                </div>
              </el-option>
            </el-option-group>
          </el-select>
          <div class="service-hint">
            {{ selectedServiceSummary }}，保存后将按当前选择覆盖业务应用的挂载关系。
          </div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="drawerVisible = false">取消</el-button>
        <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.business-app-view {
  min-width: 0;
}

.list-pane {
  min-width: 0;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.service-cell-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.service-cell-item {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 12px;
}

.service-cell-item__name {
  color: var(--im-text-primary);
  font-weight: 500;
}

.service-cell-item__address {
  color: var(--im-text-secondary);
}

.service-hint {
  margin-top: 8px;
  color: var(--im-text-secondary);
  font-size: 12px;
  line-height: 1.5;
}

.service-option {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 2px 0;
}

.service-option__name {
  color: var(--im-text-primary);
  font-size: 13px;
}

.service-option__meta {
  color: var(--im-text-secondary);
  font-size: 12px;
}

.service-option__lock {
  color: var(--im-danger);
  font-size: 12px;
}

.owner-tags {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 4px;
}
</style>
