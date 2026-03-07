<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import {
  TOPOLOGY_ENV_LABELS,
  TOPOLOGY_ENV_ORDER,
  type TopologyEdgeType,
  type TopologyFilterState,
} from "@/components/topology/topologyGraph.utils";
import type { TopologyEnv, TopologyGroupKind, TopologyLegendStats, TopologyNode } from "@/types";

const props = defineProps<{
  nodes: TopologyNode[];
  stats: TopologyLegendStats | null;
  filter: TopologyFilterState;
  focusNeighborhoodEnabled: boolean;
  layout: "force" | "dagre";
}>();

const emit = defineEmits<{
  (e: "search", payload: { matchIds: string[]; focusId?: string }): void;
  (e: "filter-change", payload: Partial<TopologyFilterState>): void;
  (e: "layout-change", type: "force" | "dagre"): void;
  (e: "export", type: "png" | "svg"): void;
  (e: "refresh"): void;
  (e: "fullscreen"): void;
  (e: "focus-mode-change", enabled: boolean): void;
}>();

const NODE_KIND_LABELS: Record<TopologyGroupKind, string> = {
  application_service: "应用服务",
  middleware: "中间件",
  nginx: "Nginx",
};

const EDGE_LABELS: Record<TopologyEdgeType, string> = {
  http_call: "HTTP 调用",
  tcp: "TCP",
  mq_produce: "MQ 生产",
  mq_consume: "MQ 消费",
  grpc_call: "gRPC",
  db_query: "数据库查询",
  cache_access: "缓存访问",
};

const NODE_KIND_ORDER: TopologyGroupKind[] = ["application_service", "middleware", "nginx"];
const EDGE_KIND_ORDER: TopologyEdgeType[] = [
  "http_call",
  "tcp",
  "mq_produce",
  "mq_consume",
  "grpc_call",
  "db_query",
  "cache_access",
];

const searchText = ref("");
const matchIds = ref<string[]>([]);
const currentMatchIndex = ref(0);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

const searchCountText = computed(() => {
  if (!searchText.value.trim()) return "";
  return `${matchIds.value.length > 0 ? currentMatchIndex.value + 1 : 0}/${matchIds.value.length}`;
});

const currentEnvCount = computed(() => {
  const env = props.filter.env;
  return props.stats?.env_counts.find((item) => item.env === env)?.count ?? 0;
});

const currentEnvAppCount = computed(() => {
  const env = props.filter.env;
  return props.stats?.env_counts.find((item) => item.env === env)?.app_count ?? 0;
});

const externalNodeCount = computed(() => props.stats?.external_node_count ?? 0);
const crossEnvEdgeCount = computed(() => props.stats?.cross_env_edge_count ?? 0);

function normalizeNodeKind(kind: string): TopologyGroupKind | null {
  if (kind === "application") return "application_service";
  if (kind === "application_service" || kind === "middleware" || kind === "nginx") return kind;
  return null;
}

const nodeItems = computed(() => {
  const countMap = new Map<TopologyGroupKind, number>();
  for (const item of props.stats?.node_type_counts || []) {
    const normalized = normalizeNodeKind(item.kind);
    if (!normalized) continue;
    countMap.set(normalized, (countMap.get(normalized) || 0) + item.count);
  }

  return NODE_KIND_ORDER.map((kind) => ({
    kind,
    label: NODE_KIND_LABELS[kind],
    count: countMap.get(kind) || 0,
  }));
});

const edgeItems = computed(() => {
  const countMap = new Map<TopologyEdgeType, number>();
  for (const item of props.stats?.edge_type_counts || []) {
    const typed = item.kind as TopologyEdgeType;
    if (!(typed in EDGE_LABELS)) continue;
    countMap.set(typed, (countMap.get(typed) || 0) + item.count);
  }

  return EDGE_KIND_ORDER.map((kind) => ({
    kind,
    label: EDGE_LABELS[kind],
    count: countMap.get(kind) || 0,
  }));
});

function emitSearchResult() {
  const focusId = matchIds.value.length > 0 ? matchIds.value[currentMatchIndex.value] : undefined;
  emit("search", { matchIds: matchIds.value, focusId });
}

