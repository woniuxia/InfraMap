<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { ElMessage } from "element-plus";
import TopologyCanvas from "@/components/topology/TopologyCanvas.vue";
import TopologyControlBar from "@/components/topology/TopologyControlBar.vue";
import TopologyDetailPanel from "@/components/topology/TopologyDetailPanel.vue";
import { getTopologyEvidence, getTopologyImpact, getTopologyPaths } from "@/api/topology";
import { getApplication } from "@/api/applications";
import { getMiddleware } from "@/api/middlewares";
import { getNginxConfig } from "@/api/nginx-configs";
import { useTopologyStore } from "@/stores/topology";
import ApplicationEditorDialog from "@/components/resource-editors/ApplicationEditorDialog.vue";
import MiddlewareEditorDialog from "@/components/resource-editors/MiddlewareEditorDialog.vue";
import NginxConfigEditorDialog from "@/components/resource-editors/NginxConfigEditorDialog.vue";
import type {
  Application,
  Middleware,
  NginxConfig,
  TopologyEvidenceItem,
  TopologyImpactResponse,
  TopologyNode,
  TopologyPathsResponse,
  TopologyTaskInsight,
  TopologyTaskViewMode,
} from "@/types";
import {
  DEFAULT_TOPOLOGY_FILTER,
  filterTopologyGraph,
  isExternalTopologyNode,
  type TopologyFilterState,
} from "@/components/topology/topologyGraph.utils";

const topologyStore = useTopologyStore();
const PERFORMANCE_OPT_STORAGE_KEY = "inframap.topology.performanceOptimizationEnabled";
const MAX_DEPTH_RANGE = {
  min: 1,
  max: 8,
};

const TASK_VIEW_OPTIONS: Array<{ value: TopologyTaskViewMode; label: string }> = [
  { value: "explore", label: "探索" },
  { value: "troubleshoot", label: "故障排查" },
  { value: "impact", label: "影响面" },
  { value: "change", label: "变更评估" },
];

const rawGraphData = computed(() => topologyStore.graphData);
const loading = computed(() => topologyStore.loading);
const taskInsights = computed(() => topologyStore.taskInsights || []);
const activeFilter = ref<TopologyFilterState>({ ...DEFAULT_TOPOLOGY_FILTER });
const performanceOptimizationEnabled = ref(false);
const effectiveFilter = computed<TopologyFilterState>(() =>
  performanceOptimizationEnabled.value
    ? activeFilter.value
    : {
        ...activeFilter.value,
        showAllEdges: true,
      },
);
const graphData = computed(() => filterTopologyGraph(rawGraphData.value, effectiveFilter.value));
const canvasRef = ref<InstanceType<typeof TopologyCanvas>>();
const selectedLayout = ref<"force" | "dagre">("force");
const warnedLayoutFallbackReasons = new Set<string>();

const panelMode = ref<"detail" | "path" | "impact" | null>(null);
const selectedNode = ref<TopologyNode | null>(null);
const pathResult = ref<TopologyPathsResponse | null>(null);
const impactResult = ref<TopologyImpactResponse | null>(null);

const pathTraceMode = ref(false);
const pathSource = ref<string | null>(null);
const pathTraceHint = ref("");

const contextMenuVisible = ref(false);
const contextMenuPos = ref({ x: 0, y: 0 });
const contextMenuNode = ref<TopologyNode | null>(null);
const applicationEditorVisible = ref(false);
const middlewareEditorVisible = ref(false);
const nginxEditorVisible = ref(false);
const applicationEditorDraft = ref<Partial<Application>>({});
const middlewareEditorDraft = ref<Partial<Middleware>>({});
const nginxEditorDraft = ref<Partial<NginxConfig>>({});

const isFullscreen = ref(false);
const focusNeighborhoodEnabled = ref(true);
const containerRef = ref<HTMLDivElement>();
const evidenceItems = ref<TopologyEvidenceItem[]>([]);
const evidenceTotal = ref(0);
const evidenceLoading = ref(false);
const evidenceCache = new Map<
  string,
  { items: TopologyEvidenceItem[]; total: number; loadedAt: number }
>();
let evidenceRequestVersion = 0;

const nodeNameMap = computed(() => {
  const map: Record<string, string> = {};
  if (rawGraphData.value) {
    rawGraphData.value.nodes.forEach((node) => {
      map[node.id] = node.name;
    });
    rawGraphData.value.lanes.forEach((lane) => {
      map[lane.id] = lane.label;
    });
  }
  return map;
});

