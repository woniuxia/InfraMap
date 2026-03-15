import type {
  TopologyEdge,
  TopologyEnv,
  TopologyGraph,
  TopologyGroupKind,
  TopologyKindCount,
  TopologyLegendStats,
  TopologyNode,
  TopologyNodeType,
} from "@/types";
import { collectDirectionalReachability } from "@/components/topology/topologyDirectionalReachability.utils";

export type TopologyEdgeType = NonNullable<TopologyEdge["edgeType"]>;

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
const UUID_LIKE_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const HOST_GENERATED_ID_PATTERN = /^host-\d{8,}-[0-9a-f]{6,}$/i;

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
  return extra?.isExternal === true || extra?.isExternal === "true";
}

export function isExternalTopologyNode(
  node: Pick<TopologyNode, "id" | "isExternal" | "extra">,
): boolean {
  return (
    node.isExternal === true ||
    node.id.startsWith(EXTERNAL_NODE_PREFIX) ||
    isExternalByExtra(node.extra)
  );
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
    if ((left.importance ?? 0) !== (right.importance ?? 0))
      return (right.importance ?? 0) - (left.importance ?? 0);
    return left.name.localeCompare(right.name);
  });
}

function createExternalNode(nodeId: string, sourceNode: TopologyNode | undefined): TopologyNode {
  if (sourceNode) {
    return {
      ...sourceNode,
      id: toExternalNodeId(sourceNode.id),
      hostId: undefined,
      name: `${sourceNode.name} (${TOPOLOGY_ENV_LABELS[sourceNode.env]})`,
      isExternal: true,
      externalRefId: sourceNode.id,
      extra: {
        ...(sourceNode.extra || {}),
        isExternal: true,
        externalRefId: sourceNode.id,
        external_env: sourceNode.env,
      },
    };
  }

  return {
    id: toExternalNodeId(nodeId),
    name: `外部节点 ${nodeId}`,
    nodeType: "service",
    env: "test",
    groupKind: "service",
    importance: 0.5,
    isExternal: true,
    externalRefId: nodeId,
    extra: {
      isExternal: true,
      externalRefId: nodeId,
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

  return sortNodes(
    nodes.filter((node) => {
      if (!isExternalTopologyNode(node)) return true;
      return linkedNodeIds.has(node.id);
    }),
  );
}

function buildLanes(
  sourceLanes: TopologyGraph["lanes"],
  nodes: TopologyNode[],
): TopologyGraph["lanes"] {
  const laneMap = new Map(sourceLanes.map((lane) => [lane.id, lane]));
  return TOPOLOGY_ENV_ORDER.map((env, order) => {
    const sameEnvNodes = nodes.filter((node) => node.env === env);
    const appCount = sameEnvNodes.filter((node) => node.groupKind === "service").length;
    const lane = laneMap.get(env);
    return {
      id: env,
      label: lane?.label || TOPOLOGY_ENV_LABELS[env],
      order: lane?.order ?? order,
      nodeCount: sameEnvNodes.length,
      appCount: appCount,
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
    const appCount = inEnv.filter((node) => node.groupKind === "service").length;
    return {
      env,
      count: inEnv.length,
      appCount: appCount,
    };
  });

  const nodeTypeCounts = countKinds(
    nodes.map((node) => node.nodeType).filter((t): t is TopologyNodeType => t != null),
  );

  const edgeTypeMap = new Map<string, number>();
  for (const edge of edges) {
    const edgeType = edge.edgeType;
    if (!edgeType) continue;
    edgeTypeMap.set(edgeType, (edgeTypeMap.get(edgeType) || 0) + Math.max(1, edge.strength ?? 1));
  }
  const edgeTypeCounts = Array.from(edgeTypeMap.entries())
    .map(([kind, count]) => ({ kind, count }))
    .sort((a, b) => a.kind.localeCompare(b.kind));

  const externalNodeCount = nodes.filter((node) => isExternalTopologyNode(node)).length;
  const crossEnvEdgeCount = edges.filter((edge) => edge.crossEnv).length;

  return {
    envCounts: envCounts,
    nodeTypeCounts: nodeTypeCounts,
    edgeTypeCounts: edgeTypeCounts,
    serviceCount: nodes.filter((node) => node.groupKind === "service").length,
    currentEnv: currentEnv,
    externalNodeCount: externalNodeCount,
    crossEnvEdgeCount: crossEnvEdgeCount,
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
      crossEnv: edge.crossEnv || isExternalNodeId(nextSource) || isExternalNodeId(nextTarget),
    });
  }

  const nodeKindSet = new Set(filter.nodeKinds);
  const edgeTypeSet = new Set(filter.edgeTypes);

  let visibleNodes = transformedNodes.filter((node) => {
    if (nodeKindSet.size > 0 && (!node.groupKind || !nodeKindSet.has(node.groupKind))) return false;
    return true;
  });
  let visibleNodeIds = new Set(visibleNodes.map((node) => node.id));

  let visibleEdges = transformedEdges.filter((edge) => {
    if (!visibleNodeIds.has(edge.source) || !visibleNodeIds.has(edge.target)) return false;
    if (edgeTypeSet.size > 0 && (!edge.edgeType || !edgeTypeSet.has(edge.edgeType))) return false;
    return true;
  });

  if (!filter.showAllEdges && visibleEdges.length > EDGE_RENDER_LIMIT) {
    visibleEdges = [...visibleEdges]
      .sort((left, right) => {
        if ((left.crossEnv ?? false) !== (right.crossEnv ?? false)) return left.crossEnv ? -1 : 1;
        if ((left.strength ?? 0) !== (right.strength ?? 0))
          return (right.strength ?? 0) - (left.strength ?? 0);
        return left.id.localeCompare(right.id);
      })
      .slice(0, EDGE_RENDER_LIMIT);
  }

  visibleNodes = trimDetachedExternalNodes(visibleNodes, visibleEdges);
  visibleNodeIds = new Set(visibleNodes.map((node) => node.id));
  visibleEdges = visibleEdges.filter(
    (edge) => visibleNodeIds.has(edge.source) && visibleNodeIds.has(edge.target),
  );

  const legendStats = computeLegendStats(visibleNodes, visibleEdges, filter.env);
  const lanes = buildLanes(graph.lanes, visibleNodes);

  return {
    lanes,
    nodes: visibleNodes,
    edges: visibleEdges,
    legendStats: legendStats,
    layoutHints: graph.layoutHints,
  };
}

export function compactLabel(value: string, maxLength = 16): string {
  if (value.length <= maxLength) return value;
  return `${value.slice(0, Math.max(1, maxLength - 1))}…`;
}

export function isOpaqueIdentifier(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed) return true;
  if (UUID_LIKE_PATTERN.test(trimmed) || HOST_GENERATED_ID_PATTERN.test(trimmed)) return true;
  const normalized = trimmed.replace(/[-_]/g, "");
  const digitCount = (normalized.match(/\d/g) || []).length;
  const letterCount = (normalized.match(/[a-z]/gi) || []).length;
  const hasChinese = /[\u4e00-\u9fa5]/.test(trimmed);
  return !hasChinese && normalized.length >= 18 && digitCount >= 8 && letterCount >= 4;
}

