import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, nextTick } from "vue";
import { mount } from "@vue/test-utils";
import ErrorDetailDialog from "@/components/ErrorDetailDialog.vue";
import { InfraError } from "@/types/error";
import {
  clearInfraError,
  presentInfraError,
  useErrorPresenter,
} from "@/composables/useErrorPresenter";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
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
  emits: ["update:modelValue"],
  template: `
    <section v-if="modelValue" data-testid="el-dialog">
      <slot />
      <slot name="footer" />
    </section>
  `,
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button v-bind="$attrs" @click="$emit('click')"><slot /></button>`,
});

function createInfraError(seed: string) {
  return new InfraError(
    {
      code: "INTERNAL_ERROR",
      message: `消息-${seed}`,
      details: `技术细节-${seed}`,
      command: `cmd-${seed}`,
      retryable: false,
    },
    null,
  );
}

function mountDialog() {
  return mount(ErrorDetailDialog, {
    global: {
      stubs: {
        ElButton: ElButtonStub,
        ElDialog: ElDialogStub,
        ElAlert: PassThroughStub,
        ElDescriptions: PassThroughStub,
        ElDescriptionsItem: PassThroughStub,
        ElCollapse: PassThroughStub,
        ElCollapseItem: PassThroughStub,
      },
    },
  });
}

describe("ErrorDetailDialog", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    clearInfraError();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows collapsed badge with count after delay", async () => {
    const wrapper = mountDialog();
    presentInfraError(createInfraError("a"));
    presentInfraError(createInfraError("b"));

    expect(wrapper.find('[data-testid="error-capsule"]').exists()).toBe(false);

    vi.advanceTimersByTime(4000);
    await nextTick();

    expect(wrapper.get('[data-testid="error-capsule"]').text()).toContain("2 条错误");
  });

  it("opens detail dialog from collapsed badge and renders list", async () => {
    const wrapper = mountDialog();
    presentInfraError(createInfraError("a"));
    presentInfraError(createInfraError("b"));
    vi.advanceTimersByTime(4000);
    await nextTick();

    await wrapper.get('[data-testid="error-capsule"]').trigger("click");

    expect(wrapper.find('[data-testid="el-dialog"]').exists()).toBe(true);
    expect(wrapper.findAll('[data-testid="error-item"]')).toHaveLength(2);
  });

  it("clears all errors from dialog action", async () => {
    const wrapper = mountDialog();
    const presenter = useErrorPresenter() as Record<string, any>;
    presentInfraError(createInfraError("a"));
    vi.advanceTimersByTime(4000);
    await nextTick();
    await wrapper.get('[data-testid="error-capsule"]').trigger("click");

    await wrapper.get('[data-testid="clear-history"]').trigger("click");
    await nextTick();

    expect(presenter.state.history).toHaveLength(0);
    expect(wrapper.find('[data-testid="error-capsule"]').exists()).toBe(false);
  });
});
