import { tauriInvoke } from "@/utils/invoke";
import type {
  BatchCreateIpParams,
  BatchCreateIpResult,
  IpAddress,
  PagedResult,
  QueryParams,
} from "@/types";

export function listIpAddresses(params: QueryParams): Promise<PagedResult<IpAddress>> {
  return tauriInvoke<PagedResult<IpAddress>>("list_ip_addresses", { params });
}

export function getIpAddress(id: string): Promise<IpAddress> {
  return tauriInvoke<IpAddress>("get_ip_address", { id });
}

export function saveIpAddress(data: Partial<IpAddress>): Promise<void> {
  return tauriInvoke<void>("save_ip_address", { data });
}

export function batchCreateIpAddresses(params: BatchCreateIpParams): Promise<BatchCreateIpResult> {
  return tauriInvoke<BatchCreateIpResult>("batch_create_ip_addresses", { params });
}

export function softDeleteIpAddress(id: string): Promise<void> {
  return tauriInvoke<void>("soft_delete_ip_address", { id });
}
