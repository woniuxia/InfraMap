import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import {
  attachServicesToBusinessApplication,
  detachServiceFromBusinessApplication,
  listBusinessApplications,
  listServicesByBusinessApplication,
  listUnassignedApplicationServices,
  saveBusinessApplication,
} from "@/api/business-applications";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("business-applications API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listBusinessApplications should invoke list_business_applications", async () => {
    __setMockHandler("list_business_applications", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          search: "pay",
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listBusinessApplications({
      page: 1,
      page_size: 20,
      search: "pay",
    });
    expect(result.total).toBe(0);
  });

  it("saveBusinessApplication should return id", async () => {
    __setMockHandler("save_business_application", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          id: "",
          name: "支付中心",
          env: "prod",
          status: "active",
        },
      });
      return "ba-1";
    });

    const id = await saveBusinessApplication({
      id: "",
      name: "支付中心",
      env: "prod",
      status: "active",
    });
    expect(id).toBe("ba-1");
  });

  it("listUnassignedApplicationServices should invoke list_unassigned_application_services", async () => {
    __setMockHandler("list_unassigned_application_services", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          filters: { env: "prod" },
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listUnassignedApplicationServices({
      page: 1,
      page_size: 20,
      filters: { env: "prod" },
    });
    expect(result.total).toBe(0);
  });

  it("attachServicesToBusinessApplication should invoke attach_services_to_business_application", async () => {
    __setMockHandler("attach_services_to_business_application", (_cmd, args) => {
      expect(args).toEqual({
        business_application_id: "ba-1",
        application_ids: ["app-a", "app-b"],
      });
      return { attached_count: 2, skipped_count: 0 };
    });

    const result = await attachServicesToBusinessApplication("ba-1", ["app-a", "app-b"]);
    expect(result).toEqual({ attached_count: 2, skipped_count: 0 });
  });

  it("detachServiceFromBusinessApplication should invoke detach_service_from_business_application", async () => {
    __setMockHandler("detach_service_from_business_application", (_cmd, args) => {
      expect(args).toEqual({
        business_application_id: "ba-1",
        application_id: "app-a",
      });
      return undefined;
    });

    await detachServiceFromBusinessApplication("ba-1", "app-a");
  });

  it("listServicesByBusinessApplication should invoke list_services_by_business_application", async () => {
    __setMockHandler("list_services_by_business_application", (_cmd, args) => {
      expect(args).toEqual({ business_application_id: "ba-1" });
      return { frontend: [], backend: [] };
    });

    const result = await listServicesByBusinessApplication("ba-1");
    expect(result).toEqual({ frontend: [], backend: [] });
  });
});
