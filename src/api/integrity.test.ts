import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

import { getIntegrityReport, repairIntegrityFindings, scanIntegrity } from "@/api/integrity";

describe("integrity API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("scanIntegrity should invoke scan_integrity", async () => {
    const mock = {
      job_id: "job-1",
      summary: { total: 0, critical: 0, warning: 0, info: 0, repairable: 0, generated_at: "" },
      findings: [],
    };

    __setMockHandler("scan_integrity", () => mock);

    const result = await scanIntegrity();
    expect(result).toEqual(mock);
  });

  it("getIntegrityReport should invoke get_integrity_report", async () => {
    const mock = {
      job_id: "job-1",
      summary: { total: 1, critical: 1, warning: 0, info: 0, repairable: 1, generated_at: "" },
      findings: [],
    };

    __setMockHandler("get_integrity_report", (_cmd, args) => {
      expect(args).toBeUndefined();
      return mock;
    });

    const result = await getIntegrityReport();
    expect(result).toEqual(mock);
  });

  it("repairIntegrityFindings should invoke repair_integrity_findings", async () => {
    const mock = {
      job_id: "job-2",
      backup_filename: "backup_pre_integrity_repair_20260307.db",
      repaired_count: 1,
      skipped_count: 0,
      report: {
        job_id: "job-2",
        summary: { total: 0, critical: 0, warning: 0, info: 0, repairable: 0, generated_at: "" },
        findings: [],
      },
    };

    __setMockHandler("repair_integrity_findings", (_cmd, args) => {
      expect(args).toEqual({ input: { finding_ids: ["orphan_deployment:dep-1"] } });
      return mock;
    });

    const result = await repairIntegrityFindings({ finding_ids: ["orphan_deployment:dep-1"] });
    expect(result).toEqual(mock);
  });
});
