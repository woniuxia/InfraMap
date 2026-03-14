import { tauriInvoke } from "@/utils/invoke";
import type { Service, QueryParams, PagedResult } from "@/types";

export function listServices(params: QueryParams): Promise<PagedResult<Service>> {
  return tauriInvoke<PagedResult<Service>>("list_services", { params });
}

export function getService(id: string): Promise<Service> {
  return tauriInvoke<Service>("get_service", { id });
}

export function saveService(data: Partial<Service>): Promise<string> {
  return tauriInvoke<string>("save_service", { data });
}

export function deleteService(id: string): Promise<void> {
  return tauriInvoke<void>("delete_service", { id });
}

/** @deprecated Use listServices */
export const listApplications = listServices;
/** @deprecated Use getService */
export const getApplication = getService;
/** @deprecated Use saveService */
export const saveApplication = saveService;
/** @deprecated Use deleteService */
export const deleteApplication = deleteService;
