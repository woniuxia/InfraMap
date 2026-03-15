import cytoscape, { type Core, type LayoutOptions } from "cytoscape";
import { isDegenerateNodeDistribution } from "@/components/topology/topologyLayoutSafety.utils";

export type TopologyLayoutType = "force" | "focus";

export interface TopologyLayoutRunResult {
  requested: TopologyLayoutType;
  applied: TopologyLayoutType;
  reason?: string;
}

interface UseTopologyLayoutOptions {
  getCy: () => Core | null;
  getIsLargeGraph: () => boolean;
  onFallbackToForce: () => void;
}

export function useTopologyLayout(options: UseTopologyLayoutOptions) {
  function snapshotLeafNodePositions(): Map<string, { x: number; y: number }> {
    const positionMap = new Map<string, { x: number; y: number }>();
    const cy = options.getCy();
    if (!cy) return positionMap;

    cy.nodes()
      .not(":parent")
      .forEach((node: cytoscape.NodeSingular) => {
        const position = node.position();
        positionMap.set(node.id(), { x: position.x, y: position.y });
      });
    return positionMap;
  }

  function restoreLeafNodePositions(positionMap: Map<string, { x: number; y: number }>) {
    const cy = options.getCy();
    if (!cy || positionMap.size === 0) return;
    cy.batch(() => {
      cy.nodes()
        .not(":parent")
        .forEach((node: cytoscape.NodeSingular) => {
          const position = positionMap.get(node.id());
          if (!position) return;
          node.position(position);
        });
    });
  }

  function hasDegenerateLayout(): boolean {
    const cy = options.getCy();
    if (!cy) return false;
    const points: Array<{ x: number; y: number }> = [];
    cy.nodes()
      .not(":parent")
      .forEach((node: cytoscape.NodeSingular) => {
        const position = node.position();
        points.push({ x: position.x, y: position.y });
      });
    return isDegenerateNodeDistribution(points);
  }

  function calculateNodeLevels(cy: Core): Map<string, number> {
    const levelMap = new Map<string, number>();
    const leafNodes = cy.nodes().not(":parent");

    if (leafNodes.length === 0) return levelMap;

    // 构建邻接表 (source -> targets)
    const adjacency = new Map<string, Set<string>>();
    const inDegree = new Map<string, number>();

    leafNodes.forEach((node: cytoscape.NodeSingular) => {
      const nodeId = node.id();
      adjacency.set(nodeId, new Set());
      inDegree.set(nodeId, 0);
    });

    cy.edges().forEach((edge: cytoscape.EdgeSingular) => {
      const sourceId = edge.source().id();
      const targetId = edge.target().id();
      if (adjacency.has(sourceId) && adjacency.has(targetId)) {
        adjacency.get(sourceId)!.add(targetId);
        inDegree.set(targetId, (inDegree.get(targetId) || 0) + 1);
      }
    });

    // 拓扑排序 + BFS 计算层级
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

        // 更新层级为所有前驱节点的最大层级 + 1
        const existingLevel = levelMap.get(neighbor) ?? -1;
        levelMap.set(neighbor, Math.max(existingLevel, currentLevel + 1));

        if (newDegree === 0) {
          queue.push(neighbor);
        }
      });
    }

    // 处理孤立节点和环中的节点
    leafNodes.forEach((node: cytoscape.NodeSingular) => {
      if (!levelMap.has(node.id())) {
        levelMap.set(node.id(), 0);
      }
    });

    return levelMap;
  }

  function buildLayoutOptions(layoutType: TopologyLayoutType): LayoutOptions {
    if (layoutType === "focus") {
      const cy = options.getCy();
      if (!cy) {
        return {
          name: "fcose",
          quality: "default",
          randomize: true,
          animate: false,
          fit: true,
          padding: 40,
        } as unknown as LayoutOptions;
      }

      // 计算节点层级
      const levelMap = calculateNodeLevels(cy);

      // 预设节点初始位置 (从左到右)
      const levelSpacing = 250;
      const verticalSpacing = 150;
      const levelGroups = new Map<number, string[]>();

      levelMap.forEach((level, nodeId) => {
        if (!levelGroups.has(level)) {
          levelGroups.set(level, []);
        }
        levelGroups.get(level)!.push(nodeId);
      });

      cy.batch(() => {
        cy.nodes()
          .not(":parent")
          .forEach((node: cytoscape.NodeSingular) => {
            const level = levelMap.get(node.id()) || 0;
            const nodesInLevel = levelGroups.get(level) || [];
            const indexInLevel = nodesInLevel.indexOf(node.id());

            const x = level * levelSpacing;
            const y = indexInLevel * verticalSpacing;

            node.position({ x, y });
          });
      });

      // 使用 fcose 布局,但基于预设位置优化
      return {
        name: "fcose",
        quality: options.getIsLargeGraph() ? "default" : "proof",
        randomize: false, // 不随机,使用预设位置
        animate: false,
        fit: true,
        padding: 40,
        packComponents: false,
        gravity: 0.25,
        gravityRange: 3.5,
        nodeRepulsion: (node: cytoscape.NodeSingular) => {
          if (node.data("isExternal")) return 6000;
          const baseRepulsion = node.isChild() ? 3500 : 8000;
          const degree = node.degree(false);
          if (degree >= 8) return Math.round(baseRepulsion * 0.5);
          if (degree >= 5) return Math.round(baseRepulsion * 0.7);
          if (degree >= 3) return Math.round(baseRepulsion * 0.85);
          return baseRepulsion;
        },
        idealEdgeLength: (edge: cytoscape.EdgeSingular) => {
          if (edge.data("crossEnv")) return 240;
          const sourceParent = edge.source().data("parent");
          const targetParent = edge.target().data("parent");
          if (sourceParent && targetParent && sourceParent === targetParent) return 100;
          return 180;
        },
        numIter: options.getIsLargeGraph() ? 1400 : 2000,
        tile: true,
      } as unknown as LayoutOptions;
    }

    return {
      name: "fcose",
      quality: options.getIsLargeGraph() ? "default" : "proof",
      randomize: true,
      animate: false,
      fit: true,
      padding: 40,
      packComponents: false,
      gravity: 0.4,
      gravityRange: 4.5,
      nodeRepulsion: (node: cytoscape.NodeSingular) => {
        if (node.data("isExternal")) return 8000;

        const baseRepulsion = node.isChild() ? 4500 : 12000;

        // 高度数节点降低斥力，使其趋向中心
        const degree = node.degree(false);
        if (degree >= 8) return Math.round(baseRepulsion * 0.45);
        if (degree >= 5) return Math.round(baseRepulsion * 0.65);
        if (degree >= 3) return Math.round(baseRepulsion * 0.82);
        return baseRepulsion;
      },
      idealEdgeLength: (edge: cytoscape.EdgeSingular) => {
        if (edge.data("crossEnv")) return 260;
        const sourceParent = edge.source().data("parent");
        const targetParent = edge.target().data("parent");
        if (sourceParent && targetParent && sourceParent === targetParent) return 90;
        return 200;
      },
      numIter: options.getIsLargeGraph() ? 1600 : 2200,
      tile: true,
    } as unknown as LayoutOptions;
  }

  function runLayoutOnce(layoutType: TopologyLayoutType): Promise<boolean> {
    const cy = options.getCy();
    if (!cy) return Promise.resolve(true);
    return new Promise((resolve) => {
      let settled = false;
      const finish = (ok: boolean) => {
        if (settled) return;
        settled = true;
        resolve(ok);
      };

      let layout: ReturnType<Core["layout"]> | null = null;
      try {
        layout = cy.layout(buildLayoutOptions(layoutType));
      } catch {
        finish(false);
        return;
      }

      layout.one("layoutstop", () => finish(true));
      try {
        layout.run();
      } catch {
        finish(false);
        return;
      }

      globalThis.setTimeout(() => finish(true), options.getIsLargeGraph() ? 2600 : 1800);
    });
  }

  async function runLayout(layoutType: TopologyLayoutType): Promise<TopologyLayoutRunResult> {
    const primaryOk = await runLayoutOnce(layoutType);
    if (layoutType !== "focus") {
      return {
        requested: layoutType,
        applied: layoutType,
        reason: primaryOk ? undefined : "force_layout_error",
      };
    }

    if (primaryOk && !hasDegenerateLayout()) {
      return {
        requested: layoutType,
        applied: layoutType,
      };
    }

    const fallbackOk = await runLayoutOnce("force");
    options.onFallbackToForce();
    return {
      requested: layoutType,
      applied: "force",
      reason: primaryOk ? "dagre_degenerate" : fallbackOk ? "dagre_error" : "dagre_and_force_error",
    };
  }

  async function runIncrementalLayout(
    fixedPositions: Map<string, { x: number; y: number }>,
    layoutType: TopologyLayoutType,
  ): Promise<TopologyLayoutRunResult> {
    const cy = options.getCy();
    if (!cy) {
      return { requested: layoutType, applied: layoutType, reason: "no_cy_instance" };
    }

    // Identify new nodes (leaf nodes not present in fixedPositions)
    const leafNodes = cy.nodes().not(":parent");
    const newNodes: cytoscape.NodeSingular[] = [];

    // Apply saved positions first
    cy.batch(() => {
      leafNodes.forEach((node: cytoscape.NodeSingular) => {
        const pos = fixedPositions.get(node.id());
        if (pos) {
          node.position(pos);
        } else {
          newNodes.push(node);
        }
      });
    });

    if (newNodes.length === 0) {
      // All nodes have saved positions — use preset layout (instant)
      return { requested: layoutType, applied: layoutType };
    }

    // Has new nodes — need incremental layout
    // Both dagre and force now use fcose, so both support fixedNodeConstraint

    // Build fixedNodeConstraint for fcose
    const fixedNodeConstraint: Array<{ nodeId: string; position: { x: number; y: number } }> = [];
    leafNodes.forEach((node: cytoscape.NodeSingular) => {
      const pos = fixedPositions.get(node.id());
      if (pos) {
        fixedNodeConstraint.push({ nodeId: node.id(), position: pos });
      }
    });

    const layoutOptions = {
      name: "fcose",
      quality: options.getIsLargeGraph() ? "default" : "proof",
      randomize: false,
      animate: false,
      fit: true,
      padding: 40,
      packComponents: false,
      gravity: 0.4,
      gravityRange: 4.5,
      nodeRepulsion: () => 6000,
      idealEdgeLength: () => 180,
      numIter: options.getIsLargeGraph() ? 1200 : 1800,
      tile: true,
      fixedNodeConstraint,
    } as unknown as cytoscape.LayoutOptions;

    return new Promise((resolve) => {
      let settled = false;
      const finish = (ok: boolean) => {
        if (settled) return;
        settled = true;
        resolve({
          requested: layoutType,
          applied: layoutType,
          reason: ok ? undefined : "incremental_layout_error",
        });
      };

      let layout: ReturnType<Core["layout"]> | null = null;
      try {
        layout = cy.layout(layoutOptions);
      } catch {
        finish(false);
        return;
      }

      layout.one("layoutstop", () => finish(true));
      try {
        layout.run();
      } catch {
        finish(false);
        return;
      }

      globalThis.setTimeout(() => finish(true), options.getIsLargeGraph() ? 2600 : 1800);
    });
  }

  return {
    snapshotLeafNodePositions,
    restoreLeafNodePositions,
    runLayout,
    runIncrementalLayout,
  };
}
