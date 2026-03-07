import { tauriInvoke } from "@/utils/invoke";
import type { PagedResult, QueryParams, SystemJobDetail, SystemJobSummary } from "@/types";

export function listSystemJobs(params: QueryParams): Promise<PagedResult<SystemJobSummary>> {
  return tauriInvoke<PagedResult<SystemJobSummary>>("list_system_jobs", { params });
}

export function getSystemJobDetail(jobId: string): Promise<SystemJobDetail> {
  return tauriInvoke<SystemJobDetail>("get_system_job_detail", { jobId });
}
