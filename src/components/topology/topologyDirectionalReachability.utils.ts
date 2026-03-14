export interface DirectionalReachabilityEdge {
  id: string;
  source: string;
  target: string;
}

export interface DirectionalReachabilityResult {
  highlightedNodeIds: Set<string>;
  highlightedEdgeIds: Set<string>;
}

type AdjacentNodeResolver = (edge: DirectionalReachabilityEdge) => string;

function buildAdjacencyMap(
  edges: readonly DirectionalReachabilityEdge[],
  from: "source" | "target",
): Map<string, DirectionalReachabilityEdge[]> {
  const adjacency = new Map<string, DirectionalReachabilityEdge[]>();
  for (const edge of edges) {
    const key = edge[from];
    const bucket = adjacency.get(key);
    if (bucket) {
      bucket.push(edge);
      continue;
    }
    adjacency.set(key, [edge]);
  }
  return adjacency;
}

function traverseDirectional(
  focusId: string,
  adjacency: Map<string, DirectionalReachabilityEdge[]>,
  resolveAdjacentNode: AdjacentNodeResolver,
  highlightedNodeIds: Set<string>,
  highlightedEdgeIds: Set<string>,
): void {
  const visitedNodeIds = new Set<string>([focusId]);
  const queue: string[] = [focusId];

  while (queue.length > 0) {
    const currentNodeId = queue.shift();
    if (!currentNodeId) continue;

    const edges = adjacency.get(currentNodeId);
    if (!edges) continue;

    for (const edge of edges) {
      highlightedEdgeIds.add(edge.id);

      const nextNodeId = resolveAdjacentNode(edge);
      if (visitedNodeIds.has(nextNodeId)) continue;

      visitedNodeIds.add(nextNodeId);
      highlightedNodeIds.add(nextNodeId);
      queue.push(nextNodeId);
    }
  }
}

export function collectDirectionalReachability(
  focusId: string,
  edges: readonly DirectionalReachabilityEdge[],
): DirectionalReachabilityResult {
  const highlightedNodeIds = new Set<string>([focusId]);
  const highlightedEdgeIds = new Set<string>();

  const downstreamAdjacency = buildAdjacencyMap(edges, "target");
  const upstreamAdjacency = buildAdjacencyMap(edges, "source");

  traverseDirectional(
    focusId,
    downstreamAdjacency,
    (edge) => edge.source,
    highlightedNodeIds,
    highlightedEdgeIds,
  );
  traverseDirectional(
    focusId,
    upstreamAdjacency,
    (edge) => edge.target,
    highlightedNodeIds,
    highlightedEdgeIds,
  );

  return {
    highlightedNodeIds,
    highlightedEdgeIds,
  };
}

// Backward-compatible alias for callers/tests using the old symbol.
export const computeDirectionalReachability = collectDirectionalReachability;
