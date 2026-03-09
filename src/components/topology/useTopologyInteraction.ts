import type { Core, EventObject, EventObjectNode } from "cytoscape";
import type { TopologyNode } from "@/types";

interface UseTopologyInteractionOptions {
  getCy: () => Core | null;
  getContainer: () => HTMLDivElement | undefined;
  focusNeighborhoodEnabled: () => boolean;
  focusNeighborhood: (
    nodeId: string,
    options?: { allowFocus?: boolean; durationMs?: number },
  ) => void;
  findNodeById: (nodeId: string) => TopologyNode | undefined;
  emitNodeClick: (node: TopologyNode) => void;
  emitNodeContextmenu: (payload: { node: TopologyNode; x: number; y: number }) => void;
  emitCanvasBlankClick: () => void;
}

export function useTopologyInteraction(options: UseTopologyInteractionOptions) {
  function getContextMenuPosition(evt: EventObjectNode): {
    x: number;
    y: number;
  } {
    const mouse = evt.originalEvent as MouseEvent | undefined;
    if (mouse && Number.isFinite(mouse.clientX) && Number.isFinite(mouse.clientY)) {
      return { x: mouse.clientX, y: mouse.clientY };
    }

    const rect = options.getContainer()?.getBoundingClientRect();
    const rendered = evt.renderedPosition || evt.target.renderedPosition();
    if (rect && rendered) {
      return {
        x: rect.left + rendered.x,
        y: rect.top + rendered.y,
      };
    }

    return { x: 0, y: 0 };
  }

  function handleCanvasTap(evt: EventObject) {
    const cy = options.getCy();
    if (!cy) return;
    if (evt.target !== cy) return;
    options.emitCanvasBlankClick();
  }

  function handleNodeTap(evt: EventObjectNode) {
    const target = evt.target;
    if (typeof target.isNode !== "function" || !target.isNode()) return;
    if (typeof target.isParent === "function" && target.isParent()) return;
    if (options.focusNeighborhoodEnabled()) {
      options.focusNeighborhood(target.id(), {
        allowFocus: false,
        durationMs: 9_000,
      });
    }
    const node = options.findNodeById(target.id());
    if (node) options.emitNodeClick(node);
  }

  function handleNodeContextTap(evt: EventObjectNode) {
    const target = evt.target;
    if (typeof target.isNode !== "function" || !target.isNode()) return;
    if (typeof target.isParent === "function" && target.isParent()) return;
    const node = options.findNodeById(target.id());
    if (!node) return;
    const position = getContextMenuPosition(evt);
    options.emitNodeContextmenu({ node, x: position.x, y: position.y });
  }

  return {
    handleCanvasTap,
    handleNodeTap,
    handleNodeContextTap,
  };
}
