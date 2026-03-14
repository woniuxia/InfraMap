<script setup lang="ts">
import { computed } from "vue";
import type {
  TopologyEvidenceItem,
  TopologyImpactResponse,
  TopologyNode,
  TopologyPathsResponse,
  TopologyTroubleshootReport,
} from "@/types";

export type TopologyWorkbenchTab = "overview" | "evidence" | "path" | "impact";

const props = defineProps<{
  visible: boolean;
  activeTab: TopologyWorkbenchTab;
  selectedNode: TopologyNode | null;
  troubleshootReport: TopologyTroubleshootReport | null;
  troubleshootLoading: boolean;
  pathResult: TopologyPathsResponse | null;
  impactResult: TopologyImpactResponse | null;
  nodeNameMap: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "tab-change", tab: TopologyWorkbenchTab): void;
  (e: "start-path-trace"): void;
  (e: "analyze-impact"): void;
  (e: "edit-node"): void;
}>();

const tabs: Array<{ key: TopologyWorkbenchTab; label: string }> = [
  { key: "overview", label: "概览" },
  { key: "evidence", label: "证据" },
  { key: "path", label: "路径" },
  { key: "impact", label: "影响" },
];

const NODE_TYPE_LABELS: Record<string, string> = {
  application: "应用",
  middleware: "中间件",
  nginx: "网关",
};

const STATUS_LABELS: Record<string, string> = {
  running: "运行中",
  stopped: "已停止",
  maintenance: "维护中",
};

const ENV_LABELS: Record<string, string> = {
  prod: "生产",
  dev: "开发",
  test: "测试",
};

const EVIDENCE_TYPE_LABELS: Record<string, string> = {
  profile: "节点画像",
  dependency_summary: "依赖摘要",
  deployment: "部署记录",
  audit: "审计事件",
  call_relation: "调用关系",
  metric: "监控指标",
  alert: "告警",
  change: "变更",
  annotation: "注释",
};

const reportSummary = computed(() => props.troubleshootReport?.summary ?? null);
const evidenceItems = computed(() => props.troubleshootReport?.evidence?.items ?? []);
const evidenceTotal = computed(
  () => props.troubleshootReport?.evidence?.total ?? evidenceItems.value.length,
);
const reportInsights = computed(() => props.troubleshootReport?.insights ?? []);
const upstreamItems = computed(() => props.troubleshootReport?.upstream ?? []);
const downstreamItems = computed(() => props.troubleshootReport?.downstream ?? []);

const overviewMetrics = computed(() => {
  if (!reportSummary.value) return [];
  return [
    { key: "inbound", label: "入边", value: reportSummary.value.inboundEdgeCount },
    { key: "outbound", label: "出边", value: reportSummary.value.outboundEdgeCount },
    { key: "deployment", label: "部署", value: reportSummary.value.deploymentCount },
    { key: "audit", label: "最近变更", value: reportSummary.value.recentAuditCount },
  ];
});

const impactByDepth = computed(() => {
  if (!props.impactResult) return [];
  const nodes = props.impactResult.affectedNodes || props.impactResult.affected_nodes || [];
  const depthMap: Record<number, { id: string; name: string; node_type: string }[]> = {};
  nodes.forEach((node) => {
    const nodeType = node.nodeType || node.node_type || "service";
    if (!depthMap[node.depth]) {
      depthMap[node.depth] = [];
    }
    depthMap[node.depth].push({ id: node.id, name: node.name, node_type: nodeType });
  });
  return Object.entries(depthMap)
    .sort(([left], [right]) => Number(left) - Number(right))
    .map(([depth, nodes]) => ({ depth: Number(depth), nodes }));
});

function extraEntries(node: TopologyNode): Array<{ key: string; value: string }> {
  if (!node.extra) return [];
  const labels: Record<string, string> = {
    type: "类型",
    category: "分类",
    tech_stack: "技术栈",
    address: "地址",
    port: "端口",
    version: "版本",
    endpoint_count: "端点数量",
    first_endpoint: "首个端点",
    strategy: "策略",
  };

  return Object.entries(node.extra)
    .filter(([, value]) => value !== undefined && value !== null && value !== "")
    .map(([key, value]) => ({
      key: labels[key] || key,
      value: String(value),
    }));
}

