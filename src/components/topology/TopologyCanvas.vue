<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import cytoscape, { type Core } from "cytoscape";
import type { TopologyGraph, TopologyNode } from "@/types";
import { buildNodeIconSpriteDataUri } from "@/icons/nodeIconSprite";
import { type ZoomDensity } from "@/components/topology/topologyDensity.utils";
import { useTopologyDensity } from "@/components/topology/useTopologyDensity";
import { useTopologyHighlight } from "@/components/topology/useTopologyHighlight";
import { useTopologyInteraction } from "@/components/topology/useTopologyInteraction";
import { useTopologyLayout } from "@/components/topology/useTopologyLayout";
import { useTopologyRenderer } from "@/components/topology/useTopologyRenderer";
import {
  getTopologyNodePositions,
  saveTopologyNodePositions,
  clearTopologyNodePositions,
} from "@/api/topologyV3";

const props = withDefaults(
  defineProps<{
    graphData: TopologyGraph | null;
    focusNeighborhood?: boolean;
    layout?: "force" | "dagre";
    performanceOptimizationEnabled?: boolean;
  }>(),
  {
    focusNeighborhood: true,
    layout: "force",
    performanceOptimizationEnabled: false,
  },
);

const emit = defineEmits<{
  (e: "node-click", node: TopologyNode): void;
  (e: "node-contextmenu", payload: { node: TopologyNode; x: number; y: number }): void;
  (e: "canvas-blank-click"): void;
  (
    e: "layout-resolved",
    payload: {
      requested: "force" | "dagre";
      applied: "force" | "dagre";
      reason?: string;
    },
  ): void;
}>();

type LayoutType = "force" | "dagre";

const containerRef = ref<HTMLDivElement>();
const activeLayout = ref<LayoutType>(props.layout);
let currentCy: Core | null = null;

let getCyImpl: () => Core | null = () => null;
let syncGraphDataImpl: (options?: {
  preserveViewport?: boolean;
  runLayout?: boolean;
  savedPositions?: Map<string, { x: number; y: number }>;
}) => Promise<void> = async () => {};
let applyRendererStylesImpl: () => void = () => {};
let graphTheme: GraphTheme | null = null;

const renderFlags = {
  isLargeGraph: false,
  hideEdgeLabels: false,
};

const viewportState = ref({
  zoom: 1,
  density: "medium" as ZoomDensity,
  totalEdges: 0,
  visibleEdges: 0,
});

const performanceOptimizationEnabled = computed(() => props.performanceOptimizationEnabled);

const {
  densityHintText,
  resolveZoomDensity,
  getActiveDensity,
  rebuildIndexes,
  resolveRenderableNodeId,
  findNodeById,
  findEdgeId,
  buildDensityGraphData,
  activateFocusDensity,
  deactivateFocusDensity,
  handleViewportZoomChange,
  resetDensityState,
  dispose: disposeDensity,
} = useTopologyDensity({
  performanceOptimizationEnabled,
  viewportState,
  requestSyncGraphData: () => {
    void syncGraphDataImpl({ preserveViewport: true, runLayout: false });
  },
});

const {
  applyHighlightState,
  clearHighlight,
  highlightPaths,
  highlightImpact,
  highlightSearch,
  focusNeighborhood: focusNeighborhoodInternal,
  isNeighborhoodHighlightActive,
} = useTopologyHighlight({
  getCy: () => getCyImpl(),
  resolveRenderableNodeId,
  findEdgeId,
  activateFocusDensity,
  deactivateFocusDensity,
});

const { handleCanvasTap, handleNodeTap, handleNodeContextTap } = useTopologyInteraction({
  getCy: () => getCyImpl(),
  getContainer: () => containerRef.value,
  focusNeighborhoodEnabled: () => props.focusNeighborhood,
  focusNeighborhood: focusNeighborhoodInternal,
  findNodeById,
  emitNodeClick: (node) => emit("node-click", node),
  emitNodeContextmenu: (payload) => emit("node-contextmenu", payload),
  emitCanvasBlankClick: () => emit("canvas-blank-click"),
});

const { snapshotLeafNodePositions, restoreLeafNodePositions, runLayout, runIncrementalLayout } =
  useTopologyLayout({
    getCy: () => getCyImpl(),
    getIsLargeGraph: () => renderFlags.isLargeGraph,
    onFallbackToForce: () => {
      activeLayout.value = "force";
      applyRendererStylesImpl();
    },
  });

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
      grpc_call: {
        stroke: cssVar("--el-color-primary", "#409eff"),
        lineDash: [6, 3],
      },
      db_query: { stroke: cssVar("--el-color-success", "#67c23a") },
      cache_access: {
        stroke: cssVar("--el-color-warning", "#e6a23c"),
        lineDash: [2, 4],
      },
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

