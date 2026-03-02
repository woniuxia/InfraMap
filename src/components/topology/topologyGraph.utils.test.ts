import { describe, expect, it } from "vitest";
import type { TopologyGraph, TopologyNode } from "@/types";
import {
  DEFAULT_TOPOLOGY_FILTER,
  buildTopologyG6Data,
  computeLegendStats,
  filterTopologyGraph,
} from "@/components/topology/topologyGraph.utils";

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
      extra: { address: "10.0.0.1" },
    }),
    createNode({
      id: "mw-prod-1",
      name: "Redis",
      node_type: "middleware",
      env: "prod",
      group_kind: "middleware",
      host_id: "host-prod-1",
      extra: { category: "cache" },
    }),
    createNode({
      id: "app-test-1",
      name: "支付服务",
      node_type: "application",
      env: "test",
      group_kind: "application_service",
      host_id: "host-test-1",
      status: "running",
      extra: { address: "10.0.1.1" },
    }),
    createNode({
      id: "ng-dev-1",
      name: "Nginx-Dev",
      node_type: "nginx",
      env: "dev",
      group_kind: "nginx",
      status: "running",
      extra: { listen_port: 8080 },
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
    {
      id: "edge-3",
      source: "ng-dev-1",
      target: "app-prod-1",
      edge_type: "tcp" as const,
      label: "ingress",
      strength: 1,
      cross_env: true,
    },
    {
      id: "edge-4",
      source: "app-test-1",
      target: "ng-dev-1",
      edge_type: "grpc_call" as const,
      label: "skip",
      strength: 1,
      cross_env: true,
    },
  ];

  return {
    lanes: [
      { id: "prod", label: "生产", order: 0, node_count: 2, app_count: 1 },
      { id: "test", label: "测试", order: 1, node_count: 1, app_count: 1 },
      { id: "dev", label: "开发", order: 2, node_count: 1, app_count: 0 },
    ],
    nodes,
    edges,
    legend_stats: computeLegendStats(nodes, edges, "prod"),
    layout_hints: {
      lane_order: ["prod", "test", "dev"],
      default_collapsed_groups: ["middleware", "nginx"],
      high_density_mode: false,
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

    expect(filtered!.legend_stats.current_env).toBe("prod");
    expect(filtered!.legend_stats.external_node_count).toBe(2);
    expect(filtered!.legend_stats.cross_env_edge_count).toBe(2);
  });

  it("should apply node and edge filters on transformed single-env graph", () => {
    const graph = createGraphFixture();
    const filtered = filterTopologyGraph(graph, {
      env: "prod",
      nodeKinds: ["application_service"],
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
      node_type: "application",
      env: "prod",
      group_kind: "application_service",
    });

    const extraNodes = Array.from({ length: 320 }, (_, index) => createNode({
      id: `app-test-${index}`,
      name: `test-${index}`,
      node_type: "application",
      env: "test",
      group_kind: "application_service",
    }));

    const edges = extraNodes.map((node, index) => ({
      id: `edge-${index}`,
      source: "app-prod-main",
      target: node.id,
      edge_type: "http_call" as const,
      label: "burst",
      strength: index % 4,
      cross_env: true,
    }));

    const graph: TopologyGraph = {
      lanes: [
        { id: "prod", label: "生产", order: 0, node_count: 1, app_count: 1 },
        { id: "test", label: "测试", order: 1, node_count: extraNodes.length, app_count: extraNodes.length },
        { id: "dev", label: "开发", order: 2, node_count: 0, app_count: 0 },
      ],
      nodes: [mainNode, ...extraNodes],
      edges,
      legend_stats: computeLegendStats([mainNode, ...extraNodes], edges, "prod"),
      layout_hints: {
        lane_order: ["prod", "test", "dev"],
        default_collapsed_groups: [],
        high_density_mode: true,
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

  it("should build host combo and external combo for g6 rendering", () => {
    const graph = createGraphFixture();
    const filtered = filterTopologyGraph(graph, {
      env: "prod",
      nodeKinds: [],
      edgeTypes: [],
      showAllEdges: true,
    });

    const g6Data = buildTopologyG6Data(filtered!);
    const combos = g6Data.combos || [];
    const hostCombos = combos.filter((combo) => combo.data?.kind === "host");
    const externalCombo = combos.find((combo) => combo.id === "external-zone");

    expect(hostCombos).toHaveLength(1);
    expect(hostCombos[0].id).toBe("host-host-prod-1");
    expect(externalCombo).toBeTruthy();
    expect(g6Data.nodes?.some((node) => node.combo === "external-zone")).toBe(true);
  });
});
