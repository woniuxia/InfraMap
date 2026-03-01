<script setup lang="ts">
import { computed, onMounted, nextTick, ref } from "vue";
import { ElMessage } from "element-plus";
import type { FormInstance, FormRules } from "element-plus";
import type { Host, IpAddress } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listHosts, saveHost, softDeleteHost } from "@/api/hosts";
import { listIpAddresses, saveIpAddress } from "@/api/ip-addresses";
import { bindHostIp, listHostIpBindings, unbindHostIp } from "@/api/host-ip-bindings";
import { useResourceList } from "@/composables/useResourceList";
import { buildHostCopyDraft } from "@/utils/resourceCopy";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import {
  COMMON_CPU_CORES_OPTIONS,
  COMMON_CPU_FREQ_OPTIONS,
  COMMON_CPU_THREADS_OPTIONS,
  COMMON_DISK_OPTIONS_GB,
  COMMON_RAM_OPTIONS_GB,
  normalizeCpuFreqValue,
  normalizePositiveIntegerValue,
} from "@/views/hostsHardwareOptions";

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
} = useResourceList<Host>({
  listFn: listHosts,
  deleteFn: softDeleteHost,
  entityLabel: "服务器",
});

const searchText = ref("");
const dialogVisible = ref(false);
const editingHost = ref<Partial<Host>>({});
const isEditing = ref(false);
const saveLoading = ref(false);
const formRef = ref<FormInstance>();
const availableIps = ref<IpAddress[]>([]);
const selectedIpIds = ref<string[]>([]);
const originalIpIds = ref<string[]>([]);
const bindingLoading = ref(false);
const allowCrossEnv = ref(false);
const bindingSearchKeyword = ref("");
const quickIpDialogVisible = ref(false);
const quickIpSaving = ref(false);
const quickIpFormRef = ref<FormInstance>();
const quickRealIpList = ref<string[]>([]);
const quickIpForm = ref<Partial<IpAddress>>({
  id: "",
  ip_address: "",
  env: "prod",
  is_vip: false,
  real_ips: undefined,
  description: undefined,
});

interface HostListFilters {
  env: string[];
  status: string[];
}

function createDefaultFilters(): HostListFilters {
  return {
    env: [],
    status: [],
  };
}

