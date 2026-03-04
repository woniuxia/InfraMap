import { defineComponent, h } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import TopologyView from "@/views/TopologyView.vue";
import type { TopologyGraph } from "@/types";

const {
  fetchGraphMock,
  highlightSearchMock,
  clearHighlightMock,
} = vi.hoisted(() => ({
  fetchGraphMock: vi.fn(async () => null),
  highlightSearchMock: vi.fn(),
  clearHighlightMock: vi.fn(),
}));

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
    fetchGraph: fetchGraphMock,
  }),
}));

vi.mock("@/api/topology", () => ({
  findPaths: vi.fn(async () => ({ paths: [], truncated: false })),
  analyzeImpact: vi.fn(async () => ({ affected_nodes: [], total_count: 0, max_depth: 0 })),
}));

const TopologyControlBarStub = defineComponent({
  name: "TopologyControlBar",
  template: `
    <div data-testid="control-bar">
      <button data-testid="emit-search-hit" @click="$emit('search', { matchIds: ['node-1'], focusId: 'node-1' })">hit</button>
      <button data-testid="emit-search-empty" @click="$emit('search', { matchIds: [] })">empty</button>
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
  setup(_props, { expose }) {
    expose({
      highlightSearch: highlightSearchMock,
      clearHighlight: clearHighlightMock,
      setLayout: vi.fn(),
      exportImage: vi.fn(async () => "data:image/png;base64,xx"),
      highlightPaths: vi.fn(),
      highlightImpact: vi.fn(),
    });
    return () => h("div", { "data-testid": "topology-canvas" });
  },
});

const TopologyDetailPanelStub = defineComponent({
  name: "TopologyDetailPanel",
  template: `<div data-testid="detail-panel" />`,
});

function mountView() {
  fetchGraphMock.mockClear();
  highlightSearchMock.mockClear();
  clearHighlightMock.mockClear();

  return mount(TopologyView, {
    global: {
      stubs: {
        TopologyControlBar: TopologyControlBarStub,
        TopologyToolbar: TopologyToolbarStub,
        TopologyLegend: TopologyLegendStub,
        TopologyCanvas: TopologyCanvasStub,
        TopologyDetailPanel: TopologyDetailPanelStub,
        ElButton: true,
        ElEmpty: true,
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
});

