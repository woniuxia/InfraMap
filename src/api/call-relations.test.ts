import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import { listCallRelations, replaceResourceCallRelations } from "@/api/call-relations";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("call-relations API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listCallRelations should invoke list_call_relations", async () => {
    __setMockHandler("list_call_relations", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          filters: { owner_id: "app-a", owner_type: "application" },
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listCallRelations({
      page: 1,
      page_size: 20,
      filters: { owner_id: "app-a", owner_type: "application" },
    });
    expect(result.total).toBe(0);
  });

  it("replaceResourceCallRelations should invoke replace_resource_call_relations", async () => {
    __setMockHandler("replace_resource_call_relations", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          resource_id: "app-a",
          resource_type: "application",
          items: [
            {
              peer_id: "app-b",
              peer_type: "application",
              direction: "upstream",
              relation_type: "http_call",
              description: "A calls B",
            },
          ],
        },
      });
      return { created_count: 2, deleted_count: 0, deduplicated_count: 0 };
    });

    const result = await replaceResourceCallRelations({
      resource_id: "app-a",
      resource_type: "application",
      items: [
        {
          peer_id: "app-b",
          peer_type: "application",
          direction: "upstream",
          relation_type: "http_call",
          description: "A calls B",
        },
      ],
    });

    expect(result).toEqual({ created_count: 2, deleted_count: 0, deduplicated_count: 0 });
  });
});