function resolveAppTypeColor(data: Record<string, unknown>, theme: GraphTheme): string {
  const appType = String(data.app_type_key || "other");
  return (
    theme.applicationTypeColors[appType as keyof GraphTheme["applicationTypeColors"]] ||
    theme.applicationTypeColors.other
  );
}

function resolveNodeBaseColor(data: Record<string, unknown>, theme: GraphTheme): string {
  const nodeType = String(data.nodeType || "service");
  const status = String(data.status || "");

  if (nodeType === "service") {
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
  const isExternal = Boolean(data.isExternal);
  const typeColor = resolveNodeBaseColor(data, theme);
  return isExternal ? withAlpha(typeColor, "A2") : withAlpha(typeColor, "EE");
}

function resolveNodeBorderWidth(data: Record<string, unknown>): number {
  const groupKind = String(data.groupKind || "service");
  if (groupKind === "nginx") return 2.8;
  if (groupKind === "middleware") return 2.2;
  return 2.6;
}

function resolveEdgeStroke(data: Record<string, unknown>, theme: GraphTheme): string {
  if (data.crossEnv) return theme.impact;
  const sourceNodeType = String(data.source_nodeType || "");
  if (sourceNodeType === "service") {
    const appType = String(data.source_app_type_key || "other");
    const color = theme.applicationTypeColors[appType as keyof GraphTheme["applicationTypeColors"]];
    if (color) return color;
  }
  const edgeType = String(data.edgeType || "http_call");
  return theme.edgeStyles[edgeType]?.stroke || theme.edgeStyles.http_call.stroke;
}

function resolveEdgeDash(data: Record<string, unknown>, theme: GraphTheme): string {
  if (data.crossEnv) return "6 4";
  const edgeType = String(data.edgeType || "http_call");
  const dash = theme.edgeStyles[edgeType]?.lineDash;
  return dash ? dash.join(" ") : "";
}

function resolveEdgeCurveStyle(
  _data: Record<string, unknown>,
  _runtime: { dense: boolean; layout: "force" | "dagre" },
): "bezier" | "taxi" {
  // dagre 模式现在使用 fcose 算法,统一使用 bezier 曲线
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
        color: (ele: cytoscape.NodeSingular) =>
          ele.data("isExternal") ? theme.labelSecondary : theme.labelPrimary,
        "text-halign": "center",
        "text-valign": "bottom",
        "text-margin-y": 6,
        "text-outline-color": withAlpha(theme.labelBg, "D9"),
        "text-outline-width": 1.2,
        "background-color": (ele: cytoscape.NodeSingular) =>
          resolveNodeBaseColor(ele.data(), theme),
        "background-image": (ele: cytoscape.NodeSingular) => {
          const src = String(ele.data("icon_src") || "").trim();
          if (src.length === 0) return "none";
          return buildNodeIconSpriteDataUri(src, resolveNodeBaseColor(ele.data(), theme));
        },
        "background-fit": "contain",
        "background-repeat": "no-repeat",
        "background-position-x": "50%",
        "background-position-y": "50%",
        "background-offset-x": 0,
        "background-offset-y": 0,
        "background-image-opacity": 0.96,
        "border-color": (ele: cytoscape.NodeSingular) => resolveNodeStroke(ele.data(), theme),
        "border-width": (ele: cytoscape.NodeSingular) => resolveNodeBorderWidth(ele.data()),
        "border-style": (ele: cytoscape.NodeSingular) => {
          if (ele.data("isExternal")) return "dashed";
          if (ele.data("groupKind") === "middleware") return "dashed";
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
        "line-style": (ele: cytoscape.EdgeSingular) =>
          resolveEdgeDash(ele.data(), theme) ? "dashed" : "solid",
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

void getTheme;
void refreshTheme;
void buildStylesheet;

const {
  syncGraphData,
  mount,
  dispose,
  exportImage: exportCanvasImage,
  applyStyles: applyRendererStyles,
} = useTopologyRenderer({
  getCy: () => currentCy,
  setCy: (cy) => {
    currentCy = cy;
  },
  getContainer: () => containerRef.value,
  getGraphData: () => props.graphData,
  getLayout: () => activeLayout.value,
  setLayout: (layout) => {
    activeLayout.value = layout;
  },
  performanceOptimizationEnabled: () => performanceOptimizationEnabled.value,
  viewportState,
  resolveZoomDensity,
  getActiveDensity,
  buildDensityGraphData,
  rebuildIndexes,
  snapshotLeafNodePositions,
  restoreLeafNodePositions,
  runLayout,
  runIncrementalLayout,
  applyHighlightState,
  emitLayoutResolved: (payload) => emit("layout-resolved", payload),
  handleCanvasTap,
  handleNodeTap,
  handleNodeContextTap,
  handleViewportZoomChange,
  onRenderFlagsChange: (flags) => {
    renderFlags.isLargeGraph = flags.isLargeGraph;
    renderFlags.hideEdgeLabels = flags.hideEdgeLabels;
  },
});

getCyImpl = () => currentCy;
syncGraphDataImpl = syncGraphData;
applyRendererStylesImpl = applyRendererStyles;

// ── Position persistence helpers ──

let saveTimer: ReturnType<typeof setTimeout> | null = null;

function collectLeafPositions(): Array<{ node_id: string; x: number; y: number }> {
  if (!currentCy) return [];
  const entries: Array<{ node_id: string; x: number; y: number }> = [];
  currentCy
    .nodes()
    .not(":parent")
    .forEach((node: cytoscape.NodeSingular) => {
      const pos = node.position();
      entries.push({ node_id: node.id(), x: pos.x, y: pos.y });
    });
  return entries;
}

function scheduleSavePositions() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    saveTimer = null;
    const positions = collectLeafPositions();
    if (positions.length === 0) return;
    void saveTopologyNodePositions(activeLayout.value, positions);
  }, 1500);
}