function doSearch() {
  const keyword = searchText.value.trim().toLowerCase();
  if (!keyword) {
    matchIds.value = [];
    currentMatchIndex.value = 0;
    emit("search", { matchIds: [] });
    return;
  }

  matchIds.value = props.nodes
    .filter((node) => {
      const name = node.name.toLowerCase();
      const nodeType = node.node_type.toLowerCase();
      const extra = node.extra || {};
      const address = ((extra.address as string) || "").toLowerCase();
      const ip = ((extra.ip as string) || "").toLowerCase();
      return name.includes(keyword) || nodeType.includes(keyword) || address.includes(keyword) || ip.includes(keyword);
    })
    .map((node) => node.id);

  currentMatchIndex.value = 0;
  emitSearchResult();
}

function handleSearchInput() {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(doSearch, 300);
}

function handleSearchEnter() {
  if (searchTimer) {
    clearTimeout(searchTimer);
    searchTimer = null;
  }
  if (matchIds.value.length === 0) {
    doSearch();
    return;
  }
  currentMatchIndex.value = (currentMatchIndex.value + 1) % matchIds.value.length;
  emitSearchResult();
}

function clearSearch() {
  if (!searchText.value) return;
  searchText.value = "";
  doSearch();
}

function switchEnv(env: TopologyEnv) {
  if (env === props.filter.env) return;
  emit("filter-change", { env });
}

function toggleNodeKind(kind: TopologyGroupKind) {
  const set = new Set(props.filter.nodeKinds);
  if (set.has(kind)) {
    set.delete(kind);
  } else {
    set.add(kind);
  }
  emit("filter-change", { nodeKinds: Array.from(set) });
}

function toggleEdgeKind(kind: TopologyEdgeType) {
  const set = new Set(props.filter.edgeTypes);
  if (set.has(kind)) {
    set.delete(kind);
  } else {
    set.add(kind);
  }
  emit("filter-change", { edgeTypes: Array.from(set) });
}

function updateShowAllEdges(event: Event) {
  const target = event.target as HTMLInputElement | null;
  emit("filter-change", { showAllEdges: Boolean(target?.checked) });
}

function updateFocusNeighborhood(event: Event) {
  const target = event.target as HTMLInputElement | null;
  emit("focus-mode-change", Boolean(target?.checked));
}

function changeLayout(layout: "force" | "dagre") {
  if (props.layout === layout) return;
  emit("layout-change", layout);
}

watch(
  () => props.nodes,
  () => {
    if (searchText.value.trim()) doSearch();
  },
);

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer);
});
</script>

<template>
  <div class="topology-control-bar">
    <div class="control-row control-row-primary">
      <div class="search-box">
        <input
          v-model="searchText"
          data-testid="topology-search-input"
          class="search-input"
          type="text"
          placeholder="搜索节点名称/IP..."
          @input="handleSearchInput"
          @keydown.enter.prevent="handleSearchEnter"
        >
        <button
          v-if="searchText"
          type="button"
          class="text-btn"
          @click="clearSearch"
        >
          清空
        </button>
        <span v-if="searchCountText" class="search-count">{{ searchCountText }}</span>
      </div>

      <div class="action-box">
        <div class="chip-group">
          <button
            data-testid="layout-dagre"
            type="button"
            class="chip-btn"
            :class="{ active: layout === 'dagre' }"
            @click="changeLayout('dagre')"
          >
            层级
          </button>
          <button
            data-testid="layout-force"
            type="button"
            class="chip-btn"
            :class="{ active: layout === 'force' }"
            @click="changeLayout('force')"
          >
            力导向
          </button>
        </div>

        <button data-testid="export-png" type="button" class="action-btn" @click="emit('export', 'png')">
          导出 PNG
        </button>
        <button data-testid="export-svg" type="button" class="action-btn" @click="emit('export', 'svg')">
          导出 SVG
        </button>
        <button data-testid="action-refresh" type="button" class="action-btn" @click="emit('refresh')">
          刷新
        </button>
        <button data-testid="action-fullscreen" type="button" class="action-btn" @click="emit('fullscreen')">
          全屏
        </button>
      </div>
    </div>

    <div class="control-row control-row-secondary">
      <div class="group">
        <span class="group-title">环境</span>
        <div class="chip-group">
          <button
            v-for="env in TOPOLOGY_ENV_ORDER"
            :key="env"
            :data-testid="`env-${env}`"
            type="button"
            class="chip-btn"
            :class="{ active: filter.env === env }"
            @click="switchEnv(env)"
          >
            {{ TOPOLOGY_ENV_LABELS[env] }}
          </button>
        </div>
      </div>

      <div class="group">
        <span class="group-title">节点类型</span>
        <div class="chip-group">
          <button
            v-for="item in nodeItems"
            :key="item.kind"
            :data-testid="`node-kind-${item.kind}`"
            type="button"
            class="chip-btn"
            :class="{ active: filter.nodeKinds.includes(item.kind) }"
            @click="toggleNodeKind(item.kind)"
          >
            {{ item.label }} · {{ item.count }}
          </button>
        </div>
      </div>

      <div class="group">
        <span class="group-title">关系类型</span>
        <div class="chip-group">
          <button
            v-for="item in edgeItems"
            :key="item.kind"
            :data-testid="`edge-kind-${item.kind}`"
            type="button"
            class="chip-btn"
            :class="{ active: filter.edgeTypes.includes(item.kind) }"
            @click="toggleEdgeKind(item.kind)"
          >
            {{ item.label }} · {{ item.count }}
          </button>
        </div>
      </div>

      <label class="switch-wrap">
        <input
          data-testid="show-all-edges"
          type="checkbox"
          :checked="filter.showAllEdges"
          @change="updateShowAllEdges"
        >
        <span>显示全部关系线</span>
      </label>

      <label class="switch-wrap">
        <input
          data-testid="focus-neighborhood"
          type="checkbox"
          :checked="focusNeighborhoodEnabled"
          @change="updateFocusNeighborhood"
        >
        <span>点击节点聚焦上下游</span>
      </label>

      <div class="metric-group">
        <span class="metric-item">当前节点 {{ currentEnvCount }}</span>
        <span class="metric-item">应用服务 {{ currentEnvAppCount }}</span>
        <span class="metric-item">跨环境节点 {{ externalNodeCount }}</span>
        <span class="metric-item">跨环境关系 {{ crossEnvEdgeCount }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.topology-control-bar {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--im-border-subtle);
  background:
    radial-gradient(120% 200% at 0% 0%, rgba(92, 163, 255, 0.07) 0%, transparent 48%),
    var(--im-surface-0);
  flex-shrink: 0;
}

