<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { getSystemJobDetail, listSystemJobs } from "@/api/system-jobs";
import type { PagedResult, SystemJobDetail, SystemJobSummary } from "@/types";

const loading = ref(false);
const detailLoading = ref(false);
const jobsPage = ref<PagedResult<SystemJobSummary> | null>(null);
const selectedJobId = ref<string>("");
const selectedDetail = ref<SystemJobDetail | null>(null);

const jobs = computed(() => jobsPage.value?.data ?? []);

const STATUS_TONE: Record<string, string> = {
  failed: "danger",
  error: "danger",
  done: "success",
  completed: "success",
  success: "success",
  succeeded: "success",
  running: "warning",
  pending: "warning",
};

const STATUS_LABEL: Record<string, string> = {
  failed: "失败",
  error: "异常",
  done: "完成",
  completed: "完成",
  success: "成功",
  succeeded: "成功",
  running: "进行中",
  pending: "等待中",
};

const JOB_TYPE_LABEL: Record<string, string> = {
  import: "批量导入",
  snapshot_import: "快照导入",
  snapshot_export: "快照导出",
  cleanup: "数据清理",
};

type ElTagType = "primary" | "success" | "warning" | "danger" | "info";

function statusTone(status: string): ElTagType {
  return (STATUS_TONE[status] as ElTagType) ?? "info";
}

function statusLabel(status: string): string {
  return STATUS_LABEL[status] ?? status;
}

function jobTypeLabel(jobType: string): string {
  return JOB_TYPE_LABEL[jobType] ?? jobType;
}

