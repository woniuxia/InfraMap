<script setup lang="ts">
import { ref, onMounted } from "vue";
import type { Service, EditorMode } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listServices, deleteService } from "@/api";
import { useResourceList } from "@/composables/useResourceList";
import { DEPLOY_MODE_OPTIONS, STATUS_OPTIONS } from "@/constants/options";
import { generateDraftId } from "@/utils/draft";
import { buildServiceCopyDraft } from "@/utils/resourceCopy";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import ServiceEditorDialog from "@/components/resource-editors/ServiceEditorDialog.vue";

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
} = useResourceList<Service>({
  listFn: listServices,
  deleteFn: deleteService,
  entityLabel: "服务",
});

const searchText = ref("");
const editorVisible = ref(false);
const editorMode = ref<EditorMode>("create");
const editorInitialDraft = ref<Partial<Service>>({});

interface ServiceListFilters {
  type: string[];
  status: string[];
  deploy_mode: string[];
}

function createDefaultFilters(): ServiceListFilters {
  return {
    type: [],
    status: [],
    deploy_mode: [],
  };
}

const listFilters = ref<ServiceListFilters>(createDefaultFilters());

function normalizeOwners(owners?: string[]) {
  const values = [...(owners ?? [])].map((item) => item.trim()).filter((item) => item.length > 0);
  return Array.from(new Set(values));
}

function ownersForRow(row: Service) {
  return normalizeOwners(row.owners);
}

function openAdd() {
  editorInitialDraft.value = {
    id: generateDraftId("svc"),
    status: "running",
    type: "backend",
    port: 8080,
    system_id: "",
    created_at: "",
    updated_at: "",
  };
  editorMode.value = "create";
  editorVisible.value = true;
}

function openEdit(row: Service) {
  editorInitialDraft.value = {
    ...row,
    owners: Array.isArray(row.owners) ? [...row.owners] : row.owners,
  };
  editorMode.value = "edit";
  editorVisible.value = true;
}

function openCopy(row: Service) {
  editorInitialDraft.value = {
    ...buildServiceCopyDraft(row),
    id: generateDraftId("svc"),
  };
  editorMode.value = "copy";
  editorVisible.value = true;
}

function handleEditorSaved(_: { id: string }) {
  fetchData();
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
  return (
    ({ running: "运行中", stopped: "已停止", maintenance: "维护中" } as Record<string, string>)[
      status
    ] || status
  );
}

function typeLabel(type: string) {
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

const typeOptions = [
  { label: "前端", value: "frontend" },
  { label: "后端", value: "backend" },
  { label: "网关", value: "gateway" },
  { label: "批处理", value: "batch_job" },
  { label: "微服务", value: "microservice" },
  { label: "其他", value: "other" },
];

const statusOptions = STATUS_OPTIONS;
const deployModeOptions = DEPLOY_MODE_OPTIONS;

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
    type: "multi-select",
    width: "md",
    options: deployModeOptions,
  },
];

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

onMounted(() => {
  fetchData();
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
        <el-button type="primary" @click="openAdd">新增服务</el-button>
      </template>
    </SearchToolbar>

    <el-table :data="data" v-loading="loading" border stripe class="w-full im-table-fixed-ops">
      <el-table-column label="所属系统" min-width="180" align="center">
        <template #default="{ row }">
          {{ row.system_name || "-" }}
        </template>
      </el-table-column>
      <el-table-column prop="type" label="类型" width="100" align="center">
        <template #default="{ row }">{{ typeLabel(row.type) }}</template>
      </el-table-column>
      <el-table-column label="地址" min-width="180" align="center">
        <template #default="{ row }">
          {{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }}
        </template>
      </el-table-column>
      <el-table-column
        prop="tech_stack"
        label="技术栈"
        width="140"
        show-overflow-tooltip
        align="center"
      />
      <el-table-column label="负责人" min-width="160" align="center">
        <template #default="{ row }">
          <div v-if="ownersForRow(row).length > 0" class="owner-tags">
            <el-tag v-for="owner in ownersForRow(row)" :key="owner" size="small" effect="plain">
              {{ owner }}
            </el-tag>
          </div>
          <span v-else>-</span>
        </template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="100" align="center">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)" size="small">
            {{ statusLabel(row.status) }}
          </el-tag>
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

    <ServiceEditorDialog
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

.im-table-fixed-ops :deep(.el-table__fixed-right) {
  background: var(--im-surface-0);
}
</style>
