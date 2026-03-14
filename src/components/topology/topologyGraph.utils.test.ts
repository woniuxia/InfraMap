import { describe, expect, it } from "vitest";
import type { TopologyGraph, TopologyNode } from "@/types";
import {
  DEFAULT_TOPOLOGY_FILTER,
  computeLegendStats,
  filterTopologyGraph,
  formatHostDisplayName,
  formatHostComboLabel,
  isOpaqueIdentifier,
} from "@/components/topology/topologyGraph.utils";

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
      extra: { address: "10.0.0.1" },
    }),
    createNode({
      id: "mw-prod-1",
      name: "Redis",
      nodeType: "middleware",
      env: "prod",
      groupKind: "middleware",
      hostId: "host-prod-1",
      extra: { category: "cache" },
    }),
    createNode({
      id: "app-test-1",
      name: "支付服务",
      nodeType: "service",
      env: "test",
      groupKind: "service",
      hostId: "host-test-1",
      status: "running",
      extra: { address: "10.0.1.1" },
    }),
    createNode({
      id: "ng-dev-1",
      name: "Nginx-Dev",
      nodeType: "nginx",
      env: "dev",
      groupKind: "nginx",
      status: "running",
      extra: { endpoint_count: 2, first_endpoint: "10.0.2.8:8080", address: "10.0.2.8:8080" },
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
    {
      id: "edge-3",
      source: "ng-dev-1",
      target: "app-prod-1",
      edgeType: "tcp" as const,
      label: "ingress",
      strength: 1,
      crossEnv: true,
    },
    {
      id: "edge-4",
      source: "app-test-1",
      target: "ng-dev-1",
      edgeType: "grpc_call" as const,
      label: "skip",
      strength: 1,
      crossEnv: true,
    },
  ];

  return {
    lanes: [
      { id: "prod", label: "生产", order: 0, nodeCount: 2, appCount: 1 },
      { id: "test", label: "测试", order: 1, nodeCount: 1, appCount: 1 },
      { id: "dev", label: "开发", order: 2, nodeCount: 1, appCount: 0 },
    ],
    nodes,
    edges,
    legendStats: computeLegendStats(nodes, edges, "prod"),
    layoutHints: {
      laneOrder: ["prod", "test", "dev"],
      defaultCollapsedGroups: ["middleware", "nginx"],
      highDensityMode: false,
    },
  };
}

describe("topologyGraph utils", () => {
  it("should use prod as default topology environment", () => {
    expect(DEFAULT_TOPOLOGY_FILTER.env).toBe("prod");
  });

  it("should build single-env graph with cross-env external nodes", () => {
    const graph = createGraphFixture();
    const filtered = filterTopologyGraph(graph, {
      env: "prod",
      nodeKinds: [],
      edgeTypes: [],
      showAllEdges: true,
    });

    expect(filtered).not.toBeNull();
    const ids = filtered!.nodes.map((node) => node.id);
    expect(ids).toContain("app-prod-1");
    expect(ids).toContain("mw-prod-1");
    expect(ids).toContain("external:app-test-1");
    expect(ids).toContain("external:ng-dev-1");
    expect(ids).not.toContain("app-test-1");
    expect(ids).not.toContain("ng-dev-1");

    const edgeIds = filtered!.edges.map((edge) => edge.id);
    expect(edgeIds).toEqual(["edge-1", "edge-2", "edge-3"]);

    const edge2 = filtered!.edges.find((edge) => edge.id === "edge-2");
    const edge3 = filtered!.edges.find((edge) => edge.id === "edge-3");
    expect(edge2?.target).toBe("external:app-test-1");
    expect(edge3?.source).toBe("external:ng-dev-1");

    expect(filtered!.legendStats.currentEnv).toBe("prod");
    expect(filtered!.legendStats.externalNodeCount).toBe(2);
    expect(filtered!.legendStats.crossEnvEdgeCount).toBe(2);
  });

  it("should apply node and edge filters on transformed single-env graph", () => {
    const graph = createGraphFixture();
    const filtered = filterTopologyGraph(graph, {
      env: "prod",
      nodeKinds: ["service"],
      edgeTypes: ["http_call"],
      showAllEdges: true,
    });

    expect(filtered).not.toBeNull();
    const nodeIds = filtered!.nodes.map((node) => node.id);
    expect(nodeIds).toEqual(["app-prod-1", "external:app-test-1"]);
    expect(filtered!.edges).toHaveLength(1);
    expect(filtered!.edges[0].id).toBe("edge-2");
  });

  it("should enforce edge render limit when showAllEdges is disabled", () => {
    const mainNode = createNode({
      id: "app-prod-main",
      name: "main",
      nodeType: "service",
      env: "prod",
      groupKind: "service",
    });

    const extraNodes = Array.from({ length: 320 }, (_, index) =>
      createNode({
        id: `app-test-${index}`,
        name: `test-${index}`,
        nodeType: "service",
        env: "test",
        groupKind: "service",
      }),
    );

    const edges = extraNodes.map((node, index) => ({
      id: `edge-${index}`,
      source: "app-prod-main",
      target: node.id,
      edgeType: "http_call" as const,
      label: "burst",
      strength: index % 4,
      crossEnv: true,
    }));

    const graph: TopologyGraph = {
      lanes: [
        { id: "prod", label: "生产", order: 0, nodeCount: 1, appCount: 1 },
        {
          id: "test",
          label: "测试",
          order: 1,
          nodeCount: extraNodes.length,
          appCount: extraNodes.length,
        },
        { id: "dev", label: "开发", order: 2, nodeCount: 0, appCount: 0 },
      ],
      nodes: [mainNode, ...extraNodes],
      edges,
      legendStats: computeLegendStats([mainNode, ...extraNodes], edges, "prod"),
      layoutHints: {
        laneOrder: ["prod", "test", "dev"],
        defaultCollapsedGroups: [],
        highDensityMode: true,
      },
    };

    const filtered = filterTopologyGraph(graph, {
      env: "prod",
      nodeKinds: [],
      edgeTypes: [],
      showAllEdges: false,
    });

    expect(filtered).not.toBeNull();
    expect(filtered!.edges).toHaveLength(260);
    expect(filtered!.nodes).toHaveLength(261);
  });

  it("should format opaque host id into readable combo label", () => {
    const hostId = "host-1772540925435-ed5c58b0";
    const label = formatHostComboLabel(hostId, []);
    expect(isOpaqueIdentifier(hostId)).toBe(true);
    expect(label.startsWith("主机 #")).toBe(true);
  });

  it("should compose host display name with hostname and ip", () => {
    expect(formatHostDisplayName("web-01", "10.0.1.12")).toBe("web-01 · 10.0.1.12");
    expect(formatHostDisplayName("web-01", null)).toBe("web-01");
    expect(formatHostDisplayName(null, "10.0.1.12")).toBe("10.0.1.12");
    expect(formatHostDisplayName(null, null)).toBeNull();
  });

  it("should summarize multiple host ips with count suffix", () => {
    expect(formatHostDisplayName(null, "10.0.1.12, 10.0.1.13,10.0.1.14")).toBe("10.0.1.12 +2");
  });

  it("should prefer host meta fields when formatting host combo label", () => {
    const label = formatHostComboLabel("host-opaque-id", [
      createNode({
        id: "app-1",
        name: "app",
        nodeType: "service",
        env: "prod",
        groupKind: "service",
        hostId: "host-opaque-id",
        hostName: "payment-host",
        hostIpDisplay: "10.6.1.8",
      }),
    ]);
    expect(label).toBe("payment-host · 10.6.1…");
  });
});
