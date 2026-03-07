export * from "./common";
export * from "./resource";
export * from "./dashboard";
export * from "./topology";
export * from "./import";
export * from "./error";
export * from "./searchToolbar";

export interface SystemJobSummary {
  id: string;
  job_type: string;
  title: string;
  status: string;
  summary?: string;
  progress_percent: number;
  retryable: boolean;
  cancellable: boolean;
  created_at: string;
  updated_at: string;
  finished_at?: string;
}

export interface SystemJobImportRow {
  row_no: number;
  resource_type: string;
  name: string;
  env: string;
  status: string;
  error_message?: string;
}

export interface SystemJobImportIssue {
  row_no: number;
  field_key?: string;
  issue_type: string;
  code: string;
  message: string;
}

export interface SystemJobDetail {
  summary: SystemJobSummary;
  payload?: Record<string, unknown> | null;
  result?: Record<string, unknown> | null;
  error_message?: string;
  import_rows?: SystemJobImportRow[];
  import_issues?: SystemJobImportIssue[];
}

export interface IntegritySummary {
  total: number;
  critical: number;
  warning: number;
  info: number;
  repairable: number;
  generated_at: string;
  last_scanned_at?: string;
}

export interface IntegrityFinding {
  id: string;
  key: string;
  category: string;
  severity: "critical" | "warning" | "info";
  title: string;
  description: string;
  resource_type?: string;
  resource_id?: string;
  resource_name?: string;
  topology_node_id?: string;
  target_route: string;
  target_filters: Record<string, string>;
  repair_supported: boolean;
}

export interface IntegrityReport {
  job_id?: string;
  summary: IntegritySummary;
  findings: IntegrityFinding[];
}

export interface IntegrityRepairInput {
  finding_ids: string[];
}

export interface IntegrityRepairResult {
  job_id: string;
  backup_filename: string;
  repaired_count: number;
  skipped_count: number;
  report: IntegrityReport;
}

export interface SnapshotManifest {
  format_version: number;
  export_time: string;
  app_version: string;
  schema_version: number;
}

export interface SnapshotTableCount {
  table: string;
  count: number;
}

export interface SnapshotPreview {
  manifest: SnapshotManifest;
  snapshot_counts: SnapshotTableCount[];
  current_counts: SnapshotTableCount[];
  compatible: boolean;
  warnings: string[];
  total_rows: number;
}

export interface SnapshotExportResult {
  job_id: string;
  filepath: string;
  total_rows: number;
  table_counts: SnapshotTableCount[];
}

export interface SnapshotImportResult {
  job_id: string;
  backup_filename: string;
  total_rows: number;
  table_counts: SnapshotTableCount[];
}
