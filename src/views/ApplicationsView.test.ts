import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import ApplicationsView from "@/views/ApplicationsView.vue";
import { saveApplication } from "@/api/applications";
import { listBusinessApplications } from "@/api/business-applications";
import { saveDeployment } from "@/api/deployments";

const { messageSuccess, messageWarning } = vi.hoisted(() => ({
  messageSuccess: vi.fn(),
  messageWarning: vi.fn(),
}));

vi.mock("element-plus", () => ({
  ElMessage: {
    success: messageSuccess,
    warning: messageWarning,
    info: vi.fn(),
    error: vi.fn(),
  },
}));

vi.mock("@/api/applications", () => ({
  listApplications: vi.fn(async () => ({
    data: [],
    total: 0,
    page: 1,
    page_size: 20,
  })),
  saveApplication: vi.fn(async () => "app-1"),
  deleteApplication: vi.fn(async () => undefined),
}));

vi.mock("@/api/business-applications", () => ({
  listBusinessApplications: vi.fn(async () => ({
    data: [
      {
        id: "ba-prod",
        name: "支付中心",
        env: "prod",
        status: "active",
        created_at: "",
        updated_at: "",
      },
      {
        id: "ba-dev",
        name: "研发平台",
        env: "dev",
        status: "active",
        created_at: "",
        updated_at: "",
      },
      {
        id: "ba-disabled",
        name: "旧业务",
        env: "prod",
        status: "inactive",
        created_at: "",
        updated_at: "",
      },
    ],
    total: 3,
    page: 1,
    page_size: 500,
  })),
}));

vi.mock("@/api/taxonomy", () => ({
  listApplicationOwnerTerms: vi.fn(async () => []),
  listApplicationTechStackTerms: vi.fn(async () => []),
}));

vi.mock("@/api/call-relations", () => ({
  replaceResourceCallRelations: vi.fn(async () => ({
    created_count: 0,
    deleted_count: 0,
    deduplicated_count: 0,
  })),
}));

vi.mock("@/api/deployments", () => ({
  saveDeployment: vi.fn(async () => undefined),
}));

const tableDataKey = Symbol("tableDataKey");

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
      type: [String, Array, Number],
      default: "",
    },
  },
  emits: ["update:modelValue", "change", "clear"],
  template: `<div class="el-select-stub"><slot /></div>`,
});

const ElOptionStub = defineComponent({
  name: "ElOption",
  props: {
    label: {
      type: String,
      default: "",
    },
    value: {
      type: [String, Number],
      default: "",
    },
  },
  template: `<div class="el-option-stub" :data-value="value">{{ label }}</div>`,
});

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
    return () => h("div", {}, slots.default ? slots.default() : []);
  },
});

const ElTableColumnStub = defineComponent({
  name: "ElTableColumn",
  props: {
    prop: {
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
        (tableProps?.data || []).map((row, index) =>
          h(
            "div",
            { key: `${props.prop}-${index}` },
            slots.default ? slots.default({ row, $index: index }) : String(row[props.prop] ?? ""),
          ),
        ),
      );
  },
});

const ElFormItemStub = defineComponent({
  name: "ElFormItem",
  props: {
    label: {
      type: String,
      default: "",
    },
  },
  template: `<div class="el-form-item-stub" :data-label="label"><slot /></div>`,
});

