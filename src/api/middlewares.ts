import { tauriInvoke } from '@/utils/invoke'
import type { Middleware, QueryParams, PagedResult } from '@/types'

export function listMiddlewares(params: QueryParams): Promise<PagedResult<Middleware>> {
  return tauriInvoke<PagedResult<Middleware>>('list_middlewares', { params })
}

export function getMiddleware(id: string): Promise<Middleware> {
  return tauriInvoke<Middleware>('get_middleware', { id })
}

export function saveMiddleware(data: Partial<Middleware>): Promise<string> {
  return tauriInvoke<string>('save_middleware', { data })
}

export function softDeleteMiddleware(id: string): Promise<void> {
  return tauriInvoke<void>('soft_delete_middleware', { id })
}
