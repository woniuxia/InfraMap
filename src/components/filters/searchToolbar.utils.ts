import type {
  SearchFieldConfig,
  SearchFieldValue,
  SearchToolbarFilters,
  SearchToolbarQueryPayload,
} from "@/types/searchToolbar";

function normalizeText(value: string | undefined | null): string {
  return (value ?? "").trim();
}

function normalizeArray(value: SearchFieldValue | undefined): string[] {
  if (!Array.isArray(value)) return [];
  return value.map((item) => normalizeText(item)).filter(Boolean);
}

export function buildSearchToolbarQuery(
  searchText: string,
  fields: SearchFieldConfig[],
  filters: SearchToolbarFilters
): SearchToolbarQueryPayload {
  const nextFilters: Record<string, string> = {};
  for (const field of fields) {
    const key = field.queryKey ?? field.key;
    const value = filters[field.key];
    if (field.type === "multi-select") {
      const values = normalizeArray(value);
      if (values.length > 0) {
        nextFilters[key] = JSON.stringify(values);
      }
      continue;
    }

    const normalized = normalizeText(typeof value === "string" ? value : "");
    if (normalized) {
      nextFilters[key] = normalized;
    }
  }

  return {
    search: normalizeText(searchText),
    filters: nextFilters,
  };
}

export function createEmptyFilters(fields: SearchFieldConfig[]): SearchToolbarFilters {
  return fields.reduce<SearchToolbarFilters>((acc, field) => {
    acc[field.key] = field.type === "multi-select" ? [] : "";
    return acc;
  }, {});
}

export function isEmptyFilterValue(field: SearchFieldConfig, value: SearchFieldValue | undefined): boolean {
  if (field.type === "multi-select") {
    return normalizeArray(value).length === 0;
  }
  return !normalizeText(typeof value === "string" ? value : "");
}
