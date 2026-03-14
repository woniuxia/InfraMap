import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import SystemsView from "@/views/SystemsView.vue";
import { listSystems, listServicesBySystem, saveSystem, deleteSystem } from "@/api";
import { listContacts } from "@/api/contacts";

const {
  businessEnvOptions,
  businessStatusOptions,
  getEnvLabelMock,
  getSystemStatusLabelMock,
  messageSuccess,
} = vi.hoisted(() => ({
  businessEnvOptions: [
    { label: "生产", value: "prod" },
    { label: "开发", value: "dev" },
    { label: "测试", value: "test" },
  ],
  businessStatusOptions: [
    { label: "激活", value: "active" },
    { label: "停用", value: "inactive" },
  ],
  getEnvLabelMock: vi.fn(
    (env?: string) =>
      (({ prod: "生产", dev: "开发", test: "测试" }) as Record<string, string>)[env ?? ""] ||
      env ||
      "-",
  ),
  getSystemStatusLabelMock: vi.fn(
    (status?: string) =>
      (({ active: "激活", inactive: "停用" }) as Record<string, string>)[status ?? ""] ||
      status ||
      "-",
  ),
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

vi.mock("@/api", () => ({
  listSystems: vi.fn(async () => ({
    data: [
      {
        id: "sys-1",
        name: "支付中心",
        owners: ["alice", "bob"],
        owner_contact_ids: ["contact-alice", "contact-bob"],
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
  saveSystem: vi.fn(async () => "sys-1"),
  deleteSystem: vi.fn(async () => undefined),
  listServicesBySystem: vi.fn(async () => ({
    frontend: [
      {
        id: "svc-fe-1",
        system_id: "sys-1",
        system_name: "支付中心",
        name: "portal-web",
        type: "frontend",
        address: "10.0.0.10",
        port: 80,
        env: "prod",
        status: "running",
        created_at: "",
        updated_at: "",
      },
    ],
    backend: [
      {
        id: "svc-be-1",
        system_id: "sys-1",
        system_name: "支付中心",
        name: "payment-api",
        type: "backend",
        address: "10.0.0.21",
        port: 8080,
        env: "prod",
        status: "running",
        created_at: "",
        updated_at: "",
      },
    ],
  })),
}));

vi.mock("@/api/contacts", () => ({
  listContacts: vi.fn(async () => ({
    data: [
      { id: "contact-alice", name: "alice", created_at: "", updated_at: "" },
      { id: "contact-bob", name: "bob", created_at: "", updated_at: "" },
    ],
    total: 2,
    page: 1,
    page_size: 500,
  })),
  getContact: vi.fn(async (id: string) => ({ id, name: id, created_at: "", updated_at: "" })),
  saveContact: vi.fn(async () => "contact-new"),
}));

vi.mock("@/constants/options", () => ({
  ENV_OPTIONS: businessEnvOptions,
  SYSTEM_STATUS_OPTIONS: businessStatusOptions,
  SYSTEM_STATUS_LABELS: {
    active: "激活",
    inactive: "停用",
  },
  getEnvLabel: getEnvLabelMock,
  getSystemStatusLabel: getSystemStatusLabelMock,
}));

const SearchToolbarStub = defineComponent({
  name: "SearchToolbar",
  props: {
    fields: {
      type: Array,
      default: () => [],
    },
  },
  template: `
    <div>
      <div data-testid="search-toolbar-fields">{{ JSON.stringify(fields) }}</div>
      <slot name="actions" :hasActiveFilters="false" :reset="() => {}" />
    </div>
  `,
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
  template: `<section v-if="modelValue" data-testid="dialog-stub"><slot /><slot name="footer" /></section>`,
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
  props: {
    label: {
      type: String,
      default: "",
    },
  },
  template: `<div class="el-option-stub">{{ label }}<slot /></div>`,
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
      h("div", {}, [
        h("div", { "data-testid": `column-${props.prop || props.label}` }, props.label || ""),
        ...(tableProps?.data || []).map((row, index) =>
          h(
            "div",
            { key: `${props.prop || props.label}-${index}` },
            slots.default ? slots.default({ row, $index: index }) : String(row[props.prop] ?? ""),
          ),
        ),
      ]);
  },
});

function mountView() {
  return mount(SystemsView, {
    global: {
      plugins: [createPinia()],
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

describe("SystemsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders frontend and backend service list columns with name and address", async () => {
    const wrapper = mountView();
    await flushPromises();
    await flushPromises();

    expect(wrapper.text()).toContain("alice");
    expect(wrapper.text()).toContain("bob");
    expect(wrapper.text()).toContain("portal-web");
    expect(wrapper.text()).toContain("10.0.0.10:80");
    expect(wrapper.text()).toContain("payment-api");
    expect(wrapper.text()).toContain("10.0.0.21:8080");
  });

  it("saves system and refreshes list", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = wrapper
      .findAll("button")
      .find((button) => button.text().trim() === "新增系统");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const saveButton = wrapper.findAll("button").find((button) => button.text().trim() === "保存");
    expect(saveButton).toBeDefined();
    await saveButton!.trigger("click");
    await flushPromises();

    const savePayload = (vi.mocked(saveSystem).mock.calls[0]?.[0] as Record<string, unknown>) ?? {};
    expect(Object.prototype.hasOwnProperty.call(savePayload, "owners")).toBe(false);
    expect(savePayload["owner_contact_ids"]).toEqual([]);
    expect(saveSystem).toHaveBeenCalled();
    expect(listContacts).toHaveBeenCalled();
    expect(listSystems).toHaveBeenCalled();
    expect(listServicesBySystem).toHaveBeenCalledWith("sys-1");
    expect(deleteSystem).not.toHaveBeenCalled();
    expect(messageSuccess).toHaveBeenCalled();
  });

  it("uses shared business status options and label helpers", async () => {
    const wrapper = mountView();
    await flushPromises();

    const renderedFields = wrapper.get('[data-testid="search-toolbar-fields"]').text();
    expect(renderedFields).toContain('"label":"激活"');
    expect(renderedFields).toContain('"value":"active"');
    expect(renderedFields).toContain('"label":"停用"');
    expect(renderedFields).toContain('"value":"inactive"');
    expect(getSystemStatusLabelMock).toHaveBeenCalledWith("active");

    const addButton = wrapper
      .findAll("button")
      .find((button) => button.text().trim() === "新增系统");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const dialogText = wrapper.get('[data-testid="dialog-stub"]').text();
    expect(dialogText).toContain("生产");
    expect(dialogText).toContain("开发");
    expect(dialogText).toContain("测试");
    expect(dialogText).toContain("激活");
    expect(dialogText).toContain("停用");
  });
});
