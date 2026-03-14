import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import {
  listServiceOwnerTerms,
  listServiceTechStackTerms,
  listSystemOwnerTerms,
  listHostCpuModelTerms,
  listHostOsTypeTerms,
  listHostOsTypeTermsByCount,
  listHostTagTerms,
  listIpTagTerms,
  listResourceTerms,
  listTaxonomyTerms,
  saveResourceTerms,
} from "@/api/taxonomy";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("taxonomy API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listTaxonomyTerms should invoke list_taxonomy_terms with all args", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "service",
        fieldKey: "tech_stack",
        limit: 20,
        sortBy: "recent",
        recencyScope: "global",
        appType: "frontend",
      });
      return ["Vue", "TypeScript"];
    });

    const result = await listTaxonomyTerms({
      resource_type: "service",
      field_key: "tech_stack",
      limit: 20,
      sort_by: "recent",
      recency_scope: "global",
      app_type: "frontend",
    });
    expect(result).toEqual(["Vue", "TypeScript"]);
  });

  it("listTaxonomyTerms should support minimum args", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "service",
        fieldKey: "owner",
        limit: undefined,
        sortBy: undefined,
        recencyScope: undefined,
        appType: undefined,
      });
      return ["alice"];
    });

    const result = await listTaxonomyTerms({
      resource_type: "service",
      field_key: "owner",
    });
    expect(result).toEqual(["alice"]);
  });

  it("listHostTagTerms should use host tags with resource_type scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "host",
        fieldKey: "tags",
        limit: 200,
        sortBy: "recent",
        recencyScope: "resource_type",
        appType: undefined,
      });
      return ["core", "edge"];
    });

    const result = await listHostTagTerms();
    expect(result).toEqual(["core", "edge"]);
  });

  it("listHostOsTypeTerms should use host os_type with resource_type scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "host",
        fieldKey: "os_type",
        limit: 200,
        sortBy: "recent",
        recencyScope: "resource_type",
        appType: undefined,
      });
      return ["openEuler", "Ubuntu 22.04"];
    });

    const result = await listHostOsTypeTerms();
    expect(result).toEqual(["openEuler", "Ubuntu 22.04"]);
  });

  it("listHostOsTypeTermsByCount should use host os_type count sort", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "host",
        fieldKey: "os_type",
        limit: 200,
        sortBy: "count",
        recencyScope: "resource_type",
        appType: undefined,
      });
      return ["Ubuntu 22.04", "openEuler"];
    });

    const result = await listHostOsTypeTermsByCount();
    expect(result).toEqual(["Ubuntu 22.04", "openEuler"]);
  });

  it("listHostCpuModelTerms should use host cpu_model with resource_type scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "host",
        fieldKey: "cpu_model",
        limit: 200,
        sortBy: "recent",
        recencyScope: "resource_type",
        appType: undefined,
      });
      return ["Intel Xeon Gold 6258R", "Hygon C86 7285"];
    });

    const result = await listHostCpuModelTerms();
    expect(result).toEqual(["Intel Xeon Gold 6258R", "Hygon C86 7285"]);
  });

  it("listIpTagTerms should use ip tags with resource_type scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "ip_address",
        fieldKey: "tags",
        limit: 200,
        sortBy: "recent",
        recencyScope: "resource_type",
        appType: undefined,
      });
      return ["vip", "gateway"];
    });

    const result = await listIpTagTerms();
    expect(result).toEqual(["vip", "gateway"]);
  });

  it("listServiceOwnerTerms should use service owner with global scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "service",
        fieldKey: "owner",
        limit: 100,
        sortBy: "recent",
        recencyScope: "global",
        appType: undefined,
      });
      return ["alice"];
    });

    const result = await listServiceOwnerTerms();
    expect(result).toEqual(["alice"]);
  });

  it("listSystemOwnerTerms should use system owner with resource_type scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "system",
        fieldKey: "owner",
        limit: 100,
        sortBy: "recent",
        recencyScope: "resource_type",
        appType: undefined,
      });
      return ["alice", "bob"];
    });

    const result = await listSystemOwnerTerms();
    expect(result).toEqual(["alice", "bob"]);
  });

  it("listServiceTechStackTerms should pass app_type with global scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "service",
        fieldKey: "tech_stack",
        limit: 10,
        sortBy: "recent",
        recencyScope: "global",
        appType: "frontend",
      });
      return ["Vue"];
    });

    const result = await listServiceTechStackTerms({ app_type: "frontend" });
    expect(result).toEqual(["Vue"]);
  });

  it("saveResourceTerms should invoke save_resource_terms_command", async () => {
    __setMockHandler("save_resource_terms_command", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "host",
        resourceId: "host-1",
        fieldKey: "tags",
        values: ["core", "db"],
      });
      return null;
    });

    await saveResourceTerms({
      resource_type: "host",
      resource_id: "host-1",
      field_key: "tags",
      values: ["core", "db"],
    });
  });

  it("listResourceTerms should invoke list_resource_terms", async () => {
    __setMockHandler("list_resource_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "ip_address",
        resourceId: "ip-1",
        fieldKey: "tags",
      });
      return ["edge", "public"];
    });

    const result = await listResourceTerms({
      resource_type: "ip_address",
      resource_id: "ip-1",
      field_key: "tags",
    });
    expect(result).toEqual(["edge", "public"]);
  });
});
