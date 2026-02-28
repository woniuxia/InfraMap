<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";
import { ElMessage } from "element-plus";
import type { FormInstance, FormRules } from "element-plus";
import type { Host } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listHosts, saveHost, softDeleteHost } from "@/api/hosts";
import { useResourceList } from "@/composables/useResourceList";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import { COMMON_DISK_OPTIONS_GB, COMMON_RAM_OPTIONS_GB } from "@/views/hostsHardwareOptions";

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

const formRules: FormRules = {
  hostname: [
    { required: true, message: "请输入主机名", trigger: "blur" },
    { min: 1, max: 200, message: "长度 1-200 个字符", trigger: "blur" },
  ],
  ip_address: [
    { required: true, message: "请输入 IP 地址", trigger: "blur" },
    { pattern: ipv4Pattern, message: "请输入有效的 IPv4 地址，如 192.168.1.100", trigger: "blur" },
  ],
  env: [{ required: true, message: "请选择环境", trigger: "change" }],
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

function openAdd() {
  editingHost.value = { status: "running", env: "prod", hostname: "", ip_address: "" };
  tagList.value = [];
  isEditing.value = false;
  dialogVisible.value = true;
}

function openEdit(row: Host) {
  editingHost.value = { ...row };
  tagList.value = parseTags(row.tags);
  isEditing.value = true;
  dialogVisible.value = true;
}

async function handleSave() {
  const valid = await formRef.value?.validate().catch(() => false);
  if (!valid) return;

  saveLoading.value = true;
  try {
    await saveHost(editingHost.value);
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
    <el-table :data="data" v-loading="loading" border stripe class="w-full">
      <el-table-column prop="hostname" label="主机名" min-width="150" align="center" />
      <el-table-column prop="ip_address" label="IP地址" width="150" align="center" />
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
      <el-table-column label="操作" width="150" fixed="right" align="center">
        <template #default="{ row }">
          <el-button text type="primary" size="small" @click="openEdit(row)">编辑</el-button>
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
        <el-form-item label="IP地址" prop="ip_address" required>
          <el-input v-model="editingHost.ip_address" placeholder="如 192.168.1.100" />
        </el-form-item>
        <el-form-item label="环境" prop="env" required>
          <el-select v-model="editingHost.env" class="w-full">
            <el-option v-for="option in envOptions" :key="option.value" :label="option.label" :value="option.value" />
          </el-select>
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
                <el-input-number
                  v-model="editingHost.cpu_cores"
                  :min="1"
                  controls-position="right"
                  placeholder="如 8"
                  class="inline-input"
                />
              </div>
            </el-col>
            <el-col :span="8">
              <div class="inline-field">
                <span class="inline-label">线程数</span>
                <el-input-number
                  v-model="editingHost.cpu_threads"
                  :min="1"
                  controls-position="right"
                  placeholder="如 16"
                  class="inline-input"
                />
              </div>
            </el-col>
            <el-col :span="8">
              <div class="inline-field">
                <span class="inline-label">频率</span>
                <el-input v-model="editingHost.cpu_freq" placeholder="如 2.40 GHz" class="inline-input" />
              </div>
            </el-col>
          </el-row>
        </el-form-item>
        <el-form-item label="硬件配置">
          <el-row :gutter="12" class="w-full">
            <el-col :span="12">
              <div class="inline-field">
                <span class="inline-label">内存</span>
                <el-input-number
                  v-model="editingHost.ram_gb"
                  :min="0"
                  controls-position="right"
                  class="inline-input"
                >
                  <template #suffix>GB</template>
                </el-input-number>
                <div class="quick-size-options">
                  <el-button
                    v-for="size in COMMON_RAM_OPTIONS_GB"
                    :key="`ram-${size}`"
                    size="small"
                    text
                    bg
                    :type="editingHost.ram_gb === size ? 'primary' : undefined"
                    class="quick-size-button"
                    @click="editingHost.ram_gb = size"
                  >
                    {{ size }} GB
                  </el-button>
                </div>
              </div>
            </el-col>
            <el-col :span="12">
              <div class="inline-field">
                <span class="inline-label">磁盘</span>
                <el-input-number
                  v-model="editingHost.disk_gb"
                  :min="0"
                  controls-position="right"
                  class="inline-input"
                >
                  <template #suffix>GB</template>
                </el-input-number>
                <div class="quick-size-options">
                  <el-button
                    v-for="size in COMMON_DISK_OPTIONS_GB"
                    :key="`disk-${size}`"
                    size="small"
                    text
                    bg
                    :type="editingHost.disk_gb === size ? 'primary' : undefined"
                    class="quick-size-button"
                    @click="editingHost.disk_gb = size"
                  >
                    {{ size }} GB
                  </el-button>
                </div>
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
.inline-input {
  width: 100%;
}
.quick-size-options {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.quick-size-button {
  margin: 0;
}
</style>


