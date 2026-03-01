import { defineComponent, ref } from "vue";
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import type { Host, IpAddress } from "@/types";
import HostEditorDialog from "@/views/hosts/HostEditorDialog.vue";

const ElDialogStub = defineComponent({
  name: "ElDialog",
  props: {
    modelValue: {
      type: Boolean,
      default: false,
    },
  },
  emits: ["update:modelValue"],
  template: `<section v-if="modelValue"><slot /><slot name="footer" /></section>`,
});

const ElInputStub = defineComponent({
  name: "ElInput",
  props: {
    modelValue: {
      type: String,
      default: "",
    },
  },
  emits: ["update:modelValue"],
  template: `
    <input
      data-testid="el-input"
      :value="modelValue"
      @input="$emit('update:modelValue', $event.target.value)"
    />
  `,
});

const ElSelectStub = defineComponent({
  name: "ElSelect",
  props: {
    modelValue: {
      type: [String, Array, Boolean],
      default: "",
    },
  },
  emits: ["update:modelValue", "visible-change"],
  template: `
    <div>
      <input
        data-testid="el-select-input"
        @input="$emit('update:modelValue', $event.target.value)"
      />
      <slot />
    </div>
  `,
});

const ElOptionStub = defineComponent({
  name: "ElOption",
  template: `<option><slot /></option>`,
});

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button v-bind="$attrs" @click="$emit('click')"><slot /></button>`,
});

const ElSwitchStub = defineComponent({
  name: "ElSwitch",
  props: {
    modelValue: {
      type: Boolean,
      default: false,
    },
  },
  emits: ["update:modelValue"],
  template: `
    <input
      data-testid="cross-env-switch"
      type="checkbox"
      :checked="modelValue"
      @change="$emit('update:modelValue', $event.target.checked)"
    />
  `,
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

function createBaseHost(): Partial<Host> {
  return {
    id: "host-1",
    hostname: "web-prod-01",
    env: "prod",
    status: "running",
  };
}

function createBaseQuickIp(): Partial<IpAddress> {
  return {
    ip_address: "10.0.0.21",
    env: "prod",
    is_vip: false,
  };
}

function mountDialog(overrides?: Record<string, unknown>) {
  return mount(HostEditorDialog, {
    props: {
      modelValue: true,
      isEditing: true,
      editingHost: createBaseHost(),
      formRef: ref(),
      formRules: {},
      tagList: [],
      formTagSuggestionOptions: [],
      selectedIpIds: [],
      allowCrossEnv: false,
      bindingLoading: false,
      filteredIpOptions: [],
      searchedIpKeyword: "10.0.0.21",
      canQuickCreateIp: true,
      quickIpDialogVisible: true,
      quickIpSaving: false,
      quickIpFormRef: ref(),
      quickIpForm: createBaseQuickIp(),
      quickIpFormRules: {},
      quickRealIpList: [],
      saveLoading: false,
      formatIpOptionLabel: (ip: Partial<IpAddress>) => ip.ip_address || "",
      envOptions: [
        { label: "生产", value: "prod" },
        { label: "开发", value: "dev" },
        { label: "测试", value: "test" },
      ],
      statusOptions: [
        { label: "运行中", value: "running" },
        { label: "已停止", value: "stopped" },
        { label: "维护中", value: "maintenance" },
      ],
      osOptions: ["Ubuntu 22.04"],
      ...overrides,
    },
    global: {
      stubs: {
        ElDialog: ElDialogStub,
        ElForm: PassThroughStub,
        ElFormItem: PassThroughStub,
        ElDivider: PassThroughStub,
        ElInput: ElInputStub,
        ElSelect: ElSelectStub,
        ElOption: ElOptionStub,
        ElButton: ElButtonStub,
        ElSwitch: ElSwitchStub,
        ElTag: PassThroughStub,
        ElRow: PassThroughStub,
        ElCol: PassThroughStub,
        ElRadioGroup: PassThroughStub,
        ElRadio: PassThroughStub,
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

describe("HostEditorDialog", () => {
  it("emits save when host form is valid", async () => {
    const wrapper = mountDialog();
    await wrapper.get('[data-testid="host-save-btn"]').trigger("click");
    expect(wrapper.emitted("save")?.length ?? 0).toBeGreaterThan(0);
  });

  it("does not emit save when hostname is missing", async () => {
    const wrapper = mountDialog({
      editingHost: {
        id: "host-2",
        hostname: "",
        env: "prod",
        status: "running",
      },
    });
    await wrapper.get('[data-testid="host-save-btn"]').trigger("click");
    expect(wrapper.emitted("save")).toBeFalsy();
  });

  it("emits quick-ip-save when quick form is valid", async () => {
    const wrapper = mountDialog();
    await wrapper.get('[data-testid="quick-ip-save-btn"]').trigger("click");
    expect(wrapper.emitted("save-quick-ip")?.length ?? 0).toBeGreaterThan(0);
  });

  it("does not emit quick-ip-save when quick ip is invalid", async () => {
    const wrapper = mountDialog({
      quickIpForm: {
        ip_address: "invalid-ip",
        env: "prod",
        is_vip: false,
      },
    });
    await wrapper.get('[data-testid="quick-ip-save-btn"]').trigger("click");
    expect(wrapper.emitted("save-quick-ip")).toBeFalsy();
  });
});
