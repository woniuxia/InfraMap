import { describe, expect, it } from "vitest";
import { normalizeInvokeError } from "@/utils/error";
import { InfraError } from "@/types/error";

describe("normalizeInvokeError", () => {
  it("accepts structured backend payload", () => {
    const error = normalizeInvokeError(
      {
        code: "NOT_FOUND",
        message: "资源不存在",
        details: "host id h-1 not found",
        command: "get_host",
        retryable: false,
      },
      "get_host",
    );

    expect(error).toBeInstanceOf(InfraError);
    expect(error).toMatchObject({
      code: "NOT_FOUND",
      message: "资源不存在",
      command: "get_host",
      retryable: false,
    });
  });

  it("parses JSON string payload", () => {
    const error = normalizeInvokeError(
      JSON.stringify({
        code: "DB_UNAVAILABLE",
        message: "数据库连接失败",
        details: "Pool error: timeout",
        command: "list_hosts",
        retryable: true,
      }),
      "list_hosts",
    );

    expect(error).toMatchObject({
      code: "DB_UNAVAILABLE",
      message: "数据库连接失败",
      command: "list_hosts",
      retryable: true,
    });
  });

  it("maps legacy unique constraint errors", () => {
    const error = normalizeInvokeError(
      "Update failed: UNIQUE constraint failed: hosts.ip_address",
      "save_host",
    );

    expect(error).toMatchObject({
      code: "CONFLICT",
      message: "保存失败，IP 地址已存在，请使用其他 IP 地址。",
      command: "save_host",
    });
  });

  it("maps ip resource unique constraint errors", () => {
    const error = normalizeInvokeError(
      "Update failed: UNIQUE constraint failed: ip_addresses.ip_address, ip_addresses.env",
      "save_ip_address",
    );

    expect(error).toMatchObject({
      code: "CONFLICT",
      message: "保存失败，IP 地址与环境组合已存在，请勿重复创建。",
      command: "save_ip_address",
    });
  });

  it("falls back to internal error for unknown payloads", () => {
    const error = normalizeInvokeError({ x: 1 }, "list_hosts");

    expect(error).toMatchObject({
      code: "INTERNAL_ERROR",
      message: "发生未知异常，请稍后重试。",
      command: "list_hosts",
      retryable: false,
    });
  });
});
