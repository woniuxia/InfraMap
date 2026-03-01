import { tauriInvoke } from '@/utils/invoke'
import type { TopologyGraph, PathResult, ImpactResult } from '@/types'

export function getTopologyGraph(): Promise<TopologyGraph> {
  return tauriInvoke<TopologyGraph>('get_topology_graph')
}

export function findPaths(
  sourceId: string,
  targetId: string,
  maxResults?: number
): Promise<PathResult> {
  return tauriInvoke<PathResult>('find_paths', { sourceId, targetId, maxResults })
}

export function analyzeImpact(nodeId: string): Promise<ImpactResult> {
  return tauriInvoke<ImpactResult>('analyze_impact', { nodeId })
}
