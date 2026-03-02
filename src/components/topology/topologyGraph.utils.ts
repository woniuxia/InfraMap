import type { ComboData, EdgeData, GraphData, NodeData } from "@antv/g6";
import type {
  TopologyEdge,
  TopologyEnv,
  TopologyGraph,
  TopologyGroupKind,
  TopologyKindCount,
  TopologyLegendStats,
  TopologyNode,
} from "@/types";

export type TopologyEdgeType = TopologyEdge["edge_type"];

export interface TopologyFilterState {
  env: TopologyEnv;
  nodeKinds: TopologyGroupKind[];
  edgeTypes: TopologyEdgeType[];
  showAllEdges: boolean;
}

export const TOPOLOGY_ENV_ORDER: TopologyEnv[] = ["prod", "test", "dev"];

export const TOPOLOGY_ENV_LABELS: Record<TopologyEnv, string> = {
  prod: "生产",
  test: "测试",
  dev: "开发",
};

export const DEFAULT_TOPOLOGY_FILTER: TopologyFilterState = {
  env: "prod",
  nodeKinds: [],
  edgeTypes: [],
  showAllEdges: false,
};

export const EDGE_RENDER_LIMIT = 260;
export const EXTERNAL_NODE_PREFIX = "external:";
export const EXTERNAL_ZONE_COMBO_ID = "external-zone";

function countKinds(items: string[]): TopologyKindCount[] {
  const map = new Map<string, number>();
  for (const item of items) {
    map.set(item, (map.get(item) || 0) + 1);
  }
  return Array.from(map.entries())
    .map(([kind, count]) => ({ kind, count }))
    .sort((a, b) => a.kind.localeCompare(b.kind));
}

function isExternalByExtra(extra: Record<string, unknown> | undefined): boolean {
  return extra?.is_external === true || extra?.is_external === "true";
}

export function isExternalTopologyNode(node: Pick<TopologyNode, "id" | "is_external" | "extra">): boolean {
  return node.is_external === true || node.id.startsWith(EXTERNAL_NODE_PREFIX) || isExternalByExtra(node.extra);
}

export function isExternalNodeId(nodeId: string): boolean {
  return nodeId.startsWith(EXTERNAL_NODE_PREFIX);
}

export function toExternalNodeId(nodeId: string): string {
  return isExternalNodeId(nodeId) ? nodeId : `${EXTERNAL_NODE_PREFIX}${nodeId}`;
}

export function fromExternalNodeId(nodeId: string): string {
  return isExternalNodeId(nodeId) ? nodeId.slice(EXTERNAL_NODE_PREFIX.length) : nodeId;
}

function sortNodes(nodes: TopologyNode[]): TopologyNode[] {
  return [...nodes].sort((left, right) => {
    if (left.importance !== right.importance) return right.importance - left.importance;
    return left.name.localeCompare(right.name);
  });
}

function createExternalNode(
  nodeId: string,
  sourceNode: TopologyNode | undefined,
): TopologyNode {
  if (sourceNode) {
    return {
      ...sourceNode,
      id: toExternalNodeId(sourceNode.id),
      host_id: undefined,
      name: `${sourceNode.name} (${TOPOLOGY_ENV_LABELS[sourceNode.env]})`,
      is_external: true,
      external_ref_id: sourceNode.id,
      extra: {
        ...(sourceNode.extra || {}),
        is_external: true,
        external_ref_id: sourceNode.id,
        external_env: sourceNode.env,
      },
    };
  }

  return {
    id: toExternalNodeId(nodeId),
    name: `外部节点 ${nodeId}`,
    node_type: "application",
    env: "test",
    group_kind: "application_service",
    importance: 0.5,
    is_external: true,
    external_ref_id: nodeId,
    extra: {
      is_external: true,
      external_ref_id: nodeId,
      external_env: "unknown",
    },
  };
}

