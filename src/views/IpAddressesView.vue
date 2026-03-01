<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { FormInstance, FormRules } from "element-plus";
import { ElMessage } from "element-plus";
import { ArrowDown } from "@element-plus/icons-vue";
import {
  batchCreateIpAddresses,
  listIpAddresses,
  saveIpAddress,
  softDeleteIpAddress,
} from "@/api/ip-addresses";
import type { IpAddress } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { useResourceList } from "@/composables/useResourceList";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import CopyableTextCell from "@/components/table/CopyableTextCell.vue";

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
} = useResourceList<IpAddress>({
  listFn: listIpAddresses,
  deleteFn: softDeleteIpAddress,
  entityLabel: "IP资源",
});

const searchText = ref("");
const listFilters = ref<{ env: string[]; is_vip: string[] }>({
  env: [],
  is_vip: [],
});
const dialogVisible = ref(false);
const isEditing = ref(false);
const saveLoading = ref(false);
const formRef = ref<FormInstance>();
const editingIp = ref<Partial<IpAddress>>({});
const realIpList = ref<string[]>([]);
const tagList = ref<string[]>([]);
const batchLoading = ref(false);
const batchExpanded = ref(false);
const batchForm = ref({
  start_ip: "",
  end_ip: "",
  env: "prod" as IpAddress["env"],
  description: "",
});
const batchTagList = ref<string[]>([]);

const ipv4Pattern = /^((25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(25[0-5]|2[0-4]\d|[01]?\d\d?)$/;

const envOptions = [
  { label: "生产", value: "prod" },
  { label: "开发", value: "dev" },
  { label: "测试", value: "test" },
];
const vipOptions = [
  { label: "VIP", value: "1" },
  { label: "普通IP", value: "0" },
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
    key: "is_vip",
    queryKey: "is_vip",
    label: "类型",
    type: "multi-select",
    width: "sm",
    options: vipOptions,
  },
];

const formRules: FormRules = {
  ip_address: [
    { required: true, message: "请输入 IP 地址", trigger: "blur" },
    { pattern: ipv4Pattern, message: "请输入有效 IPv4 地址", trigger: "blur" },
  ],
  env: [{ required: true, message: "请选择环境", trigger: "change" }],
};

function parseRealIps(json?: string): string[] {
  if (!json) return [];
  try {
    const parsed = JSON.parse(json);
    return Array.isArray(parsed)
      ? parsed.filter((item: unknown) => typeof item === "string" && item.trim().length > 0)
      : [];
  } catch {
    return [];
  }
}

function parseTags(json?: string): string[] {
  if (!json) return [];
  try {
    const parsed = JSON.parse(json);
    return Array.isArray(parsed)
      ? parsed
          .filter((item: unknown) => typeof item === "string")
          .map((item: string) => item.trim())
          .filter((item) => item.length > 0)
      : [];
  } catch {
    return [];
  }
}

function tagsToJson(values: string[]): string | undefined {
  const normalized = Array.from(
    new Set(values.map((item) => item.trim()).filter((item) => item.length > 0))
  );
  return normalized.length > 0 ? JSON.stringify(normalized) : undefined;
}

function formatTags(json?: string): string {
  const tags = parseTags(json);
  return tags.length > 0 ? tags.join(", ") : "-";
}

function formatRealIpsCount(json?: string): string {
  const count = parseRealIps(json).length;
  return count > 0 ? String(count) : "-";
}

function envLabel(env: string): string {
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] ?? env;
}

function envTagType(env: string): "primary" | "success" | "warning" | "info" | "danger" {
  const map: Record<string, "primary" | "success" | "warning" | "info" | "danger"> = {
    prod: "danger",
    dev: "info",
    test: "warning",
  };
  return map[env] ?? "info";
}

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

function openAdd() {
  editingIp.value = {
    ip_address: "",
    env: "prod",
    is_vip: false,
    real_ips: undefined,
    tags: undefined,
  };
  realIpList.value = [];
  tagList.value = [];
  isEditing.value = false;
  dialogVisible.value = true;
}

function openEdit(row: IpAddress) {
  editingIp.value = { ...row };
  realIpList.value = parseRealIps(row.real_ips);
  tagList.value = parseTags(row.tags);
  isEditing.value = true;
  dialogVisible.value = true;
}

