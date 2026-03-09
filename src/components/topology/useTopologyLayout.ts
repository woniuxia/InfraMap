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
        nodeSep: 72,
        rankSep: 180,
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
      nodeRepulsion: (node: cytoscape.NodeSingular) => (node.data("isExternal") ? 8000 : 12000),
      idealEdgeLength: (edge: cytoscape.EdgeSingular) => (edge.data("crossEnv") ? 240 : 180),
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

  return {
    snapshotLeafNodePositions,
    restoreLeafNodePositions,
    runLayout,
  };
}
