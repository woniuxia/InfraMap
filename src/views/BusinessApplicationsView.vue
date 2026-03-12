<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import type { Application, BusinessApplication, Contact } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listApplications } from "@/api/applications";
import { getContact, listContacts, saveContact } from "@/api/contacts";
import {
  listBusinessApplications,
  listServicesByBusinessApplication,
  replaceServicesByBusinessApplication,
  saveBusinessApplication,
  deleteBusinessApplication,
} from "@/api/business-applications";
import { useResourceList } from "@/composables/useResourceList";
import { useEnvStore } from "@/stores/env";
import {
  BUSINESS_APPLICATION_STATUS_OPTIONS,
  ENV_OPTIONS,
  getBusinessApplicationStatusLabel,
  getEnvLabel,
} from "@/constants/options";

const envStore = useEnvStore();
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
const listFilters = ref<{ status: string[]; owner: string[] }>({
  status: [],
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
const ownerContactIdList = ref<string[]>([]);
const ownerContactOptionsLoading = ref(false);
const ownerContactOptions = ref<Contact[]>([]);
const searchedOwnerKeyword = ref("");
const quickCreatingOwner = ref(false);

const ownerFilterContactsLoading = ref(false);
const ownerFilterContacts = ref<Contact[]>([]);

const ownerFilterOptions = computed(() =>
  ownerFilterContacts.value.map((contact) => ({
    label: contactOptionLabel(contact),
    value: contact.id,
  })),
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
const canQuickCreateOwner = computed(() => {
  const keyword = searchedOwnerKeyword.value.trim();
  return keyword.length > 0 && ownerContactOptions.value.length === 0;
});

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

function normalizeOwnerContactIds(ownerContactIds?: string[]) {
  const values = [...(ownerContactIds ?? [])]
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
  return Array.from(new Set(values));
}

function contactOptionLabel(contact: Contact) {
  const name = contact.name?.trim() || "-";
  const phone = contact.phone?.trim();
  const email = contact.email?.trim();
  const meta = [phone, email].filter(Boolean).join(" / ");
  return meta ? `${name}（${meta}）` : name;
}

async function fetchOwnerFilterContacts() {
  ownerFilterContactsLoading.value = true;
  try {
    const result = await listContacts({
      page: 1,
      page_size: 500,
      search: "",
    });
    ownerFilterContacts.value = result.data;
  } catch {
    // error shown by tauriInvoke
  } finally {
    ownerFilterContactsLoading.value = false;
  }
}

async function fetchOwnerContactOptions(keyword: string) {
  ownerContactOptionsLoading.value = true;
  try {
    const result = await listContacts({
      page: 1,
      page_size: 50,
      search: keyword.trim(),
    });
    ownerContactOptions.value = result.data;
  } catch {
    // error shown by tauriInvoke
  } finally {
    ownerContactOptionsLoading.value = false;
  }
}

async function preloadOwnerContacts(contactIds: string[]) {
  const normalizedIds = normalizeOwnerContactIds(contactIds);
  if (normalizedIds.length === 0) return;

  try {
    const results = await Promise.all(
      normalizedIds.map(async (id) => {
        try {
          return await getContact(id);
        } catch {
          return null;
        }
      }),
    );

    const contacts = results.filter((item): item is Contact => Boolean(item));
    if (contacts.length > 0) {
      ownerContactOptions.value = contacts;
    }
  } catch {
    // error shown by tauriInvoke
  }
}

function handleOwnerSearch(keyword: string) {
  searchedOwnerKeyword.value = keyword.trim();
  void fetchOwnerContactOptions(searchedOwnerKeyword.value);
}

function handleOwnerDropdownVisible(visible: boolean) {
  if (!visible) return;
  void fetchOwnerContactOptions(searchedOwnerKeyword.value);
}

async function handleQuickCreateOwner() {
  const keyword = searchedOwnerKeyword.value.trim();
  if (!keyword || quickCreatingOwner.value) return;

  quickCreatingOwner.value = true;
  try {
    const contactId = await saveContact({
      id: "",
      name: keyword,
    });
    ownerContactIdList.value = normalizeOwnerContactIds([...ownerContactIdList.value, contactId]);
    ownerContactOptions.value = [
      {
        id: contactId,
        name: keyword,
        phone: undefined,
        email: undefined,
        remark: undefined,
        created_at: "",
        updated_at: "",
      },
    ];
    ownerFilterContacts.value = [
      ...ownerFilterContacts.value,
      {
        id: contactId,
        name: keyword,
        phone: undefined,
        email: undefined,
        remark: undefined,
        created_at: "",
        updated_at: "",
      },
    ];
    ElMessage.success("联系人已创建并回填");
  } catch {
    // error shown by tauriInvoke
  } finally {
    quickCreatingOwner.value = false;
  }
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
  await fetchOwnerFilterContacts();
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
    env: envStore.currentEnv,
    created_at: "",
    updated_at: "",
  };
  isEditing.value = false;
  selectedServiceIds.value = [];
  ownerContactIdList.value = [];
  searchedOwnerKeyword.value = "";
  ownerContactOptions.value = [];
  drawerVisible.value = true;
  await Promise.all([loadServiceOptions(), fetchOwnerOptions()]);
}

async function openEdit(row: BusinessApplication) {
  editingBusiness.value = { ...row };
  isEditing.value = true;
  selectedServiceIds.value = [];
  ownerContactIdList.value = normalizeOwnerContactIds(row.owner_contact_ids);
  searchedOwnerKeyword.value = "";
  ownerContactOptions.value = [];
  void preloadOwnerContacts(ownerContactIdList.value);
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
  } catch {
    // error shown by tauriInvoke
  } finally {
    serviceOptionsLoading.value = false;
  }
  await fetchOwnerOptions();
}

async function handleSave() {
  const editingBeforeSave = isEditing.value;
  const ownerContactIds = normalizeOwnerContactIds(ownerContactIdList.value);
  const {
    owners: _ignoredOwners,
    owner_contact_ids: _ignoredIds,
    ...businessDraft
  } = editingBusiness.value;
  const payload: Partial<BusinessApplication> = {
    id: "",
    created_at: "",
    updated_at: "",
    ...businessDraft,
    owner_contact_ids: ownerContactIds,
  };
  saveLoading.value = true;
  try {
    const id = await saveBusinessApplication(payload);
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
            v-model="ownerContactIdList"
            multiple
            filterable
            remote
            clearable
            class="w-full"
            placeholder="搜索联系人（支持姓名/电话/邮箱，多选）"
            :loading="ownerContactOptionsLoading"
            :remote-method="handleOwnerSearch"
            @visible-change="handleOwnerDropdownVisible"
          >
            <el-option
              v-for="contact in ownerContactOptions"
              :key="contact.id"
              :label="contactOptionLabel(contact)"
              :value="contact.id"
            />
            <template #empty>
              <div class="owner-empty">
                <template v-if="!searchedOwnerKeyword">暂无联系人</template>
                <template v-else-if="canQuickCreateOwner">
                  <span>未找到联系人：{{ searchedOwnerKeyword }}</span>
                  <el-button
                    type="primary"
                    text
                    :loading="quickCreatingOwner"
                    @click="handleQuickCreateOwner"
                  >
                    点击新增并回填
                  </el-button>
                </template>
                <template v-else>未找到匹配联系人</template>
              </div>
            </template>
          </el-select>
          <div class="owner-hint">负责人基于联系人 ID 绑定，联系人姓名后续可修改。</div>
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

.owner-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--im-text-tertiary);
}

.owner-empty {
  padding: 8px 12px;
  font-size: 12px;
  color: var(--im-text-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
</style>