function addRealIp() {
  realIpList.value.push("");
}

function removeRealIp(index: number) {
  realIpList.value.splice(index, 1);
}

function normalizeRealIps(): string[] {
  return Array.from(
    new Set(
      realIpList.value
        .map((item) => item.trim())
        .filter((item) => item.length > 0)
    )
  );
}

function validateVipIps(): boolean {
  if (!editingIp.value.is_vip) {
    return true;
  }

  const normalized = normalizeRealIps();
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

async function handleSave() {
  const valid = await formRef.value?.validate().catch(() => false);
  if (!valid) return;
  if (!validateVipIps()) return;

  const normalizedRealIps = normalizeRealIps();
  const payload: Partial<IpAddress> = {
    id: editingIp.value.id ?? "",
    ip_address: editingIp.value.ip_address?.trim() ?? "",
    env: editingIp.value.env ?? "prod",
    is_vip: Boolean(editingIp.value.is_vip),
    real_ips: editingIp.value.is_vip ? JSON.stringify(normalizedRealIps) : undefined,
    tags: tagsToJson(tagList.value),
    description: editingIp.value.description?.trim() || undefined,
    is_deleted: 0,
    created_at: editingIp.value.created_at ?? "",
    updated_at: editingIp.value.updated_at ?? "",
  };

  saveLoading.value = true;
  try {
    await saveIpAddress(payload);
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    dialogVisible.value = false;
    fetchData();
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

async function handleBatchCreate() {
  const startIp = batchForm.value.start_ip.trim();
  const endIp = batchForm.value.end_ip.trim();
  if (!ipv4Pattern.test(startIp) || !ipv4Pattern.test(endIp)) {
    ElMessage.warning("请填写有效的起始和结束 IPv4 地址");
    return;
  }

  batchLoading.value = true;
  try {
    const result = await batchCreateIpAddresses({
      start_ip: startIp,
      end_ip: endIp,
      env: batchForm.value.env,
      tags: tagsToJson(batchTagList.value),
      description: batchForm.value.description.trim() || undefined,
    });
    ElMessage.success(
      `批量生成完成：新增 ${result.created_count} 条，跳过 ${result.skipped_count} 条`
    );
    fetchData();
  } catch {
    // error shown by tauriInvoke
  } finally {
    batchLoading.value = false;
  }
}

onMounted(() => fetchData());
</script>

<template>
  <div class="resource-view">
    <el-card class="batch-card" shadow="never">
      <template #header>
        <button
          type="button"
          class="batch-card-header"
          @click="batchExpanded = !batchExpanded"
        >
          <span class="batch-card-title">批量生成 IP 资源</span>
          <span class="batch-card-action">
            {{ batchExpanded ? "收起" : "展开" }}
            <el-icon class="batch-card-arrow" :class="{ 'is-expanded': batchExpanded }">
              <ArrowDown />
            </el-icon>
          </span>
        </button>
      </template>
      <el-form v-show="batchExpanded" label-width="92px">
        <el-row :gutter="12">
          <el-col :span="6">
            <el-form-item label="起始IP" required>
              <el-input v-model="batchForm.start_ip" placeholder="如 10.0.2.1" />
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item label="结束IP" required>
              <el-input v-model="batchForm.end_ip" placeholder="如 10.0.2.20" />
            </el-form-item>
          </el-col>
          <el-col :span="5">
            <el-form-item label="环境" required>
              <el-select v-model="batchForm.env" class="w-full">
                <el-option v-for="option in envOptions" :key="option.value" :label="option.label" :value="option.value" />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="7">
            <el-form-item label="标签">
              <el-select
                v-model="batchTagList"
                multiple
                filterable
                allow-create
                default-first-option
                :reserve-keyword="false"
                placeholder="输入标签后回车"
                class="w-full"
              />
            </el-form-item>
          </el-col>
        </el-row>
        <el-row :gutter="12">
          <el-col :span="18">
            <el-form-item label="描述">
              <el-input v-model="batchForm.description" placeholder="批量导入备注（可选）" />
            </el-form-item>
          </el-col>
          <el-col :span="6" class="batch-action-col">
            <el-button type="primary" :loading="batchLoading" @click="handleBatchCreate">
              批量生成
            </el-button>
          </el-col>
        </el-row>
      </el-form>
    </el-card>

    <SearchToolbar
      v-model:search-text="searchText"
      v-model:filters="listFilters"
      search-placeholder="搜索IP/描述..."
      :fields="toolbarFields"
      @query="handleToolbarQuery"
    >
      <template #actions="{ hasActiveFilters, reset }">
        <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
        <el-button type="primary" @click="openAdd">新增IP资源</el-button>
      </template>
    </SearchToolbar>

    <el-table :data="data" v-loading="loading" border stripe class="w-full im-table-fixed-ops">
      <el-table-column label="IP地址" min-width="180" align="center">
        <template #default="{ row }">
          <CopyableTextCell :text="row.ip_address" aria-label="复制IP地址" />
        </template>
      </el-table-column>
      <el-table-column prop="env" label="环境" width="90" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="类型" width="100" align="center">
        <template #default="{ row }">
          <el-tag :type="row.is_vip ? 'warning' : 'info'" size="small">
            {{ row.is_vip ? "VIP" : "普通IP" }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="真实IP数" width="100" align="center">
        <template #default="{ row }">{{ formatRealIpsCount(row.real_ips) }}</template>
      </el-table-column>
      <el-table-column label="标签" min-width="180" show-overflow-tooltip align="center">
        <template #default="{ row }">{{ formatTags(row.tags) }}</template>
      </el-table-column>
      <el-table-column prop="description" label="描述" min-width="220" show-overflow-tooltip align="center" />
      <el-table-column label="操作" width="160" fixed="right" align="center">
        <template #default="{ row }">
          <el-button text type="primary" size="small" @click="openEdit(row)">编辑</el-button>
          <el-button text type="danger" size="small" @click="handleDelete(row.id, row.ip_address)">删除</el-button>
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
      :title="isEditing ? '编辑IP资源' : '新增IP资源'"
      width="700px"
      align-center
      destroy-on-close
    >
      <el-form ref="formRef" :model="editingIp" :rules="formRules" label-width="96px">
        <el-form-item label="IP地址" prop="ip_address" required>
          <el-input v-model="editingIp.ip_address" placeholder="如 192.168.1.100" />
        </el-form-item>
        <el-form-item label="环境" prop="env" required>
          <el-select v-model="editingIp.env" class="w-full">
            <el-option v-for="option in envOptions" :key="option.value" :label="option.label" :value="option.value" />
          </el-select>
        </el-form-item>
        <el-form-item label="是否VIP">
          <el-radio-group v-model="editingIp.is_vip">
            <el-radio :value="false">否</el-radio>
            <el-radio :value="true">是</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="editingIp.is_vip" label="真实IP列表" required>
          <div class="real-ip-editor">
            <div v-for="(_ip, index) in realIpList" :key="`real-ip-${index}`" class="real-ip-row">
              <el-input v-model="realIpList[index]" placeholder="如 10.0.0.11" />
              <el-button text type="danger" @click="removeRealIp(index)">删除</el-button>
            </div>
            <el-button size="small" @click="addRealIp">+ 添加真实IP</el-button>
          </div>
        </el-form-item>
        <el-form-item label="标签">
          <el-select
            v-model="tagList"
            multiple
            filterable
            allow-create
            default-first-option
            :reserve-keyword="false"
            class="w-full"
            placeholder="输入标签后回车"
          />
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="editingIp.description"
            type="textarea"
            :rows="3"
            maxlength="300"
            show-word-limit
            placeholder="可填写用途、归属、备注等"
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

.batch-card {
  margin-bottom: 12px;
}

.batch-card-title {
  font-weight: 600;
  color: var(--im-text-primary);
}

.batch-card-header {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border: 0;
  background: transparent;
  padding: 0;
  cursor: pointer;
  color: inherit;
}

.batch-card-action {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--im-text-secondary);
  font-size: 13px;
}

.batch-card-arrow {
  transition: transform var(--im-duration-base) var(--im-ease-standard);
}

.batch-card-arrow.is-expanded {
  transform: rotate(180deg);
}

.batch-action-col {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.real-ip-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.real-ip-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
}
</style>
