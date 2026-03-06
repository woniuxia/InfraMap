<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import cytoscape, { type Core, type EventObjectNode, type LayoutOptions } from "cytoscape";
import dagre from "cytoscape-dagre";
import fcose from "cytoscape-fcose";
import cytoscapeSvg from "cytoscape-svg";
import type { TopologyGraph, TopologyNode } from "@/types";
import { toExternalNodeId } from "@/components/topology/topologyGraph.utils";
import { buildTopologyCyElements } from "@/components/topology/topologyCytoscape.utils";
import { colorizeSvgDataUri } from "@/icons/iconColorize";
import {
  DENSITY_OPTIONS,
  getDensityByZoom,
  normalizeEdgesForDensity,
  normalizeNodesForDensity,
  selectVisibleEdgeIds,
  type ZoomDensity,
} from "@/components/topology/topologyDensity.utils";
import {
  hasTopologyNodeSetChanged,
  isDegenerateNodeDistribution,
} from "@/components/topology/topologyLayoutSafety.utils";

const props = withDefaults(defineProps<{
  graphData: TopologyGraph | null;
  focusNeighborhood?: boolean;
  layout?: "force" | "dagre";
}>(), {
  focusNeighborhood: true,
  layout: "force",
});

const emit = defineEmits<{
  (e: "node-click", node: TopologyNode): void;
  (e: "node-contextmenu", payload: { node: TopologyNode; x: number; y: number }): void;
  (e: "layout-resolved", payload: { requested: "force" | "dagre"; applied: "force" | "dagre"; reason?: string }): void;
}>();

type LayoutType = "force" | "dagre";

let cytoscapeExtensionsRegistered = false;

function registerCytoscapeExtensions() {
  if (cytoscapeExtensionsRegistered) return;
  cytoscape.use(dagre);
  cytoscape.use(fcose);
  cytoscapeSvg(cytoscape);
  cytoscapeExtensionsRegistered = true;
}

const containerRef = ref<HTMLDivElement>();
const activeLayout = ref<LayoutType>(props.layout);

let cy: Core | null = null;
let resizeObserver: ResizeObserver | null = null;
let themeObserver: MutationObserver | null = null;
let resizeAnimationFrame: number | null = null;
let syncVersion = 0;
let graphTheme: GraphTheme | null = null;
let hasRenderedGraph = false;

const renderFlags = {
  isLargeGraph: false,
  hideEdgeLabels: false,
};

let nodeById = new Map<string, TopologyNode>();
let renderableNodeIds = new Set<string>();
let edgeIdByPair = new Map<string, string>();
let zoomDensityRaf: number | null = null;
let densityWorker: Worker | null = null;
let densityWorkerRequestId = 0;
const densityWorkerResolvers = new Map<number, (edgeIds?: string[]) => void>();
const densitySelectionCache = new WeakMap<TopologyGraph, Partial<Record<ZoomDensity, string[]>>>();
let forcedDensity: ZoomDensity | null = null;
let pendingDensityAfterFocus: ZoomDensity | null = null;
let focusDensityTimer: number | null = null;

type HighlightState
  = { kind: "none" }
  | { kind: "paths"; paths: string[][] }
  | { kind: "impact"; nodeId: string; result: { affected_nodes: { id: string; depth: number }[] } }
  | { kind: "neighborhood"; nodeId: string; depth: number }
  | { kind: "search"; nodeIds: string[]; focusId?: string };

let activeHighlightState: HighlightState = { kind: "none" };

const viewportState = ref({
  zoom: 1,
  density: "medium" as ZoomDensity,
  totalEdges: 0,
  visibleEdges: 0,
});

interface DensityWorkerRequest {
  requestId: number;
  density: ZoomDensity;
  nodes: { id: string; importance: number }[];
  edges: { id: string; source: string; target: string; strength: number; cross_env: boolean }[];
}

interface DensityWorkerResponse {
  requestId: number;
  visibleEdgeIds: string[];
}

interface LayoutRunResult {
  requested: LayoutType;
  applied: LayoutType;
  reason?: string;
}

