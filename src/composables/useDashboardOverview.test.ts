import { describe, expect, it, beforeEach, vi } from "vitest";
import { useDashboardOverview } from "@/composables/useDashboardOverview";
import type { DashboardOverview, DashboardStats } from "@/types";

const { getDashboardOverviewMock, getDashboardStatsMock, listAuditLogsMock } = vi.hoisted(
  () => ({
    getDashboardOverviewMock: vi.fn(),
    getDashboardStatsMock: vi.fn(),
    listAuditLogsMock: vi.fn(),
  }),
);

vi.mock("@/api/dashboard", () => ({
  getDashboardOverview: getDashboardOverviewMock,
  getDashboardStats: getDashboardStatsMock,
}));

vi.mock("@/api/audit-logs", () => ({
  listAuditLogs: listAuditLogsMock,
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

function createStats(): DashboardStats {
  return {
    host_total: 1,
    host_abnormal: 1,
    application_total: 1,
    application_abnormal: 1,
    middleware_total: 1,
    nginx_total: 1,
    nginx_abnormal: 0,
    deployment_total: 1,
    dependency_total: 1,
    env_distribution: [{ env: "prod", count: 2 }],
  };
}

describe("useDashboardOverview", () => {
  beforeEach(() => {
    getDashboardOverviewMock.mockReset();
    getDashboardStatsMock.mockReset();
    listAuditLogsMock.mockReset();
  });

  it("loads overview directly when new API succeeds", async () => {
    getDashboardOverviewMock.mockResolvedValue(createOverview());

    const vm = useDashboardOverview();
    await vm.loadDashboardOverview();

    expect(vm.usedFallback.value).toBe(false);
    expect(vm.overview.value?.totals.host_total).toBe(2);
    expect(getDashboardStatsMock).not.toHaveBeenCalled();
  });

  it("falls back to legacy stats and audit logs", async () => {
    getDashboardOverviewMock.mockRejectedValue(new Error("unsupported"));
    getDashboardStatsMock.mockResolvedValue(createStats());
    listAuditLogsMock.mockResolvedValue({
      data: [
        {
          id: "log-1",
          action: "create",
          resource_type: "application",
          resource_id: "app-1",
          resource_name: "Portal",
          created_at: "2026-03-01T10:00:00.000Z",
        },
      ],
      total: 1,
      page: 1,
      page_size: 20,
    });

    const vm = useDashboardOverview();
    await vm.loadDashboardOverview();

    expect(vm.usedFallback.value).toBe(true);
    expect(vm.overview.value?.health.abnormal_total).toBe(2);
    expect(vm.overview.value?.recent_changes.length).toBe(1);
    expect(vm.overview.value?.risk_items.some((item) => item.key === "host_abnormal")).toBe(true);
  });
});
