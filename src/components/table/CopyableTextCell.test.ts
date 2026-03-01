import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { defineComponent } from "vue";
import CopyableTextCell from "@/components/table/CopyableTextCell.vue";
import { ElMessage } from "element-plus";

vi.mock("element-plus", () => ({
  ElMessage: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button data-testid="copy-btn" @click="$emit('click', $event)"><slot /></button>`,
});

const ElIconStub = defineComponent({
  name: "ElIcon",
  template: `<i v-bind="$attrs"><slot /></i>`,
});

function mountCell(props?: Partial<InstanceType<typeof CopyableTextCell>["$props"]>) {
  return mount(CopyableTextCell, {
    props: {
      text: "10.0.0.1",
      ...props,
    },
    global: {
      stubs: {
        ElButton: ElButtonStub,
        ElIcon: ElIconStub,
      },
    },
  });
}

describe("CopyableTextCell", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("renders placeholder when text is empty and hides copy button", () => {
    const wrapper = mountCell({ text: "" });

    expect(wrapper.get(".im-copyable-cell__text").text()).toBe("-");
    expect(wrapper.find('[data-testid="copy-btn"]').exists()).toBe(false);
  });

  it("shows copied state after successful clipboard write", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    const wrapper = mountCell();

    await wrapper.get('[data-testid="copy-btn"]').trigger("click");

    expect(writeText).toHaveBeenCalledWith("10.0.0.1");
    expect(ElMessage.success).toHaveBeenCalledWith("复制成功");
    expect(wrapper.get('[data-testid="copy-btn"]').find(".im-copyable-cell__icon").exists()).toBe(true);
    expect(wrapper.get('[data-testid="copy-btn"]').text()).not.toContain("已复制");
  });

  it("falls back to execCommand when clipboard API is unavailable", async () => {
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: undefined,
    });
    const execSpy = vi.fn().mockReturnValue(true);
    (document as Document & { execCommand: (command: string) => boolean }).execCommand = execSpy;

    const wrapper = mountCell();
    await wrapper.get('[data-testid="copy-btn"]').trigger("click");

    expect(execSpy).toHaveBeenCalledWith("copy");
    expect(ElMessage.success).toHaveBeenCalledWith("复制成功");
    expect(wrapper.get('[data-testid="copy-btn"]').find(".im-copyable-cell__icon").exists()).toBe(true);
  });

  it("shows retry state when all copy strategies fail", async () => {
    const writeText = vi.fn().mockRejectedValue(new Error("denied"));
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    const execSpy = vi.fn().mockReturnValue(false);
    (document as Document & { execCommand: (command: string) => boolean }).execCommand = execSpy;

    const wrapper = mountCell();
    await wrapper.get('[data-testid="copy-btn"]').trigger("click");

    expect(ElMessage.error).toHaveBeenCalledWith("复制失败，请重试");
    expect(wrapper.get('[data-testid="copy-btn"]').find(".im-copyable-cell__icon").exists()).toBe(true);
    expect(wrapper.get('[data-testid="copy-btn"]').text()).not.toContain("重试");
  });

  it("blurs button after pointer click so focus does not keep it visible", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    const wrapper = mount(CopyableTextCell, {
      attachTo: document.body,
      props: {
        text: "10.0.0.1",
      },
      global: {
        stubs: {
          ElButton: ElButtonStub,
          ElIcon: ElIconStub,
        },
      },
    });
    const button = wrapper.get('[data-testid="copy-btn"]');
    const blurSpy = vi.spyOn(button.element as HTMLButtonElement, "blur");
    vi.spyOn(document, "activeElement", "get").mockReturnValue(button.element);

    await button.trigger("click");
    await wrapper.vm.$nextTick();

    expect(blurSpy).toHaveBeenCalled();
    wrapper.unmount();
  });

  it("does not render hidden width sizer placeholder", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    const wrapper = mountCell();
    const button = wrapper.get('[data-testid="copy-btn"]');

    expect(button.find(".im-copyable-cell__btn-sizer").exists()).toBe(false);

    await button.trigger("click");
    await wrapper.vm.$nextTick();

    expect(button.find(".im-copyable-cell__btn-sizer").exists()).toBe(false);
    expect(ElMessage.success).toHaveBeenCalledWith("复制成功");
    expect(button.find(".im-copyable-cell__icon").exists()).toBe(true);
  });

  it("uses icon instead of plain text in idle state", () => {
    const wrapper = mountCell();
    expect(wrapper.get('[data-testid="copy-btn"]').find(".im-copyable-cell__icon").exists()).toBe(true);
    expect(wrapper.get('[data-testid="copy-btn"]').text()).not.toContain("复制");
  });
});
