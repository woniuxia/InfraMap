import { computed, type Ref } from "vue";
import type { TopologyGraph, TopologyNode } from "@/types";
import { toExternalNodeId } from "@/components/topology/topologyGraph.utils";
import {
  DENSITY_OPTIONS,
  getDensityByZoom,
  normalizeEdgesForDensity,
  normalizeNodesForDensity,
  selectVisibleEdgeIds,
  type ZoomDensity,
} from "@/components/topology/topologyDensity.utils";

interface ViewportState {
  zoom: number;
  density: ZoomDensity;
  totalEdges: number;
  visibleEdges: number;
}

interface DensityWorkerRequest {
  requestId: number;
  density: ZoomDensity;
  nodes: { id: string; importance: number }[];
  edges: {
    id: string;
    source: string;
    target: string;
    strength: number;
    crossEnv: boolean;
  }[];
}

interface DensityWorkerResponse {
  requestId: number;
  visibleEdgeIds: string[];
}

interface UseTopologyDensityOptions {
  performanceOptimizationEnabled: Readonly<Ref<boolean>>;
  viewportState: Ref<ViewportState>;
  requestSyncGraphData: () => void;
}

export function useTopologyDensity(options: UseTopologyDensityOptions) {
  let nodeById = new Map<string, TopologyNode>();
  let renderableNodeIds = new Set<string>();
  let edgeIdByPair = new Map<string, string>();
  let zoomDensityRaf: number | null = null;
  let densityWorker: Worker | null = null;
  let densityWorkerRequestId = 0;
  const densityWorkerResolvers = new Map<number, (edgeIds?: string[]) => void>();
  const densitySelectionCache = new WeakMap<
    TopologyGraph,
    Partial<Record<ZoomDensity, string[]>>
  >();
  let forcedDensity: ZoomDensity | null = null;
  let pendingDensityAfterFocus: ZoomDensity | null = null;
  let focusDensityTimer: number | null = null;

  function resolveZoomDensity(zoom: number): ZoomDensity {
    if (!options.performanceOptimizationEnabled.value) return "detail";
    return getDensityByZoom(zoom);
  }

  function getActiveDensity(): ZoomDensity {
    if (!options.performanceOptimizationEnabled.value) return "detail";
    return forcedDensity || options.viewportState.value.density;
  }

  const densityHintText = computed(() => {
    const density = DENSITY_OPTIONS[getActiveDensity()];
    const suffix = !options.performanceOptimizationEnabled.value
      ? "（性能优化关闭）"
      : forcedDensity
        ? "（聚焦）"
        : "";
    return `渲染: ${density.label}${suffix} · 关系 ${options.viewportState.value.visibleEdges}/${options.viewportState.value.totalEdges}`;
  });

  function edgePairKey(source: string, target: string): string {
    return `${source}=>${target}`;
  }

  function rebuildIndexes(graphData: TopologyGraph | null) {
    nodeById = new Map();
    renderableNodeIds = new Set();
    edgeIdByPair = new Map();

    if (!graphData) return;

    graphData.nodes.forEach((node) => {
      nodeById.set(node.id, node);
      renderableNodeIds.add(node.id);
    });

    graphData.edges.forEach((edge) => {
      edgeIdByPair.set(edgePairKey(edge.source, edge.target), edge.id);
    });
  }

  function resolveRenderableNodeId(rawNodeId: string): string | null {
    if (renderableNodeIds.has(rawNodeId)) return rawNodeId;
    const externalId = toExternalNodeId(rawNodeId);
    if (renderableNodeIds.has(externalId)) return externalId;
    return null;
  }

  function findNodeById(nodeId: string): TopologyNode | undefined {
    return nodeById.get(nodeId);
  }

  function findEdgeId(source: string, target: string): string | undefined {
    return edgeIdByPair.get(edgePairKey(source, target));
  }

  function cacheVisibleEdgeIds(graphData: TopologyGraph, density: ZoomDensity, edgeIds: string[]) {
    const cache = densitySelectionCache.get(graphData) || {};
    cache[density] = edgeIds;
    densitySelectionCache.set(graphData, cache);
  }

  function ensureDensityWorker(): Worker | null {
    if (densityWorker || typeof Worker === "undefined") return densityWorker;

    densityWorker = new Worker(new URL("./topologyDensity.worker.ts", import.meta.url), {
      type: "module",
    });
    densityWorker.onmessage = (event: MessageEvent<DensityWorkerResponse>) => {
      const { requestId, visibleEdgeIds } = event.data;
      const resolve = densityWorkerResolvers.get(requestId);
      if (!resolve) return;
      densityWorkerResolvers.delete(requestId);
      resolve(visibleEdgeIds);
    };
    densityWorker.onerror = () => {
      densityWorkerResolvers.forEach((resolve) => resolve(undefined));
      densityWorkerResolvers.clear();
      densityWorker?.terminate();
      densityWorker = null;
    };

    return densityWorker;
  }

  async function resolveVisibleEdgeIds(
    graphData: TopologyGraph,
    density: ZoomDensity,
  ): Promise<string[]> {
    const cache = densitySelectionCache.get(graphData);
    const cachedEdgeIds = cache?.[density];
    if (cachedEdgeIds) return cachedEdgeIds;

    const nodes = normalizeNodesForDensity(graphData.nodes);
    const edges = normalizeEdgesForDensity(graphData.edges);

    const computeSynchronously = () => {
      const edgeIds = selectVisibleEdgeIds(density, nodes, edges);
      cacheVisibleEdgeIds(graphData, density, edgeIds);
      return edgeIds;
    };

    if (density === "detail" || edges.length < 260) {
      return computeSynchronously();
    }

    const worker = ensureDensityWorker();
    if (!worker) {
      return computeSynchronously();
    }

    const requestId = ++densityWorkerRequestId;
    const payload: DensityWorkerRequest = {
      requestId,
      density,
      nodes,
      edges,
    };

    return new Promise((resolve) => {
      const timeoutId = globalThis.setTimeout(() => {
        densityWorkerResolvers.delete(requestId);
        resolve(computeSynchronously());
      }, 180);

      densityWorkerResolvers.set(requestId, (edgeIds?: string[]) => {
        globalThis.clearTimeout(timeoutId);
        if (edgeIds === undefined) {
          resolve(computeSynchronously());
          return;
        }
        cacheVisibleEdgeIds(graphData, density, edgeIds);
        resolve(edgeIds);
      });

      try {
        worker.postMessage(payload);
      } catch {
        densityWorkerResolvers.delete(requestId);
        globalThis.clearTimeout(timeoutId);
        resolve(computeSynchronously());
      }
    });
  }

  async function buildDensityGraphData(
    graphData: TopologyGraph,
    density: ZoomDensity,
  ): Promise<TopologyGraph> {
    if (!options.performanceOptimizationEnabled.value) {
      return graphData;
    }
    const edgeIds = await resolveVisibleEdgeIds(graphData, density);
    const edgeIdSet = new Set(edgeIds);
    return {
      ...graphData,
      edges: graphData.edges.filter((edge) => edgeIdSet.has(edge.id)),
    };
  }

  function clearFocusDensityTimer() {
    if (focusDensityTimer) {
      globalThis.clearTimeout(focusDensityTimer);
      focusDensityTimer = null;
    }
  }

  function scheduleDensitySync() {
    if (zoomDensityRaf) {
      cancelAnimationFrame(zoomDensityRaf);
    }
    zoomDensityRaf = requestAnimationFrame(() => {
      zoomDensityRaf = null;
      options.requestSyncGraphData();
    });
  }

  function deactivateFocusDensity(optionsArg: { sync?: boolean } = {}) {
    const shouldSync = optionsArg.sync !== false;
    if (!forcedDensity) return;

    forcedDensity = null;
    clearFocusDensityTimer();
    const nextDensity =
      pendingDensityAfterFocus || resolveZoomDensity(options.viewportState.value.zoom);
    pendingDensityAfterFocus = null;
    if (options.viewportState.value.density !== nextDensity) {
      options.viewportState.value.density = nextDensity;
      if (shouldSync) scheduleDensitySync();
    } else if (shouldSync) {
      scheduleDensitySync();
    }
  }

  function activateFocusDensity(durationMs = 14_000) {
    if (!options.performanceOptimizationEnabled.value) return;
    forcedDensity = "detail";
    pendingDensityAfterFocus = resolveZoomDensity(options.viewportState.value.zoom);
    clearFocusDensityTimer();
    focusDensityTimer = globalThis.setTimeout(() => {
      focusDensityTimer = null;
      deactivateFocusDensity();
    }, durationMs);

    if (options.viewportState.value.density !== "detail") {
      options.viewportState.value.density = "detail";
      scheduleDensitySync();
    }
  }

  function handleViewportZoomChange(zoom: number) {
    options.viewportState.value.zoom = zoom;
    const nextDensity = resolveZoomDensity(zoom);
    if (forcedDensity) {
      pendingDensityAfterFocus = nextDensity;
      return;
    }
    if (nextDensity !== options.viewportState.value.density) {
      options.viewportState.value.density = nextDensity;
      scheduleDensitySync();
    }
  }

  function resetDensityState() {
    forcedDensity = null;
    pendingDensityAfterFocus = null;
    clearFocusDensityTimer();
    options.viewportState.value.density = resolveZoomDensity(options.viewportState.value.zoom);
  }

  function dispose() {
    if (zoomDensityRaf) {
      cancelAnimationFrame(zoomDensityRaf);
      zoomDensityRaf = null;
    }

    clearFocusDensityTimer();
    forcedDensity = null;
    pendingDensityAfterFocus = null;

    if (densityWorker) {
      densityWorker.terminate();
      densityWorker = null;
    }
    densityWorkerResolvers.forEach((resolve) => resolve(undefined));
    densityWorkerResolvers.clear();
  }

  return {
    densityHintText,
    resolveZoomDensity,
    getActiveDensity,
    rebuildIndexes,
    resolveRenderableNodeId,
    findNodeById,
    findEdgeId,
    buildDensityGraphData,
    activateFocusDensity,
    deactivateFocusDensity,
    handleViewportZoomChange,
    clearFocusDensityTimer,
    resetDensityState,
    dispose,
  };
}
