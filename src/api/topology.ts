import { tauriInvoke } from "@/utils/invoke";
import type {
  TopologyEvidenceQuery,
  TopologyEvidenceResponse,
  TopologyImpactQuery,
  TopologyImpactResponse,
  TopologyPathsQuery,
  TopologyPathsResponse,
  TopologySnapshotQuery,
  TopologySnapshotResponse,
  TopologyTaskViewQuery,
  TopologyTaskViewResponse,
} from "@/types";

export function getTopologySnapshot(
  query: TopologySnapshotQuery = {},
): Promise<TopologySnapshotResponse> {
  return tauriInvoke<TopologySnapshotResponse>("get_topology_snapshot_v3", {
    query,
  });
}

export function getTopologyTaskView(
  query: TopologyTaskViewQuery,
): Promise<TopologyTaskViewResponse> {
  return tauriInvoke<TopologyTaskViewResponse>("get_topology_task_view_v3", {
    query,
  });
}

export function getTopologyPaths(query: TopologyPathsQuery): Promise<TopologyPathsResponse> {
  return tauriInvoke<TopologyPathsResponse>("get_topology_paths_v3", { query });
}

export function getTopologyImpact(query: TopologyImpactQuery): Promise<TopologyImpactResponse> {
  return tauriInvoke<TopologyImpactResponse>("get_topology_impact_v3", {
    query,
  });
}

export function getTopologyEvidence(
  query: TopologyEvidenceQuery,
): Promise<TopologyEvidenceResponse> {
  return tauriInvoke<TopologyEvidenceResponse>("get_topology_evidence_v3", {
    query,
  });
}
