<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import type {
  Application,
  CallRelation,
  CallRelationType,
  Middleware,
  NginxConfig,
  ReplaceCallRelationItem,
} from "@/types";
import type {
  EditableCallRelationDraft,
  EditorCallDirection,
} from "@/components/callRelationsEditor.utils";
import { listApplications } from "@/api/applications";
import { listMiddlewares } from "@/api/middlewares";
import { listNginxConfigs } from "@/api/nginx-configs";
import { listCallRelations } from "@/api/call-relations";
import { formatRelationTargetLabel } from "@/utils/resourceLabel";
import {
  expandEditorDraftsForSave,
  hasDuplicateExpandedItems,
  mergeCallRelationsForEditor,
} from "@/components/callRelationsEditor.utils";

type ResourceType = "application" | "middleware" | "nginx";

interface RelationTargetOption {
  id: string;
  type: ResourceType;
  name: string;
  displayLabel: string;
}

interface EditableRelationRow {
  id: string;
  peer_id: string;
  peer_type: ResourceType;
  direction: EditorCallDirection;
  relation_type: CallRelationType;
  description: string;
}

const props = defineProps<{
  resourceId?: string;
  resourceType: ResourceType;
}>();

const loading = ref(false);
const optionsLoading = ref(false);
const relationRows = ref<EditableRelationRow[]>([]);
const relationTargetOptions = ref<RelationTargetOption[]>([]);
const sequence = ref(0);

const relationTypeOptions: Array<{ label: string; value: CallRelationType }> = [
  { label: "HTTP调用", value: "http_call" },
  { label: "TCP连接", value: "tcp" },
  { label: "MQ生产", value: "mq_produce" },
  { label: "MQ消费", value: "mq_consume" },
  { label: "gRPC调用", value: "grpc_call" },
  { label: "数据库访问", value: "db_query" },
  { label: "缓存访问", value: "cache_access" },
  { label: "请求转发", value: "forward" },
];

const groupedTargetOptions = computed(() => {
  return [
    { label: "应用", type: "application" as const },
    { label: "中间件", type: "middleware" as const },
    { label: "网关", type: "nginx" as const },
  ].map((group) => ({
    ...group,
    options: relationTargetOptions.value.filter((item) => item.type === group.type),
  }));
});

function nextRowId() {
  sequence.value += 1;
  return `row-${sequence.value}`;
}

function createEmptyRow(): EditableRelationRow {
  return {
    id: nextRowId(),
    peer_id: "",
    peer_type: "application",
    direction: "downstream",
    relation_type: "http_call",
    description: "",
  };
}

function addRow() {
  relationRows.value.push(createEmptyRow());
}

function removeRow(rowId: string) {
  relationRows.value = relationRows.value.filter((item) => item.id !== rowId);
}

function onPeerChange(row: EditableRelationRow, peerId: string) {
  const target = relationTargetOptions.value.find((item) => item.id === peerId);
  if (target) {
    row.peer_type = target.type;
  }
}

async function fetchTargetOptions() {
  optionsLoading.value = true;
  try {
    const [apps, mws, ngs] = await Promise.all([
      listApplications({ page: 1, page_size: 999 }),
      listMiddlewares({ page: 1, page_size: 999 }),
      listNginxConfigs({ page: 1, page_size: 999 }),
    ]);

    const options: RelationTargetOption[] = [];
    const currentId = props.resourceId ?? "";
    const currentType = props.resourceType;

    apps.data.forEach((item: Application) => {
      if (!(item.id === currentId && currentType === "application")) {
        options.push({
          id: item.id,
          name: item.name,
          type: "application",
          displayLabel: formatRelationTargetLabel({
            resourceType: "application",
            name: item.name,
            appType: item.type,
          }),
        });
      }
    });
    mws.data.forEach((item: Middleware) => {
      if (!(item.id === currentId && currentType === "middleware")) {
        options.push({
          id: item.id,
          name: item.name,
          type: "middleware",
          displayLabel: formatRelationTargetLabel({
            resourceType: "middleware",
            name: item.name,
            middlewareType: item.type,
          }),
        });
      }
    });
    ngs.data.forEach((item: NginxConfig) => {
      if (!(item.id === currentId && currentType === "nginx")) {
        options.push({
          id: item.id,
          name: item.name,
          type: "nginx",
          displayLabel: formatRelationTargetLabel({
            resourceType: "nginx",
            name: item.name,
          }),
        });
      }
    });
    relationTargetOptions.value = options;
  } catch {
    // error shown by tauriInvoke
  } finally {
    optionsLoading.value = false;
  }
}

