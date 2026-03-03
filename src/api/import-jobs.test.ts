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

import {
  previewImportRows,
  executeImportRows,
  listImportJobs,
  getImportJobDetail,
} from "@/api/import-jobs";

describe("import-jobs API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("previewImportRows should invoke preview_import_rows", async () => {
    const mock = {
      rows: [],
      issues: [],
      error_count: 0,
      warning_count: 0,
      conflict_count: 0,
      valid_count: 0,
    };

    __setMockHandler("preview_import_rows", (_cmd, args) => {
      expect(args).toEqual({
        input: {
          rows: [{ resource_type: "application", name: "app-a" }],
        },
      });
      return mock;
    });

    const result = await previewImportRows({
      rows: [{ resource_type: "application", name: "app-a" }],
    });
    expect(result).toEqual(mock);
  });

  it("executeImportRows should invoke execute_import_rows", async () => {
    const mock = {
      job_id: "job-1",
      status: "completed",
      total_rows: 1,
      created_count: 1,
      updated_count: 0,
      skipped_count: 0,
      failed_count: 0,
    };
    __setMockHandler("execute_import_rows", (_cmd, args) => {
      expect(args).toEqual({
        input: {
          rows: [{ resource_type: "application", name: "app-a" }],
          strategy: "skip",
        },
      });
      return mock;
    });

    const result = await executeImportRows({
      rows: [{ resource_type: "application", name: "app-a" }],
      strategy: "skip",
    });
    expect(result).toEqual(mock);
  });

  it("listImportJobs should invoke list_import_jobs", async () => {
    const mock = {
      data: [],
      total: 0,
      page: 1,
      page_size: 20,
    };
    __setMockHandler("list_import_jobs", (_cmd, args) => {
      expect(args).toEqual({ params: { page: 1, page_size: 20 } });
      return mock;
    });

    const result = await listImportJobs({ page: 1, page_size: 20 });
    expect(result).toEqual(mock);
  });

  it("getImportJobDetail should invoke get_import_job_detail", async () => {
    const mock = {
      summary: {
        id: "job-1",
        status: "completed",
        strategy: "skip",
        total_rows: 1,
        created_count: 1,
        updated_count: 0,
        skipped_count: 0,
        failed_count: 0,
        created_at: "",
        updated_at: "",
      },
      rows: [],
      issues: [],
    };
    __setMockHandler("get_import_job_detail", (_cmd, args) => {
      expect(args).toEqual({ jobId: "job-1" });
      return mock;
    });

    const result = await getImportJobDetail("job-1");
    expect(result).toEqual(mock);
  });
});