function isExternalNode(node: TopologyNode | null): boolean {
  if (!node) return false;
  return isExternalTopologyNode(node);
}

function evidenceCacheKey(nodeId: string): string {
  return `${topologyStore.taskView}:${topologyStore.maxDepth}:${nodeId}`;
}

function resetEvidenceState() {
  evidenceRequestVersion += 1;
  evidenceLoading.value = false;
  evidenceItems.value = [];
  evidenceTotal.value = 0;
}

function resolveInsightNodeIds(insight: TopologyTaskInsight): string[] {
  return Array.from(new Set((insight.nodeIds || []).filter(Boolean)));
}

function insightSeverityLabel(insight: TopologyTaskInsight): string {
  if (insight.severity === "critical") return "高风险";
  if (insight.severity === "warning") return "需关注";
  return "提示";
}

async function loadEvidenceForNode(node: TopologyNode, forceRefresh = false) {
  const currentVersion = ++evidenceRequestVersion;
  const cacheKey = evidenceCacheKey(node.id);
  if (!forceRefresh) {
    const cached = evidenceCache.get(cacheKey);
    if (cached) {
      evidenceItems.value = cached.items;
      evidenceTotal.value = cached.total;
      evidenceLoading.value = false;
      return;
    }
  }

  evidenceLoading.value = true;

  try {
    const response = await getTopologyEvidence({
      nodeId: node.id,
      taskView: topologyStore.taskView,
      maxItems: 20,
    });
    if (currentVersion !== evidenceRequestVersion) return;

    const items = (response.items || []).slice(0, 20);
    const total = response.total ?? items.length;
    evidenceItems.value = items;
    evidenceTotal.value = total;
    evidenceCache.set(cacheKey, {
      items,
      total,
      loadedAt: Date.now(),
    });
  } catch {
    if (currentVersion !== evidenceRequestVersion) return;
    evidenceItems.value = [];
    evidenceTotal.value = 0;
  } finally {
    if (currentVersion === evidenceRequestVersion) {
      evidenceLoading.value = false;
    }
  }
}

async function loadData() {
  await topologyStore.fetchGraph(true);
}

function normalizeMaxDepth(value: number): number {
  const normalized = Number.isFinite(value) ? Math.round(value) : topologyStore.maxDepth;
  return Math.min(MAX_DEPTH_RANGE.max, Math.max(MAX_DEPTH_RANGE.min, normalized));
}

function readPerformanceOptimizationPreference(): boolean {
  try {
    const raw = localStorage.getItem(PERFORMANCE_OPT_STORAGE_KEY);
    if (raw === "true") return true;
    if (raw === "false") return false;
  } catch {
    // ignore storage read failures and fallback to default
  }
  return false;
}

function persistPerformanceOptimizationPreference(enabled: boolean) {
  try {
    localStorage.setItem(PERFORMANCE_OPT_STORAGE_KEY, enabled ? "true" : "false");
  } catch {
    // ignore storage write failures
  }
}

async function handleTaskViewChange(taskView: TopologyTaskViewMode) {
  if (topologyStore.taskView === taskView) return;
  topologyStore.setTaskView(taskView);
  await loadData();
  if (panelMode.value === "detail" && selectedNode.value) {
    await loadEvidenceForNode(selectedNode.value, true);
  }
}

async function handleMaxDepthChange(event: Event) {
  const input = event.target as HTMLInputElement | null;
  if (!input) return;
  const nextDepth = normalizeMaxDepth(Number(input.value));
  if (topologyStore.maxDepth === nextDepth) return;
  topologyStore.setMaxDepth(nextDepth);
  await loadData();
  if (panelMode.value === "detail" && selectedNode.value) {
    await loadEvidenceForNode(selectedNode.value, true);
  }
}

function handleNodeClick(node: TopologyNode) {
  if (pathTraceMode.value) {
    if (isExternalNode(node)) {
      ElMessage.info("跨环境外部节点不支持路径追踪，请选择当前环境内节点");
      return;
    }

    if (!pathSource.value) {
      pathSource.value = node.id;
      pathTraceHint.value = `已选起点: ${node.name}，请点击终点`;
      ElMessage.info(pathTraceHint.value);
    } else {
      handleFindPaths(pathSource.value, node.id);
      pathTraceMode.value = false;
      pathSource.value = null;
      pathTraceHint.value = "";
    }
    return;
  }

  selectedNode.value = node;
  panelMode.value = "detail";
  void loadEvidenceForNode(node);
}

