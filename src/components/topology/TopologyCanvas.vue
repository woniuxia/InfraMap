<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from "vue";
import { Graph } from "@antv/g6";
import type { ComboData, EdgeData, GraphData, IElementEvent, NodeData } from "@antv/g6";
import type { TopologyGraph, TopologyNode } from "@/types";
import {
  buildTopologyG6Data,
  compactLabel,
  isOpaqueIdentifier,
  toExternalNodeId,
} from "@/components/topology/topologyGraph.utils";
import { getMiddlewareIconByType } from "@/utils/middlewareCatalog";

const props = defineProps<{
  graphData: TopologyGraph | null;
}>();

const emit = defineEmits<{
  (e: "node-click", node: TopologyNode): void;
  (e: "node-contextmenu", payload: { node: TopologyNode; x: number; y: number }): void;
}>();

const containerRef = ref<HTMLDivElement>();
const activeLayout = ref<"force" | "dagre">("dagre");

let graph: Graph | null = null;
let resizeObserver: ResizeObserver | null = null;
let themeObserver: MutationObserver | null = null;

interface GraphTheme {
  statusColors: Record<string, string>;
  envColors: Record<string, string>;
  edgeStyles: Record<string, { stroke: string; lineDash?: number[] }>;
  labelPrimary: string;
  labelSecondary: string;
  labelMuted: string;
  labelBg: string;
  highlight: string;
  impact: string;
}

