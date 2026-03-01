import { computed, ref } from "vue";
import { ElMessage } from "element-plus";
import type { FormInstance, FormRules } from "element-plus";
import type { Host, IpAddress } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listHosts, saveHost, softDeleteHost } from "@/api/hosts";
import { listIpAddresses, saveIpAddress } from "@/api/ip-addresses";
import { listHostTagTerms } from "@/api/taxonomy";
import { bindHostIp, listHostIpBindings, unbindHostIp } from "@/api/host-ip-bindings";
import { useResourceList } from "@/composables/useResourceList";
import { buildHostCopyDraft } from "@/utils/resourceCopy";
import {
  COMMON_CPU_CORES_OPTIONS,
  COMMON_CPU_FREQ_OPTIONS,
  COMMON_CPU_THREADS_OPTIONS,
  DEFAULT_HOST_HARDWARE,
  COMMON_DISK_OPTIONS_GB,
  COMMON_RAM_OPTIONS_GB,
  normalizeCpuFreqValue,
  normalizePositiveIntegerValue,
} from "@/views/hostsHardwareOptions";
import { envLabel } from "@/views/hosts/hostDisplay";

interface HostListFilters {
  env: string[];
  status: string[];
  tags: string[];
}

function createDefaultFilters(): HostListFilters {
  return {
    env: [],
    status: [],
    tags: [],
  };
}

const ipv4Pattern = /^((25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(25[0-5]|2[0-4]\d|[01]?\d\d?)$/;

export function useHostViewModel() {
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
  const listFilters = ref<HostListFilters>(createDefaultFilters());
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
  const tagFilterOptions = ref<Array<{ label: string; value: string }>>([]);

  const toolbarFields = computed<SearchFieldConfig[]>(() => [
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
      key: "tags",
      queryKey: "tags",
      label: "标签",
      type: "multi-select",
      width: "md",
      maxCollapseTags: 2,
      options: tagFilterOptions.value,
    },
  ]);

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

  const quickIpFormRules: FormRules = {
    ip_address: [
      { required: true, message: "请输入 IP 地址", trigger: "blur" },
      { pattern: ipv4Pattern, message: "请输入有效 IPv4 地址", trigger: "blur" },
    ],
    env: [{ required: true, message: "请选择环境", trigger: "change" }],
  };

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

  const tagList = ref<string[]>([]);
  function buildTagSuggestionOptions(currentValues: string[]) {
    const merged = new Set<string>(tagFilterOptions.value.map((item) => item.value));
    for (const value of currentValues) {
      const normalized = value.trim();
      if (normalized) {
        merged.add(normalized);
      }
    }
    return Array.from(merged).map((value) => ({ label: value, value }));
  }

  const formTagSuggestionOptions = computed(() => buildTagSuggestionOptions(tagList.value));
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

  function parseTags(json?: string): string[] {
    if (!json) return [];
    try {
      const arr = JSON.parse(json);
      if (!Array.isArray(arr)) {
        return [];
      }
      return Array.from(
        new Set(
          arr
            .filter((item: unknown) => typeof item === "string")
            .map((item: string) => item.trim())
            .filter((item) => item.length > 0)
        )
      );
    } catch {
      return [];
    }
  }

  function tagsToJson(arr: string[]): string {
    const normalized = Array.from(
      new Set(arr.map((item) => item.trim()).filter((item) => item.length > 0))
    );
    return normalized.length > 0 ? JSON.stringify(normalized) : "";
  }

  function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
    handleQuery(payload);
  }

  function generateHostId() {
    return `host-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`;
  }

  function createAddHostDraft(): Partial<Host> {
    return {
      id: "",
      status: "running",
      env: "prod",
      hostname: "",
      ...DEFAULT_HOST_HARDWARE,
    };
  }

  function formatIpOptionLabel(ip: IpAddress) {
    const vipLabel = ip.is_vip ? "VIP" : "普通";
    return `${ip.ip_address} [${envLabel(ip.env)} | ${vipLabel}]`;
  }

  async function loadIpOptions() {
    const result = await listIpAddresses({ page: 1, page_size: 999 });
    availableIps.value = result.data;
  }

  async function loadTagOptions() {
    try {
      const tags = await listHostTagTerms(200);
      tagFilterOptions.value = tags.map((item) => ({ label: item, value: item }));
    } catch {
      // error shown by tauriInvoke
    }
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
    editingHost.value = createAddHostDraft();
    tagList.value = [];
    allowCrossEnv.value = false;
    bindingSearchKeyword.value = "";
    isEditing.value = false;
    dialogVisible.value = true;
    await Promise.all([refreshBindingContext(), loadTagOptions()]);
  }

  async function openEdit(row: Host) {
    editingHost.value = { ...row };
    tagList.value = parseTags(row.tags);
    allowCrossEnv.value = false;
    bindingSearchKeyword.value = "";
    isEditing.value = true;
    dialogVisible.value = true;
    await Promise.all([refreshBindingContext(row.id), loadTagOptions()]);
  }

  async function openCopy(row: Host) {
    editingHost.value = buildHostCopyDraft(row);
    tagList.value = parseTags(editingHost.value.tags);
    allowCrossEnv.value = false;
    bindingSearchKeyword.value = "";
    isEditing.value = false;
    dialogVisible.value = true;
    await Promise.all([refreshBindingContext(), loadTagOptions()]);
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
      tags: tagsToJson(tagList.value) || undefined,
      ip_display: undefined,
    };

    saveLoading.value = true;
    try {
      await saveHost(payload);
      await syncHostBindings(hostId);
      ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
      dialogVisible.value = false;
      await Promise.all([fetchData(), loadTagOptions()]);
    } catch {
      // error shown by tauriInvoke
    } finally {
      saveLoading.value = false;
    }
  }

  async function init() {
    await Promise.all([fetchData(), loadTagOptions()]);
  }

  return {
    loading,
    data,
    total,
    queryParams,
    searchText,
    listFilters,
    toolbarFields,
    dialogVisible,
    editingHost,
    isEditing,
    saveLoading,
    formRef,
    formRules,
    availableIps,
    selectedIpIds,
    bindingLoading,
    allowCrossEnv,
    quickIpDialogVisible,
    quickIpSaving,
    quickIpFormRef,
    quickRealIpList,
    quickIpForm,
    quickIpFormRules,
    envOptions,
    statusOptions,
    osOptions,
    tagList,
    formTagSuggestionOptions,
    filteredIpOptions,
    searchedIpKeyword,
    canQuickCreateIp,
    fetchData,
    handleToolbarQuery,
    handlePageChange,
    handlePageSizeChange,
    handleDelete,
    openAdd,
    openEdit,
    openCopy,
    handleSave,
    formatIpOptionLabel,
    refreshBindingContext,
    handleBindingSearch,
    handleBindingDropdownVisible,
    openQuickCreateIpDialog,
    handleQuickCreateIpSave,
    addQuickRealIp,
    removeQuickRealIp,
    init,
    COMMON_CPU_CORES_OPTIONS,
    COMMON_CPU_THREADS_OPTIONS,
    COMMON_CPU_FREQ_OPTIONS,
    COMMON_RAM_OPTIONS_GB,
    COMMON_DISK_OPTIONS_GB,
  };
}

export type HostViewModel = ReturnType<typeof useHostViewModel>;