function handleCanvasNodeClick(node: TopologyNode) {
  handleNodeClick(node);
}

function handleCanvasBlankClick() {
  if (panelMode.value !== "detail" || !selectedNode.value) return;
  panelMode.value = null;
  selectedNode.value = null;
  resetEvidenceState();
  canvasRef.value?.clearHighlight();
}

function handleContextMenu(payload: { node: TopologyNode; x: number; y: number }) {
  if (isExternalNode(payload.node)) {
    ElMessage.info("外部节点仅用于展示跨环境依赖，不支持操作");
    return;
  }
  contextMenuNode.value = payload.node;
  contextMenuPos.value = { x: payload.x, y: payload.y };
  contextMenuVisible.value = true;
}

function closeContextMenu() {
  contextMenuVisible.value = false;
}

function closeNodeEditors() {
  applicationEditorVisible.value = false;
  middlewareEditorVisible.value = false;
  nginxEditorVisible.value = false;
}

function resolveEnvLabel(env: string) {
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env;
}

async function handleEditNode() {
  const node = contextMenuNode.value;
  closeContextMenu();
  if (!node) return;
  if (isExternalNode(node)) {
    ElMessage.info("外部节点仅用于展示跨环境依赖，不支持编辑");
    return;
  }

  closeNodeEditors();

  try {
    if (node.nodeType === "application") {
      const detail = await getApplication(node.id);
      applicationEditorDraft.value = {
        ...detail,
        owners: Array.isArray(detail.owners) ? [...detail.owners] : detail.owners,
      };
      applicationEditorVisible.value = true;
      return;
    }

    if (node.nodeType === "middleware") {
      const detail = await getMiddleware(node.id);
      middlewareEditorDraft.value = { ...detail };
      middlewareEditorVisible.value = true;
      return;
    }

    if (node.nodeType === "nginx") {
      const detail = await getNginxConfig(node.id);
      nginxEditorDraft.value = {
        ...detail,
        endpoints: Array.isArray(detail.endpoints)
          ? detail.endpoints.map((item) => ({ ...item }))
          : detail.endpoints,
      };
      nginxEditorVisible.value = true;
    }
  } catch {
    // error shown by tauriInvoke
  }
}

async function handleNodeEditorSaved(payload: { id: string }) {
  await loadData();
  const refreshedNode = rawGraphData.value?.nodes.find((node) => node.id === payload.id) || null;
  if (!refreshedNode) {
    if (selectedNode.value?.id === payload.id) {
      handlePanelClose();
    }
    return;
  }

  if (activeFilter.value.env !== refreshedNode.env) {
    if (selectedNode.value?.id === payload.id) {
      handlePanelClose();
    }
    ElMessage.info(`节点已迁移到${resolveEnvLabel(refreshedNode.env)}环境，当前筛选下不可见`);
    return;
  }

  if (selectedNode.value?.id === payload.id) {
    selectedNode.value = refreshedNode;
    if (panelMode.value === "detail") {
      void loadEvidenceForNode(refreshedNode, true);
    }
  }
}

function startPathTrace() {
  if (isExternalNode(contextMenuNode.value)) {
    closeContextMenu();
    ElMessage.info("外部节点不支持路径追踪");
    return;
  }

  closeContextMenu();
  pathTraceMode.value = true;
  pathSource.value = contextMenuNode.value?.id || null;
  if (pathSource.value) {
    pathTraceHint.value = `已选起点: ${contextMenuNode.value?.name}，请点击终点`;
    ElMessage.info(pathTraceHint.value);
  }
}

async function handleFindPaths(sourceId: string, targetId: string) {
  try {
    const result = await getTopologyPaths({
      sourceId,
      targetId,
      taskView: topologyStore.taskView,
      maxDepth: topologyStore.maxDepth,
    });

    pathResult.value = {
      ...result,
      paths: result.paths.filter((path) => path.nodeIds.length > 0),
    };

    panelMode.value = "path";
    if (pathResult.value.paths.length > 0) {
      canvasRef.value?.highlightPaths(pathResult.value.paths.map((path) => path.nodeIds));
    } else {
      ElMessage.warning("未找到连接路径");
    }
  } catch {
    // error shown by tauriInvoke
  }
}