interface GraphTheme {
  statusColors: Record<string, string>;
  envColors: Record<string, string>;
  nodeTypeColors: {
    application: string;
    middleware: string;
    nginx: string;
  };
  applicationTypeColors: {
    frontend: string;
    backend: string;
    gateway: string;
    batch_job: string;
    microservice: string;
    other: string;
  };
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

function parseHexColor(hex: string): { r: number; g: number; b: number } | null {
  if (!hex.startsWith("#")) return null;
  if (hex.length === 4) {
    const r = Number.parseInt(`${hex[1]}${hex[1]}`, 16);
    const g = Number.parseInt(`${hex[2]}${hex[2]}`, 16);
    const b = Number.parseInt(`${hex[3]}${hex[3]}`, 16);
    if (Number.isNaN(r) || Number.isNaN(g) || Number.isNaN(b)) return null;
    return { r, g, b };
  }
  if (hex.length === 7) {
    const r = Number.parseInt(hex.slice(1, 3), 16);
    const g = Number.parseInt(hex.slice(3, 5), 16);
    const b = Number.parseInt(hex.slice(5, 7), 16);
    if (Number.isNaN(r) || Number.isNaN(g) || Number.isNaN(b)) return null;
    return { r, g, b };
  }
  return null;
}

function withAlpha(hex: string, alphaHex: string): string {
  const rgb = parseHexColor(hex);
  const alphaInt = Number.parseInt(alphaHex, 16);
  if (!rgb || Number.isNaN(alphaInt)) return hex;
  const alpha = Math.max(0, Math.min(255, alphaInt)) / 255;
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha.toFixed(3)})`;
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
    nodeTypeColors: {
      application: accent,
      middleware: warning,
      nginx: success,
    },
    applicationTypeColors: {
      frontend: cssVar("--im-app-type-frontend", "#5ca3ff"),
      backend: cssVar("--im-app-type-backend", "#41c58a"),
      gateway: cssVar("--im-app-type-gateway", "#f2b645"),
      batch_job: cssVar("--im-app-type-batch-job", "#ef8f62"),
      microservice: cssVar("--im-app-type-microservice", "#3ec7c5"),
      other: cssVar("--im-app-type-other", "#8b9bb9"),
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

function getTheme(): GraphTheme {
  if (!graphTheme) graphTheme = buildGraphTheme();
  return graphTheme;
}

function refreshTheme() {
  graphTheme = buildGraphTheme();
}

function getActiveDensity(): ZoomDensity {
  return forcedDensity || viewportState.value.density;
}

const densityHintText = computed(() => {
  const density = DENSITY_OPTIONS[getActiveDensity()];
  const focusSuffix = forcedDensity ? "（聚焦）" : "";
  return `渲染: ${density.label}${focusSuffix} · 关系 ${viewportState.value.visibleEdges}/${viewportState.value.totalEdges}`;
});

function edgePairKey(source: string, target: string): string {
  return `${source}=>${target}`;
}

function rebuildIndexes(graphData: TopologyGraph | null) {
  nodeById = new Map();
  renderableNodeIds = new Set();
  edgeIdByPair = new Map();

  if (!graphData) return;

  graphData.nodes.forEach((node) => {
    nodeById.set(node.id, node);
    renderableNodeIds.add(node.id);
  });

  graphData.edges.forEach((edge) => {
    edgeIdByPair.set(edgePairKey(edge.source, edge.target), edge.id);
  });
}

function resolveRenderableNodeId(rawNodeId: string): string | null {
  if (renderableNodeIds.has(rawNodeId)) return rawNodeId;
  const externalId = toExternalNodeId(rawNodeId);
  if (renderableNodeIds.has(externalId)) return externalId;
  return null;
}

function cacheVisibleEdgeIds(graphData: TopologyGraph, density: ZoomDensity, edgeIds: string[]) {
  const cache = densitySelectionCache.get(graphData) || {};
  cache[density] = edgeIds;
  densitySelectionCache.set(graphData, cache);
}

function ensureDensityWorker(): Worker | null {
  if (densityWorker || typeof Worker === "undefined") return densityWorker;

  densityWorker = new Worker(new URL("./topologyDensity.worker.ts", import.meta.url), { type: "module" });
  densityWorker.onmessage = (event: MessageEvent<DensityWorkerResponse>) => {
    const { requestId, visibleEdgeIds } = event.data;
    const resolve = densityWorkerResolvers.get(requestId);
    if (!resolve) return;
    densityWorkerResolvers.delete(requestId);
    resolve(visibleEdgeIds);
  };
  densityWorker.onerror = () => {
    densityWorkerResolvers.forEach((resolve) => resolve(undefined));
    densityWorkerResolvers.clear();
    densityWorker?.terminate();
    densityWorker = null;
  };

  return densityWorker;
}

async function resolveVisibleEdgeIds(graphData: TopologyGraph, density: ZoomDensity): Promise<string[]> {
  const cache = densitySelectionCache.get(graphData);
  const cachedEdgeIds = cache?.[density];
  if (cachedEdgeIds) return cachedEdgeIds;

  const nodes = normalizeNodesForDensity(graphData.nodes);
  const edges = normalizeEdgesForDensity(graphData.edges);

  const computeSynchronously = () => {
    const edgeIds = selectVisibleEdgeIds(density, nodes, edges);
    cacheVisibleEdgeIds(graphData, density, edgeIds);
    return edgeIds;
  };

  if (density === "detail" || edges.length < 260) {
    return computeSynchronously();
  }

  const worker = ensureDensityWorker();
  if (!worker) {
    return computeSynchronously();
  }

  const requestId = ++densityWorkerRequestId;
  const payload: DensityWorkerRequest = {
    requestId,
    density,
    nodes,
    edges,
  };

  return new Promise((resolve) => {
    const timeoutId = globalThis.setTimeout(() => {
      densityWorkerResolvers.delete(requestId);
      resolve(computeSynchronously());
    }, 180);

    densityWorkerResolvers.set(requestId, (edgeIds?: string[]) => {
      globalThis.clearTimeout(timeoutId);
      if (edgeIds === undefined) {
        resolve(computeSynchronously());
        return;
      }
      cacheVisibleEdgeIds(graphData, density, edgeIds);
      resolve(edgeIds);
    });

    try {
      worker.postMessage(payload);
    } catch {
      densityWorkerResolvers.delete(requestId);
      globalThis.clearTimeout(timeoutId);
      resolve(computeSynchronously());
    }
  });
}

async function buildDensityGraphData(graphData: TopologyGraph, density: ZoomDensity): Promise<TopologyGraph> {
  const edgeIds = await resolveVisibleEdgeIds(graphData, density);
  const edgeIdSet = new Set(edgeIds);
  return {
    ...graphData,
    edges: graphData.edges.filter((edge) => edgeIdSet.has(edge.id)),
  };
}

function clearFocusDensityTimer() {
  if (focusDensityTimer) {
    globalThis.clearTimeout(focusDensityTimer);
    focusDensityTimer = null;
  }
}

function scheduleDensitySync() {
  if (zoomDensityRaf) {
    cancelAnimationFrame(zoomDensityRaf);
  }
  zoomDensityRaf = requestAnimationFrame(() => {
    zoomDensityRaf = null;
    void syncGraphData({ preserveViewport: true, runLayout: false });
  });
}

function deactivateFocusDensity(options: { sync?: boolean } = {}) {
  const shouldSync = options.sync !== false;
  if (!forcedDensity) return;

  forcedDensity = null;
  clearFocusDensityTimer();
  const nextDensity = pendingDensityAfterFocus || getDensityByZoom(viewportState.value.zoom);
  pendingDensityAfterFocus = null;
  if (viewportState.value.density !== nextDensity) {
    viewportState.value.density = nextDensity;
    if (shouldSync) scheduleDensitySync();
  } else if (shouldSync) {
    scheduleDensitySync();
  }
}

function activateFocusDensity(durationMs = 14_000) {
  forcedDensity = "detail";
  pendingDensityAfterFocus = getDensityByZoom(viewportState.value.zoom);
  clearFocusDensityTimer();
  focusDensityTimer = globalThis.setTimeout(() => {
    focusDensityTimer = null;
    deactivateFocusDensity();
  }, durationMs);

  if (viewportState.value.density !== "detail") {
    viewportState.value.density = "detail";
    scheduleDensitySync();
  }
}

function handleViewportTransform() {
  if (!cy) return;
  const zoom = cy.zoom();
  viewportState.value.zoom = zoom;
  const nextDensity = getDensityByZoom(zoom);
  if (forcedDensity) {
    pendingDensityAfterFocus = nextDensity;
    return;
  }
  if (nextDensity !== viewportState.value.density) {
    viewportState.value.density = nextDensity;
    scheduleDensitySync();
  }
}

function resolveAppTypeColor(data: Record<string, unknown>, theme: GraphTheme): string {
  const appType = String(data.app_type_key || "other");
  return theme.applicationTypeColors[appType as keyof GraphTheme["applicationTypeColors"]]
    || theme.applicationTypeColors.other;
}

function resolveNodeBaseColor(data: Record<string, unknown>, theme: GraphTheme): string {
  const nodeType = String(data.node_type || "application");
  const status = String(data.status || "");

  if (nodeType === "application") {
    return resolveAppTypeColor(data, theme);
  }
  if (nodeType === "middleware") {
    return theme.nodeTypeColors.middleware;
  }
  if (nodeType === "nginx") {
    return theme.nodeTypeColors.nginx;
  }

  return theme.statusColors[status] || theme.nodeTypeColors.application;
}

function resolveNodeStroke(data: Record<string, unknown>, theme: GraphTheme): string {
  const isExternal = Boolean(data.is_external);
  const typeColor = resolveNodeBaseColor(data, theme);
  return isExternal ? withAlpha(typeColor, "A2") : withAlpha(typeColor, "EE");
}

function resolveNodeBorderWidth(data: Record<string, unknown>): number {
  const groupKind = String(data.group_kind || "application_service");
  if (groupKind === "nginx") return 2.8;
  if (groupKind === "middleware") return 2.2;
  return 2.6;
}

function resolveEdgeStroke(data: Record<string, unknown>, theme: GraphTheme): string {
  if (Boolean(data.cross_env)) return theme.impact;
  const sourceNodeType = String(data.source_node_type || "");
  if (sourceNodeType === "application") {
    const appType = String(data.source_app_type_key || "other");
    const color = theme.applicationTypeColors[appType as keyof GraphTheme["applicationTypeColors"]];
    if (color) return color;
  }
  const edgeType = String(data.edge_type || "http_call");
  return theme.edgeStyles[edgeType]?.stroke || theme.edgeStyles.http_call.stroke;
}

function resolveEdgeDash(data: Record<string, unknown>, theme: GraphTheme): string {
  if (Boolean(data.cross_env)) return "6 4";
  const edgeType = String(data.edge_type || "http_call");
  const dash = theme.edgeStyles[edgeType]?.lineDash;
  return dash ? dash.join(" ") : "";
}

function resolveEdgeCurveStyle(
  data: Record<string, unknown>,
  runtime: { dense: boolean; layout: "force" | "dagre" },
): "bezier" | "taxi" {
  if (Boolean(data.cross_env)) return "bezier";
  if (runtime.layout === "dagre" && runtime.dense) {
    return "taxi";
  }
  return "bezier";
}

function buildStylesheet(
  theme: GraphTheme,
  runtime: { dense: boolean; layout: "force" | "dagre" },
) {
  const labelBg = withAlpha(theme.labelBg, "E6");
  return [
    {
      selector: "node",
      style: {
        label: "data(label)",
        "overlay-opacity": 0,
      },
    },
    {
      selector: "node[shape][size][label_font_size]",
      style: {
        shape: "data(shape)",
        width: "data(size)",
        height: "data(size)",
        "font-size": "data(label_font_size)",
        color: (ele: cytoscape.NodeSingular) => ele.data("is_external") ? theme.labelSecondary : theme.labelPrimary,
        "text-halign": "center",
        "text-valign": "bottom",
        "text-margin-y": 6,
        "text-outline-color": withAlpha(theme.labelBg, "D9"),
        "text-outline-width": 1.2,
        "background-color": (ele: cytoscape.NodeSingular) => resolveNodeBaseColor(ele.data(), theme),
        "background-image": (ele: cytoscape.NodeSingular) => {
          const src = String(ele.data("icon_src") || "").trim();
          if (src.length === 0) return "none";
          return colorizeSvgDataUri(src, resolveNodeBaseColor(ele.data(), theme));
        },
        "background-fit": "contain",
        "background-repeat": "no-repeat",
        "background-width": "66%",
        "background-height": "66%",
        "background-position-x": "50%",
        "background-position-y": "50%",
        "background-image-opacity": 0.96,
        "border-color": (ele: cytoscape.NodeSingular) => resolveNodeStroke(ele.data(), theme),
        "border-width": (ele: cytoscape.NodeSingular) => resolveNodeBorderWidth(ele.data()),
        "border-style": (ele: cytoscape.NodeSingular) => {
          if (ele.data("is_external")) return "dashed";
          if (ele.data("group_kind") === "middleware") return "dashed";
          return "solid";
        },
        "background-opacity": 0,
      },
    },
    {
      selector: "node:parent",
      style: {
        label: "data(label)",
        shape: "round-rectangle",
        "background-color": withAlpha(theme.labelBg, "16"),
        "border-color": withAlpha(theme.labelMuted, "AA"),
        "border-width": 1.2,
        "border-style": "dashed",
        "text-valign": "top",
        "text-halign": "center",
        "font-size": 11,
        color: theme.labelSecondary,
        padding: "14px",
        "overlay-opacity": 0,
      },
    },
    {
      selector: 'node[kind = "external"]',
      style: {
        "border-color": withAlpha(theme.impact, "B8"),
        color: theme.impact,
      },
    },
    {
      selector: "edge",
      style: {
        "curve-style": (ele: cytoscape.EdgeSingular) => resolveEdgeCurveStyle(ele.data(), runtime),
        "taxi-direction": "rightward",
        "taxi-turn": 20,
        "taxi-turn-min-distance": 12,
        "line-color": (ele: cytoscape.EdgeSingular) => resolveEdgeStroke(ele.data(), theme),
        "target-arrow-color": (ele: cytoscape.EdgeSingular) => resolveEdgeStroke(ele.data(), theme),
        "line-style": (ele: cytoscape.EdgeSingular) => resolveEdgeDash(ele.data(), theme) ? "dashed" : "solid",
        width: "data(line_width)",
        opacity: "data(opacity)",
        "target-arrow-shape": "data(arrow)",
        "arrow-scale": 0.9,
        label: "data(display_label)",
        "font-size": "data(label_font_size)",
        color: theme.labelSecondary,
        "text-background-color": labelBg,
        "text-background-opacity": 0.85,
        "text-background-shape": "roundrectangle",
        "text-background-padding": "2px",
        "overlay-opacity": 0,
      },
    },
    {
      selector: "node.im-highlight",
      style: {
        "border-color": theme.highlight,
        "border-width": 3,
      },
    },
    {
      selector: "node.im-impact",
      style: {
        "border-color": theme.impact,
        "border-width": 3.2,
      },
    },
    {
      selector: "node.im-dim",
      style: {
        opacity: 0.2,
        "text-opacity": 0.35,
      },
    },
    {
      selector: "node:parent.im-dim",
      style: {
        opacity: 0.18,
      },
    },
    {
      selector: "edge.im-highlight",
      style: {
        opacity: 1,
        "line-color": theme.highlight,
        "target-arrow-color": theme.highlight,
        label: "data(label)",
        "font-size": 10,
        color: theme.labelPrimary,
        "text-background-opacity": 0.92,
      },
    },
    {
      selector: "edge.im-dim",
      style: {
        opacity: 0.13,
      },
    },
  ];
}

function syncParentDimState() {
  if (!cy) return;
  cy.nodes(":parent").forEach((parent) => {
    const children = parent.children();
    const hasActiveChild = children.filter(".im-highlight, .im-impact").nonempty();
    if (hasActiveChild) {
      parent.removeClass("im-dim");
    } else {
      parent.addClass("im-dim");
    }
  });
}

function clearHighlightStates() {
  if (!cy) return;
  cy.nodes().removeClass("im-highlight im-dim im-impact");
  cy.edges().removeClass("im-highlight im-dim");
}

function applyPathHighlight(paths: string[][]) {
  if (!cy) return;
  clearHighlightStates();

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
      const edgeId = edgeIdByPair.get(edgePairKey(source, target));
      if (edgeId) edgeIds.add(edgeId);
    }
  });

  cy.nodes().forEach((node) => {
    if (node.isParent()) return;
    node.addClass(nodeIds.has(node.id()) ? "im-highlight" : "im-dim");
  });
  cy.edges().forEach((edge) => {
    edge.addClass(edgeIds.has(edge.id()) ? "im-highlight" : "im-dim");
  });
  syncParentDimState();
}

function applyImpactHighlight(nodeId: string, result: { affected_nodes: { id: string; depth: number }[] }) {
  if (!cy) return;
  clearHighlightStates();

  const focusNodeId = resolveRenderableNodeId(nodeId) || nodeId;
  const affectedIds = new Set<string>([focusNodeId]);
  result.affected_nodes.forEach((item) => {
    const renderId = resolveRenderableNodeId(item.id);
    if (renderId) affectedIds.add(renderId);
  });

  cy.nodes().forEach((node) => {
    if (node.isParent()) return;
    if (node.id() === focusNodeId) {
      node.addClass("im-impact");
    } else if (affectedIds.has(node.id())) {
      node.addClass("im-highlight");
    } else {
      node.addClass("im-dim");
    }
  });

  cy.edges().forEach((edge) => {
    const sourceHit = affectedIds.has(edge.source().id());
    const targetHit = affectedIds.has(edge.target().id());
    edge.addClass(sourceHit && targetHit ? "im-highlight" : "im-dim");
  });
  syncParentDimState();
}

function applyNeighborhoodHighlight(nodeId: string, depth: number, allowFocus = true) {
  if (!cy) return;
  clearHighlightStates();

  const renderFocusId = resolveRenderableNodeId(nodeId);
  if (!renderFocusId) return;

  const focus = cy.getElementById(renderFocusId);
  if (focus.empty()) return;

  const maxDepth = Math.max(1, Math.min(2, depth));
  const visited = new Set<string>([renderFocusId]);
  const frontier: string[] = [renderFocusId];
  const highlightedNodeIds = new Set<string>([renderFocusId]);
  const highlightedEdgeIds = new Set<string>();

  for (let level = 0; level < maxDepth; level += 1) {
    const nextFrontier: string[] = [];
    for (const currentId of frontier) {
      const currentNode = cy.getElementById(currentId);
      if (currentNode.empty()) continue;

      currentNode.connectedEdges().forEach((edge) => {
        if (edge.empty()) return;
        highlightedEdgeIds.add(edge.id());
        const sourceId = edge.source().id();
        const targetId = edge.target().id();
        const neighborId = sourceId === currentId ? targetId : sourceId;
        if (visited.has(neighborId)) return;
        visited.add(neighborId);
        highlightedNodeIds.add(neighborId);
        nextFrontier.push(neighborId);
      });
    }
    frontier.length = 0;
    frontier.push(...nextFrontier);
    if (frontier.length === 0) break;
  }

  cy.nodes().forEach((node) => {
    if (node.isParent()) return;
    if (node.id() === renderFocusId) {
      node.addClass("im-impact");
    } else if (highlightedNodeIds.has(node.id())) {
      node.addClass("im-highlight");
    } else {
      node.addClass("im-dim");
    }
  });

  cy.edges().forEach((edge) => {
    edge.addClass(highlightedEdgeIds.has(edge.id()) ? "im-highlight" : "im-dim");
  });

  if (allowFocus) {
    cy.animate({
      center: { eles: focus },
      duration: 220,
    });
  }

  syncParentDimState();
}

function applySearchHighlight(nodeIds: string[], focusId?: string, allowFocus = true) {
  if (!cy) return;
  clearHighlightStates();

  const matchSet = new Set<string>();
  nodeIds.forEach((id) => {
    const renderId = resolveRenderableNodeId(id);
    if (renderId) matchSet.add(renderId);
  });

  cy.nodes().forEach((node) => {
    if (node.isParent()) return;
    node.addClass(matchSet.has(node.id()) ? "im-highlight" : "im-dim");
  });

  if (allowFocus && focusId) {
    const renderFocusId = resolveRenderableNodeId(focusId);
    if (renderFocusId) {
      const target = cy.getElementById(renderFocusId);
      if (target.nonempty()) {
        cy.center(target);
      }
    }
  }
  syncParentDimState();
}

function applyHighlightState(options: { allowFocus?: boolean } = {}) {
  const allowFocus = options.allowFocus !== false;
  if (!cy) return;

  if (activeHighlightState.kind === "none") {
    clearHighlightStates();
    return;
  }
  if (activeHighlightState.kind === "paths") {
    applyPathHighlight(activeHighlightState.paths);
    return;
  }
  if (activeHighlightState.kind === "impact") {
    applyImpactHighlight(activeHighlightState.nodeId, activeHighlightState.result);
    return;
  }
  if (activeHighlightState.kind === "neighborhood") {
    applyNeighborhoodHighlight(activeHighlightState.nodeId, activeHighlightState.depth, allowFocus);
    return;
  }
  applySearchHighlight(activeHighlightState.nodeIds, activeHighlightState.focusId, allowFocus);
}

function snapshotLeafNodePositions(): Map<string, { x: number; y: number }> {
  const positionMap = new Map<string, { x: number; y: number }>();
  if (!cy) return positionMap;

  cy.nodes()
    .not(":parent")
    .forEach((node: cytoscape.NodeSingular) => {
      const position = node.position();
      positionMap.set(node.id(), { x: position.x, y: position.y });
    });
  return positionMap;
}

function restoreLeafNodePositions(positionMap: Map<string, { x: number; y: number }>) {
  if (!cy || positionMap.size === 0) return;
  cy.batch(() => {
    cy!.nodes()
      .not(":parent")
      .forEach((node: cytoscape.NodeSingular) => {
        const position = positionMap.get(node.id());
        if (!position) return;
        node.position(position);
      });
  });
}

function hasDegenerateLayout(): boolean {
  if (!cy) return false;
  const points: Array<{ x: number; y: number }> = [];
  cy.nodes()
    .not(":parent")
    .forEach((node: cytoscape.NodeSingular) => {
      const position = node.position();
      points.push({ x: position.x, y: position.y });
    });
  return isDegenerateNodeDistribution(points);
}

function buildLayoutOptions(layoutType: LayoutType): LayoutOptions {
  if (layoutType === "dagre") {
    return {
      name: "dagre",
      rankDir: "LR",
      nodeSep: 72,
      rankSep: 180,
      fit: true,
      padding: 36,
      animate: false,
    } as unknown as LayoutOptions;
  }

  return {
    name: "fcose",
    quality: renderFlags.isLargeGraph ? "default" : "proof",
    randomize: true,
    animate: false,
    fit: true,
    padding: 40,
    packComponents: false,
    nodeRepulsion: (node: cytoscape.NodeSingular) => node.data("is_external") ? 8000 : 12000,
    idealEdgeLength: (edge: cytoscape.EdgeSingular) => edge.data("cross_env") ? 240 : 180,
    numIter: renderFlags.isLargeGraph ? 1600 : 2200,
    tile: true,
  } as unknown as LayoutOptions;
}

function runLayoutOnce(layoutType: LayoutType): Promise<boolean> {
  if (!cy) return Promise.resolve(true);
  return new Promise((resolve) => {
    let settled = false;
    const finish = (ok: boolean) => {
      if (settled) return;
      settled = true;
      resolve(ok);
    };

    let layout: ReturnType<Core["layout"]> | null = null;
    try {
      layout = cy!.layout(buildLayoutOptions(layoutType));
    } catch {
      finish(false);
      return;
    }

    layout.one("layoutstop", () => finish(true));
    try {
      layout.run();
    } catch {
      finish(false);
      return;
    }

    globalThis.setTimeout(() => finish(true), renderFlags.isLargeGraph ? 2600 : 1800);
  });
}

async function runLayout(layoutType: LayoutType): Promise<LayoutRunResult> {
  const primaryOk = await runLayoutOnce(layoutType);
  if (layoutType !== "dagre") {
    return {
      requested: layoutType,
      applied: layoutType,
      reason: primaryOk ? undefined : "force_layout_error",
    };
  }

  if (primaryOk && !hasDegenerateLayout()) {
    return {
      requested: layoutType,
      applied: layoutType,
    };
  }

  const fallbackOk = await runLayoutOnce("force");
  activeLayout.value = "force";
  applyStyles();
  return {
    requested: layoutType,
    applied: "force",
    reason: primaryOk ? "dagre_degenerate" : (fallbackOk ? "dagre_error" : "dagre_and_force_error"),
  };
}

function applyStyles() {
  if (!cy) return;
  const theme = getTheme();
  const dense = renderFlags.hideEdgeLabels || viewportState.value.visibleEdges > 140;
  cy.style(buildStylesheet(theme, {
    dense,
    layout: activeLayout.value,
  }) as unknown as cytoscape.StylesheetJsonBlock[]);
}

function getContextMenuPosition(evt: EventObjectNode): { x: number; y: number } {
  const mouse = evt.originalEvent as MouseEvent | undefined;
  if (mouse && Number.isFinite(mouse.clientX) && Number.isFinite(mouse.clientY)) {
    return { x: mouse.clientX, y: mouse.clientY };
  }

  const rect = containerRef.value?.getBoundingClientRect();
  const rendered = evt.renderedPosition || evt.target.renderedPosition();
  if (rect && rendered) {
    return {
      x: rect.left + rendered.x,
      y: rect.top + rendered.y,
    };
  }

  return { x: 0, y: 0 };
}

function handleNodeTap(evt: EventObjectNode) {
  const target = evt.target;
  if (!target.isNode() || target.isParent()) return;
  if (props.focusNeighborhood) {
    activeHighlightState = { kind: "neighborhood", nodeId: target.id(), depth: 1 };
    activateFocusDensity(9_000);
    applyHighlightState({ allowFocus: false });
  }
  const node = nodeById.get(target.id());
  if (node) emit("node-click", node);
}

function handleNodeContextTap(evt: EventObjectNode) {
  const target = evt.target;
  if (!target.isNode() || target.isParent()) return;
  const node = nodeById.get(target.id());
  if (!node) return;
  const position = getContextMenuPosition(evt);
  emit("node-contextmenu", { node, x: position.x, y: position.y });
}

async function ensureCy() {
  if (!containerRef.value || cy) return;

  registerCytoscapeExtensions();
  refreshTheme();

  cy = cytoscape({
    container: containerRef.value,
    elements: [],
    style: buildStylesheet(getTheme(), {
      dense: false,
      layout: activeLayout.value,
    }) as unknown as cytoscape.StylesheetJsonBlock[],
    zoomingEnabled: true,
    userZoomingEnabled: true,
    panningEnabled: true,
    userPanningEnabled: true,
    boxSelectionEnabled: false,
    autounselectify: true,
    minZoom: 0.15,
    maxZoom: 4,
  });

  cy.on("tap", "node", (evt) => handleNodeTap(evt as EventObjectNode));
  cy.on("cxttap", "node", (evt) => handleNodeContextTap(evt as EventObjectNode));
  cy.on("zoom", handleViewportTransform);

  viewportState.value.zoom = cy.zoom();
  viewportState.value.density = getDensityByZoom(viewportState.value.zoom);
}

async function syncGraphData(options: { preserveViewport?: boolean; runLayout?: boolean } = {}) {
  const currentVersion = ++syncVersion;
  await nextTick();

  if (!containerRef.value) return;
  await ensureCy();
  if (!cy || currentVersion !== syncVersion) return;

  const preservedZoom = options.preserveViewport ? cy.zoom() : null;
  const preservedPan = options.preserveViewport ? { ...cy.pan() } : null;
  const preservedNodePositions = snapshotLeafNodePositions();
  const previousNodeIds = new Set(
    cy
      .nodes()
      .not(":parent")
      .map((node) => node.id()),
  );

  if (!props.graphData) {
    viewportState.value.totalEdges = 0;
    viewportState.value.visibleEdges = 0;
    rebuildIndexes(null);
    cy.elements().remove();
    hasRenderedGraph = false;
    return;
  }

  const density = getActiveDensity();
  const densityGraph = await buildDensityGraphData(props.graphData, density);
  if (!cy || currentVersion !== syncVersion) return;

  viewportState.value.totalEdges = props.graphData.edges.length;
  viewportState.value.visibleEdges = densityGraph.edges.length;
  renderFlags.isLargeGraph = densityGraph.nodes.length > 500;
  renderFlags.hideEdgeLabels = DENSITY_OPTIONS[density].hideEdgeLabels
    || props.graphData.layout_hints.high_density_mode
    || densityGraph.edges.length > 90;
  applyStyles();

  const elements = buildTopologyCyElements(densityGraph, {
    density,
    hideEdgeLabels: renderFlags.hideEdgeLabels,
    isLargeGraph: renderFlags.isLargeGraph,
  });

  rebuildIndexes(densityGraph);

  cy.batch(() => {
    cy!.elements().remove();
    cy!.add(elements);
  });

  let shouldRunLayout = options.runLayout ?? !hasRenderedGraph;
  if (!shouldRunLayout && hasTopologyNodeSetChanged(previousNodeIds, densityGraph.nodes)) {
    shouldRunLayout = true;
  }

  let layoutResult: LayoutRunResult | null = null;
  if (shouldRunLayout) {
    layoutResult = await runLayout(activeLayout.value);
  } else {
    // 仅做边密度切换时保持节点坐标，避免 remove/add 导致节点回到原点聚团。
    restoreLeafNodePositions(preservedNodePositions);
  }

  if (layoutResult && (layoutResult.reason || layoutResult.applied !== layoutResult.requested)) {
    emit("layout-resolved", layoutResult);
  }

  if (options.preserveViewport && preservedZoom !== null && preservedPan) {
    cy.zoom(preservedZoom);
    cy.pan(preservedPan);
  } else if (!hasRenderedGraph && cy.elements().nonempty()) {
    cy.fit(undefined, 40);
  }

  hasRenderedGraph = true;
  applyHighlightState({ allowFocus: false });
}

watch(
  () => props.graphData,
  () => {
    void syncGraphData();
  },
);

watch(
  () => props.focusNeighborhood,
  (enabled) => {
    if (!enabled && activeHighlightState.kind === "neighborhood") {
      clearHighlight();
    }
  },
);

watch(
  () => props.layout,
  (layout) => {
    if (layout === activeLayout.value) return;
    void setLayout(layout);
  },
);

onMounted(() => {
  void syncGraphData();

  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      if (!cy || !containerRef.value) return;
      if (resizeAnimationFrame) {
        cancelAnimationFrame(resizeAnimationFrame);
      }
      resizeAnimationFrame = requestAnimationFrame(() => {
        resizeAnimationFrame = null;
        if (!cy || !containerRef.value) return;
        cy.resize();
      });
    });
    resizeObserver.observe(containerRef.value);
  }

  themeObserver = new MutationObserver((mutations) => {
    const changed = mutations.some(
      (mutation) => mutation.type === "attributes" && mutation.attributeName === "data-theme",
    );
    if (changed) {
      refreshTheme();
      applyStyles();
    }
  });
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  themeObserver?.disconnect();

  if (resizeAnimationFrame) {
    cancelAnimationFrame(resizeAnimationFrame);
    resizeAnimationFrame = null;
  }
  if (zoomDensityRaf) {
    cancelAnimationFrame(zoomDensityRaf);
    zoomDensityRaf = null;
  }

  clearFocusDensityTimer();
  forcedDensity = null;
  pendingDensityAfterFocus = null;

  if (densityWorker) {
    densityWorker.terminate();
    densityWorker = null;
  }
  densityWorkerResolvers.forEach((resolve) => resolve(undefined));
  densityWorkerResolvers.clear();

  if (cy) {
    cy.destroy();
    cy = null;
  }

  themeObserver = null;
});

function clearHighlight() {
  activeHighlightState = { kind: "none" };
  deactivateFocusDensity();
  clearHighlightStates();
}

function highlightPaths(paths: string[][]) {
  activeHighlightState = { kind: "paths", paths };
  activateFocusDensity();
  applyHighlightState();
}

function highlightImpact(nodeId: string, result: { affected_nodes: { id: string; depth: number }[] }) {
  activeHighlightState = { kind: "impact", nodeId, result };
  activateFocusDensity();
  applyHighlightState();
}

function highlightSearch(nodeIds: string[], focusId?: string) {
  if (nodeIds.length === 0) {
    clearHighlight();
    return;
  }
  activeHighlightState = { kind: "search", nodeIds, focusId };
  activateFocusDensity(10_000);
  applyHighlightState({ allowFocus: true });
}

function focusNeighborhood(nodeId: string, depth = 1) {
  activeHighlightState = { kind: "neighborhood", nodeId, depth };
  activateFocusDensity(9_000);
  applyHighlightState({ allowFocus: true });
}

async function setLayout(type: LayoutType) {
  if (activeLayout.value === type) return;
  activeLayout.value = type;
  applyStyles();
  await syncGraphData({ runLayout: true, preserveViewport: false });
}

function applyFilter(_payload: unknown) {
  void syncGraphData({ preserveViewport: true, runLayout: false });
}

async function exportImage(type: "png" | "svg"): Promise<string | undefined> {
  if (!cy) return;
  if (type === "svg" && typeof cy.svg === "function") {
    const svgText = cy.svg({
      full: true,
      scale: 1,
      bg: cssVar("--im-surface-0", "#0f1728"),
    });
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svgText)}`;
  }

  return cy.png({
    full: true,
    scale: 2,
    bg: cssVar("--im-surface-0", "#0f1728"),
    output: "base64uri",
  });
}

