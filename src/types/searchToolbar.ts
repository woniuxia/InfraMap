export type SearchFieldType = "text" | "select" | "multi-select";
export type SearchFieldSection = "basic" | "advanced";
export type SearchFieldWidth = "sm" | "md" | "lg";

export interface SearchFieldOption {
  label: string;
  value: string;
}

export interface SearchFieldConfig {
  key: string;
  queryKey?: string;
  label: string;
  type: SearchFieldType;
  section?: SearchFieldSection;
  width?: SearchFieldWidth;
  placeholder?: string;
  clearable?: boolean;
  multiple?: boolean;
  options?: SearchFieldOption[];
  maxCollapseTags?: number;
}

export type SearchFieldValue = string | string[];
export type SearchToolbarFilters = Record<string, SearchFieldValue>;

export interface SearchToolbarQueryPayload {
  search: string;
  filters: Record<string, string>;
}

export interface SearchToolbarChip {
  id: string;
  key: string;
  label: string;
  value: string;
}
