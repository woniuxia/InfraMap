import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { defineComponent } from "vue";
import CopyableTextCell from "@/components/table/CopyableTextCell.vue";

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button data-testid="copy-btn" @click="$emit('click')"><slot /></button>`,
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
    expect(wrapper.get('[data-testid="copy-btn"]').text()).toContain("已复制");

    vi.advanceTimersByTime(1200);
    await wrapper.vm.$nextTick();

    expect(wrapper.get('[data-testid="copy-btn"]').text()).toContain("复制");
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
    expect(wrapper.get('[data-testid="copy-btn"]').text()).toContain("已复制");
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

    expect(wrapper.get('[data-testid="copy-btn"]').text()).toContain("重试");
    vi.advanceTimersByTime(1200);
    await wrapper.vm.$nextTick();
    expect(wrapper.get('[data-testid="copy-btn"]').text()).toContain("复制");
  });
});
