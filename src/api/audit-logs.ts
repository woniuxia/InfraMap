import { tauriInvoke } from '@/utils/invoke'
import type { AuditLog, QueryParams, PagedResult } from '@/types'

export function listAuditLogs(params: QueryParams): Promise<PagedResult<AuditLog>> {
  return tauriInvoke<PagedResult<AuditLog>>('list_audit_logs', { params })
}
