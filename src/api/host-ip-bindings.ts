import { tauriInvoke } from "@/utils/invoke";
import type { HostIpBindingPayload, IpAddress } from "@/types";

export function listHostIpBindings(hostId: string): Promise<IpAddress[]> {
  return tauriInvoke<IpAddress[]>("list_host_ip_bindings", { hostId });
}

export function bindHostIp(payload: HostIpBindingPayload): Promise<void> {
  return tauriInvoke<void>("bind_host_ip", {
    hostId: payload.host_id,
    ipId: payload.ip_id,
  });
}

export function unbindHostIp(payload: HostIpBindingPayload): Promise<void> {
  return tauriInvoke<void>("unbind_host_ip", {
    hostId: payload.host_id,
    ipId: payload.ip_id,
  });
}
