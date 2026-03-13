export type EditorMode = "create" | "edit" | "copy";

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
