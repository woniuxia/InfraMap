import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import type { TopologyLegendStats, TopologyNode } from "@/types";
import type { TopologyFilterState } from "@/components/topology/topologyGraph.utils";
import TopologyControlBar from "@/components/topology/TopologyControlBar.vue";

function createNode(
  partial: Partial<TopologyNode> & Pick<TopologyNode, "id" | "name">,
): TopologyNode {
  return {
    id: partial.id,
    name: partial.name,
    nodeType: partial.nodeType ?? "service",
    env: partial.env ?? "prod",
    groupKind: partial.groupKind ?? "service",
    importance: partial.importance ?? 1,
    extra: partial.extra,
  };
}

const statsFixture: TopologyLegendStats = {
  envCounts: [
    { env: "prod", count: 3, appCount: 2 },
    { env: "test", count: 1, appCount: 1 },
    { env: "dev", count: 0, appCount: 0 },
  ],
  nodeTypeCounts: [
    { kind: "service", count: 2 },
    { kind: "middleware", count: 1 },
  ],
  edgeTypeCounts: [
    { kind: "http_call", count: 2 },
    { kind: "tcp", count: 1 },
  ],
  serviceCount: 2,
  currentEnv: "prod",
  externalNodeCount: 1,
  crossEnvEdgeCount: 1,
};

const filterFixture: TopologyFilterState = {
  env: "prod",
  nodeKinds: [],
  edgeTypes: [],
  showAllEdges: false,
};

function mountControlBar(
  layout: "force" | "dagre" = "force",
  performanceOptimizationEnabled = true,
) {
  const nodes = [
    createNode({ id: "n-1", name: "订单服务", extra: { address: "10.0.0.11" } }),
    createNode({
      id: "n-2",
      name: "订单网关",
      nodeType: "nginx",
      groupKind: "nginx",
      extra: { ip: "10.0.0.21" },
    }),
    createNode({ id: "n-3", name: "Redis", nodeType: "middleware", groupKind: "middleware" }),
  ];

  return mount(TopologyControlBar, {
    props: {
      nodes,
      stats: statsFixture,
      filter: filterFixture,
      focusNeighborhoodEnabled: true,
      layout,
      performanceOptimizationEnabled,
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
    const firstPayload = wrapper.emitted("search")?.[0]?.[0] as {
      matchIds: string[];
      focusId?: string;
    };
    expect(firstPayload.matchIds).toEqual(["n-1", "n-2"]);
    expect(firstPayload.focusId).toBe("n-1");

    await input.trigger("keydown.enter");
    const secondPayload = wrapper.emitted("search")?.[1]?.[0] as {
      matchIds: string[];
      focusId?: string;
    };
    expect(secondPayload.matchIds).toEqual(["n-1", "n-2"]);
    expect(secondPayload.focusId).toBe("n-2");

    vi.useRealTimers();
  });

  it("emits filter changes for node type, edge type and showAllEdges", async () => {
    const wrapper = mountControlBar();

    await wrapper.get('[data-testid="node-kind-middleware"]').trigger("click");
    expect(wrapper.emitted("filter-change")?.[0]?.[0]).toEqual({ nodeKinds: ["middleware"] });

    await wrapper.get('[data-testid="edge-kind-tcp"]').trigger("click");
    expect(wrapper.emitted("filter-change")?.[1]?.[0]).toEqual({ edgeTypes: ["tcp"] });

    await wrapper.get('[data-testid="show-all-edges"]').setValue(true);
    expect(wrapper.emitted("filter-change")?.[2]?.[0]).toEqual({ showAllEdges: true });
  });

  it("emits performance optimization change and disables showAllEdges when optimization is off", async () => {
    const wrapper = mountControlBar("force", false);

    const showAllEdges = wrapper.get('[data-testid="show-all-edges"]');
    expect(showAllEdges.attributes("disabled")).toBeDefined();

    await wrapper.get('[data-testid="performance-optimization"]').setValue(true);
    expect(wrapper.emitted("performance-optimization-change")?.[0]?.[0]).toBe(true);
  });

  it("emits focus mode changes", async () => {
    const wrapper = mountControlBar();

    await wrapper.get('[data-testid="focus-neighborhood"]').setValue(false);
    expect(wrapper.emitted("focus-mode-change")?.[0]?.[0]).toBe(false);
  });

  it("uses controlled layout active state and emits layout-change only when target layout differs", async () => {
    const wrapper = mountControlBar();

    const forceButton = wrapper.get('[data-testid="layout-force"]');
    const dagreButton = wrapper.get('[data-testid="layout-dagre"]');

    expect(forceButton.classes()).toContain("active");
    expect(dagreButton.classes()).not.toContain("active");

    await wrapper.get('[data-testid="layout-force"]').trigger("click");
    expect(wrapper.emitted("layout-change")).toBeFalsy();

    await wrapper.get('[data-testid="layout-dagre"]').trigger("click");
    expect(wrapper.emitted("layout-change")?.[0]?.[0]).toBe("dagre");

    expect(forceButton.classes()).toContain("active");
    expect(dagreButton.classes()).not.toContain("active");

    await wrapper.setProps({ layout: "dagre" });
    expect(forceButton.classes()).not.toContain("active");
    expect(dagreButton.classes()).toContain("active");
  });

  it("emits export and action events", async () => {
    const wrapper = mountControlBar();

    await wrapper.get('[data-testid="export-png"]').trigger("click");
    await wrapper.get('[data-testid="export-svg"]').trigger("click");
    expect(wrapper.emitted("export")?.map((item) => item[0])).toEqual(["png", "svg"]);

    await wrapper.get('[data-testid="action-refresh"]').trigger("click");
    await wrapper.get('[data-testid="action-fullscreen"]').trigger("click");

    expect(wrapper.emitted("refresh")).toHaveLength(1);
    expect(wrapper.emitted("fullscreen")).toHaveLength(1);
  });
});
