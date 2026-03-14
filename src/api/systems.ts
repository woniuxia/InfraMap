import { tauriInvoke } from "@/utils/invoke";
import type { InfraSystem, SystemServices, PagedResult, QueryParams } from "@/types";

export function listSystems(params: QueryParams): Promise<PagedResult<InfraSystem>> {
  return tauriInvoke<PagedResult<InfraSystem>>("list_systems", { params });
}

export function getSystem(id: string): Promise<InfraSystem> {
  return tauriInvoke<InfraSystem>("get_system", { id });
}

export function saveSystem(data: Partial<InfraSystem>): Promise<string> {
  return tauriInvoke<string>("save_system", { data });
}

export function deleteSystem(id: string): Promise<void> {
  return tauriInvoke<void>("delete_system", { id });
}

export function listServicesBySystem(systemId: string): Promise<SystemServices> {
  return tauriInvoke<SystemServices>("list_services_by_system", {
    systemId,
  });
}

/** @deprecated Use listSystems */
export const listBusinessApplications = listSystems;
/** @deprecated Use getSystem */
export const getBusinessApplication = getSystem;
/** @deprecated Use saveSystem */
export const saveBusinessApplication = saveSystem;
/** @deprecated Use deleteSystem */
export const deleteBusinessApplication = deleteSystem;
