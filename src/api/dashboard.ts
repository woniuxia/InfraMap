import { tauriInvoke } from '@/utils/invoke'
import type { DashboardOverview, DashboardStats } from '@/types'

export function getDashboardOverview(): Promise<DashboardOverview> {
  return tauriInvoke<DashboardOverview>('get_dashboard_overview')
}

export function getDashboardStats(): Promise<DashboardStats> {
  return tauriInvoke<DashboardStats>('get_dashboard_stats')
}
