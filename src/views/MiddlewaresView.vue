<script setup lang="ts">
import { ref, onMounted } from "vue";
import type { EditorMode, Middleware } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listMiddlewares, deleteMiddleware } from "@/api/middlewares";
import { useResourceList } from "@/composables/useResourceList";
import { ENV_OPTIONS } from "@/constants/options";
import { generateDraftId } from "@/utils/draft";
import {
  MIDDLEWARE_CATEGORY_OPTIONS,
  getMiddlewareCategoryLabel,
  getMiddlewareIconByType,
} from "@/utils/middlewareCatalog";
import { buildMiddlewareCopyDraft } from "@/utils/resourceCopy";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import MiddlewareEditorDialog from "@/components/resource-editors/MiddlewareEditorDialog.vue";

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
const editorVisible = ref(false);
const editorMode = ref<EditorMode>("create");
const editorInitialDraft = ref<Partial<Middleware>>({});

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

const envOptions = ENV_OPTIONS;

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

function openAdd() {
  editorInitialDraft.value = {
    id: generateDraftId("mw"),
    env: "prod",
    category: "database",
    created_at: "",
    updated_at: "",
  };
  editorMode.value = "create";
  editorVisible.value = true;
}

function openEdit(row: Middleware) {
  editorInitialDraft.value = { ...row };
  editorMode.value = "edit";
  editorVisible.value = true;
}

function openCopy(row: Middleware) {
  editorInitialDraft.value = {
    ...buildMiddlewareCopyDraft(row),
    id: generateDraftId("mw"),
  };
  editorMode.value = "copy";
  editorVisible.value = true;
}

function handleEditorSaved() {
  fetchData();
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
        <template #default="{ row }">
          {{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }}
        </template>
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
          <el-button text type="danger" size="small" @click="handleDelete(row.id, row.name)">
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

    <MiddlewareEditorDialog
      v-model="editorVisible"
      :mode="editorMode"
      :initial-draft="editorInitialDraft"
      @saved="handleEditorSaved"
    />
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
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--im-surface-1) 82%, transparent),
    var(--im-surface-0)
  );
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
