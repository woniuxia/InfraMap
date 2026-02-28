import type {
  SearchFieldConfig,
  SearchFieldOption,
  SearchFieldValue,
  SearchToolbarChip,
  SearchToolbarFilters,
  SearchToolbarQueryPayload,
} from "@/types/searchToolbar";

const SEARCH_CHIP_KEY = "__search__";
const DEFAULT_MAX_CHIP_VALUES = 2;

function normalizeText(value: string | undefined | null): string {
  return (value ?? "").trim();
}

function normalizeArray(value: SearchFieldValue | undefined): string[] {
  if (!Array.isArray(value)) return [];
  return value.map((item) => normalizeText(item)).filter(Boolean);
}

function optionLabelMap(options: SearchFieldOption[] | undefined): Record<string, string> {
  if (!options) return {};
  return options.reduce<Record<string, string>>((acc, option) => {
    acc[option.value] = option.label;
    return acc;
  }, {});
}

function formatMultiValue(values: string[], options: SearchFieldOption[] | undefined, maxChipValues: number): string {
  const labelByValue = optionLabelMap(options);
  const normalized = values.map((item) => labelByValue[item] ?? item);
  if (normalized.length <= maxChipValues) {
    return normalized.join(", ");
  }
  const visible = normalized.slice(0, maxChipValues).join(", ");
  return `${visible} +${normalized.length - maxChipValues}`;
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

export function getActiveFilterChips(
  searchText: string,
  searchLabel: string,
  fields: SearchFieldConfig[],
  filters: SearchToolbarFilters,
  maxChipValues = DEFAULT_MAX_CHIP_VALUES
): SearchToolbarChip[] {
  const chips: SearchToolbarChip[] = [];
  const normalizedSearch = normalizeText(searchText);
  if (normalizedSearch) {
    chips.push({
      id: SEARCH_CHIP_KEY,
      key: SEARCH_CHIP_KEY,
      label: searchLabel,
      value: normalizedSearch,
    });
  }

  for (const field of fields) {
    const rawValue = filters[field.key];
    if (field.type === "multi-select") {
      const values = normalizeArray(rawValue);
      if (values.length === 0) continue;
      chips.push({
        id: field.key,
        key: field.key,
        label: field.label,
        value: formatMultiValue(values, field.options, maxChipValues),
      });
      continue;
    }

    const value = normalizeText(typeof rawValue === "string" ? rawValue : "");
    if (!value) continue;
    chips.push({
      id: field.key,
      key: field.key,
      label: field.label,
      value,
    });
  }

  return chips;
}

export function isEmptyFilterValue(field: SearchFieldConfig, value: SearchFieldValue | undefined): boolean {
  if (field.type === "multi-select") {
    return normalizeArray(value).length === 0;
  }
  return !normalizeText(typeof value === "string" ? value : "");
}
