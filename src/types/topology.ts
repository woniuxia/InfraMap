export type TopologyEnv = "prod" | "test" | "dev";

export type TopologyNodeType = "application" | "middleware" | "nginx";

export type TopologyGroupKind = "application_service" | "middleware" | "nginx";

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
  nodeType: TopologyNodeType;
  env: TopologyEnv;
  groupKind: TopologyGroupKind;
  hostId?: string;
  hostName?: string;
  hostIpDisplay?: string;
  status?: string;
  importance: number;
  tags?: string[];
  extra?: Record<string, unknown>;
  meta?: Record<string, unknown>;
  isExternal?: boolean;
  externalRefId?: string;
}

export interface TopologyEdge {
  id: string;
  source: string;
  target: string;
  edgeType: TopologyEdgeType;
  label?: string;
  strength: number;
  crossEnv: boolean;
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
  applicationServiceCount: number;
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

export interface TopologySnapshotQuery {
  env?: TopologyEnv;
  taskView?: TopologyTaskViewMode;
  maxDepth?: number;
  focusNodeId?: string;
}

export interface TopologyTaskViewQuery {
  taskView: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
  focusNodeId?: string;
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

export interface TopologySnapshotResponse extends TopologyGraph {
  meta: TopologyMeta;
  focusNode?: TopologyNode;
  nodeCount?: number;
  edgeCount?: number;
}

export interface TopologyTaskInsight {
  kind: string;
  title: string;
  description?: string;
  severity?: "info" | "warning" | "critical";
  nodeIds?: string[];
  edgeIds?: string[];
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
  nodeIds: string[];
  edgeIds?: string[];
  score?: number;
}

export interface TopologyPathsResponse {
  paths: TopologyPath[];
  truncated: boolean;
  totalCount: number;
  meta?: TopologyMeta;
}

export interface TopologyImpactNode {
  id: string;
  name: string;
  nodeType: string;
  depth: number;
  reason?: string;
}

export interface TopologyImpactResponse {
  affectedNodes: TopologyImpactNode[];
  totalCount: number;
  maxDepth: number;
  severity: "info" | "warning" | "critical";
  truncated?: boolean;
  meta?: TopologyMeta;
}

export type TopologyEvidenceType = "profile" | "dependency_summary" | "deployment" | "audit";

export interface TopologyEvidenceItem {
  id: string;
  evidenceType: TopologyEvidenceType;
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

// Legacy topology result types (used by troubleshoot workbench)
export interface PathResult {
  paths: string[][];
  truncated: boolean;
}

export interface AffectedNode {
  id: string;
  name: string;
  node_type: string;
  depth: number;
}

export interface ImpactResult {
  affected_nodes: AffectedNode[];
  total_count: number;
  max_depth: number;
}

// Topology V3 types (dual-cased for serde compatibility)
export type TopologyDrilldownDirection = "upstream" | "downstream" | "both";

export interface TopologyV3SnapshotQuery {
  env?: TopologyEnv;
  taskView?: TopologyTaskViewMode;
  maxDepth?: number;
  focusNodeId?: string;
  includeEvidence?: boolean;
}

export interface TopologyV3DrilldownQuery {
  nodeId: string;
  taskView?: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
  direction?: TopologyDrilldownDirection;
}

export interface TopologyV3TaskViewQuery {
  taskView: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
  focusNodeId?: string;
  nodeId?: string;
}

export interface TopologyV3PathsQuery {
  sourceId: string;
  targetId: string;
  taskView?: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
  maxResults?: number;
}

export interface TopologyV3ImpactQuery {
  nodeId: string;
  taskView?: TopologyTaskViewMode;
  env?: TopologyEnv;
  maxDepth?: number;
}

export interface TopologyV3EvidenceQuery {
  nodeId: string;
  edgeId?: string;
  relatedNodeId?: string;
  taskView?: TopologyTaskViewMode;
  maxItems?: number;
}

export interface TopologyV3TroubleshootReportQuery {
  nodeId: string;
  taskView?: TopologyTaskViewMode;
  evidenceLimit?: number;
}

export interface TopologyV3Node {
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

export interface TopologyV3Edge {
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

export interface TopologyV3Meta {
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

export interface TopologyV3SnapshotResponse {
  meta: TopologyV3Meta;
  lanes?: TopologyLane[];
  nodes: TopologyV3Node[];
  edges: TopologyV3Edge[];
  legendStats?: TopologyLegendStats;
  legend_stats?: TopologyLegendStats;
  layoutHints?: TopologyLayoutHints;
  layout_hints?: TopologyLayoutHints;
  focusNode?: TopologyV3Node;
  focus_node?: TopologyV3Node;
  nodeCount?: number;
  node_count?: number;
  edgeCount?: number;
  edge_count?: number;
}

export interface TopologyV3Neighbor {
  id: string;
  name: string;
  nodeType?: TopologyNodeType;
  node_type?: TopologyNodeType;
  env: TopologyEnv;
}

export interface TopologyV3DrilldownResponse {
  node: TopologyV3Node;
  upstream: TopologyV3Neighbor[];
  downstream: TopologyV3Neighbor[];
  inboundEdgeCount?: number;
  inbound_edge_count?: number;
  outboundEdgeCount?: number;
  outbound_edge_count?: number;
}

export interface TopologyV3TaskInsight {
  kind: string;
  title: string;
  description?: string;
  severity?: "info" | "warning" | "critical";
  nodeIds?: string[];
  node_ids?: string[];
  edgeIds?: string[];
  edge_ids?: string[];
}

export interface TopologyV3TaskViewResponse {
  taskView: TopologyTaskViewMode;
  snapshot: TopologyV3SnapshotResponse;
  title?: string;
  summary?: string;
  insights?: TopologyV3TaskInsight[];
  meta?: TopologyV3Meta;
}

export interface TopologyV3Path {
  nodeIds?: string[];
  node_ids?: string[];
  edgeIds?: string[];
  edge_ids?: string[];
  score?: number;
}

export interface TopologyV3PathsResponse {
  paths: TopologyV3Path[];
  truncated: boolean;
  totalCount?: number;
  total_count?: number;
  meta?: TopologyV3Meta;
}

export interface TopologyV3ImpactNode {
  id: string;
  name: string;
  nodeType?: string;
  node_type?: string;
  depth: number;
  reason?: string;
}

export interface TopologyV3ImpactResponse {
  affectedNodes?: TopologyV3ImpactNode[];
  affected_nodes?: TopologyV3ImpactNode[];
  totalCount?: number;
  total_count?: number;
  maxDepth?: number;
  max_depth?: number;
  truncated?: boolean;
  meta?: TopologyV3Meta;
}

export type TopologyV3EvidenceType =
  | "profile"
  | "dependency_summary"
  | "deployment"
  | "audit"
  | "call_relation"
  | "metric"
  | "alert"
  | "change"
  | "annotation";

export interface TopologyV3EvidenceItem {
  id: string;
  evidenceType?: TopologyV3EvidenceType;
  evidence_type?: TopologyV3EvidenceType;
  title: string;
  description?: string;
  severity?: "info" | "warning" | "critical";
  source?: string;
  timestamp?: string;
  nodeId?: string;
  edgeId?: string;
  payload?: Record<string, unknown>;
}

export interface TopologyV3EvidenceResponse {
  items: TopologyV3EvidenceItem[];
  total: number;
  meta?: TopologyV3Meta;
}

export interface TopologyV3TroubleshootSummary {
  inboundEdgeCount: number;
  outboundEdgeCount: number;
  deploymentCount: number;
  recentAuditCount: number;
  statusSeverity: "info" | "warning" | "critical";
}

export interface TopologyV3TroubleshootReport {
  node: TopologyV3Node;
  summary: TopologyV3TroubleshootSummary;
  upstream: TopologyV3Neighbor[];
  downstream: TopologyV3Neighbor[];
  evidence: TopologyV3EvidenceResponse;
  insights: TopologyV3TaskInsight[];
}
