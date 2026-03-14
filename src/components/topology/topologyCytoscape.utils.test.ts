import { describe, expect, it } from "vitest";
import type { TopologyGraph, TopologyNode } from "@/types";
import { computeLegendStats, filterTopologyGraph } from "@/components/topology/topologyGraph.utils";
import {
  buildTopologyCyElements,
  toHostCompoundId,
} from "@/components/topology/topologyCytoscape.utils";

function createNode(
  partial: Partial<TopologyNode> &
    Pick<TopologyNode, "id" | "name" | "nodeType" | "env" | "groupKind">,
): TopologyNode {
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
      nodeType: "service",
      env: "prod",
      groupKind: "service",
      hostId: "host-prod-1",
      status: "running",
      extra: { address: "10.0.0.1", type: "frontend" },
    }),
    createNode({
      id: "mw-prod-1",
      name: "Redis",
      nodeType: "middleware",
      env: "prod",
      groupKind: "middleware",
      hostId: "host-prod-1",
      extra: { category: "cache", type: "Redis", icon_key: "heroicons:window" },
    }),
    createNode({
      id: "app-test-1",
      name: "支付服务",
      nodeType: "service",
      env: "test",
      groupKind: "service",
      hostId: "host-test-1",
      status: "running",
    }),
  ];

  const edges = [
    {
      id: "edge-1",
      source: "app-prod-1",
      target: "mw-prod-1",
      edgeType: "cache_access" as const,
      label: "cache",
      strength: 2,
      crossEnv: false,
    },
    {
      id: "edge-2",
      source: "app-prod-1",
      target: "app-test-1",
      edgeType: "http_call" as const,
      label: "api",
      strength: 1,
      crossEnv: true,
    },
  ];

  return {
    lanes: [
      { id: "prod", label: "生产", order: 0, nodeCount: 2, appCount: 1 },
      { id: "test", label: "测试", order: 1, nodeCount: 1, appCount: 1 },
      { id: "dev", label: "开发", order: 2, nodeCount: 0, appCount: 0 },
    ],
    nodes,
    edges,
    legendStats: computeLegendStats(nodes, edges, "prod"),
    layoutHints: {
      laneOrder: ["prod", "test", "dev"],
      defaultCollapsedGroups: [],
      highDensityMode: false,
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

    expect(Number(detailEdge?.data?.line_width)).toBeGreaterThan(
      Number(overviewEdge?.data?.line_width),
    );
    expect(overviewEdge?.data?.arrow).toBe("none");
    expect(detailEdge?.data?.arrow).toBe("triangle");
  });

  it("should attach node icons and source app type metadata", () => {
    const elements = buildTopologyCyElements(createGraphFixture(), {
      density: "medium",
      hideEdgeLabels: false,
      isLargeGraph: false,
    });

    const frontendAppNode = elements.find((item) => item.data?.id === "app-prod-1");
    const otherAppNode = elements.find((item) => item.data?.id === "app-test-1");
    const middlewareNode = elements.find((item) => item.data?.id === "mw-prod-1");
    const edge = elements.find((item) => item.data?.id === "edge-1");

    expect(frontendAppNode?.data?.app_type_key).toBe("frontend");
    expect(String(frontendAppNode?.data?.icon_src || "")).toContain("data:image/svg+xml");
    expect(String(frontendAppNode?.data?.icon_key || "")).toContain("heroicons:window");
    expect(otherAppNode?.data?.app_type_key).toBe("other");
    expect(String(otherAppNode?.data?.icon_key || "")).toContain("heroicons:server-stack");
    expect(String(middlewareNode?.data?.icon_src || "")).toContain("data:image/svg+xml");
    expect(String(middlewareNode?.data?.icon_key || "")).toContain("redis");
    expect(String(middlewareNode?.data?.icon_key || "")).not.toBe("heroicons:window");
    expect(edge?.data?.source_nodeType).toBe("service");
    expect(edge?.data?.source_app_type_key).toBe("frontend");
  });
});