function trimDetachedExternalNodes(nodes: TopologyNode[], edges: TopologyEdge[]): TopologyNode[] {
  const linkedNodeIds = new Set<string>();
  for (const edge of edges) {
    linkedNodeIds.add(edge.source);
    linkedNodeIds.add(edge.target);
  }

  return sortNodes(nodes.filter((node) => {
    if (!isExternalTopologyNode(node)) return true;
    return linkedNodeIds.has(node.id);
  }));
}

function buildLanes(
  sourceLanes: TopologyGraph["lanes"],
  nodes: TopologyNode[],
): TopologyGraph["lanes"] {
  const laneMap = new Map(sourceLanes.map((lane) => [lane.id, lane]));
  return TOPOLOGY_ENV_ORDER.map((env, order) => {
    const sameEnvNodes = nodes.filter((node) => node.env === env);
    const appCount = sameEnvNodes.filter((node) => node.group_kind === "application_service").length;
    const lane = laneMap.get(env);
    return {
      id: env,
      label: lane?.label || TOPOLOGY_ENV_LABELS[env],
      order: lane?.order ?? order,
      node_count: sameEnvNodes.length,
      app_count: appCount,
    };
  });
}

export function computeLegendStats(
  nodes: TopologyNode[],
  edges: TopologyEdge[],
  currentEnv?: TopologyEnv,
): TopologyLegendStats {
  const envCounts = TOPOLOGY_ENV_ORDER.map((env) => {
    const inEnv = nodes.filter((node) => node.env === env);
    const appCount = inEnv.filter((node) => node.group_kind === "application_service").length;
    return {
      env,
      count: inEnv.length,
      app_count: appCount,
    };
  });

  const nodeTypeCounts = countKinds(nodes.map((node) => node.node_type));

  const edgeTypeMap = new Map<string, number>();
  for (const edge of edges) {
    edgeTypeMap.set(edge.edge_type, (edgeTypeMap.get(edge.edge_type) || 0) + Math.max(1, edge.strength));
  }
  const edgeTypeCounts = Array.from(edgeTypeMap.entries())
    .map(([kind, count]) => ({ kind, count }))
    .sort((a, b) => a.kind.localeCompare(b.kind));

  const externalNodeCount = nodes.filter((node) => isExternalTopologyNode(node)).length;
  const crossEnvEdgeCount = edges.filter((edge) => edge.cross_env).length;

  return {
    env_counts: envCounts,
    node_type_counts: nodeTypeCounts,
    edge_type_counts: edgeTypeCounts,
    application_service_count: nodes.filter((node) => node.group_kind === "application_service").length,
    current_env: currentEnv,
    external_node_count: externalNodeCount,
    cross_env_edge_count: crossEnvEdgeCount,
  };
}

