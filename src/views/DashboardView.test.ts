import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import DashboardView from "@/views/DashboardView.vue";
import type { DashboardOverview } from "@/types";

const { pushMock, getDashboardOverviewMock } = vi.hoisted(() => ({
  pushMock: vi.fn(),
  getDashboardOverviewMock: vi.fn(),
}));

vi.mock("vue-router", () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock("@/api/dashboard", () => ({
  getDashboardOverview: getDashboardOverviewMock,
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
        (tableProps?.data || []).map((row, index) => {
          const slot = slots.default ? slots.default({ row }) : [String(row[props.prop] ?? "")];
          return h("div", { "data-testid": `cell-${props.prop || props.label}-${index}` }, slot);
        }),
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
      service_total: 3,
      service_abnormal: 1,
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
      undeployed_service_total: 1,
      undeployed_middleware_total: 1,
      undeployed_nginx_total: 0,
    },
    env_distribution: [
      { env: "prod", count: 4 },
      { env: "dev", count: 2 },
    ],
    risk_items: [
      {
        key: "service_abnormal",
        label: "异常服务",
        count: 1,
        severity: "critical",
        target_route: "Services",
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

function mountDashboard(componentStubs: Record<string, unknown> = {}) {
  return mount(DashboardView, {
    global: {
      stubs: {
        ...componentStubs,
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
        Upload: true,
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
  });

  it("renders overview data from dashboard overview API", async () => {
    getDashboardOverviewMock.mockResolvedValue(createOverview());

    const wrapper = mountDashboard();
    await flushPromises();

    expect(wrapper.text()).toContain("Redis");
    expect(wrapper.find('[data-testid="quick-action-import-workbench"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="resource-entry-row"]').exists()).toBe(true);

    const resourceTypeCell = wrapper.find('[data-testid="cell-resource_type-0"]');
    expect(resourceTypeCell.exists()).toBe(true);
    expect(resourceTypeCell.find('[data-testid="el-tag"]').exists()).toBe(true);
  });

  it("composes dashboard presentation with dedicated child sections", async () => {
    getDashboardOverviewMock.mockResolvedValue(createOverview());

    const wrapper = mountDashboard({
      DashboardQuickHub: defineComponent({
        name: "DashboardQuickHub",
        props: {
          primaryActions: {
            type: Array,
            default: () => [],
          },
          secondaryActions: {
            type: Array,
            default: () => [],
          },
        },
        template:
          '<div data-testid="dashboard-quick-hub-stub">{{ primaryActions.length }}-{{ secondaryActions.length }}</div>',
      }),
      DashboardKpiGrid: defineComponent({
        name: "DashboardKpiGrid",
        props: {
          items: {
            type: Array,
            default: () => [],
          },
        },
        template: '<div data-testid="dashboard-kpi-grid-stub">{{ items.length }}</div>',
      }),
      DashboardRecentChanges: defineComponent({
        name: "DashboardRecentChanges",
        props: {
          items: {
            type: Array,
            default: () => [],
          },
        },
        template: '<div data-testid="dashboard-recent-changes-stub">{{ items.length }}</div>',
      }),
      DashboardOverviewPanels: defineComponent({
        name: "DashboardOverviewPanels",
        props: {
          overview: {
            type: Object,
            default: null,
          },
        },
        template:
          '<div data-testid="dashboard-overview-panels-stub">{{ overview?.risk_items?.length ?? 0 }}</div>',
      }),
    });
    await flushPromises();

    expect(wrapper.get('[data-testid="dashboard-quick-hub-stub"]').text()).toBe("5-4");
    expect(wrapper.get('[data-testid="dashboard-kpi-grid-stub"]').text()).toBe("6");
    expect(wrapper.get('[data-testid="dashboard-recent-changes-stub"]').text()).toBe("1");
    expect(wrapper.get('[data-testid="dashboard-overview-panels-stub"]').text()).toBe("1");
  });

  it("navigates by quick actions and supports refresh action", async () => {
    getDashboardOverviewMock.mockResolvedValue(createOverview());

    const wrapper = mountDashboard();
    await flushPromises();

    expect(getDashboardOverviewMock).toHaveBeenCalledTimes(1);

    await wrapper.get('[data-testid="quick-action-services"]').trigger("click");
    expect(pushMock).toHaveBeenCalledWith({
      name: "Services",
      query: undefined,
    });

    await wrapper.get('[data-testid="quick-action-import-workbench"]').trigger("click");
    expect(pushMock).toHaveBeenCalledWith({
      name: "ImportWorkbench",
      query: undefined,
    });

    const beforeRefreshCalls = getDashboardOverviewMock.mock.calls.length;
    await wrapper.get('[data-testid="quick-action-refresh"]').trigger("click");
    await flushPromises();
    expect(getDashboardOverviewMock).toHaveBeenCalledTimes(beforeRefreshCalls + 1);
  });

  it("renders empty state when overview endpoint fails", async () => {
    getDashboardOverviewMock.mockRejectedValue(new Error("overview unavailable"));

    const wrapper = mountDashboard();
    await flushPromises();

    expect(getDashboardOverviewMock).toHaveBeenCalledTimes(1);
    expect(wrapper.text()).not.toContain("Redis");
  });
});
