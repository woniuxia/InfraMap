import { tauriInvoke } from "@/utils/invoke";
import type {
  IntegrityRepairInput,
  IntegrityRepairResult,
  IntegrityReport,
} from "@/types";

export function scanIntegrity(): Promise<IntegrityReport> {
  return tauriInvoke<IntegrityReport>("scan_integrity");
}

export function getIntegrityReport(jobId?: string): Promise<IntegrityReport> {
  if (jobId) {
    return tauriInvoke<IntegrityReport>("get_integrity_report", { jobId });
  }

  return tauriInvoke<IntegrityReport>("get_integrity_report");
}

export function repairIntegrityFindings(input: IntegrityRepairInput): Promise<IntegrityRepairResult> {
  return tauriInvoke<IntegrityRepairResult>("repair_integrity_findings", { input });
}
