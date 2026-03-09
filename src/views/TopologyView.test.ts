import { defineComponent, h } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import TopologyView from "@/views/TopologyView.vue";
import type { TopologyGraph } from "@/types";

const {
  storeState,
  fetchGraphMock,
  setTaskViewMock,
  setMaxDepthMock,
  getTopologyPathsMock,
  getTopologyImpactMock,
  getTopologyEvidenceMock,
  getApplicationMock,
  getMiddlewareMock,
  getNginxConfigMock,
  highlightSearchMock,
  clearHighlightMock,
  highlightPathsMock,
  highlightImpactMock,
} = vi.hoisted(() => ({
  storeState: {
    taskView: "explore",
    maxDepth: 3,
    taskInsights: [] as Array<{
      kind: string;
      title: string;
      severity?: "info" | "warning" | "critical";
      nodeIds?: string[];
    }>,
  },
  fetchGraphMock: vi.fn(async () => null),
  setTaskViewMock: vi.fn(),
  setMaxDepthMock: vi.fn(),
  getTopologyPathsMock: vi.fn(async () => ({
    paths: [{ nodeIds: ["node-1"] }],
    truncated: false,
  })),
  getTopologyImpactMock: vi.fn(async () => ({
    affectedNodes: [],
    totalCount: 0,
    maxDepth: 0,
  })),
  getTopologyEvidenceMock: vi.fn(async () => ({
    items: [],
    total: 0,
  })),
  getApplicationMock: vi.fn(async () => ({
    id: "node-1",
    name: "订单服务",
    type: "backend",
    env: "prod",
    status: "running",
    created_at: "",
    updated_at: "",
  })),
  getMiddlewareMock: vi.fn(async () => ({
    id: "mw-1",
    name: "redis-main",
    category: "cache",
    type: "redis",
    address: "10.0.0.1",
    env: "prod",
    created_at: "",
    updated_at: "",
  })),
  getNginxConfigMock: vi.fn(async () => ({
    id: "ng-1",
    name: "nginx-main",
    endpoints: [{ host: "10.0.0.2", port: 80 }],
    env: "prod",
    status: "running",
    created_at: "",
    updated_at: "",
  })),
  highlightSearchMock: vi.fn(),
  clearHighlightMock: vi.fn(),
  highlightPathsMock: vi.fn(),
  highlightImpactMock: vi.fn(),
}));

setTaskViewMock.mockImplementation((taskView: string) => {
  storeState.taskView = taskView;
});

setMaxDepthMock.mockImplementation((maxDepth: number) => {
  storeState.maxDepth = maxDepth;
});

