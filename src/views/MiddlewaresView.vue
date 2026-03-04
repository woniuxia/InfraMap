<script setup lang="ts">
import { computed, ref, onMounted, watch } from "vue";
import { ElMessage } from "element-plus";
import type { Middleware } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listMiddlewares, saveMiddleware, deleteMiddleware } from "@/api/middlewares";
import { replaceResourceCallRelations } from "@/api/call-relations";
import { saveDeployment } from "@/api/deployments";
import { useResourceList } from "@/composables/useResourceList";
import {
  MIDDLEWARE_CATEGORY_OPTIONS,
  getMiddlewareCategoryLabel,
  getMiddlewareDefaultPort,
  getMiddlewareIconByType,
  getMiddlewareTypeOptionsWithIcon,
} from "@/utils/middlewareCatalog";
import { buildMiddlewareCopyDraft } from "@/utils/resourceCopy";
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
} = useResourceList<Middleware>({
  listFn: listMiddlewares,
  deleteFn: deleteMiddleware,
  entityLabel: "中间件",
});

const searchText = ref("");
const drawerVisible = ref(false);
const editingMw = ref<Partial<Middleware>>({});
const isEditing = ref(false);
const saveLoading = ref(false);
const autoFilledPort = ref<number | undefined>(undefined);
const callRelationsEditorRef = ref<InstanceType<typeof CallRelationsEditor> | null>(null);
interface DraftDeploymentItem {
  host_id: string;
  port?: number;
}
interface DeploymentPanelExposed {
  getDraftDeployments: () => DraftDeploymentItem[];
}
const deploymentPanelRef = ref<DeploymentPanelExposed | null>(null);

interface MiddlewareListFilters {
  category: string[];
  env: string[];
}

function createDefaultFilters(): MiddlewareListFilters {
  return {
    category: [],
    env: [],
  };
}

const listFilters = ref<MiddlewareListFilters>(createDefaultFilters());
const categoryOptions = MIDDLEWARE_CATEGORY_OPTIONS;

const middlewareTypeOptionsWithIcon = computed(() =>
  getMiddlewareTypeOptionsWithIcon(editingMw.value.category, editingMw.value.type)
);

const envOptions = [
  { label: "生产", value: "prod" },
  { label: "开发", value: "dev" },
  { label: "测试", value: "test" },
];

