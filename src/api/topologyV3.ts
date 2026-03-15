import { tauriInvoke } from "@/utils/invoke";
import type {
  TopologyDrilldownQuery,
  TopologyDrilldownResponse,
  TopologyEvidenceQuery,
  TopologyEvidenceResponse,
  TopologyImpactQuery,
  TopologyImpactResponse,
  TopologyNodePosition,
  TopologyNodePositionEntry,
  TopologyPathsQuery,
  TopologyPathsResponse,
  TopologySnapshotQuery,
  TopologySnapshotResponse,
  TopologyTaskViewQuery,
  TopologyTaskViewResponse,
  TopologyTroubleshootReport,
  TopologyTroubleshootReportQuery,
} from "@/types";

export function getTopologySnapshotV3(
  query: TopologySnapshotQuery = {},
): Promise<TopologySnapshotResponse> {
  return tauriInvoke<TopologySnapshotResponse>("get_topology_snapshot_v3", { query });
}

export function getTopologyDrilldownV3(
  query: TopologyDrilldownQuery,
): Promise<TopologyDrilldownResponse> {
  return tauriInvoke<TopologyDrilldownResponse>("get_topology_drilldown_v3", { query });
}

export function getTopologyTaskViewV3(
  query: TopologyTaskViewQuery,
): Promise<TopologyTaskViewResponse> {
  return tauriInvoke<TopologyTaskViewResponse>("get_topology_task_view_v3", { query });
}

export function getTopologyPathsV3(query: TopologyPathsQuery): Promise<TopologyPathsResponse> {
  return tauriInvoke<TopologyPathsResponse>("get_topology_paths_v3", { query });
}

export function getTopologyImpactV3(query: TopologyImpactQuery): Promise<TopologyImpactResponse> {
  return tauriInvoke<TopologyImpactResponse>("get_topology_impact_v3", { query });
}

export function getTopologyEvidenceV3(
  query: TopologyEvidenceQuery,
): Promise<TopologyEvidenceResponse> {
  return tauriInvoke<TopologyEvidenceResponse>("get_topology_evidence_v3", { query });
}

export function getTopologyTroubleshootReportV3(
  query: TopologyTroubleshootReportQuery,
): Promise<TopologyTroubleshootReport> {
  return tauriInvoke<TopologyTroubleshootReport>("get_topology_troubleshoot_report_v3", { query });
}

// ── Node position persistence ──

export function getTopologyNodePositions(layoutType: string): Promise<TopologyNodePosition[]> {
  return tauriInvoke<TopologyNodePosition[]>("get_topology_node_positions", {
    layoutType,
  });
}

export function saveTopologyNodePositions(
  layoutType: string,
  positions: TopologyNodePositionEntry[],
): Promise<void> {
  return tauriInvoke<void>("save_topology_node_positions", {
    data: { layout_type: layoutType, positions },
  });
}

export function clearTopologyNodePositions(layoutType: string): Promise<void> {
  return tauriInvoke<void>("clear_topology_node_positions", { layoutType });
}