const graphFixture: TopologyGraph = {
  lanes: [
    { id: "prod", label: "生产", order: 0, nodeCount: 1, appCount: 1 },
    { id: "test", label: "测试", order: 1, nodeCount: 0, appCount: 0 },
    { id: "dev", label: "开发", order: 2, nodeCount: 0, appCount: 0 },
  ],
  nodes: [
    {
      id: "node-1",
      name: "订单服务",
      nodeType: "application",
      env: "prod",
      groupKind: "application_service",
      importance: 1,
      extra: {
        address: "10.0.0.11",
      },
    },
  ],
  edges: [],
  legendStats: {
    envCounts: [
      { env: "prod", count: 1, appCount: 1 },
      { env: "test", count: 0, appCount: 0 },
      { env: "dev", count: 0, appCount: 0 },
    ],
    nodeTypeCounts: [{ kind: "application", count: 1 }],
    edgeTypeCounts: [],
    applicationServiceCount: 1,
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

vi.mock("@/stores/topology", () => ({
  useTopologyStore: () => ({
    graphData: graphFixture,
    loading: false,
    get taskView() {
      return storeState.taskView;
    },
    get maxDepth() {
      return storeState.maxDepth;
    },
    get taskInsights() {
      return storeState.taskInsights;
    },
    fetchGraph: fetchGraphMock,
    setTaskView: setTaskViewMock,
    setMaxDepth: setMaxDepthMock,
  }),
}));

vi.mock("@/api/topology", () => ({
  getTopologyPaths: getTopologyPathsMock,
  getTopologyImpact: getTopologyImpactMock,
  getTopologyEvidence: getTopologyEvidenceMock,
}));

vi.mock("@/api/applications", () => ({
  getApplication: getApplicationMock,
}));

vi.mock("@/api/middlewares", () => ({
  getMiddleware: getMiddlewareMock,
}));

vi.mock("@/api/nginx-configs", () => ({
  getNginxConfig: getNginxConfigMock,
}));

const TopologyControlBarStub = defineComponent({
  name: "TopologyControlBar",
  props: {
    layout: {
      type: String,
      default: "force",
    },
    performanceOptimizationEnabled: {
      type: Boolean,
      default: false,
    },
    filter: {
      type: Object,
      default: () => ({
        showAllEdges: false,
      }),
    },
  },
  template: `
    <div data-testid="control-bar">
      <span data-testid="control-layout">{{ layout }}</span>
      <span data-testid="control-perf-opt">{{ performanceOptimizationEnabled ? 'on' : 'off' }}</span>
      <span data-testid="control-show-all-edges">{{ filter.showAllEdges ? 'on' : 'off' }}</span>
      <button data-testid="emit-search-hit" @click="$emit('search', { matchIds: ['node-1'], focusId: 'node-1' })">hit</button>
      <button data-testid="emit-search-empty" @click="$emit('search', { matchIds: [] })">empty</button>
      <button data-testid="emit-focus-off" @click="$emit('focus-mode-change', false)">focus-off</button>
      <button data-testid="emit-layout-dagre" @click="$emit('layout-change', 'dagre')">layout-dagre</button>
      <button data-testid="emit-layout-force" @click="$emit('layout-change', 'force')">layout-force</button>
      <button data-testid="emit-perf-opt-on" @click="$emit('performance-optimization-change', true)">perf-on</button>
      <button data-testid="emit-perf-opt-off" @click="$emit('performance-optimization-change', false)">perf-off</button>
    </div>
  `,
});

const TopologyToolbarStub = defineComponent({
  name: "TopologyToolbar",
  template: `<div data-testid="legacy-toolbar" />`,
});

const TopologyLegendStub = defineComponent({
  name: "TopologyLegend",
  template: `<div data-testid="legacy-legend" />`,
});

const TopologyCanvasStub = defineComponent({
  name: "TopologyCanvas",
  props: {
    layout: {
      type: String,
      default: "force",
    },
    performanceOptimizationEnabled: {
      type: Boolean,
      default: false,
    },
  },
  setup(props, { expose, emit }) {
    const sampleNode = graphFixture.nodes[0];

    expose({
      highlightSearch: highlightSearchMock,
      clearHighlight: clearHighlightMock,
      focusNeighborhood: vi.fn(),
      setLayout: vi.fn(),
      exportImage: vi.fn(async () => "data:image/png;base64,xx"),
      highlightPaths: highlightPathsMock,
      highlightImpact: highlightImpactMock,
    });

    return () =>
      h(
        "div",
        {
          "data-testid": "topology-canvas",
          "data-layout": props.layout,
          "data-perf-opt": props.performanceOptimizationEnabled ? "on" : "off",
        },
        [
          h(
            "button",
            {
              "data-testid": "emit-node-click",
              onClick: (event: MouseEvent) => {
                event.stopPropagation();
                emit("node-click", sampleNode);
              },
            },
            "emit-node-click",
          ),
          h(
            "button",
            {
              "data-testid": "emit-node-contextmenu",
              onClick: (event: MouseEvent) => {
                event.stopPropagation();
                emit("node-contextmenu", { node: sampleNode, x: 120, y: 160 });
              },
            },
            "emit-node-contextmenu",
          ),
          h(
            "button",
            {
              "data-testid": "emit-canvas-blank-click",
              onClick: (event: MouseEvent) => {
                event.stopPropagation();
                emit("canvas-blank-click");
              },
            },
            "emit-canvas-blank-click",
          ),
          h(
            "button",
            {
              "data-testid": "emit-layout-resolved-fallback",
              onClick: (event: MouseEvent) => {
                event.stopPropagation();
                emit("layout-resolved", {
                  requested: "dagre",
                  applied: "force",
                  reason: "graph-density",
                });
              },
            },
            "emit-layout-resolved-fallback",
          ),
        ],
      );
  },
});

const TopologyDetailPanelStub = defineComponent({
  name: "TopologyDetailPanel",
  template: `<div data-testid="detail-panel" />`,
});

const ApplicationEditorDialogStub = defineComponent({
  name: "ApplicationEditorDialog",
  props: {
    modelValue: {
      type: Boolean,
      default: false,
    },
  },
  emits: ["saved", "update:modelValue"],
  template: `
    <div v-if="modelValue" data-testid="application-editor-dialog">
      <button data-testid="emit-application-editor-saved" @click="$emit('saved', { id: 'node-1', mode: 'edit' })">
        emit-application-editor-saved
      </button>
    </div>
  `,
});

const MiddlewareEditorDialogStub = defineComponent({
  name: "MiddlewareEditorDialog",
  template: `<div data-testid="middleware-editor-dialog" />`,
});

const NginxConfigEditorDialogStub = defineComponent({
  name: "NginxConfigEditorDialog",
  template: `<div data-testid="nginx-editor-dialog" />`,
});

function mountView(options?: {
  taskInsights?: Array<{
    kind: string;
    title: string;
    severity?: "info" | "warning" | "critical";
    nodeIds?: string[];
  }>;
}) {
  storeState.taskView = "explore";
  storeState.maxDepth = 3;
  storeState.taskInsights = options?.taskInsights || [];

  fetchGraphMock.mockClear();
  setTaskViewMock.mockClear();
  setMaxDepthMock.mockClear();
  getTopologyPathsMock.mockClear();
  getTopologyImpactMock.mockClear();
  getTopologyEvidenceMock.mockClear();
  getApplicationMock.mockClear();
  getMiddlewareMock.mockClear();
  getNginxConfigMock.mockClear();
  highlightSearchMock.mockClear();
  clearHighlightMock.mockClear();
  highlightPathsMock.mockClear();
  highlightImpactMock.mockClear();

  return mount(TopologyView, {
    global: {
      stubs: {
        TopologyControlBar: TopologyControlBarStub,
        TopologyToolbar: TopologyToolbarStub,
        TopologyLegend: TopologyLegendStub,
        TopologyCanvas: TopologyCanvasStub,
        TopologyDetailPanel: TopologyDetailPanelStub,
        ApplicationEditorDialog: ApplicationEditorDialogStub,
        MiddlewareEditorDialog: MiddlewareEditorDialogStub,
        NginxConfigEditorDialog: NginxConfigEditorDialogStub,
        ElButton: true,
        ElEmpty: true,
        Teleport: true,
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

describe("TopologyView", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("defaults performance optimization to off and forces showAllEdges in control filter", async () => {
    localStorage.removeItem("inframap.topology.performanceOptimizationEnabled");
    const wrapper = mountView();
    await flushPromises();

    expect(wrapper.get('[data-testid="control-perf-opt"]').text()).toBe("off");
    expect(wrapper.get('[data-testid="control-show-all-edges"]').text()).toBe("on");
    expect(wrapper.get('[data-testid="topology-canvas"]').attributes("data-perf-opt")).toBe("off");
  });

  it("restores performance optimization preference from localStorage", async () => {
    localStorage.setItem("inframap.topology.performanceOptimizationEnabled", "true");
    const wrapper = mountView();
    await flushPromises();

    expect(wrapper.get('[data-testid="control-perf-opt"]').text()).toBe("on");
    expect(wrapper.get('[data-testid="topology-canvas"]').attributes("data-perf-opt")).toBe("on");
  });

  it("persists performance optimization changes from control bar", async () => {
    localStorage.removeItem("inframap.topology.performanceOptimizationEnabled");
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-perf-opt-on"]').trigger("click");
    await flushPromises();

    expect(localStorage.getItem("inframap.topology.performanceOptimizationEnabled")).toBe("true");
    expect(wrapper.get('[data-testid="control-perf-opt"]').text()).toBe("on");
    expect(wrapper.get('[data-testid="topology-canvas"]').attributes("data-perf-opt")).toBe("on");
  });

  it("renders new unified control bar and removes legacy toolbar/legend", async () => {
    const wrapper = mountView();
    await flushPromises();

    expect(wrapper.find('[data-testid="control-bar"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="legacy-toolbar"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="legacy-legend"]').exists()).toBe(false);
  });

  it("forwards search events from control bar to canvas highlight api", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-search-hit"]').trigger("click");
    expect(highlightSearchMock).toHaveBeenCalledWith(["node-1"], "node-1");

    await wrapper.get('[data-testid="emit-search-empty"]').trigger("click");
    expect(clearHighlightMock).toHaveBeenCalled();
  });

  it("clears canvas highlight when focus mode is disabled", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-focus-off"]').trigger("click");
    expect(clearHighlightMock).toHaveBeenCalled();
  });

  it("clears selected node when canvas blank area is clicked", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-node-click"]').trigger("click");
    await flushPromises();
    clearHighlightMock.mockClear();

    await wrapper.get('[data-testid="emit-canvas-blank-click"]').trigger("click");
    expect(clearHighlightMock).toHaveBeenCalledTimes(1);
  });

  it("does nothing on canvas blank click when no node is selected", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-canvas-blank-click"]').trigger("click");
    expect(clearHighlightMock).not.toHaveBeenCalled();
  });

  it("syncs selected layout between control bar and canvas, including fallback resolve", async () => {
    const wrapper = mountView();
    await flushPromises();

    expect(wrapper.get('[data-testid="control-layout"]').text()).toBe("force");
    expect(wrapper.get('[data-testid="topology-canvas"]').attributes("data-layout")).toBe("force");

    await wrapper.get('[data-testid="emit-layout-dagre"]').trigger("click");
    await flushPromises();

    expect(wrapper.get('[data-testid="control-layout"]').text()).toBe("dagre");
    expect(wrapper.get('[data-testid="topology-canvas"]').attributes("data-layout")).toBe("dagre");

    await wrapper.get('[data-testid="emit-layout-resolved-fallback"]').trigger("click");
    await flushPromises();

    expect(wrapper.get('[data-testid="control-layout"]').text()).toBe("force");
    expect(wrapper.get('[data-testid="topology-canvas"]').attributes("data-layout")).toBe("force");
  });

  it("switches task view and refreshes topology snapshot", async () => {
    const wrapper = mountView();
    await flushPromises();

    expect(fetchGraphMock).toHaveBeenCalledTimes(1);

    await wrapper.get('[data-testid="task-view-impact"]').trigger("click");

    expect(setTaskViewMock).toHaveBeenCalledWith("impact");
    expect(fetchGraphMock).toHaveBeenCalledTimes(2);
  });

  it("updates maxDepth and refreshes topology snapshot", async () => {
    const wrapper = mountView();
    await flushPromises();

    expect(fetchGraphMock).toHaveBeenCalledTimes(1);

    const input = wrapper.get('[data-testid="max-depth-number"]');
    await input.setValue("5");
    await input.trigger("change");

    expect(setMaxDepthMock).toHaveBeenCalledWith(5);
    expect(fetchGraphMock).toHaveBeenCalledTimes(2);
  });

  it("uses topology paths api for path tracing", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-node-contextmenu"]').trigger("click");
    await wrapper.get('[data-testid="context-path-trace"]').trigger("click");
    await wrapper.get('[data-testid="emit-node-click"]').trigger("click");

    expect(getTopologyPathsMock).toHaveBeenCalledWith({
      sourceId: "node-1",
      targetId: "node-1",
      taskView: "explore",
      maxDepth: 3,
    });
    expect(highlightPathsMock).toHaveBeenCalledWith([["node-1"]]);
  });

  it("uses topology impact api for impact analysis", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-node-contextmenu"]').trigger("click");
    await wrapper.get('[data-testid="context-impact"]').trigger("click");

    expect(getTopologyImpactMock).toHaveBeenCalledWith({
      nodeId: "node-1",
      taskView: "explore",
      maxDepth: 3,
    });
    expect(highlightImpactMock).toHaveBeenCalledWith("node-1", {
      affectedNodes: [],
      totalCount: 0,
      maxDepth: 0,
    });
  });

  it("opens editor from context menu and refreshes topology after save", async () => {
    const wrapper = mountView();
    await flushPromises();

    expect(fetchGraphMock).toHaveBeenCalledTimes(1);

    await wrapper.get('[data-testid="emit-node-contextmenu"]').trigger("click");
    await wrapper.get('[data-testid="context-edit"]').trigger("click");
    await flushPromises();

    expect(getApplicationMock).toHaveBeenCalledWith("node-1");
    expect(wrapper.find('[data-testid="application-editor-dialog"]').exists()).toBe(true);

    await wrapper.get('[data-testid="emit-application-editor-saved"]').trigger("click");
    await flushPromises();

    expect(fetchGraphMock).toHaveBeenCalledTimes(2);
  });

  it("highlights related nodes when task insight chip is clicked", async () => {
    const wrapper = mountView({
      taskInsights: [
        {
          kind: "dependency",
          title: "依赖关系摘要",
          severity: "warning",
          nodeIds: ["node-1"],
        },
      ],
    });
    await flushPromises();

    await wrapper.get('[data-testid="task-insight-0"]').trigger("click");
    expect(highlightSearchMock).toHaveBeenCalledWith(["node-1"], "node-1");
    expect(getTopologyEvidenceMock).toHaveBeenCalledWith({
      nodeId: "node-1",
      taskView: "explore",
      maxItems: 20,
    });
  });

  it("requests topology evidence when opening node detail panel", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-node-click"]').trigger("click");
    await flushPromises();

    expect(getTopologyEvidenceMock).toHaveBeenCalledWith({
      nodeId: "node-1",
      taskView: "explore",
      maxItems: 20,
    });
  });
});
