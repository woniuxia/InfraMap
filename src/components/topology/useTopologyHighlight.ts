import cytoscape, { type Core } from "cytoscape";
import {
  collectDirectionalReachability,
  type DirectionalReachabilityEdge,
} from "@/components/topology/topologyDirectionalReachability.utils";

type HighlightState =
  | { kind: "none" }
  | { kind: "paths"; paths: string[][] }
  | {
      kind: "impact";
      nodeId: string;
      result: { affectedNodes: { id: string; depth: number }[] };
    }
  | { kind: "neighborhood"; nodeId: string }
  | { kind: "search"; nodeIds: string[]; focusId?: string };

interface UseTopologyHighlightOptions {
  getCy: () => Core | null;
  resolveRenderableNodeId: (rawNodeId: string) => string | null;
  findEdgeId: (source: string, target: string) => string | undefined;
  activateFocusDensity: (durationMs?: number) => void;
  deactivateFocusDensity: (options?: { sync?: boolean }) => void;
}

export function useTopologyHighlight(options: UseTopologyHighlightOptions) {
  let activeHighlightState: HighlightState = { kind: "none" };

  function syncParentDimState() {
    const cy = options.getCy();
    if (!cy) return;
    cy.nodes(":parent").forEach((parent) => {
      const children = parent.children().filter((node) => !node.isParent());
      if (children.length === 0) {
        parent.removeClass("im-dim");
        return;
      }
      const allDim = children.toArray().every((child) => child.hasClass("im-dim"));
      parent.toggleClass("im-dim", allDim);
    });
  }

  function clearHighlightStates() {
    const cy = options.getCy();
    if (!cy) return;
    cy.nodes().removeClass("im-highlight im-dim im-impact");
    cy.edges().removeClass("im-highlight im-dim");
  }

  function applyPathHighlight(paths: string[][]) {
    const cy = options.getCy();
    if (!cy) return;
    clearHighlightStates();

    const nodeIds = new Set<string>();
    const edgeIds = new Set<string>();

    paths.forEach((path) => {
      path.forEach((rawId) => {
        const renderId = options.resolveRenderableNodeId(rawId);
        if (renderId) nodeIds.add(renderId);
      });

      for (let index = 0; index < path.length - 1; index += 1) {
        const source = options.resolveRenderableNodeId(path[index]);
        const target = options.resolveRenderableNodeId(path[index + 1]);
        if (!source || !target) continue;
        const edgeId = options.findEdgeId(source, target);
        if (edgeId) edgeIds.add(edgeId);
      }
    });

    cy.nodes().forEach((node) => {
      if (node.isParent()) return;
      node.addClass(nodeIds.has(node.id()) ? "im-highlight" : "im-dim");
    });
    cy.edges().forEach((edge) => {
      edge.addClass(edgeIds.has(edge.id()) ? "im-highlight" : "im-dim");
    });
    syncParentDimState();
  }

  function applyImpactHighlight(
    nodeId: string,
    result: { affectedNodes: { id: string; depth: number }[] },
  ) {
    const cy = options.getCy();
    if (!cy) return;
    clearHighlightStates();

    const focusNodeId = options.resolveRenderableNodeId(nodeId) || nodeId;
    const affectedIds = new Set<string>([focusNodeId]);
    result.affectedNodes.forEach((item) => {
      const renderId = options.resolveRenderableNodeId(item.id);
      if (renderId) affectedIds.add(renderId);
    });

    cy.nodes().forEach((node) => {
      if (node.isParent()) return;
      if (node.id() === focusNodeId) {
        node.addClass("im-impact");
      } else if (affectedIds.has(node.id())) {
        node.addClass("im-highlight");
      } else {
        node.addClass("im-dim");
      }
    });

    cy.edges().forEach((edge) => {
      const sourceHit = affectedIds.has(edge.source().id());
      const targetHit = affectedIds.has(edge.target().id());
      edge.addClass(sourceHit && targetHit ? "im-highlight" : "im-dim");
    });
    syncParentDimState();
  }

  function applyNeighborhoodHighlight(nodeId: string, allowFocus = true) {
    const cy = options.getCy();
    if (!cy) return;
    clearHighlightStates();

    const renderFocusId = options.resolveRenderableNodeId(nodeId);
    if (!renderFocusId) return;

    const focus = cy.getElementById(renderFocusId);
    if (focus.empty()) return;

    const visibleEdges: DirectionalReachabilityEdge[] = [];
    cy.edges().forEach((edge: cytoscape.EdgeSingular) => {
      visibleEdges.push({
        id: edge.id(),
        source: edge.source().id(),
        target: edge.target().id(),
      });
    });

    const { highlightedNodeIds, highlightedEdgeIds } = collectDirectionalReachability(
      renderFocusId,
      visibleEdges,
    );
    highlightedNodeIds.add(renderFocusId);

    cy.nodes().forEach((node) => {
      if (node.isParent()) return;
      if (node.id() === renderFocusId) {
        node.addClass("im-impact");
      } else if (highlightedNodeIds.has(node.id())) {
        node.addClass("im-highlight");
      } else {
        node.addClass("im-dim");
      }
    });

    cy.edges().forEach((edge) => {
      edge.addClass(highlightedEdgeIds.has(edge.id()) ? "im-highlight" : "im-dim");
    });

    if (allowFocus) {
      cy.animate({
        center: { eles: focus },
        duration: 220,
      });
    }

    syncParentDimState();
  }

  function applySearchHighlight(nodeIds: string[], focusId?: string, allowFocus = true) {
    const cy = options.getCy();
    if (!cy) return;
    clearHighlightStates();

    const matchSet = new Set<string>();
    nodeIds.forEach((id) => {
      const renderId = options.resolveRenderableNodeId(id);
      if (renderId) matchSet.add(renderId);
    });

    cy.nodes().forEach((node) => {
      if (node.isParent()) return;
      node.addClass(matchSet.has(node.id()) ? "im-highlight" : "im-dim");
    });

    if (allowFocus && focusId) {
      const renderFocusId = options.resolveRenderableNodeId(focusId);
      if (renderFocusId) {
        const target = cy.getElementById(renderFocusId);
        if (target.nonempty()) {
          cy.center(target);
        }
      }
    }
    syncParentDimState();
  }

  function applyHighlightState(optionsArg: { allowFocus?: boolean } = {}) {
    const allowFocus = optionsArg.allowFocus !== false;
    if (activeHighlightState.kind === "none") {
      clearHighlightStates();
      return;
    }
    if (activeHighlightState.kind === "paths") {
      applyPathHighlight(activeHighlightState.paths);
      return;
    }
    if (activeHighlightState.kind === "impact") {
      applyImpactHighlight(activeHighlightState.nodeId, activeHighlightState.result);
      return;
    }
    if (activeHighlightState.kind === "neighborhood") {
      applyNeighborhoodHighlight(activeHighlightState.nodeId, allowFocus);
      return;
    }
    applySearchHighlight(activeHighlightState.nodeIds, activeHighlightState.focusId, allowFocus);
  }

  function clearHighlight() {
    activeHighlightState = { kind: "none" };
    options.deactivateFocusDensity();
    applyHighlightState();
  }

  function highlightPaths(paths: string[][]) {
    activeHighlightState = { kind: "paths", paths };
    options.activateFocusDensity();
    applyHighlightState();
  }

  function highlightImpact(
    nodeId: string,
    result: { affectedNodes: { id: string; depth: number }[] },
  ) {
    activeHighlightState = { kind: "impact", nodeId, result };
    options.activateFocusDensity();
    applyHighlightState();
  }

  function highlightSearch(nodeIds: string[], focusId?: string) {
    if (nodeIds.length === 0) {
      clearHighlight();
      return;
    }
    activeHighlightState = { kind: "search", nodeIds, focusId };
    options.activateFocusDensity(10_000);
    applyHighlightState();
  }

  function focusNeighborhood(
    nodeId: string,
    optionsArg: { allowFocus?: boolean; durationMs?: number } = {},
  ) {
    activeHighlightState = { kind: "neighborhood", nodeId };
    options.activateFocusDensity(optionsArg.durationMs ?? 9_000);
    applyHighlightState({ allowFocus: optionsArg.allowFocus });
  }

  function isNeighborhoodHighlightActive(): boolean {
    return activeHighlightState.kind === "neighborhood";
  }

  return {
    applyHighlightState,
    clearHighlight,
    highlightPaths,
    highlightImpact,
    highlightSearch,
    focusNeighborhood,
    isNeighborhoodHighlightActive,
  };
}
