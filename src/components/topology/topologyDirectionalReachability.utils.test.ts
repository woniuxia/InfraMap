import { describe, expect, it } from "vitest";
import { collectDirectionalReachability } from "@/components/topology/topologyDirectionalReachability.utils";

interface TestEdge {
  id: string;
  source: string;
  target: string;
}

function sortIds(ids: Set<string>): string[] {
  return [...ids].sort((left, right) => left.localeCompare(right));
}

describe("topologyDirectionalReachability utils", () => {
  const edges: TestEdge[] = [
    { id: "e-ab", source: "A", target: "B" },
    { id: "e-bc", source: "B", target: "C" },
    { id: "e-db", source: "D", target: "B" },
  ];

  it("focus A should only include downstream branch and not include D", () => {
    const result = collectDirectionalReachability("A", edges);

    expect(sortIds(result.highlightedNodeIds)).toEqual(["A", "B", "C"]);
    expect(sortIds(result.highlightedEdgeIds)).toEqual(["e-ab", "e-bc"]);
  });

  it("focus B should include upstream A/D and downstream C", () => {
    const result = collectDirectionalReachability("B", edges);

    expect(sortIds(result.highlightedNodeIds)).toEqual(["A", "B", "C", "D"]);
    expect(sortIds(result.highlightedEdgeIds)).toEqual(["e-ab", "e-bc", "e-db"]);
  });

  it("should be stable on A<->B cycle", () => {
    const cycleEdges: TestEdge[] = [
      { id: "e-ab", source: "A", target: "B" },
      { id: "e-ba", source: "B", target: "A" },
    ];

    const result = collectDirectionalReachability("A", cycleEdges);

    expect(sortIds(result.highlightedNodeIds)).toEqual(["A", "B"]);
    expect(sortIds(result.highlightedEdgeIds)).toEqual(["e-ab", "e-ba"]);
  });

  it("should keep isolated focus node highlighted", () => {
    const result = collectDirectionalReachability("Z", edges);

    expect(sortIds(result.highlightedNodeIds)).toEqual(["Z"]);
    expect(sortIds(result.highlightedEdgeIds)).toEqual([]);
  });
});
