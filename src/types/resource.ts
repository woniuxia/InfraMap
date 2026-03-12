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
  created_at: string;
  updated_at: string;
}

export interface Contact {
  id: string;
  name: string;
  phone?: string;
  email?: string;
  remark?: string;
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
  owners?: string[];
  owner_contact_ids?: string[];
  business_application_id?: string;
  business_application_name?: string;
  status: "running" | "stopped" | "maintenance";
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface BusinessApplication {
  id: string;
  name: string;
  code?: string;
  owners?: string[];
  owner_contact_ids?: string[];
  description?: string;
  env?: "prod" | "dev" | "test";
  status: "active" | "inactive";
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
  created_at: string;
  updated_at: string;
}

export interface NginxEndpoint {
  host: string;
  port: number;
}

export interface NginxConfig {
  id: string;
  name: string;
  endpoints: NginxEndpoint[];
  strategy?: "roundrobin" | "ip_hash";
  env: "prod" | "dev" | "test";
  status: "running" | "stopped" | "maintenance";
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface Deployment {
  id: string;
  resource_id: string;
  resource_type: "application" | "middleware" | "nginx";
  host_id: string;
  port?: number;
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
  | "cache_access"
  | "forward";

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
