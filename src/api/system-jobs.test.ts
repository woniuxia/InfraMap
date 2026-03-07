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

import { getSystemJobDetail, listSystemJobs } from "@/api/system-jobs";

describe("system-jobs API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listSystemJobs should invoke list_system_jobs", async () => {
    const mock = { data: [], total: 0, page: 1, page_size: 20 };

    __setMockHandler("list_system_jobs", (_cmd, args) => {
      expect(args).toEqual({ params: { page: 1, page_size: 20 } });
      return mock;
    });

    const result = await listSystemJobs({ page: 1, page_size: 20 });
    expect(result).toEqual(mock);
  });

  it("getSystemJobDetail should invoke get_system_job_detail", async () => {
    const mock = {
      summary: {
        id: "job-1",
        job_type: "import_rows",
        title: "批量录入",
        status: "completed",
        summary: "导入完成",
        progress_percent: 100,
        retryable: true,
        cancellable: false,
        created_at: "",
        updated_at: "",
      },
    };

    __setMockHandler("get_system_job_detail", (_cmd, args) => {
      expect(args).toEqual({ jobId: "job-1" });
      return mock;
    });

    const result = await getSystemJobDetail("job-1");
    expect(result).toEqual(mock);
  });
});
