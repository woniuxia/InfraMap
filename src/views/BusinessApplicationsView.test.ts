import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import BusinessApplicationsView from "@/views/BusinessApplicationsView.vue";
import {
  listBusinessApplications,
  listServicesByBusinessApplication,
  replaceServicesByBusinessApplication,
  saveBusinessApplication,
  deleteBusinessApplication,
} from "@/api/business-applications";
import { listApplications } from "@/api/applications";
import {
  listBusinessApplicationOwnerTerms,
  listResourceTerms,
  saveResourceTerms,
} from "@/api/taxonomy";

const { messageSuccess } = vi.hoisted(() => ({
  messageSuccess: vi.fn(),
}));

vi.mock("element-plus", () => ({
  ElMessage: {
    success: messageSuccess,
    warning: vi.fn(),
    info: vi.fn(),
    error: vi.fn(),
  },
}));

vi.mock("@/api/business-applications", () => ({
  listBusinessApplications: vi.fn(async () => ({
    data: [
      {
        id: "ba-1",
        name: "支付中心",
        owners: ["alice", "bob"],
        env: "prod",
        status: "active",
        created_at: "",
        updated_at: "",
      },
    ],
    total: 1,
    page: 1,
    page_size: 20,
  })),
  saveBusinessApplication: vi.fn(async () => "ba-1"),
  deleteBusinessApplication: vi.fn(async () => undefined),
  listServicesByBusinessApplication: vi.fn(async () => ({
    frontend: [
      {
        id: "app-fe-1",
        name: "portal-web",
        type: "frontend",
        address: "10.0.0.10",
        port: 80,
      },
    ],
    backend: [
      {
        id: "app-be-1",
        name: "payment-api",
        type: "backend",
        address: "10.0.0.21",
        port: 8080,
      },
    ],
  })),
  replaceServicesByBusinessApplication: vi.fn(async () => ({
    attached_count: 0,
    detached_count: 0,
    unchanged_count: 0,
  })),
}));

vi.mock("@/api/applications", () => ({
  listApplications: vi.fn(async () => ({
    data: [],
    total: 0,
    page: 1,
    page_size: 500,
  })),
}));

vi.mock("@/api/taxonomy", () => ({
  listBusinessApplicationOwnerTerms: vi.fn(async () => ["alice", "bob"]),
  listResourceTerms: vi.fn(async () => ["alice", "bob"]),
  saveResourceTerms: vi.fn(async () => undefined),
}));

const SearchToolbarStub = defineComponent({
  name: "SearchToolbar",
  template: `<div><slot name="actions" :hasActiveFilters="false" :reset="() => {}" /></div>`,
});

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button type="button" @click="$emit('click')"><slot /></button>`,
});

const ElDialogStub = defineComponent({
  name: "ElDialog",
  props: {
    modelValue: {
      type: Boolean,
      default: false,
    },
  },
  template: `<section v-if="modelValue"><slot /><slot name="footer" /></section>`,
});

const ElInputStub = defineComponent({
  name: "ElInput",
  props: {
    modelValue: {
      type: [String, Number],
      default: "",
    },
  },
  emits: ["update:modelValue"],
  template: `<input :value="modelValue" @input="$emit('update:modelValue', $event.target.value)" />`,
});

const ElSelectStub = defineComponent({
  name: "ElSelect",
  props: {
    modelValue: {
      type: [String, Array],
      default: "",
    },
    multiple: {
      type: Boolean,
      default: false,
    },
  },
  emits: ["update:modelValue", "change"],
  template: `<div><slot /></div>`,
});

const ElOptionStub = defineComponent({
  name: "ElOption",
  template: `<div><slot /></div>`,
});

const ElOptionGroupStub = defineComponent({
  name: "ElOptionGroup",
  template: `<div><slot /></div>`,
});

const ElTagStub = defineComponent({
  name: "ElTag",
  template: `<span><slot /></span>`,
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

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
    return () => h("div", { "data-testid": "table-stub" }, slots.default ? slots.default() : []);
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
        {},
        [
          h("div", { "data-testid": `column-${props.prop || props.label}` }, props.label || ""),
          ...(tableProps?.data || []).map((row, index) =>
            h(
              "div",
              { key: `${props.prop || props.label}-${index}` },
              slots.default ? slots.default({ row, $index: index }) : String(row[props.prop] ?? ""),
            ),
          ),
        ],
      );
  },
});

function mountView() {
  return mount(BusinessApplicationsView, {
    global: {
      stubs: {
        SearchToolbar: SearchToolbarStub,
        ElButton: ElButtonStub,
        ElDialog: ElDialogStub,
        ElInput: ElInputStub,
        ElSelect: ElSelectStub,
        ElOption: ElOptionStub,
        ElOptionGroup: ElOptionGroupStub,
        ElTag: ElTagStub,
        ElForm: PassThroughStub,
        ElFormItem: PassThroughStub,
        ElTable: ElTableStub,
        ElTableColumn: ElTableColumnStub,
        ElPagination: PassThroughStub,
        ElCard: PassThroughStub,
        ElEmpty: PassThroughStub,
        ElDivider: PassThroughStub,
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

describe("BusinessApplicationsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders frontend and backend service list columns with name and address", async () => {
    const wrapper = mountView();
    await flushPromises();
    await flushPromises();

    expect(wrapper.text()).toContain("alice");
    expect(wrapper.text()).toContain("bob");
    expect(wrapper.text()).toContain("前端服务");
    expect(wrapper.text()).toContain("后端服务");
    expect(wrapper.text()).toContain("portal-web");
    expect(wrapper.text()).toContain("10.0.0.10:80");
    expect(wrapper.text()).toContain("payment-api");
    expect(wrapper.text()).toContain("10.0.0.21:8080");
  });

  it("saves business application and replaces mounted services", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = wrapper
      .findAll("button")
      .find((button) => button.text().trim() === "新增业务应用");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const saveButton = wrapper
      .findAll("button")
      .find((button) => button.text().trim() === "保存");
    expect(saveButton).toBeDefined();
    await saveButton!.trigger("click");
    await flushPromises();

    const savePayload = vi.mocked(saveBusinessApplication).mock.calls[0]?.[0] ?? {};
    expect(Object.prototype.hasOwnProperty.call(savePayload, "owner")).toBe(false);
    expect(listApplications).toHaveBeenCalled();
    expect(saveBusinessApplication).toHaveBeenCalled();
    expect(saveResourceTerms).toHaveBeenCalledWith({
      resource_type: "business_application",
      resource_id: "ba-1",
      field_key: "owner",
      values: [],
    });
    expect(listBusinessApplicationOwnerTerms).toHaveBeenCalled();
    expect(listResourceTerms).not.toHaveBeenCalled();
    expect(replaceServicesByBusinessApplication).toHaveBeenCalledWith("ba-1", []);
    expect(listBusinessApplications).toHaveBeenCalled();
    expect(listServicesByBusinessApplication).toHaveBeenCalledWith("ba-1");
    expect(deleteBusinessApplication).not.toHaveBeenCalled();
    expect(messageSuccess).toHaveBeenCalled();
  });
});

