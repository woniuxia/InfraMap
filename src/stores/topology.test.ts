import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";

const { getTopologySnapshotMock, getTopologyTaskViewMock } = vi.hoisted(() => ({
  getTopologySnapshotMock: vi.fn(),
  getTopologyTaskViewMock: vi.fn(),
}));

vi.mock("@/api/topologyV3", () => ({
  getTopologySnapshotV3: getTopologySnapshotMock,
  getTopologyTaskViewV3: getTopologyTaskViewMock,
}));

import { useTopologyStore } from "@/stores/topology";

function createSnapshot() {
  return {
    meta: {
      version: "3.0",
      taskView: "explore",
      generatedAt: "2026-03-07T00:00:00.000Z",
      nodeCount: 1,
      edgeCount: 0,
    },
    lanes: [{ id: "prod", label: "生产", order: 0, nodeCount: 1, appCount: 1 }],
    nodes: [
      {
        id: "node-1",
        name: "订单服务",
        nodeType: "service",
        groupKind: "service",
        env: "prod",
        importance: 1,
      },
    ],
    edges: [],
    legendStats: {
      envCounts: [{ env: "prod", count: 1, appCount: 1 }],
      nodeTypeCounts: [{ kind: "service", count: 1 }],
      edgeTypeCounts: [],
      serviceCount: 1,
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
}

describe("useTopologyStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    getTopologySnapshotMock.mockReset();
    getTopologyTaskViewMock.mockReset();
  });

  it("loads explore snapshot from topology snapshot api", async () => {
    getTopologySnapshotMock.mockResolvedValue(createSnapshot());

    const store = useTopologyStore();
    const result = await store.fetchGraph();

    expect(getTopologySnapshotMock).toHaveBeenCalledWith({
      taskView: "explore",
      maxDepth: 3,
    });
    expect(result?.nodes[0]?.nodeType).toBe("service");
    expect(store.taskInsights).toEqual([]);
  });

  it("does not fall back to snapshot api when task view response misses snapshot", async () => {
    getTopologyTaskViewMock.mockResolvedValue({
      taskView: "impact",
      insights: [],
      meta: {
        version: "3.0",
        taskView: "impact",
        generatedAt: "2026-03-07T00:00:00.000Z",
        nodeCount: 0,
        edgeCount: 0,
      },
    });
    getTopologySnapshotMock.mockResolvedValue(createSnapshot());

    const store = useTopologyStore();
    store.setTaskView("impact");
    const result = await store.fetchGraph();

    expect(result).toBeNull();
    expect(getTopologyTaskViewMock).toHaveBeenCalledWith({
      taskView: "impact",
      maxDepth: 3,
    });
    expect(getTopologySnapshotMock).not.toHaveBeenCalled();
  });
});
