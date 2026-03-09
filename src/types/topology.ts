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
