import { tauriInvoke } from "@/utils/invoke";
import type { Host, QueryParams, PagedResult } from "@/types";

export function listHosts(params: QueryParams): Promise<PagedResult<Host>> {
  return tauriInvoke<PagedResult<Host>>("list_hosts", { params });
}

export function getHost(id: string): Promise<Host> {
  return tauriInvoke<Host>("get_host", { id });
}

export function saveHost(data: Partial<Host>): Promise<string> {
  return tauriInvoke<string>("save_host", { data });
}

export function deleteHost(id: string): Promise<void> {
  return tauriInvoke<void>("delete_host", { id });
}
