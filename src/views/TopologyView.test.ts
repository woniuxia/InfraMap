import { defineComponent, h } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import TopologyView from "@/views/TopologyView.vue";
import type { TopologyGraph } from "@/types";

const {
  storeState,
  fetchGraphMock,
  setTaskViewMock,
  setMaxDepthMock,
  getTopologyPathsV3Mock,
  getTopologyImpactV3Mock,
  getTopologyEvidenceV3Mock,
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
    taskInsights: [] as Array<{ kind: string; title: string; severity?: "info" | "warning" | "critical"; nodeIds?: string[] }>,
  },
  fetchGraphMock: vi.fn(async () => null),
  setTaskViewMock: vi.fn(),
  setMaxDepthMock: vi.fn(),
  getTopologyPathsV3Mock: vi.fn(async () => ({
    paths: [{ nodeIds: ["node-1"] }],
    truncated: false,
  })),
  getTopologyImpactV3Mock: vi.fn(async () => ({
    affectedNodes: [],
    totalCount: 0,
    maxDepth: 0,
  })),
  getTopologyEvidenceV3Mock: vi.fn(async () => ({
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
    { id: "prod", label: "生产", order: 0, node_count: 1, app_count: 1 },
    { id: "test", label: "测试", order: 1, node_count: 0, app_count: 0 },
    { id: "dev", label: "开发", order: 2, node_count: 0, app_count: 0 },
  ],
  nodes: [
    {
      id: "node-1",
      name: "订单服务",
      node_type: "application",
      env: "prod",
      group_kind: "application_service",
      importance: 1,
      extra: {
        address: "10.0.0.11",
      },
    },
  ],
  edges: [],
  legend_stats: {
    env_counts: [
      { env: "prod", count: 1, app_count: 1 },
      { env: "test", count: 0, app_count: 0 },
      { env: "dev", count: 0, app_count: 0 },
    ],
    node_type_counts: [{ kind: "application", count: 1 }],
    edge_type_counts: [],
    application_service_count: 1,
    current_env: "prod",
    external_node_count: 0,
    cross_env_edge_count: 0,
  },
  layout_hints: {
    lane_order: ["prod", "test", "dev"],
    default_collapsed_groups: [],
    high_density_mode: false,
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

vi.mock("@/api/topologyV3", () => ({
  getTopologyPathsV3: getTopologyPathsV3Mock,
  getTopologyImpactV3: getTopologyImpactV3Mock,
  getTopologyEvidenceV3: getTopologyEvidenceV3Mock,
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
  },
  template: `
    <div data-testid="control-bar">
      <span data-testid="control-layout">{{ layout }}</span>
      <button data-testid="emit-search-hit" @click="$emit('search', { matchIds: ['node-1'], focusId: 'node-1' })">hit</button>
      <button data-testid="emit-search-empty" @click="$emit('search', { matchIds: [] })">empty</button>
      <button data-testid="emit-focus-off" @click="$emit('focus-mode-change', false)">focus-off</button>
      <button data-testid="emit-layout-dagre" @click="$emit('layout-change', 'dagre')">layout-dagre</button>
      <button data-testid="emit-layout-force" @click="$emit('layout-change', 'force')">layout-force</button>
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
      h("div", { "data-testid": "topology-canvas", "data-layout": props.layout }, [
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
      ]);
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
  taskInsights?: Array<{ kind: string; title: string; severity?: "info" | "warning" | "critical"; nodeIds?: string[] }>;
}) {
  storeState.taskView = "explore";
  storeState.maxDepth = 3;
  storeState.taskInsights = options?.taskInsights || [];

  fetchGraphMock.mockClear();
  setTaskViewMock.mockClear();
  setMaxDepthMock.mockClear();
  getTopologyPathsV3Mock.mockClear();
  getTopologyImpactV3Mock.mockClear();
  getTopologyEvidenceV3Mock.mockClear();
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

  it("uses V3 paths api for path tracing", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-node-contextmenu"]').trigger("click");
    await wrapper.get('[data-testid="context-path-trace"]').trigger("click");
    await wrapper.get('[data-testid="emit-node-click"]').trigger("click");

    expect(getTopologyPathsV3Mock).toHaveBeenCalledWith({
      sourceId: "node-1",
      targetId: "node-1",
      taskView: "explore",
      maxDepth: 3,
    });
    expect(highlightPathsMock).toHaveBeenCalledWith([["node-1"]]);
  });

  it("uses V3 impact api for impact analysis", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-node-contextmenu"]').trigger("click");
    await wrapper.get('[data-testid="context-impact"]').trigger("click");

    expect(getTopologyImpactV3Mock).toHaveBeenCalledWith({
      nodeId: "node-1",
      taskView: "explore",
      maxDepth: 3,
    });
    expect(highlightImpactMock).toHaveBeenCalledWith("node-1", {
      affected_nodes: [],
      total_count: 0,
      max_depth: 0,
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
      taskInsights: [{
        kind: "dependency",
        title: "依赖关系摘要",
        severity: "warning",
        nodeIds: ["node-1"],
      }],
    });
    await flushPromises();

    await wrapper.get('[data-testid="task-insight-0"]').trigger("click");
    expect(highlightSearchMock).toHaveBeenCalledWith(["node-1"], "node-1");
    expect(getTopologyEvidenceV3Mock).toHaveBeenCalledWith({
      nodeId: "node-1",
      taskView: "explore",
      maxItems: 20,
    });
  });

  it("requests V3 evidence when opening node detail panel", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="emit-node-click"]').trigger("click");
    await flushPromises();

    expect(getTopologyEvidenceV3Mock).toHaveBeenCalledWith({
      nodeId: "node-1",
      taskView: "explore",
      maxItems: 20,
    });
  });
});
