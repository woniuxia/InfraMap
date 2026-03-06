import { describe, expect, it } from "vitest";
import { getDensityByZoom, selectVisibleEdgeIds } from "@/components/topology/topologyDensity.utils";

describe("topologyDensity.utils", () => {
  it("maps zoom level to density tier", () => {
    expect(getDensityByZoom(0.6)).toBe("overview");
    expect(getDensityByZoom(1)).toBe("medium");
    expect(getDensityByZoom(1.8)).toBe("detail");
  });

  it("keeps all edges in detail mode", () => {
    const nodes = [
      { id: "A", importance: 1 },
      { id: "B", importance: 1 },
    ];
    const edges = [
      { id: "e1", source: "A", target: "B", strength: 1, cross_env: false },
      { id: "e2", source: "B", target: "A", strength: 2, cross_env: false },
    ];

    const ids = selectVisibleEdgeIds("detail", nodes, edges);
    expect(ids).toEqual(["e1", "e2"]);
  });

  it("prioritizes cross-env and stronger edges in overview mode", () => {
    const nodes = [
      { id: "A", importance: 1 },
      { id: "B", importance: 1 },
      { id: "C", importance: 1 },
      { id: "D", importance: 1 },
    ];
    const edges = [
      { id: "e1", source: "A", target: "B", strength: 1, cross_env: false },
      { id: "e2", source: "A", target: "C", strength: 1, cross_env: false },
      { id: "e3", source: "A", target: "D", strength: 1, cross_env: false },
      { id: "e4", source: "B", target: "C", strength: 4, cross_env: false },
      { id: "e5", source: "C", target: "D", strength: 2, cross_env: false },
      { id: "e6", source: "D", target: "A", strength: 1, cross_env: true },
    ];

    const ids = selectVisibleEdgeIds("overview", nodes, edges);
    expect(ids).toEqual(["e4", "e5", "e6"]);
  });

  it("applies per-node cap for non-critical edges", () => {
    const nodes = [
      { id: "A", importance: 1 },
      { id: "B", importance: 1 },
      { id: "C", importance: 1 },
      { id: "D", importance: 1 },
      { id: "E", importance: 1 },
      { id: "F", importance: 1 },
      { id: "G", importance: 1 },
    ];
    const edges = [
      { id: "e1", source: "A", target: "B", strength: 2, cross_env: false },
      { id: "e2", source: "A", target: "C", strength: 2, cross_env: false },
      { id: "e3", source: "A", target: "D", strength: 2, cross_env: false },
      { id: "e4", source: "A", target: "E", strength: 2, cross_env: false },
      { id: "e5", source: "A", target: "F", strength: 2, cross_env: false },
      { id: "e6", source: "A", target: "G", strength: 2, cross_env: false },
    ];

    const ids = selectVisibleEdgeIds("overview", nodes, edges);
    expect(ids.length).toBe(4);
  });
});

