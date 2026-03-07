import { ref } from "vue";
import { getDashboardOverview } from "@/api/dashboard";
import type { DashboardOverview } from "@/types";

export function useDashboardOverview() {
  const overview = ref<DashboardOverview | null>(null);
  const loading = ref(false);

  async function loadDashboardOverview() {
    loading.value = true;
    try {
      overview.value = await getDashboardOverview();
    } catch {
      overview.value = null;
    } finally {
      loading.value = false;
    }
  }

  return {
    overview,
    loading,
    loadDashboardOverview,
  };
}
