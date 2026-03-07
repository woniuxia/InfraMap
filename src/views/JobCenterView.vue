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

function statusTone(status: string) {
  if (["failed", "error"].includes(status)) return "danger";
  if (["done", "completed", "success", "succeeded"].includes(status)) return "success";
  if (["running", "pending"].includes(status)) return "warning";
  return "info";
}

function formatJson(value: Record<string, unknown> | null | undefined) {
  if (value == null) {
    return "{}";
  }

  return JSON.stringify(value, null, 2);
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
  selectedJobId.value = jobId;
  detailLoading.value = true;
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
        <el-button data-testid="refresh-jobs" :loading="loading" @click="loadJobs">刷新任务</el-button>
      </div>
    </div>

    <div class="job-layout">
      <el-card class="panel-card">
        <template #header>
          <div class="card-header">
            <div>
              <span class="card-title">任务列表</span>
              <span class="card-hint">共 {{ jobsPage?.total ?? 0 }} 个任务</span>
            </div>
          </div>
        </template>

        <div v-if="loading" class="empty-state">正在加载任务列表...</div>
        <div v-else-if="jobs.length === 0" class="empty-state" data-testid="empty-jobs">暂无系统任务</div>
        <div v-else class="table-wrap">
          <table class="job-table" data-testid="jobs-table">
            <thead>
              <tr>
                <th>任务</th>
                <th>类型</th>
                <th>状态</th>
                <th>进度</th>
                <th>更新时间</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="job in jobs"
                :key="job.id"
                :data-testid="`job-row-${job.id}`"
                :class="['job-row', { active: selectedJobId === job.id }]"
              >
                <td>
                  <div class="job-title">{{ job.title }}</div>
                  <div v-if="job.summary" class="job-summary">{{ job.summary }}</div>
                </td>
                <td>{{ job.job_type }}</td>
                <td>
                  <el-tag :type="statusTone(job.status)" effect="light">{{ job.status }}</el-tag>
                </td>
                <td>{{ job.progress_percent }}%</td>
                <td>{{ job.updated_at || job.created_at }}</td>
                <td>
                  <el-button :data-testid="`detail-${job.id}`" @click="loadJobDetail(job.id)">详情</el-button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </el-card>

      <el-card class="panel-card detail-card">
        <template #header>
          <div class="card-header">
            <div>
              <span class="card-title">任务详情</span>
              <span class="card-hint">展示摘要、异常与导入明细</span>
            </div>
          </div>
        </template>

        <div v-if="detailLoading" class="empty-state">正在加载任务详情...</div>
        <div v-else-if="!selectedDetail" class="empty-state" data-testid="empty-detail">请选择左侧任务查看详情</div>
        <div v-else class="detail-content" data-testid="job-detail">
          <section class="detail-section">
            <h3 class="section-title">摘要</h3>
            <div class="detail-summary-grid">
              <div class="summary-item">
                <span class="summary-label">标题</span>
                <span class="summary-value">{{ selectedDetail.summary.title }}</span>
              </div>
              <div class="summary-item">
                <span class="summary-label">状态</span>
                <span class="summary-value">{{ selectedDetail.summary.status }}</span>
              </div>
              <div class="summary-item">
                <span class="summary-label">类型</span>
                <span class="summary-value">{{ selectedDetail.summary.job_type }}</span>
              </div>
              <div class="summary-item">
                <span class="summary-label">进度</span>
                <span class="summary-value">{{ selectedDetail.summary.progress_percent }}%</span>
              </div>
            </div>
          </section>

          <section class="detail-section">
            <h3 class="section-title">错误信息</h3>
            <div class="detail-block" data-testid="error-message">
              {{ selectedDetail.error_message || "无" }}
            </div>
          </section>

          <section class="detail-section detail-grid">
            <div>
              <h3 class="section-title">Payload</h3>
              <pre class="json-block" data-testid="payload-json">{{ formatJson(selectedDetail.payload) }}</pre>
            </div>
            <div>
              <h3 class="section-title">Result</h3>
              <pre class="json-block" data-testid="result-json">{{ formatJson(selectedDetail.result) }}</pre>
            </div>
          </section>

          <section class="detail-section">
            <h3 class="section-title">导入记录</h3>
            <div v-if="!selectedDetail.import_rows?.length" class="detail-block">暂无导入记录</div>
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
                  <tr v-for="row in selectedDetail.import_rows" :key="`${row.row_no}-${row.name}`">
                    <td>{{ row.row_no }}</td>
                    <td>{{ row.resource_type }}</td>
                    <td>{{ row.name }}</td>
                    <td>{{ row.env }}</td>
                    <td>{{ row.status }}</td>
                    <td>{{ row.error_message || "-" }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>

          <section class="detail-section">
            <h3 class="section-title">导入问题</h3>
            <div v-if="!selectedDetail.import_issues?.length" class="detail-block">暂无导入问题</div>
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
                  <tr v-for="issue in selectedDetail.import_issues" :key="`${issue.row_no}-${issue.code}`">
                    <td>{{ issue.row_no }}</td>
                    <td>{{ issue.field_key || "-" }}</td>
                    <td>{{ issue.issue_type }}</td>
                    <td>{{ issue.code }}</td>
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
  color: var(--im-text-primary);
}

.page-subtitle {
  margin: 8px 0 0;
  color: var(--im-text-secondary);
  max-width: 720px;
}

.job-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr);
  gap: 16px;
}

.panel-card {
  min-height: 420px;
  background: var(--im-surface-1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.card-hint {
  margin-left: 12px;
  color: var(--im-text-secondary);
  font-size: 13px;
}

.empty-state {
  padding: 32px 16px;
  text-align: center;
  color: var(--im-text-secondary);
}

.table-wrap {
  overflow: auto;
}

.job-table,
.detail-table {
  width: 100%;
  border-collapse: collapse;
}

.job-table th,
.job-table td,
.detail-table th,
.detail-table td {
  padding: 12px;
  border-bottom: 1px solid var(--im-border-light);
  text-align: left;
  vertical-align: top;
  color: var(--im-text-primary);
}

.job-row.active {
  background: var(--im-accent-dim);
}

.job-title,
.summary-value {
  color: var(--im-text-primary);
  font-weight: 600;
}

.job-summary,
.summary-label,
.detail-block {
  margin-top: 6px;
  color: var(--im-text-secondary);
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.section-title {
  margin: 0;
  font-size: 15px;
  color: var(--im-text-primary);
}

.detail-summary-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.summary-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-md);
  background: var(--im-surface-0);
}

.json-block {
  margin: 0;
  padding: 12px;
  min-height: 140px;
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-md);
  background: var(--im-surface-0);
  color: var(--im-text-primary);
  white-space: pre-wrap;
  word-break: break-word;
}

@media (max-width: 1080px) {
  .job-layout,
  .detail-grid,
  .detail-summary-grid {
    grid-template-columns: 1fr;
  }
}
</style>
