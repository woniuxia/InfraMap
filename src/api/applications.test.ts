import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import { listApplications, saveApplication } from "@/api/applications";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("applications API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listApplications should pass params as-is", async () => {
    __setMockHandler("list_applications", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          search: "alice",
          filters: {
            env: "prod",
          },
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listApplications({
      page: 1,
      page_size: 20,
      search: "alice",
      filters: {
        env: "prod",
      },
    });
    expect(result.total).toBe(0);
  });

  it("saveApplication should pass owner_contact_ids array", async () => {
    __setMockHandler("save_application", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          name: "payment-api",
          type: "backend",
          env: "prod",
          status: "running",
          owner_contact_ids: ["contact-alice", "contact-bob"],
        },
      });
      return "app-123";
    });

    const result = await saveApplication({
      name: "payment-api",
      type: "backend",
      env: "prod",
      status: "running",
      owner_contact_ids: ["contact-alice", "contact-bob"],
    });
    expect(result).toBe("app-123");
  });
});