function evidenceTypeLabel(item: TopologyEvidenceItem): string {
  const type = item.evidenceType || item.evidence_type || "annotation";
  return EVIDENCE_TYPE_LABELS[type] || type;
}

function evidenceTone(item: TopologyEvidenceItem): string {
  if (item.severity === "critical") return "danger";
  if (item.severity === "warning") return "warning";
  return "info";
}

function formatEvidenceTime(timestamp?: string): string {
  if (!timestamp) return "未知时间";
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return timestamp;
  return date.toLocaleString("zh-CN", { hour12: false });
}

function getNodeName(nodeId: string): string {
  return props.nodeNameMap[nodeId] || nodeId.slice(0, 8);
}

function insightSeverityLabel(severity?: string): string {
  if (severity === "critical") return "高风险";
  if (severity === "warning") return "需关注";
  return "提示";
}
</script>

<template>
  <transition name="slide">
    <aside v-if="visible" class="detail-panel">
      <div class="panel-header">
        <div>
          <div class="panel-eyebrow">排障工作台</div>
          <div class="panel-title">{{ selectedNode?.name || "未选择节点" }}</div>
        </div>
        <button type="button" class="text-btn" @click="emit('close')">关闭</button>
      </div>

      <div class="panel-tab-bar">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          class="panel-tab"
          :class="{ active: activeTab === tab.key }"
          @click="emit('tab-change', tab.key)"
        >
          {{ tab.label }}
        </button>
      </div>

      <div class="panel-body">
        <template v-if="activeTab === 'overview'">
          <div v-if="selectedNode" class="overview-section">
            <div class="node-card">
              <div class="node-card-main">
                <span class="node-name">{{ selectedNode.name }}</span>
                <span class="status-chip" :data-tone="reportSummary?.statusSeverity || 'info'">
                  {{
                    STATUS_LABELS[selectedNode.status || "running"] || selectedNode.status || "未知"
                  }}
                </span>
              </div>
              <div class="node-meta">
                <span>
                  {{
                    NODE_TYPE_LABELS[selectedNode.node_type || ""] ||
                    selectedNode.node_type ||
                    "未知"
                  }}
                </span>
                <span>·</span>
                <span>{{ ENV_LABELS[selectedNode.env] || selectedNode.env }}</span>
                <template v-if="selectedNode.host_name">
                  <span>·</span>
                  <span>{{ selectedNode.host_name }}</span>
                </template>
              </div>
            </div>

            <div class="metric-grid">
              <div v-for="item in overviewMetrics" :key="item.key" class="metric-card">
                <div class="metric-label">{{ item.label }}</div>
                <div class="metric-value">{{ item.value }}</div>
              </div>
            </div>

            <div class="action-row">
              <button type="button" class="action-btn" @click="emit('start-path-trace')">
                发起路径追踪
              </button>
              <button type="button" class="action-btn" @click="emit('analyze-impact')">
                分析影响面
              </button>
              <button type="button" class="action-btn ghost" @click="emit('edit-node')">
                编辑资源
              </button>
            </div>

            <div v-if="extraEntries(selectedNode).length > 0" class="section-block">
              <div class="section-title">节点属性</div>
              <div class="kv-list">
                <div v-for="entry in extraEntries(selectedNode)" :key="entry.key" class="kv-row">
                  <span class="kv-key">{{ entry.key }}</span>
                  <span class="kv-value">{{ entry.value }}</span>
                </div>
              </div>
            </div>

            <div v-if="reportInsights.length > 0" class="section-block">
              <div class="section-title">线索提示</div>
              <div class="insight-list">
                <article
                  v-for="(item, index) in reportInsights"
                  :key="`${item.kind}-${index}`"
                  class="insight-item"
                >
                  <div class="insight-head">
                    <span class="insight-badge" :data-tone="item.severity || 'info'">
                      {{ insightSeverityLabel(item.severity) }}
                    </span>
                    <span class="insight-title">{{ item.title }}</span>
                  </div>
                  <div v-if="item.description" class="insight-desc">{{ item.description }}</div>
                </article>
              </div>
            </div>

            <div class="section-block">
              <div class="section-title">一跳邻居</div>
              <div class="neighbor-grid">
                <div class="neighbor-column">
                  <div class="neighbor-title">上游 {{ upstreamItems.length }}</div>
                  <div v-if="upstreamItems.length === 0" class="muted-text">暂无上游</div>
                  <div v-else class="neighbor-list">
                    <span v-for="item in upstreamItems" :key="item.id" class="neighbor-chip">
                      {{ item.name }}
                    </span>
                  </div>
                </div>
                <div class="neighbor-column">
                  <div class="neighbor-title">下游 {{ downstreamItems.length }}</div>
                  <div v-if="downstreamItems.length === 0" class="muted-text">暂无下游</div>
                  <div v-else class="neighbor-list">
                    <span v-for="item in downstreamItems" :key="item.id" class="neighbor-chip">
                      {{ item.name }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div v-else class="empty-block">请选择一个节点开始排查</div>
        </template>

        <template v-else-if="activeTab === 'evidence'">
          <div class="section-block evidence-section">
            <div class="section-head">
              <div class="section-title">证据</div>
              <div class="section-meta">共 {{ evidenceTotal }} 条</div>
            </div>
            <div v-if="troubleshootLoading" data-testid="evidence-loading" class="loading-block">
              证据加载中...
            </div>
            <div v-else-if="evidenceItems.length === 0" class="empty-block">暂无关联证据</div>
            <div v-else class="evidence-list">
              <article v-for="item in evidenceItems" :key="item.id" class="evidence-item">
                <div class="evidence-head">
                  <span class="evidence-type" :data-tone="evidenceTone(item)">
                    {{ evidenceTypeLabel(item) }}
                  </span>
                  <span class="evidence-time">{{ formatEvidenceTime(item.timestamp) }}</span>
                </div>
                <div class="evidence-title">{{ item.title }}</div>
                <div v-if="item.description" class="evidence-desc">{{ item.description }}</div>
                <div class="evidence-source">{{ item.source || "topology_v3" }}</div>
              </article>
            </div>
          </div>
        </template>

        <template v-else-if="activeTab === 'path'">
          <div v-if="!pathResult" class="empty-block">请先从当前节点发起路径追踪</div>
          <div v-else-if="pathResult.paths.length === 0" class="empty-block">未找到连接路径</div>
          <div v-else class="section-block">
            <div class="section-head">
              <div class="section-title">路径结果</div>
              <div class="section-meta">找到 {{ pathResult.paths.length }} 条</div>
            </div>
            <div
              v-for="(path, index) in pathResult.paths"
              :key="`${index}-${(path.nodeIds || path.node_ids || []).join('-')}`"
              class="path-item"
            >
              <div class="path-index">#{{ index + 1 }}</div>
              <div class="path-chain">
                <template
                  v-for="(nodeId, nodeIndex) in path.nodeIds || path.node_ids || []"
                  :key="`${nodeId}-${nodeIndex}`"
                >
                  <span class="path-node">{{ getNodeName(nodeId) }}</span>
                  <span
                    v-if="nodeIndex < (path.nodeIds || path.node_ids || []).length - 1"
                    class="path-arrow"
                  >
                    →
                  </span>
                </template>
              </div>
            </div>
          </div>
        </template>

        <template v-else-if="activeTab === 'impact'">
          <div v-if="!impactResult" class="empty-block">请先执行影响分析</div>
          <div v-else class="section-block">
            <div class="metric-grid compact">
              <div class="metric-card">
                <div class="metric-label">受影响节点</div>
                <div class="metric-value">
                  {{ impactResult.totalCount ?? impactResult.total_count ?? 0 }}
                </div>
              </div>
              <div class="metric-card">
                <div class="metric-label">最大层级</div>
                <div class="metric-value">
                  {{ impactResult.maxDepth ?? impactResult.max_depth ?? 0 }}
                </div>
              </div>
            </div>
            <div
              v-if="(impactResult.affectedNodes || impactResult.affected_nodes || []).length === 0"
              class="empty-block inline"
            >
              无上游依赖
            </div>
            <div v-else class="impact-list">
              <div v-for="group in impactByDepth" :key="group.depth" class="impact-group">
                <div class="impact-title">第 {{ group.depth }} 层</div>
                <div class="impact-tags">
                  <span v-for="node in group.nodes" :key="node.id" class="impact-chip">
                    {{ node.name }}
                    <span class="impact-node-type">
                      ({{ NODE_TYPE_LABELS[node.node_type] || node.node_type }})
                    </span>
                  </span>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>
    </aside>
  </transition>
</template>

<style scoped>
.detail-panel {
  width: 380px;
  border-left: 1px solid var(--im-border-subtle);
  background: var(--im-surface-0);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  flex-shrink: 0;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 12px;
  border-bottom: 1px solid var(--im-border-subtle);
}

.panel-eyebrow {
  font-size: 12px;
  color: var(--im-text-secondary);
}

.panel-title {
  margin-top: 4px;
  font-size: 18px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.text-btn {
  border: none;
  background: transparent;
  color: var(--im-accent);
  cursor: pointer;
}

.panel-tab-bar {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--im-border-subtle);
}

.panel-tab {
  min-height: 32px;
  border-radius: var(--im-radius-sm);
  border: 1px solid var(--im-border-subtle);
  background: var(--im-surface-1);
  color: var(--im-text-secondary);
  cursor: pointer;
}

.panel-tab.active {
  background: var(--im-accent-dim);
  color: var(--im-accent);
  border-color: color-mix(in srgb, var(--im-accent) 35%, var(--im-border-subtle));
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px 18px;
}

.overview-section,
.section-block,
.evidence-section,
.impact-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.node-card,
.metric-card,
.insight-item,
.evidence-item,
.path-item,
.impact-group,
.neighbor-column {
  border: 1px solid var(--im-border-subtle);
  border-radius: var(--im-radius-md);
  background: var(--im-surface-1);
}

.node-card {
  padding: 12px;
}

.node-card-main,
.section-head,
.evidence-head,
.insight-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.node-name,
.section-title,
.evidence-title,
.insight-title,
.neighbor-title,
.impact-title {
  font-weight: 600;
  color: var(--im-text-primary);
}

.node-meta,
.section-meta,
.muted-text,
.evidence-time,
.evidence-source,
.insight-desc,
.kv-key,
.impact-node-type {
  color: var(--im-text-secondary);
  font-size: 12px;
}

.node-meta,
.kv-row,
.path-chain,
.impact-tags,
.neighbor-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.status-chip,
.evidence-type,
.insight-badge,
.neighbor-chip,
.impact-chip {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 12px;
}

.status-chip[data-tone="critical"],
.evidence-type[data-tone="danger"],
.insight-badge[data-tone="critical"] {
  background: color-mix(in srgb, var(--im-danger) 16%, transparent);
  color: var(--im-danger);
}

.status-chip[data-tone="warning"],
.evidence-type[data-tone="warning"],
.insight-badge[data-tone="warning"] {
  background: color-mix(in srgb, var(--im-warning) 16%, transparent);
  color: var(--im-warning);
}

.status-chip[data-tone="info"],
.evidence-type[data-tone="info"],
.insight-badge[data-tone="info"],
.neighbor-chip,
.impact-chip {
  background: var(--im-accent-dim);
  color: var(--im-accent);
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.metric-grid.compact {
  margin-bottom: 12px;
}

.metric-card {
  padding: 12px;
}

.metric-label {
  font-size: 12px;
  color: var(--im-text-secondary);
}

.metric-value {
  margin-top: 6px;
  font-size: 24px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.action-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.action-btn {
  min-height: 32px;
  border: 1px solid transparent;
  border-radius: var(--im-radius-sm);
  background: var(--im-accent);
  color: #fff;
  padding: 0 12px;
  cursor: pointer;
}

.action-btn.ghost {
  background: var(--im-surface-1);
  border-color: var(--im-border-subtle);
  color: var(--im-text-primary);
}

.kv-list,
.evidence-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.kv-row,
.path-item,
.impact-group,
.evidence-item,
.neighbor-column,
.insight-item {
  padding: 12px;
}

.kv-row {
  justify-content: space-between;
}

.kv-value {
  color: var(--im-text-primary);
  text-align: right;
}

.path-index {
  font-size: 12px;
  color: var(--im-text-secondary);
  margin-bottom: 8px;
}

.path-arrow {
  color: var(--im-text-secondary);
}

.loading-block,
.empty-block {
  padding: 28px 12px;
  text-align: center;
  color: var(--im-text-secondary);
}

.empty-block.inline {
  padding: 12px 0 0;
}

.slide-enter-active,
.slide-leave-active {
  transition:
    transform 0.18s ease,
    opacity 0.18s ease;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  transform: translateX(12px);
}
</style>