function formatTime(val: string | null | undefined): string {
  if (!val) return "-";
  const d = new Date(val);
  if (isNaN(d.getTime())) return val;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function formatJson(value: Record<string, unknown> | null | undefined) {
  if (value == null) return "{}";
  return JSON.stringify(value, null, 2);
}

function progressStatus(percent: number, status: string): "" | "success" | "exception" | "warning" {
  if (["failed", "error"].includes(status)) return "exception";
  if (["done", "completed", "success", "succeeded"].includes(status)) return "success";
  if (percent > 0 && percent < 100) return "";
  return "";
}

async function loadJobs() {
  loading.value = true;
  try {
    jobsPage.value = await listSystemJobs({ page: 1, page_size: 20 });
    const exists = jobs.value.some((item) => item.id === selectedJobId.value);
    if (!exists) {
      selectedJobId.value = "";
      selectedDetail.value = null;
    }
  } catch {
    jobsPage.value = null;
    selectedJobId.value = "";
    selectedDetail.value = null;
  } finally {
    loading.value = false;
  }
}

async function loadJobDetail(jobId: string) {
  if (selectedJobId.value === jobId) return;
  selectedJobId.value = jobId;
  detailLoading.value = true;
  selectedDetail.value = null;
  try {
    selectedDetail.value = await getSystemJobDetail(jobId);
  } catch {
    selectedDetail.value = null;
  } finally {
    detailLoading.value = false;
  }
}

onMounted(loadJobs);
</script>

<template>
  <div class="job-center">
    <div class="page-header">
      <div>
        <h2 class="page-title">任务中心</h2>
        <p class="page-subtitle">查看系统任务执行摘要，并按需钻取导入类任务的执行细节。</p>
      </div>
      <div class="page-actions">
        <el-button
          data-testid="refresh-jobs"
          :loading="loading"
          :icon="loading ? undefined : 'Refresh'"
          @click="loadJobs"
        >
          刷新任务
        </el-button>
      </div>
    </div>

    <div class="job-layout">
      <!-- 左侧：任务列表 -->
      <el-card class="panel-card">
        <template #header>
          <div class="card-header">
            <div class="card-header-left">
              <span class="card-title">任务列表</span>
              <el-tag v-if="!loading" type="info" size="small" round class="count-badge">
                {{ jobsPage?.total ?? 0 }}
              </el-tag>
            </div>
          </div>
        </template>

        <!-- 骨架屏 -->
        <div v-if="loading" class="skeleton-wrap">
          <div v-for="i in 5" :key="i" class="skeleton-row">
            <el-skeleton :rows="2" animated />
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else-if="jobs.length === 0" class="empty-state" data-testid="empty-jobs">
          <el-empty description="暂无系统任务" :image-size="64" />
        </div>

        <!-- 任务列表 -->
        <div v-else class="table-wrap">
          <table class="job-table" data-testid="jobs-table">
            <thead>
              <tr>
                <th>任务</th>
                <th>类型</th>
                <th>状态</th>
                <th>进度</th>
                <th>更新时间</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="job in jobs"
                :key="job.id"
                :data-testid="`job-row-${job.id}`"
                :class="['job-row', { active: selectedJobId === job.id }]"
                @click="loadJobDetail(job.id)"
              >
                <td>
                  <div class="job-title">{{ job.title }}</div>
                  <div v-if="job.summary" class="job-summary">{{ job.summary }}</div>
                </td>
                <td>
                  <span class="type-badge">{{ jobTypeLabel(job.job_type) }}</span>
                </td>
                <td>
                  <el-tag :type="statusTone(job.status)" effect="light" size="small">
                    {{ statusLabel(job.status) }}
                  </el-tag>
                </td>
                <td class="progress-cell">
                  <el-progress
                    :percentage="job.progress_percent"
                    :status="progressStatus(job.progress_percent, job.status)"
                    :stroke-width="5"
                    :show-text="false"
                    class="progress-bar"
                  />
                  <span class="progress-text">{{ job.progress_percent }}%</span>
                </td>
                <td class="time-cell">{{ formatTime(job.updated_at || job.created_at) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </el-card>

      <!-- 右侧：任务详情 -->
      <el-card class="panel-card detail-card">
        <template #header>
          <div class="card-header">
            <div class="card-header-left">
              <span class="card-title">任务详情</span>
              <span class="card-hint">摘要 · 异常 · 导入明细</span>
            </div>
          </div>
        </template>

        <!-- 详情骨架屏 -->
        <div v-if="detailLoading" class="skeleton-wrap">
          <el-skeleton :rows="6" animated />
        </div>

        <!-- 未选中 -->
        <div v-else-if="!selectedDetail" class="empty-state" data-testid="empty-detail">
          <el-empty description="点击左侧任务行查看详情" :image-size="64" />
        </div>

        <!-- 详情内容 -->
        <div v-else class="detail-content" data-testid="job-detail">
          <!-- 摘要 -->
          <section class="detail-section">
            <h3 class="section-title">摘要</h3>
            <div class="detail-summary-grid">
              <div class="summary-item">
                <span class="summary-label">标题</span>
                <span class="summary-value">{{ selectedDetail.summary.title }}</span>
              </div>
              <div class="summary-item">
                <span class="summary-label">状态</span>
                <el-tag
                  :type="statusTone(selectedDetail.summary.status)"
                  effect="light"
                  size="small"
                >
                  {{ statusLabel(selectedDetail.summary.status) }}
                </el-tag>
              </div>
              <div class="summary-item">
                <span class="summary-label">类型</span>
                <span class="summary-value">
                  {{ jobTypeLabel(selectedDetail.summary.job_type) }}
                </span>
              </div>
              <div class="summary-item">
                <span class="summary-label">进度</span>
                <div class="summary-progress">
                  <el-progress
                    :percentage="selectedDetail.summary.progress_percent"
                    :status="
                      progressStatus(
                        selectedDetail.summary.progress_percent,
                        selectedDetail.summary.status,
                      )
                    "
                    :stroke-width="6"
                  />
                </div>
              </div>
            </div>
          </section>

          <!-- 错误信息（仅在有错时展示） -->
          <section v-if="selectedDetail.error_message" class="detail-section">
            <h3 class="section-title section-title--danger">错误信息</h3>
            <div class="error-block" data-testid="error-message">
              {{ selectedDetail.error_message }}
            </div>
          </section>

          <!-- Payload / Result -->
          <section class="detail-section detail-grid">
            <div>
              <h3 class="section-title">Payload</h3>
              <pre class="json-block" data-testid="payload-json">{{
                formatJson(selectedDetail.payload)
              }}</pre>
            </div>
            <div>
              <h3 class="section-title">Result</h3>
              <pre class="json-block" data-testid="result-json">{{
                formatJson(selectedDetail.result)
              }}</pre>
            </div>
          </section>

          <!-- 导入记录 -->
          <section class="detail-section">
            <h3 class="section-title">导入记录</h3>
            <div v-if="!selectedDetail.import_rows?.length" class="hint-block">暂无导入记录</div>
            <div v-else class="table-wrap">
              <table class="detail-table" data-testid="import-rows">
                <thead>
                  <tr>
                    <th>行号</th>
                    <th>资源类型</th>
                    <th>名称</th>
                    <th>环境</th>
                    <th>状态</th>
                    <th>错误</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="row in selectedDetail.import_rows"
                    :key="`${row.row_no}-${row.name}`"
                    :class="{ 'row-error': row.status === 'failed' || row.status === 'error' }"
                  >
                    <td class="mono">{{ row.row_no }}</td>
                    <td>{{ row.resource_type }}</td>
                    <td>{{ row.name }}</td>
                    <td>{{ row.env }}</td>
                    <td>
                      <el-tag :type="statusTone(row.status)" effect="light" size="small">
                        {{ statusLabel(row.status) }}
                      </el-tag>
                    </td>
                    <td class="error-cell">{{ row.error_message || "-" }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>

          <!-- 导入问题 -->
          <section class="detail-section">
            <h3 class="section-title">导入问题</h3>
            <div v-if="!selectedDetail.import_issues?.length" class="hint-block">暂无导入问题</div>
            <div v-else class="table-wrap">
              <table class="detail-table" data-testid="import-issues">
                <thead>
                  <tr>
                    <th>行号</th>
                    <th>字段</th>
                    <th>问题类型</th>
                    <th>代码</th>
                    <th>说明</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="issue in selectedDetail.import_issues"
                    :key="`${issue.row_no}-${issue.code}`"
                  >
                    <td class="mono">{{ issue.row_no }}</td>
                    <td>{{ issue.field_key || "-" }}</td>
                    <td>{{ issue.issue_type }}</td>
                    <td class="mono">{{ issue.code }}</td>
                    <td>{{ issue.message }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>
        </div>
      </el-card>
    </div>
  </div>
</template>

<style scoped lang="scss">
.job-center {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  flex-wrap: wrap;
}

.page-title {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: var(--im-text-primary);
}

.page-subtitle {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--im-text-secondary);
  max-width: 720px;
}

.job-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);
  gap: 16px;
  align-items: start;
}

.panel-card {
  min-height: 480px;
  background: var(--im-surface-1);

  :deep(.el-card__body) {
    padding: 0;
  }
}

.detail-card {
  :deep(.el-card__body) {
    padding: 0 0 16px;
    max-height: calc(100vh - 240px);
    overflow-y: auto;
  }
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.card-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.card-hint {
  font-size: 12px;
  color: var(--im-text-muted);
}

.count-badge {
  font-size: 11px;
}

// 骨架屏
.skeleton-wrap {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.skeleton-row {
  padding: 12px;
  border-bottom: 1px solid var(--im-border-subtle);

  &:last-child {
    border-bottom: none;
  }
}

// 空状态
.empty-state {
  padding: 48px 16px;
  display: flex;
  justify-content: center;
}

// 表格
.table-wrap {
  overflow-x: auto;
}

.job-table,
.detail-table {
  width: 100%;
  border-collapse: collapse;
}

.job-table th,
.detail-table th {
  padding: 10px 14px;
  border-bottom: 1px solid var(--im-border);
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--im-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  white-space: nowrap;
  background: var(--im-surface-2);
}

.job-table td,
.detail-table td {
  padding: 11px 14px;
  border-bottom: 1px solid var(--im-border-subtle);
  text-align: left;
  vertical-align: middle;
  color: var(--im-text-regular);
  font-size: 13px;
}

.job-table tbody tr:last-child td,
.detail-table tbody tr:last-child td {
  border-bottom: none;
}

// 可点击行
.job-row {
  cursor: pointer;
  transition: background 150ms ease;

  &:hover {
    background: var(--im-accent-soft);
  }

  &.active {
    background: var(--im-accent-dim);

    td {
      color: var(--im-text-primary);
    }
  }
}

// 错误行高亮
.row-error td {
  background: color-mix(in srgb, var(--el-color-danger) 5%, transparent);
}

.job-title {
  font-weight: 600;
  color: var(--im-text-primary);
  font-size: 13px;
}

.job-summary {
  margin-top: 3px;
  font-size: 12px;
  color: var(--im-text-muted);
}

.type-badge {
  font-size: 12px;
  color: var(--im-text-secondary);
  background: var(--im-surface-3);
  padding: 2px 8px;
  border-radius: 4px;
  white-space: nowrap;
}

// 进度列
.progress-cell {
  min-width: 100px;
}

.progress-bar {
  margin-bottom: 3px;

  :deep(.el-progress-bar__outer) {
    background: var(--im-surface-3);
  }
}

.progress-text {
  font-size: 11px;
  color: var(--im-text-muted);
  font-variant-numeric: tabular-nums;
}

.time-cell {
  white-space: nowrap;
  font-size: 12px;
  color: var(--im-text-muted);
  font-variant-numeric: tabular-nums;
}

.mono {
  font-family: ui-monospace, "Cascadia Code", monospace;
  font-size: 12px;
  color: var(--im-text-secondary);
}

.error-cell {
  color: var(--el-color-danger);
  font-size: 12px;
}

// 详情区
.detail-content {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.detail-section {
  padding: 16px 20px;
  border-bottom: 1px solid var(--im-border-subtle);

  &:last-child {
    border-bottom: none;
  }
}

.section-title {
  margin: 0 0 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--im-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;

  &--danger {
    color: var(--el-color-danger);
  }
}

.detail-summary-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.summary-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px 14px;
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-sm);
  background: var(--im-surface-0);
}

.summary-label {
  font-size: 11px;
  color: var(--im-text-muted);
  font-weight: 500;
}

.summary-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.summary-progress {
  :deep(.el-progress__text) {
    font-size: 12px !important;
    color: var(--im-text-secondary);
  }
}

.error-block {
  padding: 12px 14px;
  border-radius: var(--im-radius-sm);
  background: color-mix(in srgb, var(--el-color-danger) 8%, var(--im-surface-0));
  border: 1px solid color-mix(in srgb, var(--el-color-danger) 24%, transparent);
  color: var(--el-color-danger);
  font-size: 13px;
  line-height: 1.6;
  word-break: break-all;
}

.hint-block {
  font-size: 13px;
  color: var(--im-text-muted);
  padding: 4px 0;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
  border-bottom: 1px solid var(--im-border-subtle);
}

.json-block {
  margin: 0;
  padding: 12px 14px;
  min-height: 120px;
  max-height: 260px;
  overflow-y: auto;
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-sm);
  background: var(--im-surface-0);
  color: var(--im-text-secondary);
  font-family: ui-monospace, "Cascadia Code", monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

@media (max-width: 1100px) {
  .job-layout {
    grid-template-columns: 1fr;
  }

  .detail-card :deep(.el-card__body) {
    max-height: none;
  }
}

@media (max-width: 720px) {
  .detail-grid,
  .detail-summary-grid {
    grid-template-columns: 1fr;
  }
}
</style>