function cssVar(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

function withAlpha(hex: string, alphaHex: string): string {
  if (!hex.startsWith("#") || (hex.length !== 7 && hex.length !== 4)) return hex;
  if (hex.length === 4) {
    const r = hex[1];
    const g = hex[2];
    const b = hex[3];
    return `#${r}${r}${g}${g}${b}${b}${alphaHex}`;
  }
  return `${hex}${alphaHex}`;
}

function buildGraphTheme(): GraphTheme {
  const accent = cssVar("--im-accent", "#5ca3ff");
  const success = cssVar("--im-success", "#41c58a");
  const warning = cssVar("--im-warning", "#f2b645");
  const danger = cssVar("--im-danger", "#ef6b73");
  const textPrimary = cssVar("--im-text-primary", "#e6eefc");
  const textSecondary = cssVar("--im-text-secondary", "#93a4c4");
  const textMuted = cssVar("--im-text-muted", "#6f7f9c");
  const surface = cssVar("--im-surface-0", "#0f1728");
  return {
    statusColors: {
      running: success,
      stopped: textMuted,
      maintenance: warning,
    },
    envColors: {
      prod: cssVar("--im-danger", "#ef6b73"),
      test: cssVar("--im-accent", "#5ca3ff"),
      dev: cssVar("--im-success", "#41c58a"),
    },
    edgeStyles: {
      http_call: { stroke: accent },
      tcp: { stroke: cssVar("--el-color-info", "#5ca3ff") },
      mq_produce: { stroke: warning, lineDash: [4, 4] },
      mq_consume: { stroke: warning, lineDash: [4, 4] },
      grpc_call: { stroke: cssVar("--el-color-primary", "#409eff"), lineDash: [6, 3] },
      db_query: { stroke: cssVar("--el-color-success", "#67c23a") },
      cache_access: { stroke: cssVar("--el-color-warning", "#e6a23c"), lineDash: [2, 4] },
    },
    labelPrimary: textPrimary,
    labelSecondary: textSecondary,
    labelMuted: textMuted,
    labelBg: surface,
    highlight: accent,
    impact: danger,
  };
}

function isExternalDatum(datum: NodeData): boolean {
  return datum.data?.is_external === true;
}

function getNodeType(datum: NodeData): string {
  const nodeType = datum.data?.node_type as string;
  if (nodeType === "middleware") return "image";
  if (nodeType === "nginx") return "hexagon";
  return "circle";
}

function humanizeOpaqueLabel(value: string): string {
  if (!isOpaqueIdentifier(value)) return value;
  const suffix = value.slice(-6);
  return `节点 #${suffix}`;
}

function nodeLabel(datum: NodeData, isLargeGraph: boolean): string {
  const raw = ((datum.data?.name as string) || `${datum.id || ""}`).trim() || "未命名节点";
  const base = humanizeOpaqueLabel(raw);
  return compactLabel(base, isLargeGraph ? 10 : 14);
}

function getNodeStyle(datum: NodeData, isLargeGraph: boolean): Record<string, unknown> {
  const theme = buildGraphTheme();
  const nodeType = datum.data?.node_type as string;
  const groupKind = datum.data?.group_kind as string;
  const env = (datum.data?.env as string) || "prod";
  const external = isExternalDatum(datum);
  const labelText = nodeLabel(datum, isLargeGraph);

  const size = isLargeGraph ? 18 : groupKind === "application_service" ? 34 : 28;
  const labelFontSize = isLargeGraph ? 9 : groupKind === "application_service" ? 11 : 10;
  const envColor = theme.envColors[env] || theme.envColors.prod;

  if (nodeType === "middleware") {
    const extra = (datum.data?.extra as Record<string, unknown> | undefined) ?? {};
    const category = typeof extra.category === "string" ? extra.category : undefined;
    const middlewareType = typeof extra.type === "string" ? extra.type : undefined;
    const icon = getMiddlewareIconByType(middlewareType, category);
    return {
      img: icon.src,
      src: icon.src,
      size,
      opacity: external ? 0.78 : 1,
      labelText,
      labelPlacement: "bottom",
      labelFontSize,
      labelFill: external ? theme.labelSecondary : theme.labelPrimary,
      labelOffsetY: 5,
      lineWidth: 1.5,
      stroke: external ? withAlpha(envColor, "88") : withAlpha(envColor, "DD"),
      lineDash: external ? [4, 4] : [],
    };
  }

  const status = datum.data?.status as string;
  const baseFill = nodeType === "application"
    ? withAlpha(envColor, external ? "66" : "CC")
    : theme.statusColors[status] || theme.labelMuted;

  return {
    fill: baseFill,
    stroke: external ? withAlpha(envColor, "88") : withAlpha(envColor, "EE"),
    lineWidth: groupKind === "application_service" ? 2.6 : 1.8,
    lineDash: external ? [4, 4] : [],
    opacity: external ? 0.82 : 1,
    labelText,
    labelPlacement: "bottom",
    labelFontSize,
    labelFill: external ? theme.labelSecondary : theme.labelPrimary,
    labelOffsetY: 4,
    size,
  };
}

function getEdgeStyle(datum: EdgeData, hideLabels: boolean): Record<string, unknown> {
  const theme = buildGraphTheme();
  const edgeType = datum.data?.edge_type as string;
  const edgeConf = theme.edgeStyles[edgeType] || theme.edgeStyles.http_call;
  const strength = Number((datum.data?.strength as number) || 1);
  const crossEnv = Boolean(datum.data?.cross_env);

  return {
    stroke: crossEnv ? theme.impact : edgeConf.stroke,
    lineWidth: Math.min(4, 1 + strength * 0.45),
    lineDash: crossEnv ? [6, 4] : (edgeConf.lineDash || []),
    endArrow: true,
    endArrowSize: 8,
    labelText: hideLabels ? "" : ((datum.data?.label as string) || ""),
    labelFontSize: 10,
    labelFill: theme.labelSecondary,
    labelBackground: true,
    labelBackgroundFill: withAlpha(theme.labelBg, "E6"),
    labelBackgroundOpacity: 0.85,
    labelPadding: [2, 4],
  };
}

function getComboStyle(datum: ComboData): Record<string, unknown> {
  const theme = buildGraphTheme();
  const kind = datum.data?.kind as string;

  if (kind === "external") {
    return {
      fill: withAlpha(theme.impact, "10"),
      stroke: withAlpha(theme.impact, "B8"),
      lineWidth: 1.4,
      lineDash: [8, 4],
      radius: 12,
      labelText: (datum.data?.label as string) || "跨环境依赖",
      labelPlacement: "top-left",
      labelOffsetX: 8,
      labelOffsetY: 8,
      labelFontSize: 12,
      labelFill: theme.impact,
      padding: [20, 12, 12, 12],
    };
  }

  const hostLabel = compactLabel(((datum.data?.label as string) || datum.id), 18);
  const nodeCount = Number((datum.data?.node_count as number) || 0);
  const labelText = nodeCount > 1 ? `${hostLabel} (${nodeCount})` : hostLabel;

  return {
    fill: withAlpha(theme.labelBg, "16"),
    stroke: withAlpha(theme.labelMuted, "AA"),
    lineWidth: 1.2,
    lineDash: [5, 3],
    collapsedSize: [140, 48],
    labelText,
    labelFontSize: 11,
    labelFill: theme.labelSecondary,
    labelPlacement: "top",
    labelOffsetY: 4,
    padding: [14, 10, 10, 10],
  };
}

function getLayoutConfig(g6Data: GraphData, layoutType: "force" | "dagre") {
  if (layoutType === "dagre") {
    return {
      type: "dagre" as const,
      rankdir: "LR" as const,
      nodesep: 72,
      ranksep: 180,
      controlPoints: true,
      sortByCombo: true,
    };
  }

  if ((g6Data.nodes?.length || 0) > 500) {
    return {
      type: "fruchterman" as const,
      maxIteration: 260,
      gravity: 4.8,
      speed: 5,
    };
  }

  return {
    type: "force" as const,
    preventOverlap: true,
    nodeSize: 40,
    linkDistance: (datum: EdgeData) => (datum.data?.cross_env ? 210 : 165),
    nodeStrength: (datum: NodeData) => {
      const groupKind = datum.data?.group_kind as string;
      const external = Boolean(datum.data?.is_external);
      if (external) return -40;
      return groupKind === "application_service" ? -95 : -78;
    },
    edgeStrength: (datum: EdgeData) => {
      const isCrossEnv = Boolean(datum.data?.cross_env);
      const strength = Number((datum.data?.strength as number) || 1);
      if (isCrossEnv) return Math.min(0.72, 0.34 + strength * 0.08);
      return Math.min(0.92, 0.55 + strength * 0.1);
    },
  };
}

function resolveRenderableNodeId(rawNodeId: string): string | null {
  const allNodeIds = new Set((props.graphData?.nodes || []).map((node) => node.id));
  if (allNodeIds.has(rawNodeId)) return rawNodeId;
  const externalId = toExternalNodeId(rawNodeId);
  if (allNodeIds.has(externalId)) return externalId;
  return null;
}

function initGraph() {
  if (!containerRef.value || !props.graphData) return;

  const container = containerRef.value;
  const { width, height } = container.getBoundingClientRect();
  const g6Data = buildTopologyG6Data(props.graphData);
  const hideEdgeLabels = props.graphData.layout_hints.high_density_mode || (g6Data.edges?.length || 0) > 90;
  const isLargeGraph = (g6Data.nodes?.length || 0) > 500;
  const theme = buildGraphTheme();

  if (graph) {
    graph.destroy();
    graph = null;
  }

  graph = new Graph({
    container,
    width: width || 800,
    height: height || 600,
    autoFit: "view",
    animation: false,
    data: g6Data,
    layout: getLayoutConfig(g6Data, activeLayout.value),
    node: {
      type: getNodeType,
      style: (datum: NodeData) => getNodeStyle(datum, isLargeGraph),
      state: {
        highlight: {
          lineWidth: 3,
          shadowColor: theme.highlight,
          shadowBlur: 10,
        },
        dim: {
          opacity: 0.2,
          labelOpacity: 0.35,
        },
        impact: {
          lineWidth: 3,
          shadowColor: theme.impact,
          shadowBlur: 10,
        },
      },
    },
    edge: {
      type: "line",
      style: (datum: EdgeData) => getEdgeStyle(datum, hideEdgeLabels),
      state: {
        highlight: {
          lineWidth: 3,
          shadowColor: theme.highlight,
          shadowBlur: 6,
        },
        dim: {
          opacity: 0.13,
        },
      },
    },
    combo: {
      type: "rect",
      style: getComboStyle,
      state: {
        highlight: {
          lineWidth: 2,
          shadowColor: theme.highlight,
          shadowBlur: 8,
        },
        dim: {
          opacity: 0.18,
        },
      },
    },
    behaviors: ["drag-canvas", "zoom-canvas", "drag-element"],
    transforms: ["process-parallel-edges"],
  });

  graph.on("node:click", (evt: IElementEvent) => {
    const id = evt.target?.id as string;
    if (!id) return;
    const node = props.graphData?.nodes.find((item) => item.id === id);
    if (node) emit("node-click", node);
  });

  graph.on("node:contextmenu", (evt: IElementEvent) => {
    const id = evt.target?.id as string;
    if (!id) return;
    const node = props.graphData?.nodes.find((item) => item.id === id);
    if (node) {
      emit("node-contextmenu", { node, x: evt.client.x, y: evt.client.y });
    }
  });

  graph.render();
}

watch(
  () => props.graphData,
  () => {
    nextTick(initGraph);
  },
);

onMounted(() => {
  nextTick(initGraph);

  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      if (graph && containerRef.value) {
        const { width, height } = containerRef.value.getBoundingClientRect();
        graph.resize(width, height);
      }
    });
    resizeObserver.observe(containerRef.value);
  }

  themeObserver = new MutationObserver((mutations) => {
    const changed = mutations.some(
      (mutation) => mutation.type === "attributes" && mutation.attributeName === "data-theme",
    );
    if (changed) nextTick(initGraph);
  });
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  themeObserver?.disconnect();
  themeObserver = null;
  if (graph) {
    graph.destroy();
    graph = null;
  }
});

