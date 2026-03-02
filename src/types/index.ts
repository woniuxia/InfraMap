export interface Host {
  id: string;
  hostname: string;
  ip_display?: string;
  env: "prod" | "dev" | "test";
  os_type?: string;
  cpu_model?: string;
  cpu_cores?: number;
  cpu_threads?: number;
  cpu_freq?: string;
  ram_gb?: number;
  disk_gb?: number;
  status: "running" | "stopped" | "maintenance";
  tags?: string;
  description?: string;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export interface IpAddress {
  id: string;
  ip_address: string;
  env: "prod" | "dev" | "test";
  is_vip: boolean;
  real_ips?: string;
  tags?: string;
  description?: string;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export interface BatchCreateIpParams {
  start_ip: string;
  end_ip: string;
  env: "prod" | "dev" | "test";
  tags?: string;
  description?: string;
}

export interface BatchCreateIpResult {
  created_count: number;
  skipped_count: number;
}

export interface HostIpBindingPayload {
  host_id: string;
  ip_id: string;
}

export interface Application {
  id: string;
  name: string;
  type: "frontend" | "backend" | "gateway" | "batch_job" | "microservice" | "other";
  address?: string;
  port?: number;
  tech_stack?: string;
  deploy_mode?: string;
  env: "prod" | "dev" | "test";
  git_repo?: string;
  owner?: string;
  owners?: string[];
  business_application_id?: string;
  business_application_name?: string;
  status: "running" | "stopped" | "maintenance";
  description?: string;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export interface BusinessApplication {
  id: string;
  name: string;
  code?: string;
  owners?: string[];
  description?: string;
  env?: "prod" | "dev" | "test";
  status: "active" | "inactive";
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export interface AttachServicesResult {
  attached_count: number;
  skipped_count: number;
}

export interface ReplaceServicesResult {
  attached_count: number;
  detached_count: number;
  unchanged_count: number;
}

export interface BusinessApplicationServices {
  frontend: Application[];
  backend: Application[];
}

export interface Middleware {
  id: string;
  name: string;
  category: "database" | "message_queue" | "cache" | "search_engine" | "config_center" | "other";
  type: string;
  address: string;
  port?: number;
  version?: string;
  env: "prod" | "dev" | "test";
  description?: string;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export interface NginxConfig {
  id: string;
  name: string;
  address: string;
  listen_port?: number;
  strategy?: "roundrobin" | "ip_hash";
  upstream_servers?: string;
  env: "prod" | "dev" | "test";
  status: "running" | "stopped" | "maintenance";
  description?: string;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export interface Deployment {
  id: string;
  resource_id: string;
  resource_type: "application" | "middleware" | "nginx";
  host_id: string;
  port?: number;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export type DeploymentResourceType = "application" | "middleware" | "nginx";

export interface ResourceDeployContext {
  resource_type: DeploymentResourceType;
  resource_id: string;
  address?: string | null;
  resource_env?: "prod" | "dev" | "test" | null;
  parsed_ip?: string | null;
  matched_host_id?: string | null;
  matched_host_name?: string | null;
}

export type CallRelationType =
  | "http_call"
  | "tcp"
  | "mq_produce"
  | "mq_consume"
  | "grpc_call"
  | "db_query"
  | "cache_access";

export type CallDirection = "upstream" | "downstream";

export interface CallRelation {
  id: string;
  pair_key: string;
  owner_id: string;
  owner_type: "application" | "middleware" | "nginx";
  peer_id: string;
  peer_type: "application" | "middleware" | "nginx";
  direction: CallDirection;
  relation_type: CallRelationType;
  description?: string;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export interface ReplaceCallRelationItem {
  peer_id: string;
  peer_type: "application" | "middleware" | "nginx";
  direction: CallDirection;
  relation_type: CallRelationType;
  description?: string;
}

export interface ReplaceResourceCallRelationsParams {
  resource_id: string;
  resource_type: "application" | "middleware" | "nginx";
  items: ReplaceCallRelationItem[];
}

export interface ReplaceResourceCallRelationsResult {
  created_count: number;
  deleted_count: number;
  deduplicated_count: number;
}

export interface AuditLog {
  id: string;
  action: "create" | "update" | "delete";
  resource_type: string;
  resource_id: string;
  resource_name?: string;
  changes?: string;
  created_at: string;
}

export interface QueryParams {
  page?: number;
  page_size?: number;
  search?: string;
  filters?: Record<string, string>;
}

export interface PagedResult<T> {
  data: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface HealthStatus {
  status: string;
  db_connected: boolean;
  db_path: string;
  table_count: number;
  schema_version: number;
}

export interface DbInfo {
  tables: string[];
  index_count: number;
}

export interface DashboardStats {
  host_total: number;
  host_abnormal: number;
  application_total: number;
  application_abnormal: number;
  middleware_total: number;
  nginx_total: number;
  nginx_abnormal: number;
  deployment_total: number;
  dependency_total: number;
  env_distribution: { env: string; count: number }[];
}

export interface DashboardTotals {
  host_total: number;
  host_abnormal: number;
  application_total: number;
  application_abnormal: number;
  middleware_total: number;
  nginx_total: number;
  nginx_abnormal: number;
  deployment_total: number;
  dependency_total: number;
}

export interface DashboardHealth {
  abnormal_total: number;
  abnormal_rate: number;
  score: number;
}

export interface DashboardCoverage {
  deployable_total: number;
  deployed_total: number;
  undeployed_total: number;
  deployment_coverage: number;
  relatable_total: number;
  related_total: number;
  isolated_total: number;
  relation_coverage: number;
  undeployed_application_total: number;
  undeployed_middleware_total: number;
  undeployed_nginx_total: number;
}

export type DashboardRiskSeverity = "critical" | "warning" | "info";

export interface DashboardRiskItem {
  key: string;
  label: string;
  count: number;
  severity: DashboardRiskSeverity;
  target_route: string;
  target_filters: Record<string, string>;
}

export interface DashboardRecentChange {
  id: string;
  action: "create" | "update" | "delete";
  resource_type: string;
  resource_id: string;
  resource_name?: string;
  created_at: string;
}

export interface DashboardOverview {
  totals: DashboardTotals;
  health: DashboardHealth;
  coverage: DashboardCoverage;
  env_distribution: { env: string; count: number }[];
  risk_items: DashboardRiskItem[];
  recent_changes: DashboardRecentChange[];
}

// Topology types
export type TopologyEnv = "prod" | "test" | "dev";

export type TopologyNodeType = "application" | "middleware" | "nginx";

export type TopologyGroupKind = "application_service" | "middleware" | "nginx";

export interface TopologyLane {
  id: TopologyEnv;
  label: string;
  order: number;
  node_count: number;
  app_count: number;
}

export interface TopologyNode {
  id: string;
  name: string;
  node_type: TopologyNodeType;
  env: TopologyEnv;
  group_kind: TopologyGroupKind;
  host_id?: string;
  status?: string;
  importance: number;
  extra?: Record<string, unknown>;
  is_external?: boolean;
  external_ref_id?: string;
}

export interface TopologyEdge {
  id: string;
  source: string;
  target: string;
  edge_type:
    | "http_call"
    | "tcp"
    | "mq_produce"
    | "mq_consume"
    | "grpc_call"
    | "db_query"
    | "cache_access";
  label?: string;
  strength: number;
  cross_env: boolean;
}

export interface TopologyEnvCount {
  env: TopologyEnv;
  count: number;
  app_count: number;
}

export interface TopologyKindCount {
  kind: string;
  count: number;
}

export interface TopologyLegendStats {
  env_counts: TopologyEnvCount[];
  node_type_counts: TopologyKindCount[];
  edge_type_counts: TopologyKindCount[];
  application_service_count: number;
  current_env?: TopologyEnv;
  external_node_count?: number;
  cross_env_edge_count?: number;
}

export interface TopologyLayoutHints {
  lane_order: TopologyEnv[];
  default_collapsed_groups: string[];
  high_density_mode: boolean;
}

export interface TopologyGraph {
  lanes: TopologyLane[];
  nodes: TopologyNode[];
  edges: TopologyEdge[];
  legend_stats: TopologyLegendStats;
  layout_hints: TopologyLayoutHints;
}

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

// Settings & Backup types
export interface SystemSettings {
  id: string;
  auto_backup_enabled: boolean;
  backup_interval_hours: number;
  max_backups: number;
  last_backup_time?: string;
  created_at: string;
  updated_at: string;
}

export interface UpdateSettingsInput {
  auto_backup_enabled: boolean;
  backup_interval_hours: number;
  max_backups: number;
}

export interface StorageProfile {
  active_root_path: string;
  db_path: string;
  backup_dir: string;
  is_default_path: boolean;
}

export interface UpdateStoragePathInput {
  root_path: string;
}

export interface UpdateStoragePathResult {
  restart_required: boolean;
  migrated: boolean;
}

export interface BackupEntry {
  filename: string;
  file_size: number;
  created_at: string;
  is_auto: boolean;
}

export interface DbPreviewSummary {
  hosts: number;
  applications: number;
  middlewares: number;
  nginx_configs: number;
  deployments: number;
  call_relations: number;
  schema_version: number;
  is_compatible: boolean;
}

export interface ImportResult {
  hosts_imported: number;
  applications_imported: number;
  application_owners_imported?: number;
  middlewares_imported: number;
  nginx_configs_imported: number;
  deployments_imported: number;
  call_relations_imported: number;
}
