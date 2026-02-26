import { tauriInvoke } from '@/utils/invoke'
import type { NginxConfig, QueryParams, PagedResult } from '@/types'

export function listNginxConfigs(params: QueryParams): Promise<PagedResult<NginxConfig>> {
  return tauriInvoke<PagedResult<NginxConfig>>('list_nginx_configs', { params })
}

export function getNginxConfig(id: string): Promise<NginxConfig> {
  return tauriInvoke<NginxConfig>('get_nginx_config', { id })
}

export function saveNginxConfig(data: Partial<NginxConfig>): Promise<void> {
  return tauriInvoke<void>('save_nginx_config', { data })
}

export function softDeleteNginxConfig(id: string): Promise<void> {
  return tauriInvoke<void>('soft_delete_nginx_config', { id })
}
