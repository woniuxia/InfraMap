import { describe, expect, it, vi } from "vitest";
import { defineComponent } from "vue";
import { mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import ServicesView from "@/views/ServicesView.vue";
import NginxConfigsView from "@/views/NginxConfigsView.vue";

vi.mock("@/api/services", () => ({
  listServices: vi.fn(async () => ({
    data: [],
    total: 0,
    page: 1,
    page_size: 20,
  })),
  saveService: vi.fn(async () => "svc-1"),
  deleteService: vi.fn(async () => undefined),
}));

vi.mock("@/api/taxonomy", () => ({
  listServiceOwnerTerms: vi.fn(async () => []),
  listServiceTechStackTerms: vi.fn(async () => []),
}));

vi.mock("@/api/nginx-configs", () => ({
  listNginxConfigs: vi.fn(async () => ({
    data: [],
    total: 0,
    page: 1,
    page_size: 20,
  })),
  saveNginxConfig: vi.fn(async () => "nginx-1"),
  deleteNginxConfig: vi.fn(async () => undefined),
}));

const SearchToolbarStub = defineComponent({
  name: "SearchToolbar",
  props: {
    fields: {
      type: Array,
      default: () => [],
    },
  },
  template: `<div><slot name="actions" :hasActiveFilters="false" :reset="() => {}" /></div>`,
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

const ElTableColumnStub = defineComponent({
  name: "ElTableColumn",
  template: `<div />`,
});

interface ToolbarFieldItem {
  key: string;
  section?: "basic" | "advanced";
}

function getToolbarFields(wrapper: ReturnType<typeof mount>): ToolbarFieldItem[] {
  const toolbar = wrapper.findComponent(SearchToolbarStub);
  expect(toolbar.exists()).toBe(true);
  return toolbar.props("fields") as ToolbarFieldItem[];
}

describe("search toolbar field placement", () => {
  it("keeps deploy mode in basic filters for services", () => {
    const wrapper = mount(ServicesView, {
      global: {
        plugins: [createPinia()],
        stubs: {
          SearchToolbar: SearchToolbarStub,
          ElButton: PassThroughStub,
          ElTable: PassThroughStub,
          ElTableColumn: ElTableColumnStub,
          ElTag: PassThroughStub,
          ElPagination: PassThroughStub,
          ElDialog: PassThroughStub,
          ElForm: PassThroughStub,
          ElFormItem: PassThroughStub,
          ElDivider: PassThroughStub,
          ElInput: PassThroughStub,
          ElInputNumber: PassThroughStub,
          ElSelect: PassThroughStub,
          ElOption: PassThroughStub,
          CallRelationsEditor: PassThroughStub,
          DeploymentPanel: PassThroughStub,
          ServiceEditorDialog: PassThroughStub,
        },
        directives: {
          loading: () => undefined,
        },
      },
    });

    const fields = getToolbarFields(wrapper);
    const deployModeField = fields.find((field) => field.key === "deploy_mode");
    expect(deployModeField).toBeDefined();
    expect(deployModeField?.section ?? "basic").toBe("basic");
  });

  it("keeps strategy in basic filters for nginx configs", () => {
    const wrapper = mount(NginxConfigsView, {
      global: {
        plugins: [createPinia()],
        stubs: {
          SearchToolbar: SearchToolbarStub,
          ElButton: PassThroughStub,
          ElTable: PassThroughStub,
          ElTableColumn: ElTableColumnStub,
          ElTag: PassThroughStub,
          ElPagination: PassThroughStub,
          ElDialog: PassThroughStub,
          ElForm: PassThroughStub,
          ElFormItem: PassThroughStub,
          ElDivider: PassThroughStub,
          ElInput: PassThroughStub,
          ElInputNumber: PassThroughStub,
          ElSelect: PassThroughStub,
          ElOption: PassThroughStub,
          CallRelationsEditor: PassThroughStub,
          DeploymentPanel: PassThroughStub,
          NginxConfigEditorDialog: PassThroughStub,
        },
        directives: {
          loading: () => undefined,
        },
      },
    });

    const fields = getToolbarFields(wrapper);
    const strategyField = fields.find((field) => field.key === "strategy");
    expect(strategyField).toBeDefined();
    expect(strategyField?.section ?? "basic").toBe("basic");
  });
});