async function loadSavedPositions(
  layoutType: string,
): Promise<Map<string, { x: number; y: number }>> {
  try {
    const rows = await getTopologyNodePositions(layoutType);
    const map = new Map<string, { x: number; y: number }>();
    for (const row of rows) {
      map.set(row.node_id, { x: row.x, y: row.y });
    }
    return map;
  } catch {
    return new Map();
  }
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
    if (!enabled && isNeighborhoodHighlightActive()) {
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

watch(
  () => props.performanceOptimizationEnabled,
  () => {
    resetDensityState();
    void syncGraphData({ preserveViewport: true, runLayout: false });
  },
);

onMounted(async () => {
  mount();
  const savedPositions = await loadSavedPositions(activeLayout.value);
  if (savedPositions.size > 0) {
    await syncGraphData({ savedPositions });
  } else {
    await syncGraphData();
  }
  // After initial layout, save positions and start listening for drags
  scheduleSavePositions();
  // Listen for node drag-end to auto-save
  const cy = currentCy;
  if (cy) {
    cy.on("free", "node", () => {
      scheduleSavePositions();
    });
  }
});

onBeforeUnmount(() => {
  if (saveTimer) clearTimeout(saveTimer);
  disposeDensity();
  dispose();
});

function focusNeighborhoodAction(nodeId: string) {
  focusNeighborhoodInternal(nodeId, { allowFocus: true, durationMs: 9_000 });
}

async function setLayout(type: LayoutType) {
  if (type === activeLayout.value) return;
  activeLayout.value = type;
  applyRendererStylesImpl();
  const savedPositions = await loadSavedPositions(type);
  if (savedPositions.size > 0) {
    await syncGraphDataImpl({ preserveViewport: true, savedPositions });
  } else {
    await syncGraphDataImpl({ preserveViewport: true, runLayout: true });
  }
  scheduleSavePositions();
}

function applyFilter() {
  void syncGraphData({ preserveViewport: true, runLayout: false });
}

async function exportImage(type: "png" | "svg"): Promise<string | undefined> {
  return exportCanvasImage(type);
}

async function resetLayout() {
  await clearTopologyNodePositions(activeLayout.value);
  await syncGraphDataImpl({ runLayout: true });
  scheduleSavePositions();
}

defineExpose({
  highlightPaths,
  highlightImpact,
  highlightSearch,
  focusNeighborhood: focusNeighborhoodAction,
  applyFilter,
  setLayout,
  exportImage,
  clearHighlight,
  resetLayout,
});
</script>

<template>
  <div class="topology-canvas">
    <div ref="containerRef" class="topology-canvas-surface" />
    <div
      v-if="viewportState.totalEdges > 0"
      class="canvas-density-hint"
      data-testid="canvas-density-hint"
    >
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
    radial-gradient(
      120% 90% at 10% 0%,
      color-mix(in srgb, var(--im-accent) 8%, transparent) 0%,
      transparent 58%
    ),
    radial-gradient(
      140% 110% at 100% 100%,
      color-mix(in srgb, var(--im-success) 8%, transparent) 0%,
      transparent 62%
    ),
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
