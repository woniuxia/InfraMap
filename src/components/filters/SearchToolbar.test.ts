import { defineComponent, h } from "vue";
import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import type { SearchFieldConfig } from "@/types/searchToolbar";

const ElInputStub = defineComponent({
  name: "ElInput",
  props: {
    modelValue: {
      type: String,
      default: "",
    },
  },
  emits: ["update:modelValue", "keyup", "clear"],
  template: `
    <div>
      <input
        data-testid="keyword-input"
        :value="modelValue"
        @input="$emit('update:modelValue', $event.target.value)"
        @keyup="$emit('keyup', $event)"
      />
      <button data-testid="clear-input" @click="$emit('clear')">clear</button>
    </div>
  `,
});

const ElSelectStub = defineComponent({
  name: "ElSelect",
  props: {
    modelValue: {
      type: [String, Array],
      default: "",
    },
  },
  emits: ["update:modelValue", "change"],
  template: `<div data-testid="select"><slot /></div>`,
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

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button data-testid="el-button" @click="$emit('click')"><slot /></button>`,
});

const ElTagStub = defineComponent({
  name: "ElTag",
  emits: ["close"],
  template: `<span data-testid="el-tag"><slot /></span>`,
});

const ElCollapseTransitionStub = defineComponent({
  name: "ElCollapseTransition",
  template: `<div><slot /></div>`,
});

const statusField: SearchFieldConfig = {
  key: "status",
  label: "状态",
  queryKey: "status",
  type: "multi-select",
  options: [
    { label: "运行中", value: "running" },
    { label: "已停止", value: "stopped" },
    { label: "维护中", value: "maintenance" },
  ],
};

function mountToolbar(props?: Partial<InstanceType<typeof SearchToolbar>["$props"]>) {
  return mount(SearchToolbar, {
    props: {
      searchText: "",
      filters: {},
      fields: [],
      ...props,
    },
    slots: {
      actions: ({ reset }: { reset: () => void }) =>
        h("button", { "data-testid": "reset-btn", onClick: reset }, "reset"),
    },
    global: {
      stubs: {
        ElInput: ElInputStub,
        ElSelect: ElSelectStub,
        ElOption: ElOptionStub,
        ElButton: ElButtonStub,
        ElTag: ElTagStub,
        ElCollapseTransition: ElCollapseTransitionStub,
      },
    },
  });
}

describe("SearchToolbar", () => {
  it("debounces keyword query and enter triggers immediately", async () => {
    vi.useFakeTimers();
    const wrapper = mountToolbar();
    const input = wrapper.get('[data-testid="keyword-input"]');

    await input.setValue("api");
    expect(wrapper.emitted("query")).toBeFalsy();

    vi.advanceTimersByTime(350);
    expect(wrapper.emitted("query")).toBeFalsy();

    vi.advanceTimersByTime(50);
    expect(wrapper.emitted("query")?.[0]?.[0]).toEqual({
      search: "api",
      filters: {},
    });

    await input.setValue("gateway");
    await input.trigger("keyup", { key: "Enter" });

    expect(wrapper.emitted("query")?.[1]?.[0]).toEqual({
      search: "gateway",
      filters: {},
    });

    vi.advanceTimersByTime(500);
    expect(wrapper.emitted("query")?.length).toBe(2);
    vi.useRealTimers();
  });

  it("reset should emit cleared model and empty query", async () => {
    const wrapper = mountToolbar({
      searchText: "prod",
      fields: [statusField],
      filters: {
        status: ["running", "stopped"],
      },
    });

    await wrapper.get('[data-testid="reset-btn"]').trigger("click");

    const updateSearchTextEvents = wrapper.emitted("update:searchText") ?? [];
    const updateFiltersEvents = wrapper.emitted("update:filters") ?? [];
    const queryEvents = wrapper.emitted("query") ?? [];

    expect(updateSearchTextEvents[updateSearchTextEvents.length - 1]?.[0]).toBe("");
    expect(updateFiltersEvents[updateFiltersEvents.length - 1]?.[0]).toEqual({
      status: [],
    });
    expect(queryEvents[queryEvents.length - 1]?.[0]).toEqual({
      search: "",
      filters: {},
    });
  });
});
