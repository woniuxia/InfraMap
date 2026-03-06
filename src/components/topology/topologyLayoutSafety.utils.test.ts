import { describe, expect, it } from "vitest";
import {
  hasTopologyNodeSetChanged,
  isDegenerateNodeDistribution,
} from "@/components/topology/topologyLayoutSafety.utils";

describe("topologyLayoutSafety utils", () => {
  it("detects topology node set changes by size and id", () => {
    const previousNodeIds = new Set(["a", "b", "c"]);

    expect(hasTopologyNodeSetChanged(previousNodeIds, [{ id: "a" }, { id: "b" }, { id: "c" }])).toBe(false);
    expect(hasTopologyNodeSetChanged(previousNodeIds, [{ id: "a" }, { id: "b" }])).toBe(true);
    expect(hasTopologyNodeSetChanged(previousNodeIds, [{ id: "a" }, { id: "b" }, { id: "x" }])).toBe(true);
  });

  it("marks highly overlapped points as degenerate", () => {
    const clustered = Array.from({ length: 20 }, () => ({ x: 0, y: 0 }));

    expect(isDegenerateNodeDistribution(clustered)).toBe(true);
  });

  it("keeps sparse distribution as non-degenerate", () => {
    const spread = Array.from({ length: 20 }, (_, index) => ({
      x: index * 24,
      y: (index % 4) * 18,
    }));

    expect(isDegenerateNodeDistribution(spread)).toBe(false);
  });
});
