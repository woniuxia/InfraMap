import { tauriInvoke } from '@/utils/invoke'
import type {
  TopologyV3DrilldownQuery,
  TopologyV3DrilldownResponse,
  TopologyV3EvidenceQuery,
  TopologyV3EvidenceResponse,
  TopologyV3ImpactQuery,
  TopologyV3ImpactResponse,
  TopologyV3PathsQuery,
  TopologyV3PathsResponse,
  TopologyV3SnapshotQuery,
  TopologyV3SnapshotResponse,
  TopologyV3TaskViewQuery,
  TopologyV3TaskViewResponse,
  TopologyV3TroubleshootReport,
  TopologyV3TroubleshootReportQuery,
} from '@/types'

export function getTopologySnapshotV3(query: TopologyV3SnapshotQuery = {}): Promise<TopologyV3SnapshotResponse> {
  return tauriInvoke<TopologyV3SnapshotResponse>('get_topology_snapshot_v3', { query })
}

export function getTopologyDrilldownV3(query: TopologyV3DrilldownQuery): Promise<TopologyV3DrilldownResponse> {
  return tauriInvoke<TopologyV3DrilldownResponse>('get_topology_drilldown_v3', { query })
}

export function getTopologyTaskViewV3(query: TopologyV3TaskViewQuery): Promise<TopologyV3TaskViewResponse> {
  return tauriInvoke<TopologyV3TaskViewResponse>('get_topology_task_view_v3', { query })
}

export function getTopologyPathsV3(query: TopologyV3PathsQuery): Promise<TopologyV3PathsResponse> {
  return tauriInvoke<TopologyV3PathsResponse>('get_topology_paths_v3', { query })
}

export function getTopologyImpactV3(query: TopologyV3ImpactQuery): Promise<TopologyV3ImpactResponse> {
  return tauriInvoke<TopologyV3ImpactResponse>('get_topology_impact_v3', { query })
}

export function getTopologyEvidenceV3(query: TopologyV3EvidenceQuery): Promise<TopologyV3EvidenceResponse> {
  return tauriInvoke<TopologyV3EvidenceResponse>('get_topology_evidence_v3', { query })
}

export function getTopologyTroubleshootReportV3(
  query: TopologyV3TroubleshootReportQuery,
): Promise<TopologyV3TroubleshootReport> {
  return tauriInvoke<TopologyV3TroubleshootReport>('get_topology_troubleshoot_report_v3', { query })
}
