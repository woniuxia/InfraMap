<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { repairIntegrityFindings, scanIntegrity } from "@/api/integrity";
import type { IntegrityFinding, IntegrityReport } from "@/types";

const router = useRouter();

const loading = ref(false);
const repairing = ref(false);
const report = ref<IntegrityReport | null>(null);
const selectedFindingIds = ref<string[]>([]);

const findings = computed(() => report.value?.findings ?? []);
const repairableCount = computed(() => findings.value.filter((item) => item.repair_supported).length);

function severityTone(severity: IntegrityFinding["severity"]) {
  if (severity === "critical") return "danger";
  if (severity === "warning") return "warning";
  return "info";
}

function isSelected(findingId: string) {
  return selectedFindingIds.value.includes(findingId);
}

function toggleSelection(findingId: string, checked: boolean) {
  if (checked) {
    if (!selectedFindingIds.value.includes(findingId)) {
      selectedFindingIds.value = [...selectedFindingIds.value, findingId];
    }
    return;
  }

  selectedFindingIds.value = selectedFindingIds.value.filter((item) => item !== findingId);
}

function handleSelectionChange(findingId: string, event: Event) {
  const target = event.target as HTMLInputElement | null;
  toggleSelection(findingId, Boolean(target?.checked));
}

async function loadReport() {
  loading.value = true;
  try {
    report.value = await scanIntegrity();
    const repairableIds = new Set(findings.value.filter((item) => item.repair_supported).map((item) => item.id));
    selectedFindingIds.value = selectedFindingIds.value.filter((item) => repairableIds.has(item));
  } catch {
    report.value = null;
  } finally {
    loading.value = false;
  }
}

async function handleRepairSelected() {
  if (selectedFindingIds.value.length === 0) {
    ElMessage.warning("请先勾选可修复的问题");
    return;
  }

  repairing.value = true;
  try {
    const result = await repairIntegrityFindings({ finding_ids: selectedFindingIds.value });
    report.value = result.report;
    selectedFindingIds.value = [];
    ElMessage.success(`已修复 ${result.repaired_count} 项问题`);
  } catch {
    // 错误提示已由 tauriInvoke 统一处理
  } finally {
    repairing.value = false;
  }
}

function handleOpenTarget(finding: IntegrityFinding) {
  const query = Object.keys(finding.target_filters || {}).length > 0 ? finding.target_filters : undefined;
  router.push({ name: finding.target_route, query });
}

onMounted(loadReport);
</script>

<template>
  <div class="integrity-center">
    <div class="page-header">
      <div>
        <h2 class="page-title">数据健康中心</h2>
        <p class="page-subtitle">集中检查孤儿关系、失效绑定与覆盖缺口，并提供受控修复入口。</p>
      </div>
      <div class="page-actions">
        <el-button data-testid="refresh-report" :loading="loading" @click="loadReport">重新扫描</el-button>
        <el-button
          data-testid="repair-selected"
          type="primary"
          :loading="repairing"
          :disabled="selectedFindingIds.length === 0"
          @click="handleRepairSelected"
        >
          修复已选
        </el-button>
      </div>
    </div>

    <div class="summary-grid">
      <el-card class="summary-card">
        <div class="summary-label">问题总数</div>
        <div class="summary-value">{{ report?.summary.total ?? 0 }}</div>
      </el-card>
      <el-card class="summary-card">
        <div class="summary-label">严重</div>
        <div class="summary-value danger">{{ report?.summary.critical ?? 0 }}</div>
      </el-card>
      <el-card class="summary-card">
        <div class="summary-label">警告</div>
        <div class="summary-value warning">{{ report?.summary.warning ?? 0 }}</div>
      </el-card>
      <el-card class="summary-card">
        <div class="summary-label">可修复</div>
        <div class="summary-value success">{{ report?.summary.repairable ?? 0 }}</div>
      </el-card>
    </div>

    <el-card class="findings-card">
      <template #header>
        <div class="card-header">
          <div>
            <span class="card-title">健康问题清单</span>
            <span class="card-hint">当前可自动修复 {{ repairableCount }} 项</span>
          </div>
          <span class="card-meta">最近扫描：{{ report?.summary.generated_at || "--" }}</span>
        </div>
      </template>

      <div v-if="loading" class="empty-state">正在扫描健康问题...</div>

      <div v-else-if="findings.length === 0" class="empty-state" data-testid="empty-findings">
        暂未发现明显的健康问题
      </div>

      <div v-else class="findings-list" data-testid="findings-list">
        <article v-for="finding in findings" :key="finding.id" class="finding-item" :data-testid="`finding-${finding.id}`">
          <div class="finding-select">
            <input
              :checked="isSelected(finding.id)"
              :disabled="!finding.repair_supported"
              type="checkbox"
              :data-testid="`select-${finding.id}`"
              @change="handleSelectionChange(finding.id, $event)"
            />
          </div>
          <div class="finding-main">
            <div class="finding-topline">
              <el-tag :type="severityTone(finding.severity)" effect="light">{{ finding.severity }}</el-tag>
              <span class="finding-title">{{ finding.title }}</span>
              <span v-if="finding.resource_name" class="finding-resource">{{ finding.resource_name }}</span>
            </div>
            <p class="finding-description">{{ finding.description }}</p>
            <div class="finding-meta">
              <span>分类：{{ finding.category }}</span>
              <span>目标页：{{ finding.target_route }}</span>
              <span>{{ finding.repair_supported ? "支持自动修复" : "仅支持人工处理" }}</span>
            </div>
          </div>
          <div class="finding-actions">
            <button class="link-button" :data-testid="`open-${finding.id}`" @click="handleOpenTarget(finding)">查看处理页</button>
          </div>
        </article>
      </div>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.integrity-center {
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

.page-actions {
  display: flex;
  gap: 12px;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.summary-card {
  background: var(--im-surface-1);
}

.summary-label {
  font-size: 13px;
  color: var(--im-text-secondary);
}

.summary-value {
  margin-top: 8px;
  font-size: 28px;
  font-weight: 700;
  color: var(--im-text-primary);
}

.summary-value.danger {
  color: var(--el-color-danger);
}

.summary-value.warning {
  color: var(--el-color-warning);
}

.summary-value.success {
  color: var(--el-color-success);
}

.card-header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: center;
  flex-wrap: wrap;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.card-hint,
.card-meta {
  margin-left: 12px;
  color: var(--im-text-secondary);
  font-size: 13px;
}

.empty-state {
  padding: 32px 16px;
  text-align: center;
  color: var(--im-text-secondary);
}

.findings-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.finding-item {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 12px;
  padding: 16px;
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-md);
  background: var(--im-surface-0);
}

.finding-topline {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.finding-title {
  font-weight: 600;
  color: var(--im-text-primary);
}

.finding-resource,
.finding-meta {
  color: var(--im-text-secondary);
  font-size: 13px;
}

.finding-description {
  margin: 8px 0;
  color: var(--im-text-primary);
  line-height: 1.6;
}

.finding-meta {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.finding-actions {
  display: flex;
  align-items: center;
}

.link-button {
  min-height: 32px;
  padding: 0 12px;
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-sm);
  background: transparent;
  color: var(--im-accent);
  cursor: pointer;
}

.link-button:hover,
.link-button:focus-visible {
  border-color: var(--im-accent);
  background: var(--im-accent-dim);
  outline: none;
}

@media (max-width: 900px) {
  .finding-item {
    grid-template-columns: 1fr;
  }

  .finding-select,
  .finding-actions {
    justify-self: start;
  }
}
</style>
