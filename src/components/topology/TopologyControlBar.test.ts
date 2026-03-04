import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import type { TopologyLegendStats, TopologyNode } from "@/types";
import type { TopologyFilterState } from "@/components/topology/topologyGraph.utils";
import TopologyControlBar from "@/components/topology/TopologyControlBar.vue";

function createNode(partial: Partial<TopologyNode> & Pick<TopologyNode, "id" | "name">): TopologyNode {
  return {
    id: partial.id,
    name: partial.name,
    node_type: partial.node_type ?? "application",
    env: partial.env ?? "prod",
    group_kind: partial.group_kind ?? "application_service",
    importance: partial.importance ?? 1,
    extra: partial.extra,
  };
}

const statsFixture: TopologyLegendStats = {
  env_counts: [
    { env: "prod", count: 3, app_count: 2 },
    { env: "test", count: 1, app_count: 1 },
    { env: "dev", count: 0, app_count: 0 },
  ],
  node_type_counts: [
    { kind: "application", count: 2 },
    { kind: "middleware", count: 1 },
  ],
  edge_type_counts: [
    { kind: "http_call", count: 2 },
    { kind: "tcp", count: 1 },
  ],
  application_service_count: 2,
  current_env: "prod",
  external_node_count: 1,
  cross_env_edge_count: 1,
};

const filterFixture: TopologyFilterState = {
  env: "prod",
  nodeKinds: [],
  edgeTypes: [],
  showAllEdges: false,
};

function mountControlBar() {
  const nodes = [
    createNode({ id: "n-1", name: "订单服务", extra: { address: "10.0.0.11" } }),
    createNode({ id: "n-2", name: "订单网关", node_type: "nginx", group_kind: "nginx", extra: { ip: "10.0.0.21" } }),
    createNode({ id: "n-3", name: "Redis", node_type: "middleware", group_kind: "middleware" }),
  ];

  return mount(TopologyControlBar, {
    props: {
      nodes,
      stats: statsFixture,
      filter: filterFixture,
    },
  });
}

describe("TopologyControlBar", () => {
  it("debounces search and cycles focus with Enter", async () => {
    vi.useFakeTimers();
    const wrapper = mountControlBar();

    const input = wrapper.get('[data-testid="topology-search-input"]');
    await input.setValue("订单");
    await input.trigger("input");

    expect(wrapper.emitted("search")).toBeFalsy();

    vi.advanceTimersByTime(299);
    expect(wrapper.emitted("search")).toBeFalsy();

    vi.advanceTimersByTime(1);
    const firstPayload = wrapper.emitted("search")?.[0]?.[0] as { matchIds: string[]; focusId?: string };
    expect(firstPayload.matchIds).toEqual(["n-1", "n-2"]);
    expect(firstPayload.focusId).toBe("n-1");

    await input.trigger("keydown.enter");
    const secondPayload = wrapper.emitted("search")?.[1]?.[0] as { matchIds: string[]; focusId?: string };
    expect(secondPayload.matchIds).toEqual(["n-1", "n-2"]);
    expect(secondPayload.focusId).toBe("n-2");

    vi.useRealTimers();
  });

  it("emits filter changes for env, node type, edge type and showAllEdges", async () => {
    const wrapper = mountControlBar();

    await wrapper.get('[data-testid="env-test"]').trigger("click");
    expect(wrapper.emitted("filter-change")?.[0]?.[0]).toEqual({ env: "test" });

    await wrapper.get('[data-testid="node-kind-middleware"]').trigger("click");
    expect(wrapper.emitted("filter-change")?.[1]?.[0]).toEqual({ nodeKinds: ["middleware"] });

    await wrapper.get('[data-testid="edge-kind-tcp"]').trigger("click");
    expect(wrapper.emitted("filter-change")?.[2]?.[0]).toEqual({ edgeTypes: ["tcp"] });

    await wrapper.get('[data-testid="show-all-edges"]').setValue(true);
    expect(wrapper.emitted("filter-change")?.[3]?.[0]).toEqual({ showAllEdges: true });
  });

  it("emits layout, export and action events", async () => {
    const wrapper = mountControlBar();

    await wrapper.get('[data-testid="layout-force"]').trigger("click");
    expect(wrapper.emitted("layout-change")?.[0]?.[0]).toBe("force");

    await wrapper.get('[data-testid="export-png"]').trigger("click");
    await wrapper.get('[data-testid="export-svg"]').trigger("click");
    expect(wrapper.emitted("export")?.map((item) => item[0])).toEqual(["png", "svg"]);

    await wrapper.get('[data-testid="action-refresh"]').trigger("click");
    await wrapper.get('[data-testid="action-fullscreen"]').trigger("click");

    expect(wrapper.emitted("refresh")).toHaveLength(1);
    expect(wrapper.emitted("fullscreen")).toHaveLength(1);
  });
});