export function filterTopologyGraph(
  graph: TopologyGraph | null,
  filter: TopologyFilterState,
): TopologyGraph | null {
  if (!graph) return null;

  const nodeById = new Map(graph.nodes.map((node) => [node.id, node]));
  const selectedEnvNodes = sortNodes(graph.nodes.filter((node) => node.env === filter.env));
  const selectedEnvNodeIds = new Set(selectedEnvNodes.map((node) => node.id));

  const transformedNodes = [...selectedEnvNodes];
  const transformedNodeIds = new Set(transformedNodes.map((node) => node.id));
  const transformedEdges: TopologyEdge[] = [];

  const ensureExternalNode = (originalNodeId: string): string => {
    const externalId = toExternalNodeId(originalNodeId);
    if (!transformedNodeIds.has(externalId)) {
      transformedNodes.push(createExternalNode(originalNodeId, nodeById.get(originalNodeId)));
      transformedNodeIds.add(externalId);
    }
    return externalId;
  };

  for (const edge of graph.edges) {
    const sourceInEnv = selectedEnvNodeIds.has(edge.source);
    const targetInEnv = selectedEnvNodeIds.has(edge.target);
    if (!sourceInEnv && !targetInEnv) continue;

    const nextSource = sourceInEnv ? edge.source : ensureExternalNode(edge.source);
    const nextTarget = targetInEnv ? edge.target : ensureExternalNode(edge.target);

    transformedEdges.push({
      ...edge,
      source: nextSource,
      target: nextTarget,
      cross_env: edge.cross_env || isExternalNodeId(nextSource) || isExternalNodeId(nextTarget),
    });
  }

  const nodeKindSet = new Set(filter.nodeKinds);
  const edgeTypeSet = new Set(filter.edgeTypes);

  let visibleNodes = transformedNodes.filter((node) => {
    if (nodeKindSet.size > 0 && !nodeKindSet.has(node.group_kind)) return false;
    return true;
  });
  let visibleNodeIds = new Set(visibleNodes.map((node) => node.id));

  let visibleEdges = transformedEdges.filter((edge) => {
    if (!visibleNodeIds.has(edge.source) || !visibleNodeIds.has(edge.target)) return false;
    if (edgeTypeSet.size > 0 && !edgeTypeSet.has(edge.edge_type)) return false;
    return true;
  });

  if (!filter.showAllEdges && visibleEdges.length > EDGE_RENDER_LIMIT) {
    visibleEdges = [...visibleEdges]
      .sort((left, right) => {
        if (left.cross_env !== right.cross_env) return left.cross_env ? -1 : 1;
        if (left.strength !== right.strength) return right.strength - left.strength;
        return left.id.localeCompare(right.id);
      })
      .slice(0, EDGE_RENDER_LIMIT);
  }

  visibleNodes = trimDetachedExternalNodes(visibleNodes, visibleEdges);
  visibleNodeIds = new Set(visibleNodes.map((node) => node.id));
  visibleEdges = visibleEdges.filter((edge) => visibleNodeIds.has(edge.source) && visibleNodeIds.has(edge.target));

  const legendStats = computeLegendStats(visibleNodes, visibleEdges, filter.env);
  const lanes = buildLanes(graph.lanes, visibleNodes);

  return {
    lanes,
    nodes: visibleNodes,
    edges: visibleEdges,
    legend_stats: legendStats,
    layout_hints: graph.layout_hints,
  };
}

function hostComboId(hostId: string): string {
  return `host-${hostId}`;
}

export function buildTopologyG6Data(graph: TopologyGraph): GraphData {
  const hostCombos: ComboData[] = [];
  const hostSeen = new Set<string>();
  for (const node of graph.nodes) {
    if (!node.host_id || isExternalTopologyNode(node) || hostSeen.has(node.host_id)) continue;
    hostSeen.add(node.host_id);
    hostCombos.push({
      id: hostComboId(node.host_id),
      type: "rect",
      data: {
        kind: "host",
        host_id: node.host_id,
        label: node.host_id,
      },
    });
  }

  const hasExternalNodes = graph.nodes.some((node) => isExternalTopologyNode(node));
  const externalCombo: ComboData[] = hasExternalNodes
    ? [{
      id: EXTERNAL_ZONE_COMBO_ID,
      type: "rect",
      data: {
        kind: "external",
        label: "跨环境依赖",
      },
    }]
    : [];

  const nodes: NodeData[] = graph.nodes.map((node) => {
    let combo: string | undefined;
    if (isExternalTopologyNode(node)) {
      combo = EXTERNAL_ZONE_COMBO_ID;
    } else if (node.host_id) {
      combo = hostComboId(node.host_id);
    }

    return {
      id: node.id,
      combo,
      data: {
        name: node.name,
        node_type: node.node_type,
        group_kind: node.group_kind,
        status: node.status,
        env: node.env,
        host_id: node.host_id,
        importance: node.importance,
        is_external: isExternalTopologyNode(node),
        external_ref_id: node.external_ref_id || node.extra?.external_ref_id,
        extra: node.extra,
      },
    };
  });

  const edges: EdgeData[] = graph.edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    data: {
      edge_type: edge.edge_type,
      label: edge.label,
      strength: edge.strength,
      cross_env: edge.cross_env,
    },
  }));

  return {
    nodes,
    edges,
    combos: [...hostCombos, ...externalCombo],
  };
}
