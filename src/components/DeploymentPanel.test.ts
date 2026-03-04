import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent } from "vue";
import { mount } from "@vue/test-utils";
import DeploymentPanel from "@/components/DeploymentPanel.vue";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";

const { messageSuccess, messageWarning } = vi.hoisted(() => ({
  messageSuccess: vi.fn(),
  messageWarning: vi.fn(),
}));

vi.mock("element-plus", () => ({
  ElMessage: {
    success: messageSuccess,
    warning: messageWarning,
  },
  ElMessageBox: {
    confirm: vi.fn().mockResolvedValue(true),
  },
}));

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

const ElSelectStub = defineComponent({
  name: "ElSelect",
  props: {
    modelValue: {
      type: String,
      default: "",
    },
    disabled: {
      type: Boolean,
      default: false,
    },
  },
  emits: ["update:modelValue"],
  template: `
    <select
      :disabled="disabled"
      :value="modelValue"
      @change="$emit('update:modelValue', $event.target.value)"
    >
      <slot />
    </select>
  `,
});

const ElOptionStub = defineComponent({
  name: "ElOption",
  props: {
    label: {
      type: String,
      default: "",
    },
    value: {
      type: String,
      default: "",
    },
  },
  template: `<option :value="value">{{ label }}</option>`,
});

