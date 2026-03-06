import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  getTopologySnapshotV3,
  getTopologyTaskViewV3,
} from '@/api/topologyV3'
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
  TopologyTaskViewMode,
  TopologyV3Edge,
  TopologyV3Node,
  TopologyV3SnapshotResponse,
  TopologyV3TaskInsight,
} from '@/types'

const CACHE_TTL_MS = 30_000 // 30 seconds
const MAX_DEPTH_RANGE = { min: 1, max: 8 }
const DEFAULT_MAX_DEPTH = 3
const DEFAULT_TASK_VIEW: TopologyTaskViewMode = 'explore'
const TOPOLOGY_ENV_ORDER: TopologyEnv[] = ['prod', 'test', 'dev']
const TOPOLOGY_ENV_LABELS: Record<TopologyEnv, string> = {
  prod: '生产',
  test: '测试',
  dev: '开发',
}

function clampMaxDepth(value: number): number {
  const normalized = Number.isFinite(value) ? Math.round(value) : DEFAULT_MAX_DEPTH
  return Math.min(MAX_DEPTH_RANGE.max, Math.max(MAX_DEPTH_RANGE.min, normalized))
}

function countKinds(items: string[]): TopologyKindCount[] {
  const counter = new Map<string, number>()
  for (const item of items) {
    counter.set(item, (counter.get(item) || 0) + 1)
  }
  return Array.from(counter.entries())
    .map(([kind, count]) => ({ kind, count }))
    .sort((left, right) => left.kind.localeCompare(right.kind))
}

function resolveGroupKind(nodeType: TopologyNodeType, raw?: string): TopologyGroupKind {
  if (raw === 'application_service' || raw === 'middleware' || raw === 'nginx') return raw
  if (nodeType === 'middleware') return 'middleware'
  if (nodeType === 'nginx') return 'nginx'
  return 'application_service'
}

function normalizeNode(node: TopologyV3Node): TopologyNode {
  const nodeType: TopologyNodeType = node.nodeType || node.node_type || 'application'
  const groupKind = resolveGroupKind(nodeType, node.groupKind || node.group_kind)
  const importance = Number.isFinite(node.importance) ? Number(node.importance) : 1
  const isExternal = node.isExternal === true || node.is_external === true
  const env = node.env || 'prod'
  const extra = {
    ...(node.extra || {}),
    ...(node.meta || {}),
  }

  return {
    id: node.id,
    name: node.name,
    node_type: nodeType,
    env,
    group_kind: groupKind,
    host_id: node.hostId || node.host_id,
    host_name: node.hostName || node.host_name,
    host_ip_display: node.hostIpDisplay || node.host_ip_display,
    status: node.status,
    importance,
    extra,
    is_external: isExternal || undefined,
    external_ref_id: node.externalRefId || node.external_ref_id,
  }
}

function normalizeEdge(edge: TopologyV3Edge): TopologyEdge {
  const edgeType: TopologyEdge['edge_type'] = edge.edgeType || edge.edge_type || 'http_call'
  const strength = Number.isFinite(edge.strength) ? Number(edge.strength) : 1

  return {
    id: edge.id,
    source: edge.source,
    target: edge.target,
    edge_type: edgeType,
    label: edge.label,
    strength,
    cross_env: edge.crossEnv === true || edge.cross_env === true,
  }
}

function buildLanes(nodes: TopologyNode[], sourceLanes: TopologyLane[] = []): TopologyLane[] {
  const laneMap = new Map(sourceLanes.map((lane) => [lane.id, lane]))
  return TOPOLOGY_ENV_ORDER.map((env, index) => {
    const inEnv = nodes.filter((node) => node.env === env)
    const lane = laneMap.get(env)

    return {
      id: env,
      label: lane?.label || TOPOLOGY_ENV_LABELS[env],
      order: lane?.order ?? index,
      node_count: inEnv.length,
      app_count: inEnv.filter((node) => node.group_kind === 'application_service').length,
    }
  })
}

