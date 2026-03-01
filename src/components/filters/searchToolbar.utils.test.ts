import { describe, expect, it } from "vitest";
import {
  buildSearchToolbarQuery,
  createEmptyFilters,
  isEmptyFilterValue,
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

  it("isEmptyFilterValue should identify empty and non-empty values", () => {
    expect(isEmptyFilterValue(fields[0], [])).toBe(true);
    expect(isEmptyFilterValue(fields[0], ["running"])).toBe(false);
    expect(isEmptyFilterValue(fields[1], "  ")).toBe(true);
    expect(isEmptyFilterValue(fields[1], "alice")).toBe(false);
  });
});
