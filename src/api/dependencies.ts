import { tauriInvoke } from '@/utils/invoke'
import type {
  Dependency,
  PagedResult,
  QueryParams,
  SaveDependenciesBatchParams,
  SaveDependenciesBatchResult,
} from '@/types'

export function listDependencies(params: QueryParams): Promise<PagedResult<Dependency>> {
  return tauriInvoke<PagedResult<Dependency>>('list_dependencies', { params })
}

export function saveDependency(data: Partial<Dependency>): Promise<void> {
  return tauriInvoke<void>('save_dependency', { data })
}

export function softDeleteDependency(id: string): Promise<void> {
  return tauriInvoke<void>('soft_delete_dependency', { id })
}

export function saveDependenciesBatch(
  params: SaveDependenciesBatchParams
): Promise<SaveDependenciesBatchResult> {
  return tauriInvoke<SaveDependenciesBatchResult>('save_dependencies_batch', { params })
}