const ElInputNumberStub = defineComponent({
  name: "ElInputNumber",
  props: {
    modelValue: {
      type: Number,
      required: false,
    },
  },
  emits: ["update:modelValue"],
  template: `<input type="number" :value="modelValue" @input="$emit('update:modelValue', Number($event.target.value))" />`,
});

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button v-bind="$attrs" @click="$emit('click')"><slot /></button>`,
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

const ElTableStub = defineComponent({
  template: `<div><slot /></div>`,
});

const ElTableColumnStub = defineComponent({
  template: `<div></div>`,
});

function flushPromises() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function mountPanel(
  props: {
    resourceId?: string;
    resourceType?: "application" | "middleware" | "nginx";
    resourcePersisted?: boolean;
    resourceAddress?: string;
    resourceEnv?: "prod" | "dev" | "test";
  } = {}
) {
  return mount(DeploymentPanel, {
    props: {
      resourceId: props.resourceId ?? "mw-1",
      resourceType: props.resourceType ?? "middleware",
      resourcePersisted: props.resourcePersisted,
      resourceAddress: props.resourceAddress,
      resourceEnv: props.resourceEnv,
    },
    global: {
      stubs: {
        ElButton: ElButtonStub,
        ElDialog: ElDialogStub,
        ElForm: PassThroughStub,
        ElFormItem: PassThroughStub,
        ElSelect: ElSelectStub,
        ElOption: ElOptionStub,
        ElInputNumber: ElInputNumberStub,
        ElTable: ElTableStub,
        ElTableColumn: ElTableColumnStub,
        ElEmpty: PassThroughStub,
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

describe("DeploymentPanel", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("offers quick-create host when parsed ip has no match", async () => {
    let hostListCount = 0;

    __setMockHandler("list_deployments", () => ({ data: [], total: 0, page: 1, page_size: 100 }));
    __setMockHandler("list_hosts", () => {
      hostListCount += 1;
      if (hostListCount >= 3) {
        return {
          data: [
            {
              id: "h-created",
              hostname: "temp-host-20260228090507",
              ip_display: "10.0.0.8",
              env: "dev",
              status: "running",
              created_at: "2026-02-28T09:05:07Z",
              updated_at: "2026-02-28T09:05:07Z",
            },
          ],
          total: 1,
          page: 1,
          page_size: 999,
        };
      }
      return { data: [], total: 0, page: 1, page_size: 999 };
    });
    __setMockHandler("get_resource_deploy_context", () => ({
      resource_type: "middleware",
      resource_id: "mw-1",
      address: "redis://10.0.0.8:6379",
      resource_env: "dev",
      parsed_ip: "10.0.0.8",
      matched_host_id: null,
      matched_host_name: null,
    }));
    __setMockHandler("save_host", (_cmd, args) => {
      const payload = (args?.data ?? {}) as Record<string, unknown>;
      expect(payload.env).toBe("dev");
      expect(payload.status).toBe("running");
      expect(String(payload.hostname)).toContain("temp-host-");
      return undefined;
    });
    __setMockHandler("save_ip_address", (_cmd, args) => {
      const payload = (args?.data ?? {}) as Record<string, unknown>;
      expect(payload.ip_address).toBe("10.0.0.8");
      expect(payload.env).toBe("dev");
      return undefined;
    });
    __setMockHandler("list_ip_addresses", () => ({
      data: [
        {
          id: "ip-8",
          ip_address: "10.0.0.8",
          env: "dev",
          is_vip: false,
          created_at: "2026-02-28T09:05:07Z",
          updated_at: "2026-02-28T09:05:07Z",
        },
      ],
      total: 1,
      page: 1,
      page_size: 50,
    }));
    __setMockHandler("bind_host_ip", (_cmd, args) => {
      expect(args).toEqual({
        hostId: expect.any(String),
        ipId: "ip-8",
      });
      return undefined;
    });

    const wrapper = mountPanel();
    await flushPromises();

    const addBtn = wrapper.findAll("button").find((btn) => btn.text() === "添加");
    expect(addBtn).toBeDefined();
    await addBtn!.trigger("click");
    await flushPromises();

    const quickCreateBtn = wrapper.findAll("button").find((btn) => btn.text().includes("快捷新建服务器"));
    expect(quickCreateBtn).toBeDefined();
    await quickCreateBtn!.trigger("click");
    await flushPromises();

    expect(messageSuccess).toHaveBeenCalled();
  });

  it("stores deployment rows locally in draft mode before resource is persisted", async () => {
    __setMockHandler("list_hosts", () => ({
      data: [
        {
          id: "h-1",
          hostname: "host-1",
          ip_display: "10.0.0.11",
          env: "prod",
          status: "running",
          created_at: "",
          updated_at: "",
        },
      ],
      total: 1,
      page: 1,
      page_size: 999,
    }));

    const wrapper = mountPanel({
      resourceId: "app-draft-1",
      resourceType: "application",
      resourcePersisted: false,
    });
    await flushPromises();

    const addBtn = wrapper.findAll("button").find((btn) => btn.text() === "添加");
    expect(addBtn).toBeDefined();
    await addBtn!.trigger("click");
    await flushPromises();

    const hostSelect = wrapper.find("select");
    expect(hostSelect.exists()).toBe(true);
    await hostSelect.setValue("h-1");
    await flushPromises();

    const confirmBtn = wrapper.findAll("button").find((btn) => btn.text() === "确定");
    expect(confirmBtn).toBeDefined();
    await confirmBtn!.trigger("click");
    await flushPromises();

    const exposed = wrapper.vm as unknown as {
      getDraftDeployments: () => Array<{ host_id: string; port?: number }>;
    };
    expect(exposed.getDraftDeployments()).toEqual([
      {
        host_id: "h-1",
        port: undefined,
      },
    ]);
    expect(messageSuccess).toHaveBeenCalledWith("部署关系已暂存，保存应用后生效");
  });

  it("passes live address/env overrides when loading deploy context", async () => {
    __setMockHandler("list_deployments", () => ({ data: [], total: 0, page: 1, page_size: 100 }));
    __setMockHandler("list_hosts", () => ({ data: [], total: 0, page: 1, page_size: 999 }));
    const contextHandler = vi.fn((_cmd, args) => {
      expect(args).toEqual({
        resourceType: "middleware",
        resourceId: "mw-1",
        addressOverride: "redis://10.0.0.9:6379",
        resourceEnvOverride: "dev",
      });
      return {
        resource_type: "middleware",
        resource_id: "mw-1",
        address: "redis://10.0.0.9:6379",
        resource_env: "dev",
        parsed_ip: "10.0.0.9",
        matched_host_id: null,
        matched_host_name: null,
      };
    });
    __setMockHandler("get_resource_deploy_context", contextHandler);

    const wrapper = mountPanel({
      resourceId: "mw-1",
      resourceType: "middleware",
      resourceAddress: "redis://10.0.0.9:6379",
      resourceEnv: "dev",
    });
    await flushPromises();

    const addBtn = wrapper.findAll("button").find((btn) => btn.text() === "添加");
    expect(addBtn).toBeDefined();
    await addBtn!.trigger("click");
    await flushPromises();
    expect(contextHandler).toHaveBeenCalledTimes(1);
  });
});
