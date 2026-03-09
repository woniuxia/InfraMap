/// <reference lib="webworker" />
import type { ZoomDensity } from "./topologyDensity.utils";
import { selectVisibleEdgeIds } from "./topologyDensity.utils";

interface DensityWorkerRequest {
  requestId: number;
  density: ZoomDensity;
  nodes: { id: string; importance: number }[];
  edges: { id: string; source: string; target: string; strength: number; crossEnv: boolean }[];
}

interface DensityWorkerResponse {
  requestId: number;
  visibleEdgeIds: string[];
}

const workerGlobal = self as DedicatedWorkerGlobalScope;

workerGlobal.onmessage = (event: MessageEvent<DensityWorkerRequest>) => {
  const payload = event.data;
  const visibleEdgeIds = selectVisibleEdgeIds(payload.density, payload.nodes, payload.edges);
  const response: DensityWorkerResponse = {
    requestId: payload.requestId,
    visibleEdgeIds,
  };
  workerGlobal.postMessage(response);
};
