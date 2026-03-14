import { nextTick, type Ref } from "vue";
import cytoscape, { type Core, type EventObject, type EventObjectNode } from "cytoscape";
import dagre from "cytoscape-dagre";
import fcose from "cytoscape-fcose";
import cytoscapeSvg from "cytoscape-svg";
import type { TopologyGraph } from "@/types";
import { buildTopologyCyElements } from "@/components/topology/topologyCytoscape.utils";
import { buildNodeIconSpriteDataUri } from "@/icons/nodeIconSprite";
import { DENSITY_OPTIONS, type ZoomDensity } from "@/components/topology/topologyDensity.utils";
import {
  type TopologyLayoutRunResult,
  type TopologyLayoutType,
} from "@/components/topology/useTopologyLayout";
import { hasTopologyNodeSetChanged } from "@/components/topology/topologyLayoutSafety.utils";

interface ViewportStateValue {
  zoom: number;
  density: ZoomDensity;
  totalEdges: number;
  visibleEdges: number;
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

export interface TopologyRendererSyncOptions {
  preserveViewport?: boolean;
  runLayout?: boolean;
}

interface UseTopologyRendererOptions {
  getCy: () => Core | null;
  setCy: (cy: Core | null) => void;
  getContainer: () => HTMLDivElement | undefined;
  getGraphData: () => TopologyGraph | null;
  getLayout: () => TopologyLayoutType;
  setLayout: (layout: TopologyLayoutType) => void;
  performanceOptimizationEnabled: () => boolean;
  viewportState: Ref<ViewportStateValue>;
  resolveZoomDensity: (zoom: number) => ZoomDensity;
  getActiveDensity: () => ZoomDensity;
  buildDensityGraphData: (graph: TopologyGraph, density: ZoomDensity) => Promise<TopologyGraph>;
  rebuildIndexes: (graph: TopologyGraph | null) => void;
  snapshotLeafNodePositions: () => Map<string, { x: number; y: number }>;
  restoreLeafNodePositions: (positions: Map<string, { x: number; y: number }>) => void;
  runLayout: (layout: TopologyLayoutType) => Promise<TopologyLayoutRunResult>;
  applyHighlightState: (options: { allowFocus: boolean }) => void;
  emitLayoutResolved: (payload: TopologyLayoutRunResult) => void;
  handleCanvasTap: (event: EventObject) => void;
  handleNodeTap: (event: EventObjectNode) => void;
  handleNodeContextTap: (event: EventObjectNode) => void;
  handleViewportZoomChange: (zoom: number) => void;
  onRenderFlagsChange: (flags: { isLargeGraph: boolean; hideEdgeLabels: boolean }) => void;
}

let cytoscapeExtensionsRegistered = false;

function registerCytoscapeExtensions() {
  if (cytoscapeExtensionsRegistered) return;
  cytoscape.use(dagre);
  cytoscape.use(fcose);
  cytoscapeSvg(cytoscape);
  cytoscapeExtensionsRegistered = true;
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

function resolveEdgeDash(data: Record<string, unknown>, theme: GraphTheme): number[] | undefined {
  if (data.crossEnv) return [4, 3];
  const edgeType = String(data.edgeType || "http_call");
  return theme.edgeStyles[edgeType]?.lineDash;
}

function resolveEdgeCurveStyle(
  data: Record<string, unknown>,
  runtime: { dense: boolean; layout: TopologyLayoutType },
): string {
  if (runtime.layout === "dagre") return "taxi";
  if (runtime.dense) return "straight";
  if (data.crossEnv) return "unbundled-bezier";
  return "bezier";
}

function buildStylesheet(
  theme: GraphTheme,
  runtime: { dense: boolean; layout: TopologyLayoutType },
) {
  const labelBg = withAlpha(theme.labelBg, runtime.dense ? "B8" : "D1");
  return [
    {
      selector: "core",
      style: {
        "selection-box-color": withAlpha(theme.highlight, "44"),
        "selection-box-border-color": withAlpha(theme.highlight, "C8"),
        "selection-box-opacity": 1,
      },
    },
    {
      selector: "node[shape][size][label_font_size]",
      style: {
        shape: "data(shape)",
        width: "data(size)",
        height: "data(size)",
        label: "data(label)",
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

export function useTopologyRenderer(options: UseTopologyRendererOptions) {
  let resizeObserver: ResizeObserver | null = null;
  let themeObserver: MutationObserver | null = null;
  let resizeAnimationFrame: number | null = null;
  let syncVersion = 0;
  let graphTheme: GraphTheme | null = null;
  let hasRenderedGraph = false;
  let pendingInitialResizeRefit = false;
  let hasUserAdjustedViewport = false;

  const renderFlags = {
    isLargeGraph: false,
    hideEdgeLabels: false,
  };

  function getTheme(): GraphTheme {
    if (!graphTheme) graphTheme = buildGraphTheme();
    return graphTheme;
  }

  function refreshTheme() {
    graphTheme = buildGraphTheme();
  }

  function markViewportAsUserAdjusted(event?: { originalEvent?: unknown }) {
    if (!event?.originalEvent) return;
    hasUserAdjustedViewport = true;
    pendingInitialResizeRefit = false;
  }

  function handleViewportTransform(event?: { originalEvent?: unknown }) {
    const cy = options.getCy();
    if (!cy) return;
    markViewportAsUserAdjusted(event);
    options.handleViewportZoomChange(cy.zoom());
  }

  function handleViewportPan(event?: { originalEvent?: unknown }) {
    markViewportAsUserAdjusted(event);
  }

  function applyStyles() {
    const cy = options.getCy();
    if (!cy) return;
    const dense =
      options.performanceOptimizationEnabled() &&
      (renderFlags.hideEdgeLabels || options.viewportState.value.visibleEdges > 140);
    cy.style(
      buildStylesheet(getTheme(), {
        dense,
        layout: options.getLayout(),
      }) as unknown as cytoscape.StylesheetJsonBlock[],
    );
  }

  async function ensureCy() {
    const container = options.getContainer();
    if (!container || options.getCy()) return;

    registerCytoscapeExtensions();
    refreshTheme();

    const cy = cytoscape({
      container,
      elements: [],
      style: buildStylesheet(getTheme(), {
        dense: false,
        layout: options.getLayout(),
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

    cy.on("tap", "node", (event) => options.handleNodeTap(event as EventObjectNode));
    cy.on("tap", (event) => options.handleCanvasTap(event as EventObject));
    cy.on("cxttap", "node", (event) => options.handleNodeContextTap(event as EventObjectNode));
    cy.on("zoom", handleViewportTransform);
    cy.on("pan", handleViewportPan);

    options.setCy(cy);
    options.viewportState.value.zoom = cy.zoom();
    options.viewportState.value.density = options.resolveZoomDensity(cy.zoom());
  }

  async function syncGraphData(syncOptions: TopologyRendererSyncOptions = {}) {
    const currentVersion = ++syncVersion;
    await nextTick();

    if (!options.getContainer()) return;
    await ensureCy();
    const cy = options.getCy();
    if (!cy || currentVersion !== syncVersion) return;

    const preservedZoom = syncOptions.preserveViewport ? cy.zoom() : null;
    const preservedPan = syncOptions.preserveViewport ? { ...cy.pan() } : null;
    const preservedNodePositions = options.snapshotLeafNodePositions();
    const previousNodeIds = new Set(
      cy
        .nodes()
        .not(":parent")
        .map((node) => node.id()),
    );

    const graphData = options.getGraphData();
    if (!graphData) {
      options.viewportState.value.totalEdges = 0;
      options.viewportState.value.visibleEdges = 0;
      pendingInitialResizeRefit = false;
      options.rebuildIndexes(null);
      cy.elements().remove();
      hasRenderedGraph = false;
      return;
    }

    const density = options.getActiveDensity();
    const densityGraph = await options.buildDensityGraphData(graphData, density);
    const latestCy = options.getCy();
    if (!latestCy || currentVersion !== syncVersion) return;

    options.viewportState.value.totalEdges = graphData.edges.length;
    options.viewportState.value.visibleEdges = densityGraph.edges.length;
    renderFlags.isLargeGraph = densityGraph.nodes.length > 500;
    renderFlags.hideEdgeLabels = options.performanceOptimizationEnabled()
      ? DENSITY_OPTIONS[density].hideEdgeLabels ||
        graphData.layoutHints.highDensityMode ||
        densityGraph.edges.length > 90
      : false;
    options.onRenderFlagsChange({ ...renderFlags });
    applyStyles();

    const elements = buildTopologyCyElements(densityGraph, {
      density,
      hideEdgeLabels: renderFlags.hideEdgeLabels,
      isLargeGraph: renderFlags.isLargeGraph,
    });

    options.rebuildIndexes(densityGraph);

    latestCy.batch(() => {
      latestCy.elements().remove();
      latestCy.add(elements);
    });

    let shouldRunLayout = syncOptions.runLayout ?? !hasRenderedGraph;
    if (!shouldRunLayout && hasTopologyNodeSetChanged(previousNodeIds, densityGraph.nodes)) {
      shouldRunLayout = true;
    }

    let layoutResult: TopologyLayoutRunResult | null = null;
    if (shouldRunLayout) {
      layoutResult = await options.runLayout(options.getLayout());
    } else {
      options.restoreLeafNodePositions(preservedNodePositions);
    }

    if (layoutResult && (layoutResult.reason || layoutResult.applied !== layoutResult.requested)) {
      options.emitLayoutResolved(layoutResult);
    }

    if (syncOptions.preserveViewport && preservedZoom !== null && preservedPan) {
      latestCy.zoom(preservedZoom);
      latestCy.pan(preservedPan);
    } else if (!hasRenderedGraph && latestCy.elements().nonempty()) {
      latestCy.fit(undefined, 40);
      pendingInitialResizeRefit = true;
      hasUserAdjustedViewport = false;
    }

    hasRenderedGraph = true;
    options.applyHighlightState({ allowFocus: false });
  }

  function mount() {
    if (options.getContainer()) {
      resizeObserver = new ResizeObserver(() => {
        const cy = options.getCy();
        if (!cy || !options.getContainer()) return;
        if (resizeAnimationFrame) {
          cancelAnimationFrame(resizeAnimationFrame);
        }
        resizeAnimationFrame = requestAnimationFrame(() => {
          resizeAnimationFrame = null;
          const latestCy = options.getCy();
          if (!latestCy || !options.getContainer()) return;
          latestCy.resize();
          if (
            pendingInitialResizeRefit &&
            !hasUserAdjustedViewport &&
            latestCy.elements().nonempty()
          ) {
            latestCy.fit(undefined, 40);
            pendingInitialResizeRefit = false;
          }
        });
      });
      resizeObserver.observe(options.getContainer()!);
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
  }

  function dispose() {
    resizeObserver?.disconnect();
    themeObserver?.disconnect();

    if (resizeAnimationFrame) {
      cancelAnimationFrame(resizeAnimationFrame);
      resizeAnimationFrame = null;
    }

    const cy = options.getCy();
    if (cy) {
      cy.destroy();
      options.setCy(null);
    }

    themeObserver = null;
  }

  async function exportImage(type: "png" | "svg"): Promise<string | undefined> {
    const cy = options.getCy();
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

  return {
    applyStyles,
    syncGraphData,
    mount,
    dispose,
    exportImage,
  };
}