function clearHighlight() {
  if (!graph) return;
  const allNodes = graph.getNodeData();
  const allEdges = graph.getEdgeData();
  const allCombos = graph.getComboData();

  allNodes.forEach((node) => graph!.setElementState(node.id, []));
  allEdges.forEach((edge) => {
    if (edge.id) graph!.setElementState(edge.id, []);
  });
  allCombos.forEach((combo) => graph!.setElementState(combo.id, []));
}

function highlightPaths(paths: string[][]) {
  if (!graph) return;
  clearHighlight();

  const nodeIds = new Set<string>();
  const edgeIds = new Set<string>();

  paths.forEach((path) => {
    path.forEach((rawId) => {
      const renderId = resolveRenderableNodeId(rawId);
      if (renderId) nodeIds.add(renderId);
    });

    for (let index = 0; index < path.length - 1; index += 1) {
      const source = resolveRenderableNodeId(path[index]);
      const target = resolveRenderableNodeId(path[index + 1]);
      if (!source || !target) continue;
      const edge = props.graphData?.edges.find((item) => item.source === source && item.target === target);
      if (edge) edgeIds.add(edge.id);
    }
  });

  graph.getNodeData().forEach((node) => {
    graph!.setElementState(node.id, nodeIds.has(node.id as string) ? "highlight" : "dim");
  });
  graph.getEdgeData().forEach((edge) => {
    if (!edge.id) return;
    graph!.setElementState(edge.id, edgeIds.has(edge.id as string) ? "highlight" : "dim");
  });
}

