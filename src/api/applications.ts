import { tauriInvoke } from '@/utils/invoke'
import type { Application, QueryParams, PagedResult } from '@/types'
export type ApplicationTechStackSide = 'frontend' | 'backend'

export function listApplications(params: QueryParams): Promise<PagedResult<Application>> {
  return tauriInvoke<PagedResult<Application>>('list_applications', { params })
}

export function getApplication(id: string): Promise<Application> {
  return tauriInvoke<Application>('get_application', { id })
}

export function listTopApplicationTechStacks(
  limit = 10,
  appType: ApplicationTechStackSide = 'backend'
): Promise<string[]> {
  return tauriInvoke<string[]>('list_top_application_tech_stacks', { limit, app_type: appType })
}

export function listApplicationOwnerCandidates(limit = 100): Promise<string[]> {
  return tauriInvoke<string[]>('list_application_owner_candidates', { limit })
}

export function saveApplication(data: Partial<Application>): Promise<string> {
  return tauriInvoke<string>('save_application', { data })
}

export function softDeleteApplication(id: string): Promise<void> {
  return tauriInvoke<void>('soft_delete_application', { id })
}
