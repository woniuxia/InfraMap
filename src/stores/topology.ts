import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { getTopologySnapshotV3, getTopologyTaskViewV3 } from "@/api/topologyV3";
import type {
  TopologyEdge,
  TopologyEnv,
  TopologyGraph,
  TopologyGroupKind,
  TopologyKindCount,
  TopologyLane,
  TopologyLegendStats,
  TopologyNode,
  TopologyNodeType,
  TopologyLayoutHints,
  TopologySnapshotResponse,
  TopologyTaskViewMode,
  TopologyTaskInsight,
} from "@/types";

const CACHE_TTL_MS = 30_000; // 30 seconds
const MAX_DEPTH_RANGE = { min: 1, max: 8 };
const DEFAULT_MAX_DEPTH = 3;
const DEFAULT_TASK_VIEW: TopologyTaskViewMode = "explore";
const TOPOLOGY_ENV_ORDER: TopologyEnv[] = ["prod", "test", "dev"];
const TOPOLOGY_ENV_LABELS: Record<TopologyEnv, string> = {
  prod: "生产",
  test: "测试",
  dev: "开发",
};

function clampMaxDepth(value: number): number {
  const normalized = Number.isFinite(value) ? Math.round(value) : DEFAULT_MAX_DEPTH;
  return Math.min(MAX_DEPTH_RANGE.max, Math.max(MAX_DEPTH_RANGE.min, normalized));
}

function countKinds(items: string[]): TopologyKindCount[] {
  const counter = new Map<string, number>();
  for (const item of items) {
    counter.set(item, (counter.get(item) || 0) + 1);
  }
  return Array.from(counter.entries())
    .map(([kind, count]) => ({ kind, count }))
    .sort((left, right) => left.kind.localeCompare(right.kind));
}

function resolveGroupKind(nodeType: TopologyNodeType, raw?: string): TopologyGroupKind {
  if (raw === "application_service" || raw === "middleware" || raw === "nginx") return raw;
  if (nodeType === "middleware") return "middleware";
  if (nodeType === "nginx") return "nginx";
  return "application_service";
}

function normalizeNode(node: TopologyNode): TopologyNode {
  const nodeType: TopologyNodeType = node.nodeType || node.node_type || "application";
  const groupKind = resolveGroupKind(nodeType, node.groupKind || node.group_kind);
  const importance = Number.isFinite(node.importance) ? Number(node.importance) : 1;
  const isExternal = node.isExternal === true || node.is_external === true;
  const env = node.env || "prod";
  const extra = {
    ...(node.extra || {}),
    ...(node.meta || {}),
  };

  return {
    id: node.id,
    name: node.name,
    nodeType: nodeType,
    env,
    groupKind: groupKind,
    hostId: node.hostId || node.host_id,
    hostName: node.hostName || node.host_name,
    hostIpDisplay: node.hostIpDisplay || node.host_ip_display,
    status: node.status,
    importance,
    extra,
    isExternal: isExternal || undefined,
    externalRefId: node.externalRefId || node.external_ref_id,
  };
}

function normalizeEdge(edge: TopologyEdge): TopologyEdge {
  const edgeType: TopologyEdge["edgeType"] = edge.edgeType || edge.edge_type || "http_call";
  const strength = Number.isFinite(edge.strength) ? Number(edge.strength) : 1;

  return {
    id: edge.id,
    source: edge.source,
    target: edge.target,
    edgeType: edgeType,
    label: edge.label,
    strength,
    crossEnv: edge.crossEnv === true || edge.cross_env === true,
  };
}

function normalizeLanes(nodes: TopologyNode[], sourceLanes: TopologyLane[] = []): TopologyLane[] {
  const laneMap = new Map(sourceLanes.map((lane) => [lane.id, lane]));
  return TOPOLOGY_ENV_ORDER.map((env, index) => {
    const lane = laneMap.get(env);
    const inEnv = nodes.filter((node) => node.env === env);
    return {
      id: env,
      label: lane?.label || TOPOLOGY_ENV_LABELS[env],
      order: lane?.order ?? index,
      nodeCount: lane?.nodeCount ?? inEnv.length,
      appCount:
        lane?.appCount ?? inEnv.filter((node) => node.groupKind === "application_service").length,
    };
  });
}

function normalizeLegendStats(
  legendStats: TopologyLegendStats | undefined,
  nodes: TopologyNode[],
  edges: TopologyEdge[],
  env?: TopologyEnv,
): TopologyLegendStats {
  if (!legendStats) {
    return buildLegendStats(nodes, edges, env);
  }

  return {
    envCounts: legendStats.envCounts.map((item) => ({
      env: item.env,
      count: item.count,
      appCount: item.appCount,
    })),
    nodeTypeCounts: legendStats.nodeTypeCounts.map((item) => ({
      kind: item.kind,
      count: item.count,
    })),
    edgeTypeCounts: legendStats.edgeTypeCounts.map((item) => ({
      kind: item.kind,
      count: item.count,
    })),
    applicationServiceCount: legendStats.applicationServiceCount,
    currentEnv: legendStats.currentEnv,
    externalNodeCount: legendStats.externalNodeCount,
    crossEnvEdgeCount: legendStats.crossEnvEdgeCount,
  };
}

