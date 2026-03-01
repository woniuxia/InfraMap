import { tauriInvoke } from "@/utils/invoke";

export type TaxonomyResourceType =
  | "host"
  | "ip_address"
  | "application"
  | "business_application";
export type TaxonomyFieldKey = "tags" | "owner" | "tech_stack";
export type TaxonomySortBy = "alpha" | "recent";
export type TaxonomyRecencyScope = "global" | "resource_type";
export type TaxonomyAppType = "frontend" | "backend";

export interface ListTaxonomyTermsParams {
  resource_type: TaxonomyResourceType;
  field_key: TaxonomyFieldKey;
  limit?: number;
  sort_by?: TaxonomySortBy;
  recency_scope?: TaxonomyRecencyScope;
  app_type?: TaxonomyAppType;
}

export interface SaveResourceTermsParams {
  resource_type: TaxonomyResourceType;
  resource_id: string;
  field_key: TaxonomyFieldKey;
  values: string[];
}

export interface ListResourceTermsParams {
  resource_type: TaxonomyResourceType;
  resource_id: string;
  field_key: TaxonomyFieldKey;
}

export function listTaxonomyTerms(params: ListTaxonomyTermsParams): Promise<string[]> {
  const { resource_type, field_key, limit, sort_by, recency_scope, app_type } = params;
  return tauriInvoke<string[]>("list_taxonomy_terms", {
    resourceType: resource_type,
    fieldKey: field_key,
    limit,
    sortBy: sort_by,
    recencyScope: recency_scope,
    appType: app_type,
  });
}

export function listHostTagTerms(limit = 200): Promise<string[]> {
  return listTaxonomyTerms({
    resource_type: "host",
    field_key: "tags",
    limit,
    sort_by: "recent",
    recency_scope: "resource_type",
  });
}

export function listIpTagTerms(limit = 200): Promise<string[]> {
  return listTaxonomyTerms({
    resource_type: "ip_address",
    field_key: "tags",
    limit,
    sort_by: "recent",
    recency_scope: "resource_type",
  });
}

export function listApplicationOwnerTerms(limit = 100): Promise<string[]> {
  return listTaxonomyTerms({
    resource_type: "application",
    field_key: "owner",
    limit,
    sort_by: "recent",
    recency_scope: "global",
  });
}

export function listBusinessApplicationOwnerTerms(limit = 100): Promise<string[]> {
  return listTaxonomyTerms({
    resource_type: "business_application",
    field_key: "owner",
    limit,
    sort_by: "recent",
    recency_scope: "resource_type",
  });
}

export interface ListApplicationTechStackTermsParams {
  app_type: TaxonomyAppType;
  limit?: number;
}

export function listApplicationTechStackTerms(
  params: ListApplicationTechStackTermsParams
): Promise<string[]> {
  const { app_type, limit = 10 } = params;
  return listTaxonomyTerms({
    resource_type: "application",
    field_key: "tech_stack",
    limit,
    sort_by: "recent",
    recency_scope: "global",
    app_type,
  });
}

export function saveResourceTerms(params: SaveResourceTermsParams): Promise<void> {
  const { resource_type, resource_id, field_key, values } = params;
  return tauriInvoke<void>("save_resource_terms_command", {
    resourceType: resource_type,
    resourceId: resource_id,
    fieldKey: field_key,
    values,
  });
}

export function listResourceTerms(params: ListResourceTermsParams): Promise<string[]> {
  const { resource_type, resource_id, field_key } = params;
  return tauriInvoke<string[]>("list_resource_terms", {
    resourceType: resource_type,
    resourceId: resource_id,
    fieldKey: field_key,
  });
}
