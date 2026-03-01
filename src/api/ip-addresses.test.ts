import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import {
  batchCreateIpAddresses,
  listIpAddresses,
  saveIpAddress,
} from "@/api/ip-addresses";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("ip-addresses API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listIpAddresses should invoke list_ip_addresses", async () => {
    __setMockHandler("list_ip_addresses", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          filters: { env: "prod" },
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listIpAddresses({
      page: 1,
      page_size: 20,
      filters: { env: "prod" },
    });
    expect(result.total).toBe(0);
  });

  it("saveIpAddress should invoke save_ip_address", async () => {
    __setMockHandler("save_ip_address", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          id: "",
          ip_address: "10.0.0.1",
          env: "prod",
          is_vip: false,
        },
      });
      return undefined;
    });

    await saveIpAddress({
      id: "",
      ip_address: "10.0.0.1",
      env: "prod",
      is_vip: false,
    });
  });

  it("batchCreateIpAddresses should invoke batch_create_ip_addresses", async () => {
    __setMockHandler("batch_create_ip_addresses", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          start_ip: "10.0.2.1",
          end_ip: "10.0.2.3",
          env: "prod",
          tags: "[\"batch\"]",
          description: "bulk",
        },
      });
      return { created_count: 3, skipped_count: 0 };
    });

    const result = await batchCreateIpAddresses({
      start_ip: "10.0.2.1",
      end_ip: "10.0.2.3",
      env: "prod",
      tags: "[\"batch\"]",
      description: "bulk",
    });
    expect(result.created_count).toBe(3);
  });
});