.control-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}

.control-row-secondary {
  justify-content: flex-start;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 280px;
  max-width: 460px;
  flex: 1 1 300px;
}

.search-input {
  width: 100%;
  min-height: 32px;
  border: 1px solid var(--im-border-subtle);
  border-radius: var(--im-radius-sm);
  background: var(--im-surface-1);
  color: var(--im-text-primary);
  padding: 6px 10px;
  outline: none;
  transition:
    border-color var(--im-duration-fast) var(--im-ease-standard),
    box-shadow var(--im-duration-fast) var(--im-ease-standard);
}

.search-input:focus-visible {
  border-color: var(--im-border-active);
  box-shadow: 0 0 0 2px var(--im-accent-soft);
}

.search-count {
  min-width: 36px;
  text-align: center;
  color: var(--im-text-secondary);
  font-size: 12px;
}

.text-btn {
  min-height: 32px;
  border: none;
  background: transparent;
  color: var(--im-text-secondary);
  cursor: pointer;
}

.text-btn:hover {
  color: var(--im-accent);
}

.action-box {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.group-title {
  color: var(--im-text-secondary);
  font-size: 12px;
  white-space: nowrap;
}

.chip-group {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.chip-btn,
.action-btn {
  min-height: 32px;
  padding: 4px 10px;
  border-radius: var(--im-radius-sm);
  border: 1px solid var(--im-border-subtle);
  background: var(--im-surface-1);
  color: var(--im-text-primary);
  cursor: pointer;
  font-size: 12px;
  transition:
    border-color var(--im-duration-fast) var(--im-ease-standard),
    background var(--im-duration-fast) var(--im-ease-standard),
    color var(--im-duration-fast) var(--im-ease-standard);
}

.chip-btn:hover,
.action-btn:hover {
  background: var(--im-surface-2);
}

.chip-btn:focus-visible,
.action-btn:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}

.chip-btn.active {
  border-color: var(--im-border-active);
  background: var(--im-accent-soft);
  color: var(--im-accent);
}

.switch-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--im-text-secondary);
  font-size: 12px;
  min-height: 32px;
}

.metric-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.metric-item {
  display: inline-flex;
  align-items: center;
  min-height: 32px;
  padding: 4px 10px;
  border: 1px dashed var(--im-border-subtle);
  border-radius: var(--im-radius-sm);
  color: var(--im-text-secondary);
  font-size: 12px;
  background: var(--im-surface-1);
}

@media (max-width: 1080px) {
  .search-box {
    flex: 1 1 100%;
    max-width: 100%;
  }

  .metric-group {
    width: 100%;
  }
}
</style>