function buildLegendStats(nodes: TopologyNode[], edges: TopologyEdge[], env?: TopologyEnv): TopologyLegendStats {
  return {
    env_counts: TOPOLOGY_ENV_ORDER.map((laneEnv) => {
      const inEnv = nodes.filter((node) => node.env === laneEnv)
      return {
        env: laneEnv,
        count: inEnv.length,
        app_count: inEnv.filter((node) => node.group_kind === 'application_service').length,
      }
    }),
    node_type_counts: countKinds(nodes.map((node) => node.node_type)),
    edge_type_counts: countKinds(edges.map((edge) => edge.edge_type)),
    application_service_count: nodes.filter((node) => node.group_kind === 'application_service').length,
    current_env: env,
    external_node_count: nodes.filter((node) => node.is_external === true).length,
    cross_env_edge_count: edges.filter((edge) => edge.cross_env).length,
  }
}

function normalizeSnapshot(snapshot: TopologyV3SnapshotResponse): TopologyGraph {
  const nodes = (snapshot.nodes || []).map(normalizeNode)
  const edges = (snapshot.edges || []).map(normalizeEdge)
  const legendStats = snapshot.legendStats || snapshot.legend_stats || buildLegendStats(nodes, edges, snapshot.meta?.env)
  const layoutHints = snapshot.layoutHints || snapshot.layout_hints || {
    lane_order: TOPOLOGY_ENV_ORDER,
    default_collapsed_groups: [],
    high_density_mode: edges.length > 260,
  }

  return {
    lanes: buildLanes(nodes, snapshot.lanes),
    nodes,
    edges,
    legend_stats: legendStats,
    layout_hints: layoutHints,
  }
}

function normalizeTaskInsight(insight: TopologyV3TaskInsight): TopologyV3TaskInsight {
  return {
    ...insight,
    nodeIds: insight.nodeIds || insight.node_ids || [],
    edgeIds: insight.edgeIds || insight.edge_ids || [],
  }
}

export const useTopologyStore = defineStore('topology', () => {
  const snapshotV3 = ref<TopologyV3SnapshotResponse | null>(null)
  const graphData = ref<TopologyGraph | null>(null)
  const taskInsights = ref<TopologyV3TaskInsight[]>([])
  const lastFetchTime = ref<number>(0)
  const loading = ref(false)
  const taskView = ref<TopologyTaskViewMode>(DEFAULT_TASK_VIEW)
  const maxDepth = ref<number>(DEFAULT_MAX_DEPTH)

  // Index node names for fast search
  const nodeNameIndex = computed(() => {
    const map = new Map<string, string[]>()
    if (!graphData.value) return map

    for (const node of graphData.value.nodes) {
      const words = node.name.toLowerCase().split(/[\s\-_./]+/)
      for (const word of words) {
        if (!word) continue
        const ids = map.get(word) || []
        ids.push(node.id)
        map.set(word, ids)
      }
    }
    return map
  })

  async function loadSnapshotV3(): Promise<TopologyV3SnapshotResponse> {
    const query = {
      taskView: taskView.value,
      maxDepth: maxDepth.value,
    }

    if (taskView.value === 'explore') {
      taskInsights.value = []
      return getTopologySnapshotV3(query)
    }

    const response = await getTopologyTaskViewV3(query)
    taskInsights.value = (response.insights || []).map(normalizeTaskInsight)
    if (response.snapshot) return response.snapshot
    taskInsights.value = []
    return getTopologySnapshotV3(query)
  }

  async function fetchGraph(forceRefresh = false): Promise<TopologyGraph | null> {
    const now = Date.now()
    if (!forceRefresh && graphData.value && now - lastFetchTime.value < CACHE_TTL_MS) {
      return graphData.value
    }

    loading.value = true
    try {
      snapshotV3.value = await loadSnapshotV3()
      graphData.value = normalizeSnapshot(snapshotV3.value)
      lastFetchTime.value = Date.now()
      return graphData.value
    } catch {
      taskInsights.value = []
      return null
    } finally {
      loading.value = false
    }
  }

  function setTaskView(nextTaskView: TopologyTaskViewMode) {
    if (taskView.value === nextTaskView) return
    taskView.value = nextTaskView
    invalidateCache()
  }

  function setMaxDepth(nextMaxDepth: number) {
    const clamped = clampMaxDepth(nextMaxDepth)
    if (maxDepth.value === clamped) return
    maxDepth.value = clamped
    invalidateCache()
  }

  function invalidateCache() {
    lastFetchTime.value = 0
  }

  return {
    snapshotV3,
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
  }
})
