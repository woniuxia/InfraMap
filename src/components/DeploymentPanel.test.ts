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

function mountPanel() {
  return mount(DeploymentPanel, {
    props: {
      resourceId: "mw-1",
      resourceType: "middleware",
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
              ip_address: "10.0.0.8",
              env: "dev",
              status: "running",
              is_deleted: 0,
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
      expect(payload.ip_address).toBe("10.0.0.8");
      expect(payload.env).toBe("dev");
      expect(payload.status).toBe("running");
      expect(String(payload.hostname)).toContain("temp-host-");
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
});