const toolbarFields: SearchFieldConfig[] = [
  {
    key: "category",
    queryKey: "category",
    label: "分类",
    type: "multi-select",
    width: "md",
    options: categoryOptions,
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

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

function generateDraftMiddlewareId() {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `mw-${crypto.randomUUID()}`;
  }
  return `mw-draft-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`;
}

function openAdd() {
  autoFilledPort.value = undefined;
  editingMw.value = {
    id: generateDraftMiddlewareId(),
    env: "prod",
    category: "database",
    created_at: "",
    updated_at: "",
  };
  isEditing.value = false;
  drawerVisible.value = true;
}

function openEdit(row: Middleware) {
  autoFilledPort.value = undefined;
  editingMw.value = { ...row };
  isEditing.value = true;
  drawerVisible.value = true;
}

function openCopy(row: Middleware) {
  autoFilledPort.value = undefined;
  editingMw.value = {
    ...buildMiddlewareCopyDraft(row),
    id: generateDraftMiddlewareId(),
  };
  isEditing.value = false;
  drawerVisible.value = true;
}

async function handleSave() {
  const draftItems = callRelationsEditorRef.value?.getDraftItems();
  if (draftItems === null) {
    return;
  }

  const wasEditing = isEditing.value;
  const draftDeployments = !wasEditing ? deploymentPanelRef.value?.getDraftDeployments?.() ?? [] : [];
  const payload: Partial<Middleware> = {
    id: "",
    created_at: "",
    updated_at: "",
    ...editingMw.value,
  };
  saveLoading.value = true;
  try {
    const middlewareId = await saveMiddleware(payload);
    try {
      await replaceResourceCallRelations({
        resource_id: middlewareId,
        resource_type: "middleware",
        items: draftItems ?? [],
      });
    } catch {
      ElMessage.warning("中间件已保存，调用关系保存失败，请重新编辑后重试。");
    }
    if (!wasEditing && draftDeployments.length > 0) {
      try {
        await Promise.all(
          draftDeployments.map((item) =>
            saveDeployment({
              id: "",
              resource_id: middlewareId,
              resource_type: "middleware",
              host_id: item.host_id,
              port: item.port,
            })
          )
        );
      } catch {
        ElMessage.warning("中间件已保存，部署关系保存失败，请在部署关系中重试。");
      }
    }
    ElMessage.success(wasEditing ? "更新成功" : "创建成功");
    drawerVisible.value = false;
    fetchData();
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

function categoryLabel(category: string) {
  return getMiddlewareCategoryLabel(category);
}

function middlewareTypeLabel(type?: string): string {
  const normalized = (type ?? "").trim();
  return normalized || "-";
}

function middlewareTypeIconSrc(row: Pick<Middleware, "category" | "type">): string {
  return getMiddlewareIconByType(row.type, row.category).src;
}

function middlewareTypeIconAlt(row: Pick<Middleware, "category" | "type">): string {
  return getMiddlewareIconByType(row.type, row.category).alt;
}

function middlewareTypeSlotValue(value: unknown): string {
  if (typeof value === "string") return value;
  if (typeof value === "number") return String(value);
  return "";
}

function middlewareTypeLabelBySlot(label: unknown, value: unknown): string {
  if (typeof label === "string" && label.trim().length > 0) return label;
  return middlewareTypeLabel(middlewareTypeSlotValue(value));
}

function middlewareTypeIconSrcBySlot(value: unknown): string {
  return getMiddlewareIconByType(middlewareTypeSlotValue(value), editingMw.value.category).src;
}

function middlewareTypeIconAltBySlot(value: unknown): string {
  return getMiddlewareIconByType(middlewareTypeSlotValue(value), editingMw.value.category).alt;
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

watch(
  () => editingMw.value.type,
  (newType) => {
    const defaultPort = getMiddlewareDefaultPort(newType);
    if (!defaultPort) return;
    const currentPort = editingMw.value.port;
    if (currentPort == null || currentPort === autoFilledPort.value) {
      editingMw.value.port = defaultPort;
      autoFilledPort.value = defaultPort;
    }
  }
);
</script>

<template>
  <div class="resource-view">
    <SearchToolbar
      v-model:search-text="searchText"
      v-model:filters="listFilters"
      search-placeholder="搜索名称/地址..."
      :fields="toolbarFields"
      @query="handleToolbarQuery"
    >
      <template #actions="{ hasActiveFilters, reset }">
        <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
        <el-button type="primary" @click="openAdd">新增中间件</el-button>
      </template>
    </SearchToolbar>
    <el-table :data="data" v-loading="loading" border stripe class="w-full im-table-fixed-ops">
      <el-table-column prop="name" label="实例名称" min-width="150" align="center" />
      <el-table-column prop="category" label="分类" width="100" align="center">
        <template #default="{ row }">{{ categoryLabel(row.category) }}</template>
      </el-table-column>
      <el-table-column prop="type" label="类型" width="140" align="center">
        <template #default="{ row }">
          <div class="middleware-type-cell">
            <img
              :src="middlewareTypeIconSrc(row)"
              :alt="middlewareTypeIconAlt(row)"
              class="middleware-type-icon"
            />
            <span>{{ middlewareTypeLabel(row.type) }}</span>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="地址" min-width="180" align="center">
        <template #default="{ row }"> {{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }} </template>
      </el-table-column>
      <el-table-column prop="version" label="版本" width="100" align="center" />
      <el-table-column prop="env" label="环境" width="80" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
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
      :title="isEditing ? '编辑中间件' : '新增中间件'"
      width="700px"
      align-center
      destroy-on-close
    >
      <el-form :model="editingMw" label-width="96px">
        <el-divider content-position="left">基础信息</el-divider>
        <el-form-item label="实例名称" required>
          <el-input v-model="editingMw.name" placeholder="请输入实例名称" />
        </el-form-item>
        <el-form-item label="分类" required>
          <el-select v-model="editingMw.category" class="w-full">
            <el-option
              v-for="option in categoryOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="类型" required>
          <el-select
            v-model="editingMw.type"
            class="w-full"
            filterable
            allow-create
            default-first-option
            clearable
            placeholder="可选择常用类型或手动输入"
          >
            <template #label="{ label, value }">
              <div class="middleware-type-option middleware-type-selected">
                <img
                  :src="middlewareTypeIconSrcBySlot(value)"
                  :alt="middlewareTypeIconAltBySlot(value)"
                  class="middleware-type-option-icon"
                />
                <span>{{ middlewareTypeLabelBySlot(label, value) }}</span>
              </div>
            </template>
            <el-option
              v-for="option in middlewareTypeOptionsWithIcon"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            >
              <div class="middleware-type-option">
                <img :src="option.icon.src" :alt="option.icon.alt" class="middleware-type-option-icon" />
                <span>{{ option.label }}</span>
              </div>
            </el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="连接地址" required>
          <el-input v-model="editingMw.address" placeholder="如 192.168.1.100" />
        </el-form-item>
        <el-form-item label="端口">
          <el-input-number v-model="editingMw.port" :min="1" :max="65535" class="w-full" />
        </el-form-item>

        <el-divider content-position="left">实例信息</el-divider>
        <el-form-item label="版本">
          <el-input v-model="editingMw.version" placeholder="如 8.0.33" />
        </el-form-item>
        <el-form-item label="环境" required>
          <el-select v-model="editingMw.env" class="w-full">
            <el-option label="生产" value="prod" />
            <el-option label="开发" value="dev" />
            <el-option label="测试" value="test" />
          </el-select>
        </el-form-item>

        <el-divider content-position="left">运维信息</el-divider>
        <el-form-item label="描述">
          <el-input v-model="editingMw.description" type="textarea" :rows="3" maxlength="300" show-word-limit />
        </el-form-item>
      </el-form>

      <CallRelationsEditor
        ref="callRelationsEditorRef"
        :resource-id="isEditing ? editingMw.id : undefined"
        resource-type="middleware"
      />

      <DeploymentPanel
        v-if="Boolean(editingMw.id)"
        ref="deploymentPanelRef"
        :resource-id="editingMw.id!"
        resource-type="middleware"
        :resource-persisted="isEditing"
        :resource-address="editingMw.address"
        :resource-env="editingMw.env"
        :default-port="editingMw.port"
      />

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
  grid-template-columns: minmax(0, 2.8fr) minmax(0, 1.7fr) minmax(0, 1.3fr);
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
.category-filter,
.env-filter {
  width: 100%;
  min-width: 0;
}
:deep(.category-filter .el-select__selection),
:deep(.env-filter .el-select__selection) {
  flex-wrap: nowrap;
  overflow: hidden;
}
:deep(.category-filter .el-select__selected-item),
:deep(.env-filter .el-select__selected-item) {
  max-width: 100%;
}
@media (max-width: 1080px) {
  .filter-row-primary {
    grid-template-columns: minmax(0, 2.3fr) minmax(0, 1.5fr) minmax(0, 1.2fr);
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
.middleware-type-cell {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.middleware-type-icon {
  width: 16px;
  height: 16px;
  display: block;
  flex-shrink: 0;
  border-radius: 4px;
}
.middleware-type-option {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.middleware-type-selected {
  max-width: 100%;
  overflow: hidden;
  white-space: nowrap;
}
.middleware-type-selected span {
  overflow: hidden;
  text-overflow: ellipsis;
}
.middleware-type-option-icon {
  width: 16px;
  height: 16px;
  display: block;
  flex-shrink: 0;
  border-radius: 4px;
}
</style>

