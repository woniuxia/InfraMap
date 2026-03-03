<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage } from "element-plus";
import type {
  ExecuteImportRowsInput,
  ImportDraftRow,
  ImportExecutionResult,
  ImportJobDetail,
  ImportJobSummary,
  ImportPreviewResult,
  ImportExecutionStrategy,
} from "@/types";
import {
  executeImportRows,
  getImportJobDetail,
  listImportJobs,
  previewImportRows,
} from "@/api/import-jobs";

const rows = ref<ImportDraftRow[]>([]);
const preview = ref<ImportPreviewResult | null>(null);
const executeResult = ref<ImportExecutionResult | null>(null);
const jobs = ref<ImportJobSummary[]>([]);
const jobDetail = ref<ImportJobDetail | null>(null);
const previewLoading = ref(false);
const executeLoading = ref(false);
const jobsLoading = ref(false);
const detailLoading = ref(false);
const detailVisible = ref(false);
const strategy = ref<ImportExecutionStrategy>("skip");
const pasteText = ref("");

const strategyOptions: { label: string; value: ImportExecutionStrategy; hint: string }[] = [
  { label: "默认跳过", value: "skip", hint: "发现冲突时跳过该行" },
  { label: "覆盖更新", value: "overwrite", hint: "冲突时更新已有记录" },
  { label: "阻断失败", value: "block", hint: "有冲突则该行失败" },
];

const activeRows = computed(() =>
  rows.value
    .map((item) => ({
      ...item,
      resource_type: item.resource_type?.trim() || "application",
      name: item.name?.trim(),
      env: item.env || "prod",
      type: item.type?.trim() || "backend",
      status: item.status || "running",
      address: item.address?.trim(),
      description: item.description?.trim(),
    }))
    .filter((item) => !!item.name)
);

function createEmptyRow(): ImportDraftRow {
  return {
    resource_type: "application",
    name: "",
    env: "prod",
    type: "backend",
    status: "running",
    address: "",
    port: 8080,
    description: "",
  };
}

function resetRows() {
  rows.value = [createEmptyRow()];
}

function addRow() {
  rows.value.push(createEmptyRow());
}

function removeRow(index: number) {
  rows.value.splice(index, 1);
  if (rows.value.length === 0) {
    addRow();
  }
}

function loadDemoRows() {
  rows.value = [
    {
      resource_type: "application",
      name: "order-api",
      env: "prod",
      type: "backend",
      status: "running",
      address: "10.1.0.21",
      port: 8080,
      description: "订单服务",
    },
    {
      resource_type: "application",
      name: "payment-api",
      env: "prod",
      type: "backend",
      status: "running",
      address: "10.1.0.22",
      port: 8080,
      description: "支付服务",
    },
  ];
}

function parsePastedRows() {
  const raw = pasteText.value.trim();
  if (!raw) {
    ElMessage.warning("请先粘贴制表符分隔的文本");
    return;
  }

  const lines = raw
    .split(/\r?\n/)
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
  if (lines.length === 0) {
    ElMessage.warning("未解析到可用行");
    return;
  }

  const known = new Set(["resource_type", "name", "env", "type", "status", "address", "port", "description"]);
  const firstCells = lines[0].split("\t").map((cell) => cell.trim().toLowerCase());
  const hasHeader = firstCells.some((item) => known.has(item));

  const dataLines = hasHeader ? lines.slice(1) : lines;
  const nextRows: ImportDraftRow[] = dataLines.map((line) => {
    const cells = line.split("\t");
    const portRaw = cells[6]?.trim();
    return {
      resource_type: (cells[0] || "application").trim() || "application",
      name: (cells[1] || "").trim(),
      env: ((cells[2] || "prod").trim() || "prod") as "prod" | "dev" | "test",
      type: (cells[3] || "backend").trim() || "backend",
      status: ((cells[4] || "running").trim() || "running") as "running" | "stopped" | "maintenance",
      address: (cells[5] || "").trim(),
      port: portRaw ? Number(portRaw) : undefined,
      description: (cells[7] || "").trim(),
    };
  });

  rows.value = nextRows.length > 0 ? nextRows : [createEmptyRow()];
  ElMessage.success(`已导入 ${nextRows.length} 行草稿`);
}

async function handlePreview() {
  if (activeRows.value.length === 0) {
    ElMessage.warning("请至少填写一行 name");
    return;
  }

  previewLoading.value = true;
  try {
    preview.value = await previewImportRows({
      rows: activeRows.value,
    });
  } catch {
    // error already presented by tauriInvoke
  } finally {
    previewLoading.value = false;
  }
}

async function handleExecute() {
  if (activeRows.value.length === 0) {
    ElMessage.warning("请至少填写一行 name");
    return;
  }

  const payload: ExecuteImportRowsInput = {
    rows: activeRows.value,
    strategy: strategy.value,
  };

  executeLoading.value = true;
  try {
    executeResult.value = await executeImportRows(payload);
    ElMessage.success("批量录入已执行");
    await loadJobs();
  } catch {
    // error already presented by tauriInvoke
  } finally {
    executeLoading.value = false;
  }
}