defineExpose({
  highlightPaths,
  highlightImpact,
  highlightSearch,
  focusNeighborhood,
  applyFilter,
  setLayout,
  exportImage,
  clearHighlight,
});
</script>

<template>
  <div class="topology-canvas">
    <div ref="containerRef" class="topology-canvas-surface" />
    <div v-if="viewportState.totalEdges > 0" class="canvas-density-hint" data-testid="canvas-density-hint">
      {{ densityHintText }}
    </div>
  </div>
</template>

<style scoped>
.topology-canvas {
  width: 100%;
  height: 100%;
  min-height: 420px;
  position: relative;
}
.topology-canvas-surface {
  width: 100%;
  height: 100%;
  min-height: 420px;
  background:
    radial-gradient(120% 90% at 10% 0%, color-mix(in srgb, var(--im-accent) 8%, transparent) 0%, transparent 58%),
    radial-gradient(140% 110% at 100% 100%, color-mix(in srgb, var(--im-success) 8%, transparent) 0%, transparent 62%),
    repeating-linear-gradient(
      0deg,
      color-mix(in srgb, var(--im-border-subtle) 26%, transparent) 0 1px,
      transparent 1px 26px
    ),
    repeating-linear-gradient(
      90deg,
      color-mix(in srgb, var(--im-border-subtle) 20%, transparent) 0 1px,
      transparent 1px 26px
    ),
    var(--im-surface-1);
}
.canvas-density-hint {
  position: absolute;
  left: 12px;
  bottom: 10px;
  padding: 5px 10px;
  border-radius: var(--im-radius-sm);
  background: color-mix(in srgb, var(--im-surface-0) 84%, transparent);
  border: 1px solid color-mix(in srgb, var(--im-border-subtle) 75%, transparent);
  font-size: 12px;
  color: var(--im-text-secondary);
  backdrop-filter: blur(6px);
  pointer-events: none;
}
</style>
