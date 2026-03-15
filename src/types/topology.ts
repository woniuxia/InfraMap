export type TopologyEnv = "prod" | "test" | "dev";

export type TopologyNodeType = "service" | "middleware" | "nginx";

export type TopologyGroupKind = "service" | "middleware" | "nginx";

export type TopologyEdgeType =
  | "http_call"
  | "tcp"
  | "mq_produce"
  | "mq_consume"
  | "grpc_call"
  | "db_query"
  | "cache_access";

export interface TopologyLane {
  id: TopologyEnv;
  label: string;
  order: number;
  nodeCount: number;
  appCount: number;
}

export interface TopologyNode {
  id: string;
  name: string;
  nodeType?: TopologyNodeType;
  node_type?: TopologyNodeType;
  groupKind?: TopologyGroupKind;
  group_kind?: TopologyGroupKind;
  env: TopologyEnv;
  hostId?: string;
  host_id?: string;
  hostName?: string;
  host_name?: string;
  hostIpDisplay?: string;
  host_ip_display?: string;
  systemId?: string;
  system_id?: string;
  systemName?: string;
  system_name?: string;
  status?: string;
  importance?: number;
  isExternal?: boolean;
  is_external?: boolean;
  externalRefId?: string;
  external_ref_id?: string;
  tags?: string[];
  extra?: Record<string, unknown>;
  meta?: Record<string, unknown>;
}

export interface TopologyEdge {
  id: string;
  source: string;
  target: string;
  edgeType?: TopologyEdgeType;
  edge_type?: TopologyEdgeType;
  label?: string;
  strength?: number;
  crossEnv?: boolean;
  cross_env?: boolean;
  extra?: Record<string, unknown>;
  meta?: Record<string, unknown>;
}

export interface TopologyEnvCount {
  env: TopologyEnv;
  count: number;
  appCount: number;
}

export interface TopologyKindCount {
  kind: string;
  count: number;
}

export interface TopologyLegendStats {
  envCounts: TopologyEnvCount[];
  nodeTypeCounts: TopologyKindCount[];
  edgeTypeCounts: TopologyKindCount[];
  serviceCount: number;
  currentEnv?: TopologyEnv;
  externalNodeCount?: number;
  crossEnvEdgeCount?: number;
}

export interface TopologyLayoutHints {
  laneOrder: TopologyEnv[];
  defaultCollapsedGroups: string[];
  highDensityMode: boolean;
}

export interface TopologyGraph {
  lanes: TopologyLane[];
  nodes: TopologyNode[];
  edges: TopologyEdge[];
  legendStats: TopologyLegendStats;
  layoutHints: TopologyLayoutHints;
}

export type TopologyTaskViewMode = "explore" | "troubleshoot" | "impact" | "change";

export type TopologyDrilldownDirection = "upstream" | "downstream" | "both";

export interface TopologySnapshotQuery {
  env?: TopologyEnv;
  taskView?: TopologyTaskViewMode;
  maxDepth?: number;
  focusNodeId?: string;
  includeEvidence?: boolean;
}

export interface TopologyDrilldownQuery {
  nodeId: string;
  taskView?: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
  direction?: TopologyDrilldownDirection;
}

export interface TopologyTaskViewQuery {
  taskView: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
  focusNodeId?: string;
  nodeId?: string;
}

export interface TopologyPathsQuery {
  sourceId: string;
  targetId: string;
  taskView?: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
  maxResults?: number;
}

export interface TopologyImpactQuery {
  nodeId: string;
  taskView?: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
}

export interface TopologyEvidenceQuery {
  nodeId: string;
  edgeId?: string;
  relatedNodeId?: string;
  taskView?: TopologyTaskViewMode;
  maxItems?: number;
}

export interface TopologyTroubleshootReportQuery {
  nodeId: string;
  taskView?: TopologyTaskViewMode;
  evidenceLimit?: number;
}

export interface TopologyMeta {
  snapshotId?: string;
  generatedAt?: string;
  version?: string;
  taskView?: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
  nodeCount?: number;
  edgeCount?: number;
  truncated?: boolean;
  warnings?: string[];
  [key: string]: unknown;
}