function highlightImpact(nodeId: string, result: { affected_nodes: { id: string; depth: number }[] }) {
  if (!graph) return;
  clearHighlight();

  const focusNodeId = resolveRenderableNodeId(nodeId) || nodeId;
  const affectedIds = new Set<string>([focusNodeId]);
  result.affected_nodes.forEach((item) => {
    const renderId = resolveRenderableNodeId(item.id);
    if (renderId) affectedIds.add(renderId);
  });

  graph.getNodeData().forEach((node) => {
    if (node.id === focusNodeId) {
      graph!.setElementState(node.id, "impact");
    } else if (affectedIds.has(node.id as string)) {
      graph!.setElementState(node.id, "highlight");
    } else {
      graph!.setElementState(node.id, "dim");
    }
  });

  graph.getEdgeData().forEach((edge) => {
    if (!edge.id) return;
    const sourceHit = affectedIds.has(edge.source as string);
    const targetHit = affectedIds.has(edge.target as string);
    graph!.setElementState(edge.id, sourceHit && targetHit ? "highlight" : "dim");
  });
}

function highlightSearch(nodeIds: string[], focusId?: string) {
  if (!graph) return;
  clearHighlight();

  const matchSet = new Set<string>();
  nodeIds.forEach((id) => {
    const renderId = resolveRenderableNodeId(id);
    if (renderId) matchSet.add(renderId);
  });

  graph.getNodeData().forEach((node) => {
    graph!.setElementState(node.id, matchSet.has(node.id as string) ? "highlight" : "dim");
  });

  if (focusId) {
    const renderFocusId = resolveRenderableNodeId(focusId);
    if (renderFocusId) graph.focusElement(renderFocusId, true);
  }
}

function setLayout(type: "force" | "dagre") {
  if (activeLayout.value === type) return;
  activeLayout.value = type;
  nextTick(initGraph);
}

function applyFilter(_payload: unknown) {
  nextTick(initGraph);
}

async function exportImage(type: "png" | "svg"): Promise<string | undefined> {
  if (!graph) return;
  // G6 dataURL export does not support SVG mime in current runtime, fallback to PNG payload.
  if (type === "svg") return graph.toDataURL({ type: "image/png" });
  return graph.toDataURL({ type: "image/png" });
}

defineExpose({
  highlightPaths,
  highlightImpact,
  highlightSearch,
  applyFilter,
  setLayout,
  exportImage,
  clearHighlight,
});
</script>

<template>
  <div ref="containerRef" class="topology-canvas" />
</template>

<style scoped>
.topology-canvas {
  width: 100%;
  height: 100%;
  min-height: 420px;
}
</style>
