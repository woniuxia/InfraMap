import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import { getResourceDeployContext, listDeployments, saveDeployment } from "@/api/deployments";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("deployments API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listDeployments should invoke command with params", async () => {
    __setMockHandler("list_deployments", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          filters: {
            resource_id: "mw-1",
            resource_type: "middleware",
          },
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listDeployments({
      page: 1,
      page_size: 20,
      filters: {
        resource_id: "mw-1",
        resource_type: "middleware",
      },
    });

    expect(result.total).toBe(0);
  });

  it("saveDeployment should invoke command with payload", async () => {
    __setMockHandler("save_deployment", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          id: "",
          resource_id: "mw-1",
          resource_type: "middleware",
          host_id: "host-1",
          port: 6379,
        },
      });
      return undefined;
    });

    await saveDeployment({
      id: "",
      resource_id: "mw-1",
      resource_type: "middleware",
      host_id: "host-1",
      port: 6379,
    });
  });

  it("getResourceDeployContext should invoke unified backend command", async () => {
    __setMockHandler("get_resource_deploy_context", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "middleware",
        resourceId: "mw-1",
      });
      return {
        resource_type: "middleware",
        resource_id: "mw-1",
        address: "10.0.0.1:6379",
        resource_env: "prod",
        parsed_ip: "10.0.0.1",
        matched_host_id: "host-1",
        matched_host_name: "redis-host",
      };
    });

    const context = await getResourceDeployContext("middleware", "mw-1");
    expect(context.matched_host_id).toBe("host-1");
    expect(context.parsed_ip).toBe("10.0.0.1");
  });

  it("getResourceDeployContext should pass address/env overrides when provided", async () => {
    __setMockHandler("get_resource_deploy_context", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "middleware",
        resourceId: "mw-1",
        addressOverride: "redis://10.0.0.9:6379",
        resourceEnvOverride: "dev",
      });
      return {
        resource_type: "middleware",
        resource_id: "mw-1",
        address: "redis://10.0.0.9:6379",
        resource_env: "dev",
        parsed_ip: "10.0.0.9",
        matched_host_id: null,
        matched_host_name: null,
      };
    });

    const context = await getResourceDeployContext("middleware", "mw-1", {
      address: "redis://10.0.0.9:6379",
      resourceEnv: "dev",
    });

    expect(context.resource_env).toBe("dev");
    expect(context.parsed_ip).toBe("10.0.0.9");
  });
});