async function handleAnalyzeImpact() {
  const node = contextMenuNode.value;
  closeContextMenu();
  if (!node) return;
  if (isExternalNode(node)) {
    ElMessage.info("外部节点不支持影响分析");
    return;
  }

  try {
    const result = await getTopologyImpact({
      nodeId: node.id,
      taskView: topologyStore.taskView,
      maxDepth: topologyStore.maxDepth,
    });

    impactResult.value = result;

    selectedNode.value = node;
    panelMode.value = "impact";
    canvasRef.value?.highlightImpact(node.id, impactResult.value);
  } catch {
    // error shown by tauriInvoke
  }
}

function handleSearch(payload: { matchIds: string[]; focusId?: string }) {
  if (payload.matchIds.length === 0) {
    canvasRef.value?.clearHighlight();
    return;
  }
  canvasRef.value?.highlightSearch(payload.matchIds, payload.focusId);
}

function handleInsightClick(insight: TopologyTaskInsight) {
  const nodeIds = resolveInsightNodeIds(insight);
  if (nodeIds.length === 0) {
    ElMessage.info("该洞察暂无可高亮节点");
    return;
  }
  canvasRef.value?.highlightSearch(nodeIds, nodeIds[0]);
  const focusNode = rawGraphData.value?.nodes.find((node) => node.id === nodeIds[0]) || null;
  if (focusNode && !isExternalNode(focusNode)) {
    selectedNode.value = focusNode;
    panelMode.value = "detail";
    void loadEvidenceForNode(focusNode);
  }
}

function handleFilterChange(payload: Partial<TopologyFilterState>) {
  activeFilter.value = {
    ...activeFilter.value,
    ...payload,
  };
}

function handlePerformanceOptimizationChange(enabled: boolean) {
  performanceOptimizationEnabled.value = enabled;
  persistPerformanceOptimizationPreference(enabled);
}

function handleLayoutChange(type: "force" | "dagre") {
  selectedLayout.value = type;
}

function handleLayoutResolved(payload: {
  requested: "force" | "dagre";
  applied: "force" | "dagre";
  reason?: string;
}) {
  selectedLayout.value = payload.applied;
  if (payload.requested === payload.applied) return;

  const reasonKey = payload.reason?.trim() || "__unknown__";
  if (warnedLayoutFallbackReasons.has(reasonKey)) return;
  warnedLayoutFallbackReasons.add(reasonKey);

  const layoutLabel = payload.applied === "dagre" ? "层次布局" : "力导向布局";
  const reasonText = payload.reason ? `（${payload.reason}）` : "";
  ElMessage.warning(`布局已自动回退为${layoutLabel}${reasonText}`);
}

function handleFocusModeChange(enabled: boolean) {
  focusNeighborhoodEnabled.value = enabled;
  if (!enabled) {
    canvasRef.value?.clearHighlight();
  }
}

async function handleExport(type: "png" | "svg") {
  const dataURL = await canvasRef.value?.exportImage(type);
  if (!dataURL) return;

  const link = document.createElement("a");
  link.download = `topology.${type}`;
  link.href = dataURL;
  link.click();
  ElMessage.success("导出成功");
}

function handlePanelClose() {
  panelMode.value = null;
  selectedNode.value = null;
  pathResult.value = null;
  impactResult.value = null;
  resetEvidenceState();
  canvasRef.value?.clearHighlight();
}

function toggleFullscreen() {
  if (!containerRef.value) return;
  if (!isFullscreen.value) {
    containerRef.value.requestFullscreen?.();
    isFullscreen.value = true;
  } else {
    document.exitFullscreen?.();
    isFullscreen.value = false;
  }
}

function handleFullscreenChange() {
  isFullscreen.value = !!document.fullscreenElement;
}

onMounted(() => {
  performanceOptimizationEnabled.value = readPerformanceOptimizationPreference();
  loadData();
  document.addEventListener("fullscreenchange", handleFullscreenChange);
});

onBeforeUnmount(() => {
  document.removeEventListener("fullscreenchange", handleFullscreenChange);
});
</script>

