import { tauriInvoke } from '@/utils/invoke'
import type { DashboardOverview } from '@/types'

export function getDashboardOverview(): Promise<DashboardOverview> {
  return tauriInvoke<DashboardOverview>('get_dashboard_overview')
}
