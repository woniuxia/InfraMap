import { tauriInvoke } from '@/utils/invoke'
import type { DashboardStats } from '@/types'

export function getDashboardStats(): Promise<DashboardStats> {
  return tauriInvoke<DashboardStats>('get_dashboard_stats')
}
