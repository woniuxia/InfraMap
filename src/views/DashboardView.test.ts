import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import DashboardView from "@/views/DashboardView.vue";

vi.mock("vue-router", () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}));

vi.mock("@/api/dashboard", () => ({
  getDashboardStats: vi.fn(async () => ({
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
  })),
}));

vi.mock("@/api/audit-logs", () => ({
  listAuditLogs: vi.fn(async () => ({
    data: [
      {
        id: "log-1",
        action: "update",
        resource_type: "middleware",
        resource_id: "mid-1",
        resource_name: "Redis",
        created_at: "2026-02-28T10:00:00.000Z",
      },
    ],
    total: 1,
    page: 1,
    page_size: 20,
  })),
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
  template: `<div><slot /></div>`,
});

const ElTagStub = defineComponent({
  name: "ElTag",
  template: `<span data-testid="el-tag"><slot /></span>`,
});

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
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

describe("DashboardView", () => {
  it("renders resource type as tag in recent changes", async () => {
    const wrapper = mountDashboard();
    await flushPromises();

    const resourceTypeCell = wrapper.find('[data-testid="cell-resource_type-0"]');
    expect(resourceTypeCell.exists()).toBe(true);
    expect(resourceTypeCell.find('[data-testid="el-tag"]').exists()).toBe(true);
    expect(resourceTypeCell.text()).toContain("中间件");
  });
});
