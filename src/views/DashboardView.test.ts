import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import DashboardView from "@/views/DashboardView.vue";
import type { DashboardOverview, DashboardStats } from "@/types";

const { pushMock, getDashboardOverviewMock, getDashboardStatsMock, listAuditLogsMock } =
  vi.hoisted(() => ({
    pushMock: vi.fn(),
    getDashboardOverviewMock: vi.fn(),
    getDashboardStatsMock: vi.fn(),
    listAuditLogsMock: vi.fn(),
  }));

vi.mock("vue-router", () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock("@/api/dashboard", () => ({
  getDashboardOverview: getDashboardOverviewMock,
  getDashboardStats: getDashboardStatsMock,
}));

vi.mock("@/api/audit-logs", () => ({
  listAuditLogs: listAuditLogsMock,
}));

const tableDataKey = Symbol("tableDataKey");

const ElTableStub = defineComponent({
  name: "ElTable",
  props: {
    data: {
      type: Array,
      default: () => [],
    },
  },
  setup(props, { slots }) {
    provide(tableDataKey, props);
    return () => h("div", { "data-testid": "el-table" }, slots.default ? slots.default() : []);
  },
});

const ElTableColumnStub = defineComponent({
  name: "ElTableColumn",
  props: {
    prop: {
      type: String,
      default: "",
    },
    label: {
      type: String,
      default: "",
    },
  },
  setup(props, { slots }) {
    const tableProps = inject<{ data: Record<string, unknown>[] }>(tableDataKey);
    return () =>
      h(
        "div",
        { "data-testid": `column-${props.prop || props.label}` },
        (tableProps?.data || []).map((row, index) =>
          h(
            "div",
            { "data-testid": `cell-${props.prop || props.label}-${index}` },
            slots.default ? slots.default({ row }) : String(row[props.prop] ?? ""),
          ),
        ),
      );
  },
});

const PassThroughStub = defineComponent({
  template: "<div><slot /></div>",
});

const ElTagStub = defineComponent({
  name: "ElTag",
  template: '<span data-testid="el-tag"><slot /></span>',
});

function createOverview(): DashboardOverview {
  return {
    totals: {
      host_total: 2,
      host_abnormal: 1,
      application_total: 3,
      application_abnormal: 1,
      middleware_total: 2,
      nginx_total: 1,
      nginx_abnormal: 0,
      deployment_total: 4,
      dependency_total: 2,
    },
    health: {
      abnormal_total: 2,
      abnormal_rate: 28.6,
      score: 72.4,
    },
    coverage: {
      deployable_total: 6,
      deployed_total: 4,
      undeployed_total: 2,
      deployment_coverage: 66.7,
      relatable_total: 6,
      related_total: 3,
      isolated_total: 3,
      relation_coverage: 50,
      undeployed_application_total: 1,
      undeployed_middleware_total: 1,
      undeployed_nginx_total: 0,
    },
    env_distribution: [
      { env: "prod", count: 4 },
      { env: "dev", count: 2 },
    ],
    risk_items: [
      {
        key: "application_abnormal",
        label: "异常应用服务",
        count: 1,
        severity: "critical",
        target_route: "Applications",
        target_filters: { status: "stopped,maintenance" },
      },
    ],
    recent_changes: [
      {
        id: "log-1",
        action: "update",
        resource_type: "middleware",
        resource_id: "mw-1",
        resource_name: "Redis",
        created_at: "2026-02-28T10:00:00.000Z",
      },
    ],
  };
}

function createStats(): DashboardStats {
  return {
    host_total: 1,
    host_abnormal: 0,
    application_total: 1,
    application_abnormal: 0,
    middleware_total: 1,
    nginx_total: 1,
    nginx_abnormal: 0,
    deployment_total: 1,
    dependency_total: 1,
    env_distribution: [{ env: "prod", count: 1 }],
  };
}

function mountDashboard() {
  return mount(DashboardView, {
    global: {
      stubs: {
        ElRow: PassThroughStub,
        ElCol: PassThroughStub,
        ElCard: PassThroughStub,
        ElIcon: PassThroughStub,
        ElButton: true,
        ElText: true,
        ElProgress: true,
        ElEmpty: true,
        ElTable: ElTableStub,
        ElTableColumn: ElTableColumnStub,
        ElTag: ElTagStub,
        Monitor: true,
        Menu: true,
        Connection: true,
        SetUp: true,
        Refresh: true,
        Warning: true,
        Files: true,
        Share: true,
        DataAnalysis: true,
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

describe("DashboardView", () => {
  beforeEach(() => {
    pushMock.mockReset();
    getDashboardOverviewMock.mockReset();
    getDashboardStatsMock.mockReset();
    listAuditLogsMock.mockReset();
  });

  it("renders overview data from new dashboard API", async () => {
    getDashboardOverviewMock.mockResolvedValue(createOverview());

    const wrapper = mountDashboard();
    await flushPromises();

    expect(wrapper.text()).toContain("资产健康驾驶舱");
    expect(wrapper.text()).toContain("异常应用服务");
    expect(wrapper.text()).toContain("Redis");

    const resourceTypeCell = wrapper.find('[data-testid="cell-resource_type-0"]');
    expect(resourceTypeCell.exists()).toBe(true);
    expect(resourceTypeCell.find('[data-testid="el-tag"]').exists()).toBe(true);
    expect(resourceTypeCell.text()).toContain("中间件");
  });

  it("falls back to legacy APIs when overview endpoint fails", async () => {
    getDashboardOverviewMock.mockRejectedValue(new Error("overview unavailable"));
    getDashboardStatsMock.mockResolvedValue(createStats());
    listAuditLogsMock.mockResolvedValue({
      data: [
        {
          id: "legacy-log",
          action: "create",
          resource_type: "application",
          resource_id: "app-1",
          resource_name: "Portal",
          created_at: "2026-02-28T10:00:00.000Z",
        },
      ],
      total: 1,
      page: 1,
      page_size: 20,
    });

    const wrapper = mountDashboard();
    await flushPromises();

    expect(wrapper.text()).toContain("统计回退模式");
    expect(wrapper.text()).toContain("Portal");
    expect(getDashboardStatsMock).toHaveBeenCalledTimes(1);
    expect(listAuditLogsMock).toHaveBeenCalledTimes(1);
  });
});