const listFilters = ref<HostListFilters>(createDefaultFilters());
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
];
const ipv4Pattern = /^((25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(25[0-5]|2[0-4]\d|[01]?\d\d?)$/;

const validateOptionalPositiveInteger = (
  _rule: unknown,
  value: unknown,
  callback: (error?: Error) => void,
) => {
  if (value === undefined || value === null) {
    callback();
    return;
  }

  if (typeof value === "string" && value.trim() === "") {
    callback();
    return;
  }

  if (normalizePositiveIntegerValue(value) === undefined) {
    callback(new Error("Please enter a positive integer"));
    return;
  }

  callback();
};

const formRules: FormRules = {
  hostname: [
    { required: true, message: "请输入主机名", trigger: "blur" },
    { min: 1, max: 200, message: "长度 1-200 个字符", trigger: "blur" },
  ],
  env: [{ required: true, message: "请选择环境", trigger: "change" }],
  cpu_cores: [{ validator: validateOptionalPositiveInteger, trigger: "change" }],
  cpu_threads: [{ validator: validateOptionalPositiveInteger, trigger: "change" }],
  ram_gb: [{ validator: validateOptionalPositiveInteger, trigger: "change" }],
  disk_gb: [{ validator: validateOptionalPositiveInteger, trigger: "change" }],
  cpu_freq: [{ min: 0, max: 50, message: "Length should be less than 50 characters", trigger: "blur" }],
  status: [
    { required: true, message: "请选择状态", trigger: "change" },
  ],
};

// 操作系统预设选项
const osOptions = [
  "CentOS 7",
  "CentOS 8",
  "Ubuntu 20.04",
  "Ubuntu 22.04",
  "Ubuntu 24.04",
  "Debian 11",
  "Debian 12",
  "RHEL 8",
  "RHEL 9",
  "Windows Server 2019",
  "Windows Server 2022",
];

// 动态标签相关
const tagList = ref<string[]>([]);
const tagInputVisible = ref(false);
const tagInputValue = ref("");
const tagInputRef = ref<InstanceType<typeof import("element-plus")["ElInput"]>>();
const filteredIpOptions = computed(() => {
  const keyword = bindingSearchKeyword.value.trim().toLowerCase();
  const candidates = allowCrossEnv.value
    ? availableIps.value
    : availableIps.value.filter(
        (ip) => !editingHost.value.env || ip.env === editingHost.value.env || selectedIpIds.value.includes(ip.id)
      );

  if (!keyword) {
    return candidates;
  }

  return candidates.filter((ip) => {
    const label = formatIpOptionLabel(ip).toLowerCase();
    return ip.ip_address.toLowerCase().includes(keyword) || label.includes(keyword);
  });
});

const searchedIpKeyword = computed(() => bindingSearchKeyword.value.trim());
const canQuickCreateIp = computed(() => {
  if (!searchedIpKeyword.value || !ipv4Pattern.test(searchedIpKeyword.value)) {
    return false;
  }

  const env = editingHost.value.env;
  return !availableIps.value.some((ip) => ip.ip_address === searchedIpKeyword.value && (!env || ip.env === env));
});

const quickIpFormRules: FormRules = {
  ip_address: [
    { required: true, message: "请输入 IP 地址", trigger: "blur" },
    { pattern: ipv4Pattern, message: "请输入有效 IPv4 地址", trigger: "blur" },
  ],
  env: [{ required: true, message: "请选择环境", trigger: "change" }],
};

function parseTags(json?: string): string[] {
  if (!json) return [];
  try {
    const arr = JSON.parse(json);
    return Array.isArray(arr) ? arr.filter((t: unknown) => typeof t === "string" && t) : [];
  } catch {
    return [];
  }
}

function tagsToJson(arr: string[]): string {
  return arr.length > 0 ? JSON.stringify(arr) : "";
}

function handleTagClose(tag: string) {
  tagList.value = tagList.value.filter((t) => t !== tag);
  editingHost.value.tags = tagsToJson(tagList.value);
}

function showTagInput() {
  tagInputVisible.value = true;
  nextTick(() => {
    tagInputRef.value?.input?.focus();
  });
}

function handleTagInputConfirm() {
  const val = tagInputValue.value.trim();
  if (val && !tagList.value.includes(val)) {
    tagList.value.push(val);
    editingHost.value.tags = tagsToJson(tagList.value);
  }
  tagInputVisible.value = false;
  tagInputValue.value = "";
}

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

function generateHostId() {
  return `host-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`;
}

function formatIpOptionLabel(ip: IpAddress) {
  const vipLabel = ip.is_vip ? "VIP" : "普通";
  return `${ip.ip_address} [${envLabel(ip.env)} | ${vipLabel}]`;
}

async function loadIpOptions() {
  const result = await listIpAddresses({ page: 1, page_size: 999 });
  availableIps.value = result.data;
}

async function loadHostBindings(hostId: string) {
  const bindings = await listHostIpBindings(hostId);
  selectedIpIds.value = bindings.map((item) => item.id);
  originalIpIds.value = [...selectedIpIds.value];
}

async function syncHostBindings(hostId: string) {
  const current = new Set(selectedIpIds.value);
  const original = new Set(originalIpIds.value);
  const addIds = [...current].filter((id) => !original.has(id));
  const removeIds = [...original].filter((id) => !current.has(id));

  for (const ipId of addIds) {
    await bindHostIp({ host_id: hostId, ip_id: ipId });
  }
  for (const ipId of removeIds) {
    await unbindHostIp({ host_id: hostId, ip_id: ipId });
  }

  originalIpIds.value = [...selectedIpIds.value];
}

async function refreshBindingContext(hostId?: string) {
  bindingLoading.value = true;
  try {
    await loadIpOptions();
    if (hostId) {
      await loadHostBindings(hostId);
    } else {
      selectedIpIds.value = [];
      originalIpIds.value = [];
    }
  } catch {
    // error shown by tauriInvoke
  } finally {
    bindingLoading.value = false;
  }
}

function handleBindingSearch(keyword: string) {
  bindingSearchKeyword.value = keyword.trim();
}

function handleBindingDropdownVisible(visible: boolean) {
  if (!visible) {
    bindingSearchKeyword.value = "";
  }
}

function normalizeQuickRealIps(): string[] {
  return Array.from(
    new Set(
      quickRealIpList.value
        .map((item) => item.trim())
        .filter((item) => item.length > 0)
    )
  );
}

function validateQuickVipIps(): boolean {
  if (!quickIpForm.value.is_vip) {
    return true;
  }

  const normalized = normalizeQuickRealIps();
  if (normalized.length === 0) {
    ElMessage.warning("VIP 模式下至少需要 1 个真实 IP");
    return false;
  }

  for (const ip of normalized) {
    if (!ipv4Pattern.test(ip)) {
      ElMessage.warning(`真实 IP 无效：${ip}`);
      return false;
    }
  }

  return true;
}

function addQuickRealIp() {
  quickRealIpList.value.push("");
}

function removeQuickRealIp(index: number) {
  quickRealIpList.value.splice(index, 1);
}

function openQuickCreateIpDialog() {
  const ip = searchedIpKeyword.value;
  if (!ipv4Pattern.test(ip)) {
    ElMessage.warning("请输入有效 IPv4 地址后再新增");
    return;
  }

  quickIpForm.value = {
    id: "",
    ip_address: ip,
    env: (editingHost.value.env as IpAddress["env"]) || "prod",
    is_vip: false,
    real_ips: undefined,
    description: undefined,
  };
  quickRealIpList.value = [];
  quickIpDialogVisible.value = true;
}

async function handleQuickCreateIpSave() {
  const valid = await quickIpFormRef.value?.validate().catch(() => false);
  if (!valid) return;
  if (!validateQuickVipIps()) return;

  const normalizedRealIps = normalizeQuickRealIps();
  const payload: Partial<IpAddress> = {
    id: "",
    ip_address: quickIpForm.value.ip_address?.trim() ?? "",
    env: (quickIpForm.value.env as IpAddress["env"]) || "prod",
    is_vip: Boolean(quickIpForm.value.is_vip),
    real_ips: quickIpForm.value.is_vip ? JSON.stringify(normalizedRealIps) : undefined,
    description: quickIpForm.value.description?.trim() || undefined,
  };

  quickIpSaving.value = true;
  try {
    await saveIpAddress(payload);
    await loadIpOptions();

    const created = availableIps.value.find(
      (ip) => ip.ip_address === payload.ip_address && ip.env === payload.env
    );
    if (created && !selectedIpIds.value.includes(created.id)) {
      selectedIpIds.value.push(created.id);
    }

    quickIpDialogVisible.value = false;
    bindingSearchKeyword.value = "";
    ElMessage.success("IP 资源新增成功，已自动回填到绑定列表");
  } catch {
    // error shown by tauriInvoke
  } finally {
    quickIpSaving.value = false;
  }
}

async function openAdd() {
  editingHost.value = { id: "", status: "running", env: "prod", hostname: "" };
  tagList.value = [];
  allowCrossEnv.value = false;
  bindingSearchKeyword.value = "";
  isEditing.value = false;
  dialogVisible.value = true;
  await refreshBindingContext();
}

async function openEdit(row: Host) {
  editingHost.value = { ...row };
  tagList.value = parseTags(row.tags);
  allowCrossEnv.value = false;
  bindingSearchKeyword.value = "";
  isEditing.value = true;
  dialogVisible.value = true;
  await refreshBindingContext(row.id);
}

async function openCopy(row: Host) {
  editingHost.value = buildHostCopyDraft(row);
  tagList.value = parseTags(editingHost.value.tags);
  allowCrossEnv.value = false;
  bindingSearchKeyword.value = "";
  isEditing.value = false;
  dialogVisible.value = true;
  await refreshBindingContext();
  ElMessage.info("已生成副本草稿，请在下方绑定 IP 后保存");
}

function hasInputValue(value: unknown): boolean {
  if (value === undefined || value === null) {
    return false;
  }

  if (typeof value === "string") {
    return value.trim() !== "";
  }

  return true;
}

function validateHardwareFields(): boolean {
  const numberFields: Array<{ key: keyof Host; label: string }> = [
    { key: "cpu_cores", label: "CPU cores" },
    { key: "cpu_threads", label: "CPU threads" },
    { key: "ram_gb", label: "RAM" },
    { key: "disk_gb", label: "Disk" },
  ];

  for (const field of numberFields) {
    const rawValue = editingHost.value[field.key];
    if (hasInputValue(rawValue) && normalizePositiveIntegerValue(rawValue) === undefined) {
      ElMessage.warning(`${field.label} must be a positive integer`);
      return false;
    }
  }

  const rawCpuFreq = editingHost.value.cpu_freq;
  if (hasInputValue(rawCpuFreq)) {
    const normalizedCpuFreq = normalizeCpuFreqValue(rawCpuFreq);
    if (!normalizedCpuFreq) {
      ElMessage.warning("CPU frequency is invalid");
      return false;
    }
    if (normalizedCpuFreq.length > 50) {
      ElMessage.warning("CPU frequency must be less than 50 characters");
      return false;
    }
  }

  return true;
}

function normalizeHardwareFields() {
  editingHost.value.cpu_cores = normalizePositiveIntegerValue(editingHost.value.cpu_cores);
  editingHost.value.cpu_threads = normalizePositiveIntegerValue(editingHost.value.cpu_threads);
  editingHost.value.ram_gb = normalizePositiveIntegerValue(editingHost.value.ram_gb);
  editingHost.value.disk_gb = normalizePositiveIntegerValue(editingHost.value.disk_gb);
  editingHost.value.cpu_freq = normalizeCpuFreqValue(editingHost.value.cpu_freq);
}

async function handleSave() {
  const valid = await formRef.value?.validate().catch(() => false);
  if (!valid) return;
  if (!validateHardwareFields()) return;

  normalizeHardwareFields();
  const hostId = editingHost.value.id || generateHostId();
  editingHost.value.id = hostId;
  const payload: Partial<Host> = {
    ...editingHost.value,
    ip_address: undefined,
    ip_display: undefined,
  };

  saveLoading.value = true;
  try {
    await saveHost(payload);
    await syncHostBindings(hostId);
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    dialogVisible.value = false;
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

function statusLabel(status: string) {
  return ({ running: "运行中", stopped: "已停止", maintenance: "维护中" } as Record<string, string>)[status] || status;
}

onMounted(() => fetchData());
</script>

<template>
  <div class="resource-view">
    <SearchToolbar
      v-model:search-text="searchText"
      v-model:filters="listFilters"
      search-placeholder="搜索主机名/IP..."
      :fields="toolbarFields"
      :show-chips="false"
      @query="handleToolbarQuery"
    >
      <template #actions="{ hasActiveFilters, reset }">
        <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
        <el-button type="primary" @click="openAdd">新增服务器</el-button>
      </template>
    </SearchToolbar>
    <el-table :data="data" v-loading="loading" border stripe class="w-full im-table-fixed-ops">
      <el-table-column prop="hostname" label="主机名" min-width="150" align="center" />
      <el-table-column label="IP地址" min-width="220" align="center">
        <template #default="{ row }">{{ row.ip_display || "-" }}</template>
      </el-table-column>
      <el-table-column prop="env" label="环境" width="90" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="os_type" label="操作系统" width="120" align="center" />
      <el-table-column prop="ram_gb" label="内存(GB)" width="100" align="center" />
      <el-table-column prop="disk_gb" label="磁盘(GB)" width="100" align="center" />
      <el-table-column prop="status" label="状态" width="100" align="center">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)" size="small">{{
            statusLabel(row.status)
          }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="210" fixed="right" align="center">
        <template #default="{ row }">
          <el-button text type="primary" size="small" @click="openEdit(row)">编辑</el-button>
          <el-button text type="primary" size="small" @click="openCopy(row)">复制</el-button>
          <el-button text type="danger" size="small" @click="handleDelete(row.id, row.hostname)"
            >删除</el-button
          >
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
      v-model="dialogVisible"
      :title="isEditing ? '编辑服务器' : '新增服务器'"
      width="700px"
      align-center
      destroy-on-close
    >
      <el-form ref="formRef" :model="editingHost" :rules="formRules" label-width="96px">
        <el-divider content-position="left">基础信息</el-divider>
        <el-form-item label="主机名" prop="hostname" required>
          <el-input v-model="editingHost.hostname" placeholder="请输入主机名，例如 web-prod-01" />
        </el-form-item>
        <el-form-item label="环境" prop="env" required>
          <el-select v-model="editingHost.env" class="w-full">
            <el-option v-for="option in envOptions" :key="option.value" :label="option.label" :value="option.value" />
          </el-select>
        </el-form-item>
        <el-form-item label="绑定IP">
          <div class="binding-editor">
            <div class="binding-toolbar">
              <el-switch
                v-model="allowCrossEnv"
                inline-prompt
                :active-text="'跨环境'"
                :inactive-text="'同环境'"
              />
              <el-button text size="small" :loading="bindingLoading" @click="refreshBindingContext(editingHost.id)">
                刷新IP列表
              </el-button>
            </div>
            <el-select
              v-model="selectedIpIds"
              multiple
              filterable
              remote
              clearable
              collapse-tags
              collapse-tags-tooltip
              class="w-full"
              placeholder="选择绑定IP（支持多选）"
              :loading="bindingLoading"
              :remote-method="handleBindingSearch"
              @visible-change="handleBindingDropdownVisible"
            >
              <el-option
                v-for="ip in filteredIpOptions"
                :key="ip.id"
                :label="formatIpOptionLabel(ip)"
                :value="ip.id"
              />
              <template #empty>
                <div class="binding-empty">
                  <template v-if="!searchedIpKeyword">暂无可选 IP</template>
                  <template v-else-if="canQuickCreateIp">
                    <span>当前环境尚未录入 IP：{{ searchedIpKeyword }}</span>
                    <el-button type="primary" text @click="openQuickCreateIpDialog">
                      点击新增并回填
                    </el-button>
                  </template>
                  <template v-else>未找到匹配 IP</template>
                </div>
              </template>
            </el-select>
            <div class="binding-hint">
              默认仅显示与服务器同环境 IP，开启“跨环境”可查看全部。支持在下拉框中输入 IP，未录入时可直接新增。
            </div>
          </div>
        </el-form-item>
        <el-form-item label="操作系统">
          <el-select
            v-model="editingHost.os_type"
            filterable
            allow-create
            clearable
            placeholder="选择或输入操作系统版本，如 Ubuntu 22.04"
            class="w-full"
          >
            <el-option v-for="os in osOptions" :key="os" :label="os" :value="os" />
          </el-select>
        </el-form-item>

        <el-divider content-position="left">硬件规格</el-divider>
        <el-form-item label="CPU 型号">
          <el-input v-model="editingHost.cpu_model" placeholder="如 Intel Xeon E5-2680 v4" />
        </el-form-item>
        <el-form-item label="CPU 参数">
          <el-row :gutter="12" class="w-full">
            <el-col :span="8">
              <div class="inline-field">
                <span class="inline-label">核心数</span>
                <el-select
                  v-model="editingHost.cpu_cores"
                  filterable
                  allow-create
                  default-first-option
                  clearable
                  placeholder="8"
                  class="inline-input"
                >
                  <el-option
                    v-for="value in COMMON_CPU_CORES_OPTIONS"
                    :key="`cpu-core-${value}`"
                    :label="String(value)"
                    :value="value"
                  />
                </el-select>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="inline-field">
                <span class="inline-label">线程数</span>
                <el-select
                  v-model="editingHost.cpu_threads"
                  filterable
                  allow-create
                  default-first-option
                  clearable
                  placeholder="16"
                  class="inline-input"
                >
                  <el-option
                    v-for="value in COMMON_CPU_THREADS_OPTIONS"
                    :key="`cpu-thread-${value}`"
                    :label="String(value)"
                    :value="value"
                  />
                </el-select>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="inline-field">
                <span class="inline-label">频率</span>
                <el-select
                  v-model="editingHost.cpu_freq"
                  filterable
                  allow-create
                  default-first-option
                  clearable
                  placeholder="2.4"
                  class="inline-input"
                >
                  <el-option v-for="value in COMMON_CPU_FREQ_OPTIONS" :key="`cpu-freq-${value}`" :label="value" :value="value" />
                </el-select>
                <span class="inline-unit">GHz</span>
              </div>
            </el-col>
          </el-row>
        </el-form-item>
        <el-form-item label="硬件配置">
          <el-row :gutter="12" class="w-full">
            <el-col :span="12">
              <div class="inline-field">
                <span class="inline-label">内存</span>
                <el-select
                  v-model="editingHost.ram_gb"
                  filterable
                  allow-create
                  default-first-option
                  clearable
                  placeholder="16"
                  class="inline-input"
                >
                  <el-option
                    v-for="value in COMMON_RAM_OPTIONS_GB"
                    :key="`ram-${value}`"
                    :label="String(value)"
                    :value="value"
                  />
                </el-select>
                <span class="inline-unit">GB</span>
              </div>
            </el-col>
            <el-col :span="12">
              <div class="inline-field">
                <span class="inline-label">磁盘</span>
                <el-select
                  v-model="editingHost.disk_gb"
                  filterable
                  allow-create
                  default-first-option
                  clearable
                  placeholder="512"
                  class="inline-input"
                >
                  <el-option
                    v-for="value in COMMON_DISK_OPTIONS_GB"
                    :key="`disk-${value}`"
                    :label="String(value)"
                    :value="value"
                  />
                </el-select>
                <span class="inline-unit">GB</span>
              </div>
            </el-col>
          </el-row>
        </el-form-item>

        <el-divider content-position="left">运维信息</el-divider>
        <el-form-item label="状态" prop="status" required>
          <el-select v-model="editingHost.status" class="w-full">
            <el-option label="运行中" value="running" />
            <el-option label="已停止" value="stopped" />
            <el-option label="维护中" value="maintenance" />
          </el-select>
        </el-form-item>
        <el-form-item label="标签">
          <div class="tag-editor">
            <el-tag
              v-for="tag in tagList"
              :key="tag"
              closable
              :disable-transitions="false"
              @close="handleTagClose(tag)"
              class="tag-item"
            >
              {{ tag }}
            </el-tag>
            <el-input
              v-if="tagInputVisible"
              ref="tagInputRef"
              v-model="tagInputValue"
              size="small"
              class="w-120"
              @keyup.enter="handleTagInputConfirm"
              @blur="handleTagInputConfirm"
            />
            <el-button v-else size="small" @click="showTagInput">+ 添加标签</el-button>
          </div>
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="editingHost.description"
            type="textarea"
            :rows="3"
            placeholder="可补充用途、机房位置、负责人等信息"
            show-word-limit
            maxlength="300"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="quickIpDialogVisible"
      title="新增IP资源"
      width="620px"
      align-center
      destroy-on-close
    >
      <el-form ref="quickIpFormRef" :model="quickIpForm" :rules="quickIpFormRules" label-width="96px">
        <el-form-item label="IP地址" prop="ip_address" required>
          <el-input v-model="quickIpForm.ip_address" placeholder="如 10.0.0.21" />
        </el-form-item>
        <el-form-item label="环境" prop="env" required>
          <el-select v-model="quickIpForm.env" class="w-full">
            <el-option
              v-for="option in envOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="是否VIP">
          <el-radio-group v-model="quickIpForm.is_vip">
            <el-radio :value="false">否</el-radio>
            <el-radio :value="true">是</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="quickIpForm.is_vip" label="真实IP列表" required>
          <div class="quick-real-ip-editor">
            <div
              v-for="(_ip, index) in quickRealIpList"
              :key="`quick-real-ip-${index}`"
              class="quick-real-ip-row"
            >
              <el-input v-model="quickRealIpList[index]" placeholder="如 10.0.0.31" />
              <el-button text type="danger" @click="removeQuickRealIp(index)">删除</el-button>
            </div>
            <el-button size="small" @click="addQuickRealIp">+ 添加真实IP</el-button>
          </div>
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="quickIpForm.description"
            type="textarea"
            :rows="3"
            maxlength="300"
            show-word-limit
            placeholder="可填写用途、备注等信息"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="quickIpDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="quickIpSaving" @click="handleQuickCreateIpSave">
          保存并回填
        </el-button>
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
  grid-template-columns: minmax(0, 2.1fr) minmax(0, 1.9fr);
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
.status-filter {
  width: 100%;
  min-width: 0;
}
:deep(.status-filter .el-select__selection) {
  flex-wrap: nowrap;
  overflow: hidden;
}
:deep(.status-filter .el-select__selected-item) {
  max-width: 100%;
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
}
.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
.tag-editor {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
}
.tag-item {
  margin-right: 6px;
  margin-bottom: 4px;
}
.inline-field {
  display: flex;
  align-items: flex-start;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}
.inline-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.2;
  min-width: 40px;
  white-space: nowrap;
}
.inline-unit {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.2;
}
.inline-input {
  width: 100%;
}
.binding-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}
.binding-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.binding-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.binding-empty {
  padding: 8px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: var(--el-text-color-secondary);
  font-size: 12px;
  text-align: center;
}
.quick-real-ip-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}
.quick-real-ip-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
}
</style>