function normalizeLayoutHints(
  layoutHints: TopologyLayoutHints | undefined,
  edges: TopologyEdge[],
): TopologyGraph["layoutHints"] {
  if (!layoutHints) {
    return {
      laneOrder: TOPOLOGY_ENV_ORDER,
      defaultCollapsedGroups: [],
      highDensityMode: edges.length > 260,
    };
  }

  return {
    laneOrder: layoutHints.laneOrder,
    defaultCollapsedGroups: layoutHints.defaultCollapsedGroups,
    highDensityMode: layoutHints.highDensityMode,
  };
}

function buildLegendStats(
  nodes: TopologyNode[],
  edges: TopologyEdge[],
  env?: TopologyEnv,
): TopologyLegendStats {
  return {
    envCounts: TOPOLOGY_ENV_ORDER.map((laneEnv) => {
      const inEnv = nodes.filter((node) => node.env === laneEnv);
      return {
        env: laneEnv,
        count: inEnv.length,
        appCount: inEnv.filter((node) => node.groupKind === "application_service").length,
      };
    }),
    nodeTypeCounts: countKinds(
      nodes.map((node) => node.nodeType || node.node_type || "application"),
    ),
    edgeTypeCounts: countKinds(edges.map((edge) => edge.edgeType || edge.edge_type || "http_call")),
    applicationServiceCount: nodes.filter((node) => node.groupKind === "application_service")
      .length,
    currentEnv: env,
    externalNodeCount: nodes.filter((node) => node.isExternal === true).length,
    crossEnvEdgeCount: edges.filter((edge) => edge.crossEnv).length,
  };
}

function normalizeSnapshot(snapshot: TopologySnapshotResponse): TopologyGraph {
  const nodes = (snapshot.nodes || []).map(normalizeNode);
  const edges = (snapshot.edges || []).map(normalizeEdge);
  const stats = snapshot.legendStats || snapshot.legend_stats;
  const hints = snapshot.layoutHints || snapshot.layout_hints;
  const legendStats = normalizeLegendStats(stats, nodes, edges, snapshot.meta?.env);
  const layoutHints = normalizeLayoutHints(hints, edges);

  return {
    lanes: normalizeLanes(nodes, snapshot.lanes),
    nodes,
    edges,
    legendStats: legendStats,
    layoutHints: layoutHints,
  };
}

function normalizeTaskInsight(insight: TopologyTaskInsight): TopologyTaskInsight {
  return {
    ...insight,
    nodeIds: insight.nodeIds || insight.node_ids || [],
    edgeIds: insight.edgeIds || insight.edge_ids || [],
  };
}

export const useTopologyStore = defineStore("topology", () => {
  const snapshot = ref<TopologySnapshotResponse | null>(null);
  const graphData = ref<TopologyGraph | null>(null);
  const taskInsights = ref<TopologyTaskInsight[]>([]);
  const lastFetchTime = ref<number>(0);
  const loading = ref(false);
  const taskView = ref<TopologyTaskViewMode>(DEFAULT_TASK_VIEW);
  const maxDepth = ref<number>(DEFAULT_MAX_DEPTH);

  // Index node names for fast search
  const nodeNameIndex = computed(() => {
    const map = new Map<string, string[]>();
    if (!graphData.value) return map;

    for (const node of graphData.value.nodes) {
      const words = node.name.toLowerCase().split(/[\s\-_./]+/);
      for (const word of words) {
        if (!word) continue;
        const ids = map.get(word) || [];
        ids.push(node.id);
        map.set(word, ids);
      }
    }
    return map;
  });

  async function loadSnapshot(): Promise<TopologySnapshotResponse> {
    const query = {
      taskView: taskView.value,
      maxDepth: maxDepth.value,
    };

    if (taskView.value === "explore") {
      taskInsights.value = [];
      return getTopologySnapshotV3(query);
    }

    const response = await getTopologyTaskViewV3(query);
    taskInsights.value = (response.insights || []).map(normalizeTaskInsight);
    return response.snapshot;
  }

  async function fetchGraph(forceRefresh = false): Promise<TopologyGraph | null> {
    const now = Date.now();
    if (!forceRefresh && graphData.value && now - lastFetchTime.value < CACHE_TTL_MS) {
      return graphData.value;
    }

    loading.value = true;
    try {
      snapshot.value = await loadSnapshot();
      graphData.value = normalizeSnapshot(snapshot.value);
      lastFetchTime.value = Date.now();
      return graphData.value;
    } catch {
      taskInsights.value = [];
      return null;
    } finally {
      loading.value = false;
    }
  }

  function setTaskView(nextTaskView: TopologyTaskViewMode) {
    if (taskView.value === nextTaskView) return;
    taskView.value = nextTaskView;
    invalidateCache();
  }

  function setMaxDepth(nextMaxDepth: number) {
    const clamped = clampMaxDepth(nextMaxDepth);
    if (maxDepth.value === clamped) return;
    maxDepth.value = clamped;
    invalidateCache();
  }

  function invalidateCache() {
    lastFetchTime.value = 0;
  }

  return {
    snapshot,
    graphData,
    taskInsights,
    lastFetchTime,
    loading,
    taskView,
    maxDepth,
    nodeNameIndex,
    fetchGraph,
    setTaskView,
    setMaxDepth,
    invalidateCache,
  };
});