<template>
  <div ref="containerRef" class="topology-view" v-loading="loading" @click="closeContextMenu">
    <TopologyControlBar
      :nodes="graphData?.nodes || []"
      :stats="graphData?.legendStats || null"
      :filter="effectiveFilter"
      :layout="selectedLayout"
      :focus-neighborhood-enabled="focusNeighborhoodEnabled"
      :performance-optimization-enabled="performanceOptimizationEnabled"
      @search="handleSearch"
      @filter-change="handleFilterChange"
      @performance-optimization-change="handlePerformanceOptimizationChange"
      @layout-change="handleLayoutChange"
      @focus-mode-change="handleFocusModeChange"
      @export="handleExport"
      @refresh="loadData"
      @fullscreen="toggleFullscreen"
    />

    <div class="task-view-bar">
      <div class="task-view-switch">
        <span class="task-view-title">任务视图</span>
        <button
          v-for="option in TASK_VIEW_OPTIONS"
          :key="option.value"
          :data-testid="`task-view-${option.value}`"
          type="button"
          class="task-view-btn"
          :class="{ active: topologyStore.taskView === option.value }"
          @click="handleTaskViewChange(option.value)"
        >
          {{ option.label }}
        </button>
      </div>

      <div class="depth-control">
        <label class="depth-label" for="max-depth-number">层级深度</label>
        <input
          id="max-depth-range"
          data-testid="max-depth-range"
          class="depth-range"
          type="range"
          :min="MAX_DEPTH_RANGE.min"
          :max="MAX_DEPTH_RANGE.max"
          :value="topologyStore.maxDepth"
          @change="handleMaxDepthChange"
        />
        <input
          id="max-depth-number"
          data-testid="max-depth-number"
          class="depth-number"
          type="number"
          :min="MAX_DEPTH_RANGE.min"
          :max="MAX_DEPTH_RANGE.max"
          :value="topologyStore.maxDepth"
          @change="handleMaxDepthChange"
        />
      </div>
    </div>

    <div v-if="taskInsights.length > 0" class="task-insight-bar">
      <span class="task-insight-title">任务洞察</span>
      <button
        v-for="(insight, index) in taskInsights"
        :key="`${insight.kind}-${index}`"
        :data-testid="`task-insight-${index}`"
        type="button"
        class="task-insight-chip"
        :class="{
          warning: insight.severity === 'warning',
          critical: insight.severity === 'critical',
        }"
        @click="handleInsightClick(insight)"
      >
        <span class="task-insight-severity">{{ insightSeverityLabel(insight) }}</span>
        <span class="task-insight-text">{{ insight.title }}</span>
      </button>
    </div>

    <!-- Path trace hint -->
    <div v-if="pathTraceMode" class="path-trace-bar">
      <span>{{ pathTraceHint || "请点击起始节点" }}</span>
      <el-button
        size="small"
        type="info"
        text
        @click="
          pathTraceMode = false;
          pathSource = null;
          pathTraceHint = '';
        "
      >
        取消
      </el-button>
    </div>

    <!-- Content area -->
    <div class="topology-content">
      <!-- Canvas -->
      <div class="canvas-wrapper">
        <TopologyCanvas
          ref="canvasRef"
          :graph-data="graphData"
          :layout="selectedLayout"
          :performance-optimization-enabled="performanceOptimizationEnabled"
          :focus-neighborhood="focusNeighborhoodEnabled && !pathTraceMode"
          @node-click="handleCanvasNodeClick"
          @canvas-blank-click="handleCanvasBlankClick"
          @node-contextmenu="handleContextMenu"
          @layout-resolved="handleLayoutResolved"
        />

        <!-- Empty state -->
        <div v-if="graphData && graphData.nodes.length === 0 && !loading" class="empty-overlay">
          <el-empty
            :description="
              (rawGraphData?.nodes.length || 0) > 0
                ? '当前筛选无数据，请调整顶部筛选项'
                : '暂无拓扑数据，请先添加资源和关系'
            "
          />
        </div>
      </div>

      <!-- Detail panel -->
      <TopologyDetailPanel
        :mode="panelMode"
        :selected-node="selectedNode"
        :path-result="pathResult"
        :impact-result="impactResult"
        :evidence-items="evidenceItems"
        :evidence-total="evidenceTotal"
        :evidence-loading="evidenceLoading"
        :node-name-map="nodeNameMap"
        @close="handlePanelClose"
      />
    </div>

    <!-- Context menu -->
    <teleport to="body">
      <div
        v-if="contextMenuVisible"
        class="context-menu"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
        @click.stop
      >
        <div data-testid="context-edit" class="context-menu-item" @click="handleEditNode">编辑</div>
        <div data-testid="context-path-trace" class="context-menu-item" @click="startPathTrace">
          路径追踪（从此节点出发）
        </div>
        <div data-testid="context-impact" class="context-menu-item" @click="handleAnalyzeImpact">
          影响分析
        </div>
      </div>
    </teleport>

    <ApplicationEditorDialog
      v-model="applicationEditorVisible"
      mode="edit"
      :initial-draft="applicationEditorDraft"
      @saved="handleNodeEditorSaved"
    />
    <MiddlewareEditorDialog
      v-model="middlewareEditorVisible"
      mode="edit"
      :initial-draft="middlewareEditorDraft"
      @saved="handleNodeEditorSaved"
    />
    <NginxConfigEditorDialog
      v-model="nginxEditorVisible"
      mode="edit"
      :initial-draft="nginxEditorDraft"
      @saved="handleNodeEditorSaved"
    />
  </div>
