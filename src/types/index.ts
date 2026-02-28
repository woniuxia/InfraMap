export interface Host {
  id: string;
  hostname: string;
  ip_address: string;
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
  status: "running" | "stopped" | "maintenance";
  description?: string;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
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

export interface Dependency {
  id: string;
  source_id: string;
  source_type: string;
  target_id: string;
  target_type: string;
  relation_type:
    | "http_call"
    | "tcp"
    | "mq_produce"
    | "mq_consume"
    | "grpc_call"
    | "db_query"
    | "cache_access";
  description?: string;
  is_deleted: number;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
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

// Topology types
export interface TopologyNode {
  id: string;
  name: string;
  node_type: "application" | "middleware" | "nginx";
  status?: string;
  env?: string;
  parent_id?: string;
  extra?: Record<string, unknown>;
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
}

export interface TopologyCombo {
  id: string;
  label: string;
  ip: string;
  status: string;
}

export interface TopologyGraph {
  nodes: TopologyNode[];
  edges: TopologyEdge[];
  combos: TopologyCombo[];
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
  dependencies: number;
  schema_version: number;
  is_compatible: boolean;
}

export interface ImportResult {
  hosts_imported: number;
  applications_imported: number;
  middlewares_imported: number;
  nginx_configs_imported: number;
  deployments_imported: number;
  dependencies_imported: number;
}