const CallRelationsEditorStub = defineComponent({
  name: "CallRelationsEditor",
  setup(_props, { expose }) {
    expose({
      getDraftItems: () => [],
    });
    return () => h("div");
  },
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

const DeploymentPanelStub = defineComponent({
  name: "DeploymentPanel",
  setup(_props, { expose }) {
    expose({
      getDraftDeployments: () => [],
    });
    return () => h("div", { "data-testid": "deployment-panel-stub" });
  },
});

function mountView() {
  return mount(ApplicationsView, {
    global: {
      stubs: {
        SearchToolbar: SearchToolbarStub,
        ElButton: ElButtonStub,
        ElDialog: ElDialogStub,
        ElInput: ElInputStub,
        ElInputNumber: PassThroughStub,
        ElSelect: ElSelectStub,
        ElOption: ElOptionStub,
        ElForm: PassThroughStub,
        ElFormItem: ElFormItemStub,
        ElTable: ElTableStub,
        ElTableColumn: ElTableColumnStub,
        ElTag: PassThroughStub,
        ElPagination: PassThroughStub,
        ElDivider: PassThroughStub,
        CallRelationsEditor: CallRelationsEditorStub,
        DeploymentPanel: DeploymentPanelStub,
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

function findButtonByText(wrapper: ReturnType<typeof mount>, text: string) {
  return wrapper.findAll("button").find((item) => item.text().trim() === text);
}

function findSelectInFormItem(wrapper: ReturnType<typeof mount>, label: string) {
  return wrapper.find(`[data-label="${label}"]`).findComponent(ElSelectStub);
}

describe("ApplicationsView business application relation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads active business application options when opening add dialog", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增应用");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    expect(listBusinessApplications).toHaveBeenCalledWith({
      page: 1,
      page_size: 500,
      filters: { status: "active" },
    });
  });

  it("shows only active options that match current app env", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增应用");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const optionsText = wrapper
      .find(`[data-label="所属业务应用"]`)
      .findAll(".el-option-stub")
      .map((item) => item.text());
    expect(optionsText).toContain("支付中心（生产）");
    expect(optionsText).not.toContain("研发平台（开发）");
    expect(optionsText).not.toContain("旧业务（生产）");
  });

  it("blocks save when selected business application env does not match app env", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增应用");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const businessSelect = findSelectInFormItem(wrapper, "所属业务应用");
    businessSelect.vm.$emit("update:modelValue", "ba-prod");
    businessSelect.vm.$emit("change", "ba-prod");
    await flushPromises();

    const envSelect = findSelectInFormItem(wrapper, "环境");
    envSelect.vm.$emit("update:modelValue", "dev");
    envSelect.vm.$emit("change", "dev");
    await flushPromises();

    const saveButton = findButtonByText(wrapper, "保存");
    expect(saveButton).toBeDefined();
    await saveButton!.trigger("click");
    await flushPromises();

    expect(messageWarning).toHaveBeenCalledWith("所属业务应用与当前环境不一致，请重新选择。");
    expect(saveApplication).not.toHaveBeenCalled();
  });

  it("allows clearing selected business application and saves unassigned relation", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增应用");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const businessSelect = findSelectInFormItem(wrapper, "所属业务应用");
    businessSelect.vm.$emit("update:modelValue", "ba-prod");
    businessSelect.vm.$emit("change", "ba-prod");
    await flushPromises();
    businessSelect.vm.$emit("update:modelValue", "");
    businessSelect.vm.$emit("change", "");
    await flushPromises();

    const saveButton = findButtonByText(wrapper, "保存");
    expect(saveButton).toBeDefined();
    await saveButton!.trigger("click");
    await flushPromises();

    expect(saveApplication).toHaveBeenCalled();
    const payload = vi.mocked(saveApplication).mock.calls[0]?.[0];
    expect(payload?.business_application_id).toBeUndefined();
    expect(messageSuccess).toHaveBeenCalledWith("创建成功");
    expect(saveDeployment).not.toHaveBeenCalled();
  });

  it("shows deployment panel before first save when creating application", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增应用");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    expect(wrapper.find('[data-testid="deployment-panel-stub"]').exists()).toBe(true);
  });

  it("closes dialog after creating a new application", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增应用");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const saveButton = findButtonByText(wrapper, "保存");
    expect(saveButton).toBeDefined();
    await saveButton!.trigger("click");
    await flushPromises();

    expect(messageSuccess).toHaveBeenCalledWith("创建成功");
    expect(wrapper.find('[data-testid="deployment-panel-stub"]').exists()).toBe(false);
  });
});
