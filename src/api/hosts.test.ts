import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import { listHosts, saveHost } from "@/api/hosts";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("hosts API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listHosts should pass tags filter as-is", async () => {
    __setMockHandler("list_hosts", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          filters: {
            env: "prod",
            tags: "[\"core\",\"edge\"]",
          },
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listHosts({
      page: 1,
      page_size: 20,
      filters: {
        env: "prod",
        tags: "[\"core\",\"edge\"]",
      },
    });
    expect(result.total).toBe(0);
  });

  it("saveHost should invoke save_host", async () => {
    __setMockHandler("save_host", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          id: "host-1",
          hostname: "web-prod-01",
          env: "prod",
          status: "running",
          tags: "[\"core\"]",
        },
      });
      return undefined;
    });

    await saveHost({
      id: "host-1",
      hostname: "web-prod-01",
      env: "prod",
      status: "running",
      tags: "[\"core\"]",
    });
  });
});
