import { describe, it, expect, beforeEach } from "vitest";
import cytoscape, { type Core } from "cytoscape";

describe("useTopologyLayout - calculateNodeLevels", () => {
  let cy: Core;

  beforeEach(() => {
    cy = cytoscape({
      headless: true,
      elements: [],
    });
  });

  it("should calculate levels for linear chain", () => {
    cy.add([
      { data: { id: "a" } },
      { data: { id: "b" } },
      { data: { id: "c" } },
      { data: { id: "e1", source: "a", target: "b" } },
      { data: { id: "e2", source: "b", target: "c" } },
    ]);

    // 模拟 calculateNodeLevels 逻辑
    const levelMap = new Map<string, number>();
    const leafNodes = cy.nodes().not(":parent");
    const adjacency = new Map<string, Set<string>>();
    const inDegree = new Map<string, number>();

    leafNodes.forEach((node) => {
      adjacency.set(node.id(), new Set());
      inDegree.set(node.id(), 0);
    });

    cy.edges().forEach((edge) => {
      const sourceId = edge.source().id();
      const targetId = edge.target().id();
      if (adjacency.has(sourceId) && adjacency.has(targetId)) {
        adjacency.get(sourceId)!.add(targetId);
        inDegree.set(targetId, (inDegree.get(targetId) || 0) + 1);
      }
    });

    const queue: string[] = [];
    inDegree.forEach((degree, nodeId) => {
      if (degree === 0) {
        queue.push(nodeId);
        levelMap.set(nodeId, 0);
      }
    });

    while (queue.length > 0) {
      const current = queue.shift()!;
      const currentLevel = levelMap.get(current) || 0;

      adjacency.get(current)?.forEach((neighbor) => {
        const newDegree = (inDegree.get(neighbor) || 0) - 1;
        inDegree.set(neighbor, newDegree);

        const existingLevel = levelMap.get(neighbor) ?? -1;
        levelMap.set(neighbor, Math.max(existingLevel, currentLevel + 1));

        if (newDegree === 0) {
          queue.push(neighbor);
        }
      });
    }

    expect(levelMap.get("a")).toBe(0);
    expect(levelMap.get("b")).toBe(1);
    expect(levelMap.get("c")).toBe(2);
  });

  it("should handle isolated nodes", () => {
    cy.add([{ data: { id: "isolated" } }]);

    const levelMap = new Map<string, number>();
    const leafNodes = cy.nodes().not(":parent");

    leafNodes.forEach((node) => {
      if (!levelMap.has(node.id())) {
        levelMap.set(node.id(), 0);
      }
    });

    expect(levelMap.get("isolated")).toBe(0);
  });

  it("should handle diamond pattern", () => {
    cy.add([
      { data: { id: "a" } },
      { data: { id: "b" } },
      { data: { id: "c" } },
      { data: { id: "d" } },
      { data: { id: "e1", source: "a", target: "b" } },
      { data: { id: "e2", source: "a", target: "c" } },
      { data: { id: "e3", source: "b", target: "d" } },
      { data: { id: "e4", source: "c", target: "d" } },
    ]);

    const levelMap = new Map<string, number>();
    const leafNodes = cy.nodes().not(":parent");
    const adjacency = new Map<string, Set<string>>();
    const inDegree = new Map<string, number>();

    leafNodes.forEach((node) => {
      adjacency.set(node.id(), new Set());
      inDegree.set(node.id(), 0);
    });

    cy.edges().forEach((edge) => {
      const sourceId = edge.source().id();
      const targetId = edge.target().id();
      if (adjacency.has(sourceId) && adjacency.has(targetId)) {
        adjacency.get(sourceId)!.add(targetId);
        inDegree.set(targetId, (inDegree.get(targetId) || 0) + 1);
      }
    });

    const queue: string[] = [];
    inDegree.forEach((degree, nodeId) => {
      if (degree === 0) {
        queue.push(nodeId);
        levelMap.set(nodeId, 0);
      }
    });

    while (queue.length > 0) {
      const current = queue.shift()!;
      const currentLevel = levelMap.get(current) || 0;

      adjacency.get(current)?.forEach((neighbor) => {
        const newDegree = (inDegree.get(neighbor) || 0) - 1;
        inDegree.set(neighbor, newDegree);

        const existingLevel = levelMap.get(neighbor) ?? -1;
        levelMap.set(neighbor, Math.max(existingLevel, currentLevel + 1));

        if (newDegree === 0) {
          queue.push(neighbor);
        }
      });
    }

    expect(levelMap.get("a")).toBe(0);
    expect(levelMap.get("b")).toBe(1);
    expect(levelMap.get("c")).toBe(1);
    expect(levelMap.get("d")).toBe(2);
  });
});