function asNonEmptyString(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function pickHostName(nodes: TopologyNode[]): string | null {
  for (const node of nodes) {
    const direct = asNonEmptyString(node.hostName);
    if (direct) return direct;

    const extra = node.extra;
    if (!extra) continue;
    const candidate =
      asNonEmptyString(extra.hostName) ||
      asNonEmptyString(extra.host_hostname) ||
      asNonEmptyString(extra.hostname) ||
      asNonEmptyString(extra.host_display);
    if (candidate) return candidate;
  }
  return null;
}

function pickHostIpDisplay(nodes: TopologyNode[]): string | null {
  for (const node of nodes) {
    const direct = asNonEmptyString(node.hostIpDisplay);
    if (direct) return direct;

    const extra = node.extra;
    if (!extra) continue;
    const candidate = asNonEmptyString(extra.hostIpDisplay) || asNonEmptyString(extra.ip_display);
    if (candidate) return candidate;
  }
  return null;
}

function formatHostIpSummary(ipDisplay: string | null): string | null {
  if (!ipDisplay) return null;
  const items = ipDisplay
    .split(/[，,]/)
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
  if (items.length === 0) return null;
  if (items.length === 1) return items[0];
  return `${items[0]} +${items.length - 1}`;
}

export function formatHostDisplayName(
  hostName: string | null,
  hostIpDisplay: string | null,
): string | null {
  const ipSummary = formatHostIpSummary(hostIpDisplay);
  if (hostName && ipSummary) return `${hostName} · ${ipSummary}`;
  if (hostName) return hostName;
  if (ipSummary) return ipSummary;
  return null;
}

export function formatHostComboLabel(hostId: string, hostNodes: TopologyNode[]): string {
  const hostName = pickHostName(hostNodes);
  const hostIpDisplay = pickHostIpDisplay(hostNodes);
  const displayName = formatHostDisplayName(hostName, hostIpDisplay);
  const baseLabel =
    displayName || (isOpaqueIdentifier(hostId) ? `主机 #${hostId.slice(-6)}` : hostId);
  return compactLabel(baseLabel, 22);
}

export function formatSystemComboLabel(systemId: string, systemNodes: TopologyNode[]): string {
  for (const node of systemNodes) {
    if (node.systemName) return compactLabel(node.systemName, 22);
  }
  return compactLabel(isOpaqueIdentifier(systemId) ? `系统 #${systemId.slice(-6)}` : systemId, 22);
}

export function filterFocusSubgraph(
  graph: TopologyGraph,
  focusNodeIds: string[],
): TopologyGraph | null {
  if (focusNodeIds.length === 0) return null;

  const allHighlightedNodeIds = new Set<string>();
  const allHighlightedEdgeIds = new Set<string>();

  // 对每个焦点节点收集可达性
  for (const focusNodeId of focusNodeIds) {
    const focusNode = graph.nodes.find((node) => node.id === focusNodeId);
    if (!focusNode) continue;

    const { highlightedNodeIds, highlightedEdgeIds } = collectDirectionalReachability(
      focusNodeId,
      graph.edges,
    );

    highlightedNodeIds.forEach((id) => allHighlightedNodeIds.add(id));
    highlightedEdgeIds.forEach((id) => allHighlightedEdgeIds.add(id));
  }

  if (allHighlightedNodeIds.size === 0) return null;

  const nodes = sortNodes(graph.nodes.filter((node) => allHighlightedNodeIds.has(node.id)));
  const edges = graph.edges.filter((edge) => allHighlightedEdgeIds.has(edge.id));

  // 使用第一个焦点节点的环境作为参考
  const firstFocusNode = graph.nodes.find((node) => node.id === focusNodeIds[0]);
  const env = firstFocusNode?.env || "prod";

  const legendStats = computeLegendStats(nodes, edges, env);
  const lanes = buildLanes(graph.lanes, nodes);

  return {
    lanes,
    nodes,
    edges,
    legendStats,
    layoutHints: graph.layoutHints,
  };
}

// 保留旧函数名以兼容现有代码
export function filterDagreFocusSubgraph(
  graph: TopologyGraph,
  focusNodeId: string,
): TopologyGraph | null {
  return filterFocusSubgraph(graph, [focusNodeId]);
}
