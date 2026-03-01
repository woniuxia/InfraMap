import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import { bindHostIp, listHostIpBindings, unbindHostIp } from "@/api/host-ip-bindings";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("host-ip-bindings API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listHostIpBindings should invoke list_host_ip_bindings", async () => {
    __setMockHandler("list_host_ip_bindings", (_cmd, args) => {
      expect(args).toEqual({ hostId: "h-1" });
      return [];
    });

    const result = await listHostIpBindings("h-1");
    expect(result).toEqual([]);
  });

  it("bindHostIp should invoke bind_host_ip", async () => {
    __setMockHandler("bind_host_ip", (_cmd, args) => {
      expect(args).toEqual({ hostId: "h-1", ipId: "ip-1" });
      return undefined;
    });

    await bindHostIp({ host_id: "h-1", ip_id: "ip-1" });
  });

  it("unbindHostIp should invoke unbind_host_ip", async () => {
    __setMockHandler("unbind_host_ip", (_cmd, args) => {
      expect(args).toEqual({ hostId: "h-1", ipId: "ip-1" });
      return undefined;
    });

    await unbindHostIp({ host_id: "h-1", ip_id: "ip-1" });
  });
});
