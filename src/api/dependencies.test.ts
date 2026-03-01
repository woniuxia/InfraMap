import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import {
  listDependencies,
  saveDependenciesBatch,
  saveDependency,
  softDeleteDependency,
} from "@/api/dependencies";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("dependencies API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listDependencies should invoke list_dependencies", async () => {
    __setMockHandler("list_dependencies", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          filters: { source_id: "app-a" },
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listDependencies({
      page: 1,
      page_size: 20,
      filters: { source_id: "app-a" },
    });
    expect(result.total).toBe(0);
  });

  it("saveDependency should invoke save_dependency", async () => {
    __setMockHandler("save_dependency", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          id: "",
          source_id: "app-a",
          source_type: "application",
          target_id: "app-b",
          target_type: "application",
          relation_type: "http_call",
        },
      });
      return undefined;
    });

    await saveDependency({
      id: "",
      source_id: "app-a",
      source_type: "application",
      target_id: "app-b",
      target_type: "application",
      relation_type: "http_call",
    });
  });

  it("softDeleteDependency should invoke soft_delete_dependency", async () => {
    __setMockHandler("soft_delete_dependency", (_cmd, args) => {
      expect(args).toEqual({ id: "dep-1" });
      return undefined;
    });

    await softDeleteDependency("dep-1");
  });

  it("saveDependenciesBatch should invoke save_dependencies_batch", async () => {
    __setMockHandler("save_dependencies_batch", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          resource_id: "app-a",
          resource_type: "application",
          items: [
            {
              target_id: "app-b",
              target_type: "application",
              relation_type: "http_call",
              direction: "downstream",
              description: "calls app-b",
            },
          ],
        },
      });
      return { created_count: 1, skipped_count: 0 };
    });

    const result = await saveDependenciesBatch({
      resource_id: "app-a",
      resource_type: "application",
      items: [
        {
          target_id: "app-b",
          target_type: "application",
          relation_type: "http_call",
          direction: "downstream",
          description: "calls app-b",
        },
      ],
    });

    expect(result).toEqual({ created_count: 1, skipped_count: 0 });
  });
});
