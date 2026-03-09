export interface ImportResult {
  hosts_imported: number;
  applications_imported: number;
  middlewares_imported: number;
  nginx_configs_imported: number;
  deployments_imported: number;
  call_relations_imported: number;
}

// Bulk import workbench
export type ImportExecutionStrategy = "skip" | "overwrite" | "block";

export interface ImportDraftRow {
  resource_type: string;
  name?: string;
  env?: "prod" | "dev" | "test";
  type?: string;
  status?: "running" | "stopped" | "maintenance";
  address?: string;
  port?: number;
  category?: string;
  description?: string;
}

export interface ImportValidationIssue {
  row_no: number;
  field_key?: string;
  issue_type: "error" | "warning" | "conflict";
  code: string;
  message: string;
}

export interface ImportNormalizedRow {
  row_no: number;
  resource_type: string;
  name: string;
  env: "prod" | "dev" | "test";
  item_type?: string;
  status?: "running" | "stopped" | "maintenance";
  address?: string;
  port?: number;
  description?: string;
  conflict_id?: string;
}

export interface ImportPreviewResult {
  rows: ImportNormalizedRow[];
  issues: ImportValidationIssue[];
  error_count: number;
  warning_count: number;
  conflict_count: number;
  valid_count: number;
}

export interface PreviewImportRowsInput {
  rows: ImportDraftRow[];
}

export interface ExecuteImportRowsInput {
  rows: ImportDraftRow[];
  strategy?: ImportExecutionStrategy;
}

export interface ImportExecutionResult {
  job_id: string;
  status: string;
  total_rows: number;
  created_count: number;
  updated_count: number;
  skipped_count: number;
  failed_count: number;
}

export interface ImportJobSummary {
  id: string;
  status: string;
  strategy: ImportExecutionStrategy;
  total_rows: number;
  created_count: number;
  updated_count: number;
  skipped_count: number;
  failed_count: number;
  created_at: string;
  updated_at: string;
}

export interface ImportJobRowResult {
  row_no: number;
  resource_type: string;
  name: string;
  env: string;
  status: string;
  error_message?: string;
}

export interface ImportJobDetail {
  summary: ImportJobSummary;
  rows: ImportJobRowResult[];
  issues: ImportValidationIssue[];
}
