import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import MiddlewaresView from "@/views/MiddlewaresView.vue";
import { saveMiddleware } from "@/api/middlewares";
import { saveDeployment } from "@/api/deployments";

const draftDeploymentsState = vi.hoisted(() => ({
  value: [] as Array<{ host_id: string; port?: number }>,
}));
const deploymentPanelPropsState = vi.hoisted(() => ({
  resourceAddress: undefined as string | undefined,
  resourceEnv: undefined as string | undefined,
}));

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

vi.mock("@/api/middlewares", () => ({
  listMiddlewares: vi.fn(async () => ({
    data: [],
    total: 0,
    page: 1,
    page_size: 20,
  })),
  saveMiddleware: vi.fn(async () => "mw-new"),
  deleteMiddleware: vi.fn(async () => undefined),
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
            slots.default ? slots.default({ row, $index: index }) : String(row[props.prop] ?? "")
          )
        )
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

const DeploymentPanelStub = defineComponent({
  name: "DeploymentPanel",
  props: {
    resourceAddress: {
      type: String,
      required: false,
    },
    resourceEnv: {
      type: String,
      required: false,
    },
  },
  setup(_props, { expose }) {
    const props = _props as { resourceAddress?: string; resourceEnv?: string };
    expose({
      getDraftDeployments: () => draftDeploymentsState.value,
    });
    return () => {
      deploymentPanelPropsState.resourceAddress = props.resourceAddress;
      deploymentPanelPropsState.resourceEnv = props.resourceEnv;
      return h("div", { "data-testid": "deployment-panel-stub" });
    };
  },
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

function mountView() {
  return mount(MiddlewaresView, {
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

describe("MiddlewaresView deployment relation on create", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    draftDeploymentsState.value = [];
    deploymentPanelPropsState.resourceAddress = undefined;
    deploymentPanelPropsState.resourceEnv = undefined;
  });

  it("shows deployment panel in add mode before first save", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增中间件");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    expect(wrapper.find('[data-testid="deployment-panel-stub"]').exists()).toBe(true);
  });

  it("saves draft deployments after creating middleware", async () => {
    draftDeploymentsState.value = [{ host_id: "host-1", port: 6379 }];
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增中间件");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const saveButton = findButtonByText(wrapper, "保存");
    expect(saveButton).toBeDefined();
    await saveButton!.trigger("click");
    await flushPromises();

    expect(saveMiddleware).toHaveBeenCalledTimes(1);
    expect(saveDeployment).toHaveBeenCalledWith({
      id: "",
      resource_id: "mw-new",
      resource_type: "middleware",
      host_id: "host-1",
      port: 6379,
    });
  });

  it("does not persist draft deployments when editing existing middleware", async () => {
    vi.mocked(saveMiddleware).mockResolvedValue("mw-1");
    const { listMiddlewares } = await import("@/api/middlewares");
    vi.mocked(listMiddlewares).mockResolvedValueOnce({
      data: [
        {
          id: "mw-1",
          name: "redis-main",
          category: "cache",
          type: "Redis",
          address: "10.0.0.8",
          port: 6379,
          version: "7.0",
          env: "prod",
          description: "",
          created_at: "",
          updated_at: "",
        },
      ],
      total: 1,
      page: 1,
      page_size: 20,
    });
    draftDeploymentsState.value = [{ host_id: "host-1", port: 6379 }];

    const wrapper = mountView();
    await flushPromises();

    const editButton = findButtonByText(wrapper, "编辑");
    expect(editButton).toBeDefined();
    await editButton!.trigger("click");
    await flushPromises();

    const saveButton = findButtonByText(wrapper, "保存");
    expect(saveButton).toBeDefined();
    await saveButton!.trigger("click");
    await flushPromises();

    expect(saveMiddleware).toHaveBeenCalledTimes(1);
    expect(saveDeployment).not.toHaveBeenCalled();
  });

  it("shows warning when deployment persistence fails after middleware save", async () => {
    draftDeploymentsState.value = [{ host_id: "host-1", port: 6379 }];
    vi.mocked(saveDeployment).mockRejectedValueOnce(new Error("deploy failed"));

    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增中间件");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const saveButton = findButtonByText(wrapper, "保存");
    expect(saveButton).toBeDefined();
    await saveButton!.trigger("click");
    await flushPromises();

    expect(saveMiddleware).toHaveBeenCalledTimes(1);
    expect(messageWarning).toHaveBeenCalledWith("中间件已保存，部署关系保存失败，请在部署关系中重试。");
    expect(messageSuccess).toHaveBeenCalled();
  });

  it("shows deployment panel in copy mode before first save", async () => {
    const { listMiddlewares } = await import("@/api/middlewares");
    vi.mocked(listMiddlewares).mockResolvedValueOnce({
      data: [
        {
          id: "mw-1",
          name: "redis-main",
          category: "cache",
          type: "Redis",
          address: "10.0.0.8",
          port: 6379,
          version: "7.0",
          env: "prod",
          description: "",
          created_at: "",
          updated_at: "",
        },
      ],
      total: 1,
      page: 1,
      page_size: 20,
    });

    const wrapper = mountView();
    await flushPromises();

    const copyButton = findButtonByText(wrapper, "复制");
    expect(copyButton).toBeDefined();
    await copyButton!.trigger("click");
    await flushPromises();

    expect(wrapper.find('[data-testid="deployment-panel-stub"]').exists()).toBe(true);
  });

  it("passes current middleware address/env to deployment panel in edit mode", async () => {
    const { listMiddlewares } = await import("@/api/middlewares");
    vi.mocked(listMiddlewares).mockResolvedValueOnce({
      data: [
        {
          id: "mw-1",
          name: "redis-main",
          category: "cache",
          type: "Redis",
          address: "redis://10.0.0.8:6379",
          port: 6379,
          version: "7.0",
          env: "prod",
          description: "",
          created_at: "",
          updated_at: "",
        },
      ],
      total: 1,
      page: 1,
      page_size: 20,
    });

    const wrapper = mountView();
    await flushPromises();

    const editButton = findButtonByText(wrapper, "编辑");
    expect(editButton).toBeDefined();
    await editButton!.trigger("click");
    await flushPromises();

    const addressInput = wrapper.find('[data-label="连接地址"] input');
    expect(addressInput.exists()).toBe(true);
    await addressInput.setValue("redis://10.0.0.9:6379");
    await flushPromises();

    const envSelect = wrapper.find('[data-label="环境"]').findComponent(ElSelectStub);
    envSelect.vm.$emit("update:modelValue", "dev");
    envSelect.vm.$emit("change", "dev");
    await flushPromises();

    expect(deploymentPanelPropsState.resourceAddress).toBe("redis://10.0.0.9:6379");
    expect(deploymentPanelPropsState.resourceEnv).toBe("dev");
  });
});