function hydrateRows(relations: CallRelation[]) {
  const mergedRows = mergeCallRelationsForEditor(relations);
  relationRows.value = mergedRows.map((relation) => ({
    id: nextRowId(),
    peer_id: relation.peer_id,
    peer_type: relation.peer_type,
    direction: relation.direction,
    relation_type: relation.relation_type,
    description: relation.description,
  }));
}

async function fetchExistingRelations() {
  if (!props.resourceId) {
    relationRows.value = [];
    return;
  }

  loading.value = true;
  try {
    const result = await listCallRelations({
      page: 1,
      page_size: 500,
      filters: {
        owner_id: props.resourceId,
        owner_type: props.resourceType,
      },
    });
    hydrateRows(result.data);
  } catch {
    // error shown by tauriInvoke
  } finally {
    loading.value = false;
  }
}

function getDraftItems(): ReplaceCallRelationItem[] | null {
  const normalizedRows: EditableCallRelationDraft[] = relationRows.value.map((row) => ({
    ...row,
    peer_id: row.peer_id.trim(),
    description: row.description.trim(),
  }));

  const missingRow = normalizedRows.find(
    (row) => row.peer_id.trim().length === 0 || row.relation_type.trim().length === 0,
  );
  if (missingRow) {
    ElMessage.warning("调用关系存在未填写完整的行，请补全后再保存。");
    return null;
  }

  const expandedItems = expandEditorDraftsForSave(normalizedRows);

  if (hasDuplicateExpandedItems(expandedItems)) {
    ElMessage.warning("调用关系存在重复项，请删除重复行后再保存。");
    return null;
  }

  return expandedItems;
}

defineExpose({
  getDraftItems,
});

async function initializeEditor() {
  await fetchTargetOptions();
  await fetchExistingRelations();
}

watch(
  () => [props.resourceId, props.resourceType],
  () => {
    initializeEditor();
  },
);

onMounted(() => {
  initializeEditor();
});
</script>

<template>
  <div class="call-relations-editor">
    <div class="editor-toolbar">
      <span class="editor-title">调用关系</span>
      <el-button type="primary" text size="small" @click="addRow">添加关系</el-button>
    </div>

    <el-table
      :data="relationRows"
      v-loading="loading || optionsLoading"
      size="small"
      stripe
      class="w-full"
      empty-text="暂无调用关系，点击“添加关系”开始维护"
    >
      <el-table-column label="方向" width="140" align="center">
        <template #default="{ row }">
          <el-select v-model="row.direction" class="w-full">
            <el-option label="上游" value="upstream" />
            <el-option label="下游" value="downstream" />
            <el-option label="互相调用" value="bidirectional" />
          </el-select>
        </template>
      </el-table-column>
      <el-table-column label="目标资源" min-width="220">
        <template #default="{ row }">
          <el-select
            v-model="row.peer_id"
            filterable
            class="w-full"
            placeholder="选择目标资源"
            @change="(value) => onPeerChange(row, value)"
          >
            <el-option-group
              v-for="group in groupedTargetOptions"
              :key="group.type"
              :label="group.label"
            >
              <el-option
                v-for="option in group.options"
                :key="option.id"
                :label="option.displayLabel"
                :value="option.id"
              />
            </el-option-group>
          </el-select>
        </template>
      </el-table-column>
      <el-table-column label="关系类型" width="170" align="center">
        <template #default="{ row }">
          <el-select v-model="row.relation_type" class="w-full">
            <el-option
              v-for="option in relationTypeOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
        </template>
      </el-table-column>
      <el-table-column label="关系说明" min-width="180">
        <template #default="{ row }">
          <el-input v-model="row.description" maxlength="120" show-word-limit />
        </template>
      </el-table-column>
      <el-table-column label="操作" width="70" align="center">
        <template #default="{ row }">
          <el-button text type="danger" size="small" @click="removeRow(row.id)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<style scoped lang="scss">
.call-relations-editor {
  margin-top: 12px;
  border-top: 1px solid var(--im-border-subtle);
  padding-top: 12px;
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.editor-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--im-text-primary);
}
</style>
