import { tauriInvoke } from "@/utils/invoke";
import type {
  ExecuteImportRowsInput,
  ImportExecutionResult,
  ImportJobDetail,
  ImportJobSummary,
  ImportPreviewResult,
  PagedResult,
  PreviewImportRowsInput,
  QueryParams,
} from "@/types";

export function previewImportRows(input: PreviewImportRowsInput): Promise<ImportPreviewResult> {
  return tauriInvoke<ImportPreviewResult>("preview_import_rows", { input });
}

export function executeImportRows(input: ExecuteImportRowsInput): Promise<ImportExecutionResult> {
  return tauriInvoke<ImportExecutionResult>("execute_import_rows", { input });
}

export function listImportJobs(params: QueryParams): Promise<PagedResult<ImportJobSummary>> {
  return tauriInvoke<PagedResult<ImportJobSummary>>("list_import_jobs", { params });
}

export function getImportJobDetail(jobId: string): Promise<ImportJobDetail> {
  return tauriInvoke<ImportJobDetail>("get_import_job_detail", { jobId });
}
