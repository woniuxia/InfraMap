import { describe, it, expect, vi, beforeEach } from "vitest";
import { __setMockHandler, __clearMockHandlers } from "@/__mocks__/tauri";
import { tauriInvoke } from "@/utils/invoke";
import { presentInfraError } from "@/composables/useErrorPresenter";
import { InfraError } from "@/types/error";

vi.mock("@/composables/useErrorPresenter", () => ({
  presentInfraError: vi.fn(),
}));

describe("tauriInvoke", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("returns data on successful invoke", async () => {
    __setMockHandler("test_cmd", () => ({ result: "ok" }));
    const data = await tauriInvoke<{ result: string }>("test_cmd");
    expect(data).toEqual({ result: "ok" });
  });

  it("normalizes structured backend errors and presents them", async () => {
    __setMockHandler("save_host", () => {
      throw {
        code: "VALIDATION_ERROR",
        message: "参数校验失败",
        details: "ip_address: Invalid IPv4 address",
        command: "save_host",
        retryable: false,
      };
    });

    await expect(tauriInvoke("save_host")).rejects.toMatchObject({
      code: "VALIDATION_ERROR",
      message: "参数校验失败",
      command: "save_host",
      retryable: false,
    });
    expect(presentInfraError).toHaveBeenCalledTimes(1);
    expect(presentInfraError).toHaveBeenCalledWith(expect.any(InfraError));
  });

  it("supports silent mode and does not present toast", async () => {
    __setMockHandler("save_host", () => {
      throw {
        code: "DB_UNAVAILABLE",
        message: "数据库暂时不可用",
        command: "save_host",
        retryable: true,
      };
    });

    await expect(tauriInvoke("save_host", undefined, { silent: true })).rejects.toMatchObject({
      code: "DB_UNAVAILABLE",
      message: "数据库暂时不可用",
      command: "save_host",
      retryable: true,
    });
    expect(presentInfraError).not.toHaveBeenCalled();
  });

  it("maps legacy unique-constraint string errors", async () => {
    __setMockHandler("save_host", () => {
      throw "Update failed: UNIQUE constraint failed: hosts.ip_address";
    });

    await expect(tauriInvoke("save_host")).rejects.toMatchObject({
      code: "CONFLICT",
      message: "保存失败，IP 地址已存在，请使用其他 IP 地址。",
    });
  });
});
