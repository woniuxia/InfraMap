import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDashboardOverview } from "@/composables/useDashboardOverview";
import type { DashboardOverview } from "@/types";

const { getDashboardOverviewMock } = vi.hoisted(() => ({
  getDashboardOverviewMock: vi.fn(),
}));

vi.mock("@/api/dashboard", () => ({
  getDashboardOverview: getDashboardOverviewMock,
}));

function createOverview(): DashboardOverview {
  return {
    totals: {
      host_total: 2,
      host_abnormal: 1,
      application_total: 2,
      application_abnormal: 1,
      middleware_total: 1,
      nginx_total: 1,
      nginx_abnormal: 0,
      deployment_total: 2,
      dependency_total: 1,
    },
    health: {
      abnormal_total: 2,
      abnormal_rate: 33.3,
      score: 66.7,
    },
    coverage: {
      deployable_total: 4,
      deployed_total: 2,
      undeployed_total: 2,
      deployment_coverage: 50,
      relatable_total: 4,
      related_total: 2,
      isolated_total: 2,
      relation_coverage: 50,
      undeployed_application_total: 1,
      undeployed_middleware_total: 1,
      undeployed_nginx_total: 0,
    },
    env_distribution: [{ env: "prod", count: 4 }],
    risk_items: [],
    recent_changes: [],
  };
}

describe("useDashboardOverview", () => {
  beforeEach(() => {
    getDashboardOverviewMock.mockReset();
  });

  it("loads overview when API succeeds", async () => {
    getDashboardOverviewMock.mockResolvedValue(createOverview());

    const vm = useDashboardOverview();
    await vm.loadDashboardOverview();

    expect(vm.overview.value?.totals.host_total).toBe(2);
  });

  it("clears overview when API fails", async () => {
    getDashboardOverviewMock.mockRejectedValue(new Error("unsupported"));

    const vm = useDashboardOverview();
    await vm.loadDashboardOverview();

    expect(vm.overview.value).toBeNull();
  });
});
