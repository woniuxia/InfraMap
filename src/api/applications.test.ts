import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import {
  listApplicationOwnerCandidates,
  listApplications,
  listTopApplicationTechStacks,
  saveApplication,
} from "@/api/applications";

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

  it("listApplicationOwnerCandidates should invoke command with limit", async () => {
    __setMockHandler("list_application_owner_candidates", (_cmd, args) => {
      expect(args).toEqual({ limit: 50 });
      return ["alice", "bob"];
    });

    const result = await listApplicationOwnerCandidates(50);
    expect(result).toEqual(["alice", "bob"]);
  });

  it("listTopApplicationTechStacks should pass app_type", async () => {
    __setMockHandler("list_top_application_tech_stacks", (_cmd, args) => {
      expect(args).toEqual({ limit: 10, app_type: "frontend" });
      return ["Vue", "TypeScript"];
    });

    const result = await listTopApplicationTechStacks(10, "frontend");
    expect(result).toEqual(["Vue", "TypeScript"]);
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

  it("saveApplication should pass owners array", async () => {
    __setMockHandler("save_application", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          name: "payment-api",
          type: "backend",
          env: "prod",
          status: "running",
          owners: ["alice", "bob"],
        },
      });
      return "app-123";
    });

    const result = await saveApplication({
      name: "payment-api",
      type: "backend",
      env: "prod",
      status: "running",
      owners: ["alice", "bob"],
    });
    expect(result).toBe("app-123");
  });
});
