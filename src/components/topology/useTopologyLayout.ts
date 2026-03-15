import cytoscape, { type Core, type LayoutOptions } from "cytoscape";
import { isDegenerateNodeDistribution } from "@/components/topology/topologyLayoutSafety.utils";

export type TopologyLayoutType = "force" | "dagre";

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

  function buildLayoutOptions(layoutType: TopologyLayoutType): LayoutOptions {
    if (layoutType === "dagre") {
      return {
        name: "dagre",
        rankDir: "LR",
        nodeSep: 88,
        rankSep: 210,
        fit: true,
        padding: 36,
        animate: false,
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
    if (layoutType !== "dagre") {
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
    if (layoutType === "dagre") {
      // dagre doesn't support fixedNodeConstraint, run full dagre
      return runLayout(layoutType);
    }

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
