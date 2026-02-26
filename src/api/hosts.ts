import { tauriInvoke } from '@/utils/invoke'
import type { Host, QueryParams, PagedResult } from '@/types'

export function listHosts(params: QueryParams): Promise<PagedResult<Host>> {
  return tauriInvoke<PagedResult<Host>>('list_hosts', { params })
}

export function getHost(id: string): Promise<Host> {
  return tauriInvoke<Host>('get_host', { id })
}

export function saveHost(data: Partial<Host>): Promise<void> {
  return tauriInvoke<void>('save_host', { data })
}

export function softDeleteHost(id: string): Promise<void> {
  return tauriInvoke<void>('soft_delete_host', { id })
}
