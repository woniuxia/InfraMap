export interface TopologyLayoutPoint {
  x: number;
  y: number;
}

export interface TopologyNodeIdentity {
  id: string;
}

export function hasTopologyNodeSetChanged(
  previousNodeIds: Set<string>,
  nextNodes: TopologyNodeIdentity[],
): boolean {
  if (previousNodeIds.size !== nextNodes.length) return true;
  for (const node of nextNodes) {
    if (!previousNodeIds.has(node.id)) return true;
  }
  return false;
}

export function isDegenerateNodeDistribution(
  points: TopologyLayoutPoint[],
  minNodeCount = 8,
): boolean {
  if (points.length < minNodeCount) return false;

  let minX = Number.POSITIVE_INFINITY;
  let minY = Number.POSITIVE_INFINITY;
  let maxX = Number.NEGATIVE_INFINITY;
  let maxY = Number.NEGATIVE_INFINITY;

  for (const point of points) {
    minX = Math.min(minX, point.x);
    minY = Math.min(minY, point.y);
    maxX = Math.max(maxX, point.x);
    maxY = Math.max(maxY, point.y);
  }

  const width = Math.max(0, maxX - minX);
  const height = Math.max(0, maxY - minY);
  if (width < 4 && height < 4) return true;

  const bucketKeys = new Set(
    points.map((point) => `${Math.round(point.x / 4)},${Math.round(point.y / 4)}`),
  );
  const minExpectedBuckets = Math.max(3, Math.floor(points.length * 0.06));
  return bucketKeys.size <= minExpectedBuckets;
}
