import type { TopologyEdge, TopologyNode } from "@/types";

export type ZoomDensity = "overview" | "medium" | "detail";

export interface DensityOptions {
  label: string;
  minEdgeStrength: number;
  maxEdges: number;
  nodeEdgeCap: number;
  hideEdgeLabels: boolean;
}

export const DENSITY_OPTIONS: Record<ZoomDensity, DensityOptions> = {
  overview: {
    label: "总览",
    minEdgeStrength: 2,
    maxEdges: 180,
    nodeEdgeCap: 4,
    hideEdgeLabels: true,
  },
  medium: {
    label: "标准",
    minEdgeStrength: 1,
    maxEdges: 420,
    nodeEdgeCap: 8,
    hideEdgeLabels: true,
  },
  detail: {
    label: "细节",
    minEdgeStrength: 1,
    maxEdges: Number.POSITIVE_INFINITY,
    nodeEdgeCap: Number.POSITIVE_INFINITY,
    hideEdgeLabels: false,
  },
};

export interface DensityNodeInput {
  id: string;
  importance: number;
}

export interface DensityEdgeInput {
  id: string;
  source: string;
  target: string;
  strength: number;
  crossEnv: boolean;
}

function scoreEdge(edge: DensityEdgeInput, nodeImportance: Map<string, number>): number {
  const sourceImportance = nodeImportance.get(edge.source) || 0;
  const targetImportance = nodeImportance.get(edge.target) || 0;
  const crossEnvWeight = edge.crossEnv ? 220 : 0;
  return crossEnvWeight + edge.strength * 18 + sourceImportance * 6 + targetImportance * 6;
}

export function getDensityByZoom(zoom: number): ZoomDensity {
  if (zoom < 0.82) return "overview";
  if (zoom < 1.42) return "medium";
  return "detail";
}

export function selectVisibleEdgeIds(
  density: ZoomDensity,
  nodes: DensityNodeInput[],
  edges: DensityEdgeInput[],
): string[] {
  if (density === "detail") {
    return edges.map((edge) => edge.id);
  }

  const options = DENSITY_OPTIONS[density];
  const nodeImportance = new Map<string, number>(nodes.map((node) => [node.id, node.importance]));
  const rankedEdges = [...edges].sort((left, right) => {
    const scoreDiff = scoreEdge(right, nodeImportance) - scoreEdge(left, nodeImportance);
    if (scoreDiff !== 0) return scoreDiff;
    if (left.crossEnv !== right.crossEnv) return left.crossEnv ? -1 : 1;
    if (left.strength !== right.strength) return right.strength - left.strength;
    return left.id.localeCompare(right.id);
  });

  const nodeDegree = new Map<string, number>();
  const visibleEdgeIds: string[] = [];

  for (const edge of rankedEdges) {
    const sourceDegree = nodeDegree.get(edge.source) || 0;
    const targetDegree = nodeDegree.get(edge.target) || 0;
    const mustKeep = edge.crossEnv || edge.strength >= Math.max(3, options.minEdgeStrength + 1);
    const weakEdge = edge.strength < options.minEdgeStrength;
    const exceedsDegreeCap =
      sourceDegree >= options.nodeEdgeCap || targetDegree >= options.nodeEdgeCap;

    if (!mustKeep && weakEdge) continue;
    if (!mustKeep && exceedsDegreeCap) continue;

    visibleEdgeIds.push(edge.id);
    nodeDegree.set(edge.source, sourceDegree + 1);
    nodeDegree.set(edge.target, targetDegree + 1);

    if (visibleEdgeIds.length >= options.maxEdges) break;
  }

  visibleEdgeIds.sort((left, right) => left.localeCompare(right));
  return visibleEdgeIds;
}

export function normalizeNodesForDensity(nodes: TopologyNode[]): DensityNodeInput[] {
  return nodes.map((node) => ({
    id: node.id,
    importance: node.importance ?? 0,
  }));
}

export function normalizeEdgesForDensity(edges: TopologyEdge[]): DensityEdgeInput[] {
  return edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    strength: edge.strength ?? 1,
    crossEnv: edge.crossEnv ?? false,
  }));
}
