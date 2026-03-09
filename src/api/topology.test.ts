import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

import {
  getTopologySnapshot,
  getTopologyTaskView,
  getTopologyPaths,
  getTopologyImpact,
  getTopologyEvidence,
} from "@/api/topology";
import type {
  TopologyEvidenceQuery,
  TopologyImpactQuery,
  TopologyPathsQuery,
  TopologySnapshotQuery,
  TopologyTaskViewQuery,
} from "@/types";

describe("topology API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("getTopologySnapshot should invoke get_topology_snapshot_v3", async () => {
    const query: TopologySnapshotQuery = {
      env: "prod",
      taskView: "explore",
      maxDepth: 3,
    };
    const mockResult = {
      meta: { taskView: "explore" },
      lanes: [],
      nodes: [],
      edges: [],
      legendStats: {
        envCounts: [],
        nodeTypeCounts: [],
        edgeTypeCounts: [],
        applicationServiceCount: 0,
      },
      layoutHints: {
        laneOrder: ["prod", "test", "dev"],
        defaultCollapsedGroups: [],
        highDensityMode: false,
      },
    };
    __setMockHandler("get_topology_snapshot_v3", (_cmd, args) => {
      expect(args).toEqual({ query });
      return mockResult;
    });

    const result = await getTopologySnapshot(query);
    expect(result).toEqual(mockResult);
  });

  it("getTopologyTaskView should invoke get_topology_task_view_v3", async () => {
    const query: TopologyTaskViewQuery = {
      taskView: "impact",
      env: "prod",
      maxDepth: 2,
    };
    const mockResult = {
      taskView: "impact",
      snapshot: {
        meta: { taskView: "impact" },
        lanes: [],
        nodes: [],
        edges: [],
        legendStats: {
          envCounts: [],
          nodeTypeCounts: [],
          edgeTypeCounts: [],
          applicationServiceCount: 0,
        },
        layoutHints: {
          laneOrder: ["prod", "test", "dev"],
          defaultCollapsedGroups: [],
          highDensityMode: false,
        },
      },
      insights: [
        {
          kind: "dependency",
          title: "summary",
          nodeIds: ["node-1"],
          edgeIds: [],
        },
      ],
    };
    __setMockHandler("get_topology_task_view_v3", (_cmd, args) => {
      expect(args).toEqual({ query });
      return mockResult;
    });

    const result = await getTopologyTaskView(query);
    expect(result).toEqual(mockResult);
  });

  it("getTopologyPaths should invoke get_topology_paths_v3", async () => {
    const query: TopologyPathsQuery = {
      sourceId: "A",
      targetId: "B",
      taskView: "explore",
      maxDepth: 3,
      maxResults: 5,
    };
    const mockResult = {
      paths: [{ nodeIds: ["A", "B"], score: 1 }],
      truncated: false,
      totalCount: 1,
    };
    __setMockHandler("get_topology_paths_v3", (_cmd, args) => {
      expect(args).toEqual({ query });
      return mockResult;
    });

    const result = await getTopologyPaths(query);
    expect(result).toEqual(mockResult);
  });

  it("getTopologyImpact should invoke get_topology_impact_v3", async () => {
    const query: TopologyImpactQuery = {
      nodeId: "C",
      taskView: "impact",
      maxDepth: 3,
    };
    const mockResult = {
      affectedNodes: [{ id: "n1", name: "App", nodeType: "application", depth: 1 }],
      totalCount: 1,
      maxDepth: 1,
      severity: "warning",
    };
    __setMockHandler("get_topology_impact_v3", (_cmd, args) => {
      expect(args).toEqual({ query });
      return mockResult;
    });

    const result = await getTopologyImpact(query);
    expect(result).toEqual(mockResult);
  });

  it("getTopologyEvidence should invoke get_topology_evidence_v3", async () => {
    const query: TopologyEvidenceQuery = {
      nodeId: "node-1",
      taskView: "troubleshoot",
      maxItems: 20,
    };
    const mockResult = {
      items: [
        {
          id: "ev-1",
          evidenceType: "audit",
          title: "recent update",
          nodeId: "node-1",
        },
      ],
      total: 1,
    };
    __setMockHandler("get_topology_evidence_v3", (_cmd, args) => {
      expect(args).toEqual({ query });
      return mockResult;
    });

    const result = await getTopologyEvidence(query);
    expect(result).toEqual(mockResult);
  });
});