</template>

<style scoped>
.topology-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--im-surface-1);
  position: relative;
}
.task-view-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  padding: 8px 12px;
  border-bottom: 1px solid var(--im-border-subtle);
  background: var(--im-surface-0);
  flex-shrink: 0;
}
.task-view-switch {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.task-view-title {
  font-size: 12px;
  color: var(--im-text-secondary);
}
.task-view-btn {
  min-height: 32px;
  padding: 4px 10px;
  border-radius: var(--im-radius-sm);
  border: 1px solid var(--im-border-subtle);
  background: var(--im-surface-1);
  color: var(--im-text-primary);
  font-size: 12px;
  cursor: pointer;
  transition:
    border-color var(--im-duration-fast) var(--im-ease-standard),
    background var(--im-duration-fast) var(--im-ease-standard),
    color var(--im-duration-fast) var(--im-ease-standard);
}
.task-view-btn:hover {
  background: var(--im-surface-2);
}
.task-view-btn.active {
  border-color: var(--im-border-active);
  background: var(--im-accent-soft);
  color: var(--im-accent);
}
.task-view-btn:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}
.depth-control {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.depth-label {
  font-size: 12px;
  color: var(--im-text-secondary);
}
.depth-range {
  width: 140px;
  accent-color: var(--im-accent);
}
.depth-number {
  width: 56px;
  min-height: 32px;
  border-radius: var(--im-radius-sm);
  border: 1px solid var(--im-border-subtle);
  background: var(--im-surface-1);
  color: var(--im-text-primary);
  padding: 4px 8px;
}
.depth-number:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}
.task-insight-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding: 8px 12px;
  border-bottom: 1px solid var(--im-border-subtle);
  background: color-mix(in srgb, var(--im-accent-soft) 40%, transparent);
  flex-shrink: 0;
}
.task-insight-title {
  font-size: 12px;
  color: var(--im-text-secondary);
  font-weight: 600;
}
.task-insight-chip {
  min-height: 32px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border: 1px solid var(--im-border-subtle);
  border-radius: var(--im-radius-sm);
  background: var(--im-surface-0);
  color: var(--im-text-primary);
  cursor: pointer;
  transition:
    border-color var(--im-duration-fast) var(--im-ease-standard),
    background var(--im-duration-fast) var(--im-ease-standard),
    transform var(--im-duration-fast) var(--im-ease-standard);
}
.task-insight-chip:hover {
  border-color: var(--im-border-active);
  background: var(--im-accent-soft);
  transform: translateY(-1px);
}
.task-insight-chip:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}
.task-insight-chip.warning {
  border-color: color-mix(in srgb, var(--im-warning) 55%, var(--im-border-subtle));
}
.task-insight-chip.critical {
  border-color: color-mix(in srgb, var(--im-danger) 55%, var(--im-border-subtle));
}
.task-insight-severity {
  font-size: 11px;
  color: var(--im-text-muted);
}
.task-insight-text {
  font-size: 12px;
  max-width: 260px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.path-trace-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  background: var(--im-accent-soft);
  border-bottom: 1px solid var(--im-border-active);
  font-size: 13px;
  color: var(--im-accent);
  flex-shrink: 0;
}
.topology-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}
.canvas-wrapper {
  flex: 1;
  position: relative;
  min-width: 0;
}
.empty-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(10, 15, 26, 0.45);
  pointer-events: none;
}
.context-menu {
  position: fixed;
  z-index: 9999;
  background: var(--im-surface-1);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-sm);
  box-shadow: var(--im-shadow-md);
  padding: 4px 0;
  min-width: 160px;
}
.context-menu-item {
  padding: 8px 16px;
  font-size: 13px;
  cursor: pointer;
  transition: background var(--im-duration-fast) var(--im-ease-standard);
}
.context-menu-item:hover {
  background: var(--im-surface-2);
  color: var(--im-accent);
}

@media (max-width: 1080px) {
  .task-view-bar {
    align-items: flex-start;
  }
  .task-insight-bar {
    align-items: flex-start;
  }
  .depth-control {
    width: 100%;
    justify-content: flex-start;
  }
  .topology-content {
    flex-direction: column;
  }
}
</style>
