import { tauriInvoke } from '@/utils/invoke'
import type { Application, QueryParams, PagedResult } from '@/types'

export function listApplications(params: QueryParams): Promise<PagedResult<Application>> {
  return tauriInvoke<PagedResult<Application>>('list_applications', { params })
}

export function getApplication(id: string): Promise<Application> {
  return tauriInvoke<Application>('get_application', { id })
}

export function saveApplication(data: Partial<Application>): Promise<string> {
  return tauriInvoke<string>('save_application', { data })
}

export function deleteApplication(id: string): Promise<void> {
  return tauriInvoke<void>('delete_application', { id })
}

