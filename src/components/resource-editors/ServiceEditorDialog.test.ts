import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import ServiceEditorDialog from "@/components/resource-editors/ServiceEditorDialog.vue";
import { saveService } from "@/api/services";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

vi.mock("@/api/services", () => ({
  saveService: vi.fn(async () => "svc-1"),
}));

vi.mock("@/api/systems", () => ({
  listSystems: vi.fn(async () => ({
    data: [],
    total: 0,
    page: 1,
    page_size: 500,
  })),
}));

vi.mock("@/api/contacts", () => ({
  listContacts: vi.fn(async () => ({
    data: [],
    total: 0,
    page: 1,
    page_size: 50,
  })),
  getContact: vi.fn(async (id: string) => ({
    id,
    name: id,
    created_at: "",
    updated_at: "",
  })),
  saveContact: vi.fn(async () => "contact-1"),
}));

vi.mock("@/api/taxonomy", () => ({
  listServiceTechStackTerms: vi.fn(async () => []),
}));

vi.mock("@/api/call-relations", () => ({
  replaceResourceCallRelations: vi.fn(async () => undefined),
}));

vi.mock("@/api/deployments", () => ({
  saveDeployment: vi.fn(async () => "deployment-1"),
}));

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

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button type="button" @click="$emit('click')"><slot /></button>`,
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

const ElInputNumberStub = defineComponent({
  name: "ElInputNumber",
  props: {
    modelValue: {
      type: Number,
      default: undefined,
    },
  },
  emits: ["update:modelValue"],
  template: `<input type="number" :value="modelValue" @input="$emit('update:modelValue', Number($event.target.value))" />`,
});

const ElSelectStub = defineComponent({
  name: "ElSelect",
  props: {
    modelValue: {
      type: [String, Number, Array, Object],
      default: undefined,
    },
  },
  emits: ["update:modelValue", "change", "clear"],
  template: `<div><slot /></div>`,
});

const ElOptionStub = defineComponent({
  name: "ElOption",
  template: `<div><slot /></div>`,
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

const CallRelationsEditorStub = defineComponent({
  name: "CallRelationsEditor",
  setup(_, { expose }) {
    expose({
      getDraftItems: () => [],
    });
    return () => h("div");
  },
});

const DeploymentPanelStub = defineComponent({
  name: "DeploymentPanel",
  setup(_, { expose }) {
    expose({
      getDraftDeployments: () => [],
    });
    return () => h("div");
  },
});

function mountDialog(initialDraft: Record<string, unknown>) {
  return mount(ServiceEditorDialog, {
    props: {
      modelValue: true,
      mode: "create",
      initialDraft,
    },
    global: {
      stubs: {
        ElDialog: ElDialogStub,
        ElForm: PassThroughStub,
        ElFormItem: PassThroughStub,
        ElDivider: PassThroughStub,
        ElInput: ElInputStub,
        ElInputNumber: ElInputNumberStub,
        ElSelect: ElSelectStub,
        ElOption: ElOptionStub,
        ElButton: ElButtonStub,
        CallRelationsEditor: CallRelationsEditorStub,
        DeploymentPanel: DeploymentPanelStub,
      },
    },
  });
}

describe("ServiceEditorDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("initializes empty owner_contact_ids when hydrating and saving", async () => {
    const wrapper = mountDialog({
      id: "draft-empty-owners",
      system_id: "sys-1",
      name: "empty-owners-app",
      type: "backend",
      env: "prod",
      status: "running",
      created_at: "",
      updated_at: "",
    });
    await flushPromises();

    const buttons = wrapper.findAll("button");
    await buttons[buttons.length - 1].trigger("click");
    await flushPromises();

    expect(vi.mocked(saveService)).toHaveBeenCalledTimes(1);
    const payload = vi.mocked(saveService).mock.calls[0][0] as Record<string, unknown>;
    expect(payload.owner_contact_ids).toEqual([]);
  });
});
