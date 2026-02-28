import { describe, expect, it } from "vitest";
import {
  buildSearchToolbarQuery,
  createEmptyFilters,
  getActiveFilterChips,
} from "@/components/filters/searchToolbar.utils";
import type { SearchFieldConfig } from "@/types/searchToolbar";

const fields: SearchFieldConfig[] = [
  {
    key: "status",
    queryKey: "status",
    label: "状态",
    type: "multi-select",
    width: "md",
    options: [
      { label: "运行中", value: "running" },
      { label: "已停止", value: "stopped" },
      { label: "维护中", value: "maintenance" },
    ],
  },
  {
    key: "owner",
    queryKey: "owner",
    label: "负责人",
    type: "text",
    width: "sm",
    section: "advanced",
  },
];

describe("searchToolbar.utils", () => {
  it("buildSearchToolbarQuery should trim search and serialize filters", () => {
    const result = buildSearchToolbarQuery("  api  ", fields, {
      status: ["running", "stopped"],
      owner: "  ",
    });

    expect(result).toEqual({
      search: "api",
      filters: {
        status: "[\"running\",\"stopped\"]",
      },
    });
  });

  it("createEmptyFilters should reset values by field type", () => {
    const result = createEmptyFilters(fields);
    expect(result).toEqual({
      status: [],
      owner: "",
    });
  });

  it("getActiveFilterChips should summarize multi values and include search", () => {
    const chips = getActiveFilterChips("gateway", "内容", fields, {
      status: ["running", "stopped", "maintenance"],
      owner: "alice",
    });

    expect(chips).toEqual([
      { id: "__search__", key: "__search__", label: "内容", value: "gateway" },
      { id: "status", key: "status", label: "状态", value: "运行中, 已停止 +1" },
      { id: "owner", key: "owner", label: "负责人", value: "alice" },
    ]);
  });
});