async function loadJobs() {
  jobsLoading.value = true;
  try {
    const result = await listImportJobs({ page: 1, page_size: 20 });
    jobs.value = result.data;
  } catch {
    // error already presented by tauriInvoke
  } finally {
    jobsLoading.value = false;
  }
}

async function openDetail(jobId: string) {
  detailVisible.value = true;
  detailLoading.value = true;
  try {
    jobDetail.value = await getImportJobDetail(jobId);
  } catch {
    // error already presented by tauriInvoke
  } finally {
    detailLoading.value = false;
  }
}

function rowClass(issueType: string) {
  if (issueType === "error") return "danger";
  if (issueType === "conflict") return "warning";
  return "info";
}

onMounted(() => {
  resetRows();
  loadJobs();
});
</script>

<template>
  <div class="import-workbench">
    <el-card class="workbench-card">
      <template #header>
        <div class="card-header">
          <span class="title">批量录入工作台</span>
          <span class="hint">当前版本支持 application 资源批量录入</span>
        </div>
      </template>

      <div class="toolbar">
        <el-button @click="addRow">新增行</el-button>
        <el-button @click="resetRows">重置</el-button>
        <el-button @click="loadDemoRows">示例数据</el-button>
      </div>

      <el-table :data="rows" border size="small" class="draft-table">
        <el-table-column type="index" label="#" width="50" />
        <el-table-column label="resource_type" width="130">
          <template #default="{ row }">
            <el-select v-model="row.resource_type" class="w-full">
              <el-option label="application" value="application" />
            </el-select>
          </template>
        </el-table-column>
        <el-table-column label="name" min-width="150">
          <template #default="{ row }">
            <el-input v-model="row.name" placeholder="应用名" />
          </template>
        </el-table-column>
        <el-table-column label="env" width="95">
          <template #default="{ row }">
            <el-select v-model="row.env">
              <el-option label="prod" value="prod" />
              <el-option label="dev" value="dev" />
              <el-option label="test" value="test" />
            </el-select>
          </template>
        </el-table-column>
        <el-table-column label="type" width="130">
          <template #default="{ row }">
            <el-select v-model="row.type">
              <el-option label="backend" value="backend" />
              <el-option label="frontend" value="frontend" />
              <el-option label="gateway" value="gateway" />
              <el-option label="microservice" value="microservice" />
              <el-option label="other" value="other" />
            </el-select>
          </template>
        </el-table-column>
        <el-table-column label="status" width="130">
          <template #default="{ row }">
            <el-select v-model="row.status">
              <el-option label="running" value="running" />
              <el-option label="stopped" value="stopped" />
              <el-option label="maintenance" value="maintenance" />
            </el-select>
          </template>
        </el-table-column>
        <el-table-column label="address" min-width="150">
          <template #default="{ row }">
            <el-input v-model="row.address" placeholder="可空" />
          </template>
        </el-table-column>
        <el-table-column label="port" width="110">
          <template #default="{ row }">
            <el-input-number v-model="row.port" :min="1" :max="65535" class="w-full" />
          </template>
        </el-table-column>
        <el-table-column label="description" min-width="150">
          <template #default="{ row }">
            <el-input v-model="row.description" placeholder="描述" />
          </template>
        </el-table-column>
        <el-table-column label="操作" width="90" fixed="right">
          <template #default="{ $index }">
            <el-button text type="danger" @click="removeRow($index)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="paste-panel">
        <el-input
          v-model="pasteText"
          type="textarea"
          :rows="4"
          placeholder="支持粘贴 TSV：resource_type\tname\tenv\ttype\tstatus\taddress\tport\tdescription"
        />
        <el-button @click="parsePastedRows">解析粘贴内容</el-button>
      </div>

      <div class="strategy-row">
        <span class="label">冲突策略</span>
        <el-radio-group v-model="strategy">
          <el-radio-button
            v-for="item in strategyOptions"
            :key="item.value"
            :label="item.value"
          >
            {{ item.label }}
          </el-radio-button>
        </el-radio-group>
      </div>
      <p class="strategy-hint">
        {{ strategyOptions.find((item) => item.value === strategy)?.hint }}
      </p>

      <div class="actions">
        <el-button :loading="previewLoading" @click="handlePreview">预检</el-button>
        <el-button type="primary" :loading="executeLoading" @click="handleExecute">执行录入</el-button>
      </div>
    </el-card>

    <el-card class="workbench-card">
      <template #header>
        <div class="card-header">
          <span class="title">预检结果</span>
        </div>
      </template>

      <div v-if="preview" class="summary-row">
        <el-tag type="success">可执行 {{ preview.valid_count }}</el-tag>
        <el-tag type="danger">错误 {{ preview.error_count }}</el-tag>
        <el-tag type="warning">冲突 {{ preview.conflict_count }}</el-tag>
        <el-tag>告警 {{ preview.warning_count }}</el-tag>
      </div>
      <el-empty v-else description="尚未执行预检" :image-size="64" />

      <el-table
        v-if="preview?.issues.length"
        :data="preview.issues"
        border
        size="small"
        max-height="260"
      >
        <el-table-column prop="row_no" label="行号" width="70" />
        <el-table-column prop="issue_type" label="类型" width="90">
          <template #default="{ row }">
            <el-tag :type="rowClass(row.issue_type)" size="small">{{ row.issue_type }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="field_key" label="字段" width="130">
          <template #default="{ row }">{{ row.field_key || "-" }}</template>
        </el-table-column>
        <el-table-column prop="message" label="信息" min-width="220" />
      </el-table>
    </el-card>

    <el-card class="workbench-card">
      <template #header>
        <div class="card-header">
          <span class="title">执行记录</span>
          <el-button text @click="loadJobs">刷新</el-button>
        </div>
      </template>

      <el-alert
        v-if="executeResult"
        type="success"
        show-icon
        :closable="false"
        class="result-alert"
      >
        <template #title>
          最新执行：created={{ executeResult.created_count }} / updated={{ executeResult.updated_count }} /
          skipped={{ executeResult.skipped_count }} / failed={{ executeResult.failed_count }}
        </template>
      </el-alert>

      <el-table :data="jobs" v-loading="jobsLoading" border size="small">
        <el-table-column prop="id" label="任务ID" min-width="250" show-overflow-tooltip />
        <el-table-column prop="status" label="状态" width="160" />
        <el-table-column prop="strategy" label="策略" width="100" />
        <el-table-column prop="total_rows" label="总行数" width="90" />
        <el-table-column prop="created_count" label="创建" width="80" />
        <el-table-column prop="updated_count" label="更新" width="80" />
        <el-table-column prop="skipped_count" label="跳过" width="80" />
        <el-table-column prop="failed_count" label="失败" width="80" />
        <el-table-column label="操作" width="90" fixed="right">
          <template #default="{ row }">
            <el-button text type="primary" @click="openDetail(row.id)">详情</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="detailVisible" title="任务详情" width="980px" destroy-on-close>
      <div v-loading="detailLoading">
        <el-descriptions v-if="jobDetail" :column="4" border size="small" class="detail-summary">
          <el-descriptions-item label="任务ID">{{ jobDetail.summary.id }}</el-descriptions-item>
          <el-descriptions-item label="状态">{{ jobDetail.summary.status }}</el-descriptions-item>
          <el-descriptions-item label="策略">{{ jobDetail.summary.strategy }}</el-descriptions-item>
          <el-descriptions-item label="总行数">{{ jobDetail.summary.total_rows }}</el-descriptions-item>
        </el-descriptions>

        <el-table v-if="jobDetail" :data="jobDetail.rows" border size="small" max-height="260">
          <el-table-column prop="row_no" label="行号" width="70" />
          <el-table-column prop="resource_type" label="类型" width="120" />
          <el-table-column prop="name" label="名称" min-width="180" />
          <el-table-column prop="env" label="环境" width="90" />
          <el-table-column prop="status" label="结果" width="130" />
          <el-table-column prop="error_message" label="错误" min-width="220" show-overflow-tooltip />
        </el-table>

        <el-table
          v-if="jobDetail?.issues?.length"
          :data="jobDetail.issues"
          border
          size="small"
          max-height="220"
          class="detail-issues"
        >
          <el-table-column prop="row_no" label="行号" width="70" />
          <el-table-column prop="issue_type" label="类型" width="90">
            <template #default="{ row }">
              <el-tag :type="rowClass(row.issue_type)" size="small">{{ row.issue_type }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="field_key" label="字段" width="120" />
          <el-table-column prop="message" label="信息" min-width="220" />
        </el-table>
      </div>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.import-workbench {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.workbench-card {
  border: 1px solid var(--im-border-light);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.title {
  font-size: 15px;
  font-weight: 600;
}

.hint {
  font-size: 12px;
  color: var(--im-text-secondary);
}

.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.draft-table {
  margin-bottom: 12px;
}

.paste-panel {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  margin-bottom: 12px;
}

.strategy-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.label {
  font-size: 13px;
  color: var(--im-text-secondary);
}

.strategy-hint {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--im-text-muted);
}

.actions {
  display: flex;
  gap: 8px;
}

.summary-row {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
}

.result-alert {
  margin-bottom: 12px;
}

.detail-summary {
  margin-bottom: 12px;
}

.detail-issues {
  margin-top: 12px;
}

@media (max-width: 920px) {
  .paste-panel {
    grid-template-columns: 1fr;
  }

  .strategy-row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
