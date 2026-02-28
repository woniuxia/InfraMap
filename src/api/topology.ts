import { tauriInvoke } from '@/utils/invoke'
import type { TopologyGraph, PathResult, ImpactResult } from '@/types'

export function getTopologyGraph(): Promise<TopologyGraph> {
  return tauriInvoke<TopologyGraph>('get_topology_graph')
}

export function findPaths(
  source_id: string,
  target_id: string,
  max_results?: number
): Promise<PathResult> {
  return tauriInvoke<PathResult>('find_paths', { source_id, target_id, max_results })
}

export function analyzeImpact(node_id: string): Promise<ImpactResult> {
  return tauriInvoke<ImpactResult>('analyze_impact', { node_id })
}
