import { describe, expect, it } from "vitest";
import { formatRelationTargetLabel, getApplicationTypeLabel } from "@/utils/resourceLabel";

describe("resourceLabel", () => {
  it("maps known application types to Chinese labels", () => {
    expect(getApplicationTypeLabel("frontend")).toBe("前端");
    expect(getApplicationTypeLabel("backend")).toBe("后端");
    expect(getApplicationTypeLabel("gateway")).toBe("网关");
    expect(getApplicationTypeLabel("batch_job")).toBe("批处理");
    expect(getApplicationTypeLabel("microservice")).toBe("微服务");
    expect(getApplicationTypeLabel("other")).toBe("其他");
  });

  it("falls back to original application type when type is unknown", () => {
    expect(getApplicationTypeLabel("legacy_service")).toBe("legacy_service");
  });

  it("formats relation target label for application/middleware/nginx", () => {
    expect(
      formatRelationTargetLabel({
        resourceType: "service",
        name: "order-api",
        appType: "backend",
      }),
    ).toBe("order-api（后端）");

    expect(
      formatRelationTargetLabel({
        resourceType: "middleware",
        name: "redis-main",
        middlewareType: "Redis",
      }),
    ).toBe("redis-main（Redis）");

    expect(
      formatRelationTargetLabel({
        resourceType: "nginx",
        name: "edge-lb",
      }),
    ).toBe("edge-lb（网关）");
  });

  it("uses middleware fallback label when middleware type is empty", () => {
    expect(
      formatRelationTargetLabel({
        resourceType: "middleware",
        name: "cache-cluster",
        middlewareType: "   ",
      }),
    ).toBe("cache-cluster（中间件）");
  });
});
