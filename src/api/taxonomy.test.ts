import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import {
  listApplicationOwnerTerms,
  listApplicationTechStackTerms,
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
        resourceType: "application",
        fieldKey: "tech_stack",
        limit: 20,
        sortBy: "recent",
        recencyScope: "global",
        appType: "frontend",
      });
      return ["Vue", "TypeScript"];
    });

    const result = await listTaxonomyTerms({
      resource_type: "application",
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
        resourceType: "application",
        fieldKey: "owner",
        limit: undefined,
        sortBy: undefined,
        recencyScope: undefined,
        appType: undefined,
      });
      return ["alice"];
    });

    const result = await listTaxonomyTerms({
      resource_type: "application",
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

  it("listApplicationOwnerTerms should use application owner with global scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "application",
        fieldKey: "owner",
        limit: 100,
        sortBy: "recent",
        recencyScope: "global",
        appType: undefined,
      });
      return ["alice"];
    });

    const result = await listApplicationOwnerTerms();
    expect(result).toEqual(["alice"]);
  });

  it("listApplicationTechStackTerms should pass app_type with global scope", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      expect(args).toEqual({
        resourceType: "application",
        fieldKey: "tech_stack",
        limit: 10,
        sortBy: "recent",
        recencyScope: "global",
        appType: "frontend",
      });
      return ["Vue"];
    });

    const result = await listApplicationTechStackTerms({ app_type: "frontend" });
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
