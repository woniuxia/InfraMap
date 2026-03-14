import { nextTick, ref } from "vue";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { TopologyGraph } from "@/types";

const { capturedCores } = vi.hoisted(() => ({
  capturedCores: [] as Array<Record<string, unknown>>,
}));

vi.mock("cytoscape-dagre", () => ({
  default: vi.fn(),
}));

vi.mock("cytoscape-fcose", () => ({
  default: vi.fn(),
}));

vi.mock("cytoscape-svg", () => ({
  default: vi.fn(),
}));

vi.mock("cytoscape", () => {
  const cytoscapeMock = Object.assign(
    vi.fn(() => {
      const core = {
        on: vi.fn(() => core),
        zoom: vi.fn(() => 1),
        pan: vi.fn(() => ({ x: 0, y: 0 })),
        style: vi.fn(),
        elements: vi.fn(() => ({
          remove: vi.fn(),
          nonempty: () => false,
        })),
        batch: vi.fn((callback: () => void) => callback()),
        add: vi.fn(() => ({ empty: () => true })),
        nodes: vi.fn(() => ({
          not: vi.fn(() => ({
            map: vi.fn(() => []),
            forEach: vi.fn(),
          })),
        })),
        fit: vi.fn(),
        resize: vi.fn(),
        destroy: vi.fn(),
        png: vi.fn(() => "data:image/png;base64,mock"),
        svg: vi.fn(() => "<svg></svg>"),
      };
      capturedCores.push(core);
      return core;
    }),
    {
      use: vi.fn(),
    },
  );

  return {
    default: cytoscapeMock,
  };
});

import { useTopologyRenderer } from "@/components/topology/useTopologyRenderer";

function createGraph(): TopologyGraph {
  return {
    lanes: [{ id: "prod", label: "生产", order: 0, nodeCount: 1, appCount: 1 }],
    nodes: [
      {
        id: "node-1",
        name: "订单服务",
        nodeType: "service",
        env: "prod",
        groupKind: "service",
        importance: 1,
      },
    ],
    edges: [],
    legendStats: {
      envCounts: [{ env: "prod", count: 1, appCount: 1 }],
      nodeTypeCounts: [{ kind: "service", count: 1 }],
      edgeTypeCounts: [],
      serviceCount: 1,
      currentEnv: "prod",
      externalNodeCount: 0,
      crossEnvEdgeCount: 0,
    },
    layoutHints: {
      laneOrder: ["prod", "test", "dev"],
      defaultCollapsedGroups: [],
      highDensityMode: false,
    },
  };
}

describe("useTopologyRenderer", () => {
  beforeEach(() => {
    capturedCores.length = 0;
    vi.stubGlobal(
      "ResizeObserver",
      class {
        observe = vi.fn();
        disconnect = vi.fn();
      },
    );
    vi.stubGlobal(
      "MutationObserver",
      class {
        observe = vi.fn();
        disconnect = vi.fn();
      },
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("creates core, exports svg, and destroys core on dispose", async () => {
    const activeLayout = ref<"force" | "dagre">("force");
    const viewportState = ref({
      zoom: 1,
      density: "medium" as const,
      totalEdges: 0,
      visibleEdges: 0,
    });
    const containerRef = ref<HTMLDivElement>();
    let currentCy: Record<string, unknown> | null = null;

    const renderer = useTopologyRenderer({
      getCy: () => currentCy,
      setCy: (cy) => {
        currentCy = cy as Record<string, unknown> | null;
      },
      getContainer: () => containerRef.value,
      getGraphData: () => createGraph(),
      getLayout: () => activeLayout.value,
      setLayout: (layout) => {
        activeLayout.value = layout;
      },
      performanceOptimizationEnabled: () => false,
      viewportState,
      resolveZoomDensity: (zoom) => (zoom < 0.8 ? "overview" : "medium"),
      getActiveDensity: () => "medium",
      buildDensityGraphData: async (graph) => graph,
      rebuildIndexes: vi.fn(),
      snapshotLeafNodePositions: () => new Map(),
      restoreLeafNodePositions: vi.fn(),
      runLayout: vi.fn(async () => ({ requested: "force", applied: "force" })),
      applyHighlightState: vi.fn(),
      emitLayoutResolved: vi.fn(),
      handleCanvasTap: vi.fn(),
      handleNodeTap: vi.fn(),
      handleNodeContextTap: vi.fn(),
      handleViewportZoomChange: vi.fn(),
      onRenderFlagsChange: vi.fn(),
    });

    containerRef.value = document.createElement("div");

    await renderer.syncGraphData();
    await nextTick();

    expect(capturedCores).toHaveLength(1);

    const svgUri = await renderer.exportImage("svg");
    expect(svgUri).toContain("data:image/svg+xml;charset=utf-8,");

    renderer.dispose();

    expect(capturedCores[0]?.destroy).toHaveBeenCalledTimes(1);
  });
});