export interface TopologySnapshotResponse {
  meta: TopologyMeta;
  lanes?: TopologyLane[];
  nodes: TopologyNode[];
  edges: TopologyEdge[];
  legendStats?: TopologyLegendStats;
  legend_stats?: TopologyLegendStats;
  layoutHints?: TopologyLayoutHints;
  layout_hints?: TopologyLayoutHints;
  focusNode?: TopologyNode;
  focus_node?: TopologyNode;
  nodeCount?: number;
  node_count?: number;
  edgeCount?: number;
  edge_count?: number;
}

export interface TopologyNeighbor {
  id: string;
  name: string;
  nodeType?: TopologyNodeType;
  node_type?: TopologyNodeType;
  env: TopologyEnv;
}

export interface TopologyDrilldownResponse {
  node: TopologyNode;
  upstream: TopologyNeighbor[];
  downstream: TopologyNeighbor[];
  inboundEdgeCount?: number;
  inbound_edge_count?: number;
  outboundEdgeCount?: number;
  outbound_edge_count?: number;
}

export interface TopologyTaskInsight {
  kind: string;
  title: string;
  description?: string;
  severity?: "info" | "warning" | "critical";
  nodeIds?: string[];
  node_ids?: string[];
  edgeIds?: string[];
  edge_ids?: string[];
}

export interface TopologyTaskViewResponse {
  taskView: TopologyTaskViewMode;
  snapshot: TopologySnapshotResponse;
  title?: string;
  summary?: string;
  insights?: TopologyTaskInsight[];
  meta?: TopologyMeta;
}

export interface TopologyPath {
  nodeIds?: string[];
  node_ids?: string[];
  edgeIds?: string[];
  edge_ids?: string[];
  score?: number;
}

export interface TopologyPathsResponse {
  paths: TopologyPath[];
  truncated: boolean;
  totalCount?: number;
  total_count?: number;
  meta?: TopologyMeta;
}

export interface TopologyImpactNode {
  id: string;
  name: string;
  nodeType?: string;
  node_type?: string;
  depth: number;
  reason?: string;
}

export interface TopologyImpactResponse {
  affectedNodes?: TopologyImpactNode[];
  affected_nodes?: TopologyImpactNode[];
  totalCount?: number;
  total_count?: number;
  maxDepth?: number;
  max_depth?: number;
  truncated?: boolean;
  meta?: TopologyMeta;
}

export type TopologyEvidenceType =
  | "profile"
  | "dependency_summary"
  | "deployment"
  | "audit"
  | "call_relation"
  | "metric"
  | "alert"
  | "change"
  | "annotation";

export interface TopologyEvidenceItem {
  id: string;
  evidenceType?: TopologyEvidenceType;
  evidence_type?: TopologyEvidenceType;
  title: string;
  description?: string;
  severity?: "info" | "warning" | "critical";
  source?: string;
  timestamp?: string;
  nodeId?: string;
  edgeId?: string;
  payload?: Record<string, unknown>;
}

export interface TopologyEvidenceResponse {
  items: TopologyEvidenceItem[];
  total: number;
  meta?: TopologyMeta;
}

export interface TopologyTroubleshootSummary {
  inboundEdgeCount: number;
  outboundEdgeCount: number;
  deploymentCount: number;
  recentAuditCount: number;
  statusSeverity: "info" | "warning" | "critical";
}

export interface TopologyTroubleshootReport {
  node: TopologyNode;
  summary: TopologyTroubleshootSummary;
  upstream: TopologyNeighbor[];
  downstream: TopologyNeighbor[];
  evidence: TopologyEvidenceResponse;
  insights: TopologyTaskInsight[];
}

// ── Node position persistence ──

export interface TopologyNodePosition {
  node_id: string;
  layout_type: string;
  focus_target?: string | null;
  x: number;
  y: number;
}

export interface TopologyNodePositionEntry {
  node_id: string;
  x: number;
  y: number;
}

export interface SystemFocusOption {
  systemId: string;
  systemName: string;
  serviceCount: number;
  serviceIds: string[];
  isStandalone: boolean;
  nodeType?: string;
}

export type TopologyLayoutType = "force" | "focus";
