import { describe, expect, it } from "vitest";
import type { TopologyGraph, TopologyNode } from "@/types";
import { computeLegendStats, filterTopologyGraph } from "@/components/topology/topologyGraph.utils";
import { buildTopologyCyElements, toHostCompoundId } from "@/components/topology/topologyCytoscape.utils";

function createNode(partial: Partial<TopologyNode> & Pick<TopologyNode, "id" | "name" | "node_type" | "env" | "group_kind">): TopologyNode {
  return {
    importance: 1,
    ...partial,
  };
}

function createGraphFixture(): TopologyGraph {
  const nodes: TopologyNode[] = [
    createNode({
      id: "app-prod-1",
      name: "订单服务",
      node_type: "application",
      env: "prod",
      group_kind: "application_service",
      host_id: "host-prod-1",
      status: "running",
      extra: { address: "10.0.0.1", type: "frontend" },
    }),
    createNode({
      id: "mw-prod-1",
      name: "Redis",
      node_type: "middleware",
      env: "prod",
      group_kind: "middleware",
      host_id: "host-prod-1",
      extra: { category: "cache", type: "Redis" },
    }),
    createNode({
      id: "app-test-1",
      name: "支付服务",
      node_type: "application",
      env: "test",
      group_kind: "application_service",
      host_id: "host-test-1",
      status: "running",
    }),
  ];

  const edges = [
    {
      id: "edge-1",
      source: "app-prod-1",
      target: "mw-prod-1",
      edge_type: "cache_access" as const,
      label: "cache",
      strength: 2,
      cross_env: false,
    },
    {
      id: "edge-2",
      source: "app-prod-1",
      target: "app-test-1",
      edge_type: "http_call" as const,
      label: "api",
      strength: 1,
      cross_env: true,
    },
  ];

  return {
    lanes: [
      { id: "prod", label: "生产", order: 0, node_count: 2, app_count: 1 },
      { id: "test", label: "测试", order: 1, node_count: 1, app_count: 1 },
      { id: "dev", label: "开发", order: 2, node_count: 0, app_count: 0 },
    ],
    nodes,
    edges,
    legend_stats: computeLegendStats(nodes, edges, "prod"),
    layout_hints: {
      lane_order: ["prod", "test", "dev"],
      default_collapsed_groups: [],
      high_density_mode: false,
    },
  };
}

describe("topologyCytoscape utils", () => {
  it("should build compound nodes and parent relationship", () => {
    const graph = createGraphFixture();
    const filtered = filterTopologyGraph(graph, {
      env: "prod",
      nodeKinds: [],
      edgeTypes: [],
      showAllEdges: true,
    });
    expect(filtered).not.toBeNull();

    const elements = buildTopologyCyElements(filtered!, {
      density: "medium",
      hideEdgeLabels: false,
      isLargeGraph: false,
    });

    const hostCompound = elements.find((item) => item.data?.id === toHostCompoundId("host-prod-1"));
    const externalCompound = elements.find((item) => item.data?.id === "external-zone");
    const externalNode = elements.find((item) => item.data?.id === "external:app-test-1");

    expect(hostCompound).toBeTruthy();
    expect(externalCompound).toBeTruthy();
    expect(externalNode?.data?.parent).toBe("external-zone");
  });

  it("should hide edge labels when hideEdgeLabels is true", () => {
    const elements = buildTopologyCyElements(createGraphFixture(), {
      density: "overview",
      hideEdgeLabels: true,
      isLargeGraph: false,
    });

    const edge = elements.find((item) => item.data?.id === "edge-1");
    expect(edge?.data?.display_label).toBe("");
  });

  it("should increase visual detail for detail density", () => {
    const overview = buildTopologyCyElements(createGraphFixture(), {
      density: "overview",
      hideEdgeLabels: false,
      isLargeGraph: false,
    });
    const detail = buildTopologyCyElements(createGraphFixture(), {
      density: "detail",
      hideEdgeLabels: false,
      isLargeGraph: false,
    });

    const overviewEdge = overview.find((item) => item.data?.id === "edge-1");
    const detailEdge = detail.find((item) => item.data?.id === "edge-1");

    expect(Number(detailEdge?.data?.line_width)).toBeGreaterThan(Number(overviewEdge?.data?.line_width));
    expect(overviewEdge?.data?.arrow).toBe("none");
    expect(detailEdge?.data?.arrow).toBe("triangle");
  });

  it("should attach node icons and source app type metadata", () => {
    const elements = buildTopologyCyElements(createGraphFixture(), {
      density: "medium",
      hideEdgeLabels: false,
      isLargeGraph: false,
    });

    const appNode = elements.find((item) => item.data?.id === "app-prod-1");
    const middlewareNode = elements.find((item) => item.data?.id === "mw-prod-1");
    const edge = elements.find((item) => item.data?.id === "edge-1");

    expect(appNode?.data?.app_type_key).toBe("frontend");
    expect(String(appNode?.data?.icon_src || "")).toContain("data:image/svg+xml");
    expect(String(middlewareNode?.data?.icon_src || "")).toContain("data:image/svg+xml");
    expect(edge?.data?.source_node_type).toBe("application");
    expect(edge?.data?.source_app_type_key).toBe("frontend");
  });
});
