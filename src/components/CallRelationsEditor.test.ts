import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, inject, provide, ref, toRef, type Ref } from "vue";
import { mount } from "@vue/test-utils";
import CallRelationsEditor from "@/components/CallRelationsEditor.vue";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";

vi.mock("element-plus", () => ({
  ElMessage: {
    warning: vi.fn(),
  },
}));

const tableRowsKey = Symbol("tableRows");

const ElTableStub = defineComponent({
  name: "ElTable",
  props: {
    data: {
      type: Array,
      default: () => [],
    },
  },
  setup(props, { slots }) {
    provide(tableRowsKey, toRef(props, "data"));
    return () => h("div", { class: "el-table-stub" }, slots.default?.());
  },
});

const ElTableColumnStub = defineComponent({
  name: "ElTableColumn",
  setup(_props, { attrs, slots }) {
    const rows = inject<Ref<Array<Record<string, unknown>>>>(tableRowsKey, ref([]));
    return () =>
      h(
        "div",
        { class: "el-table-column-stub", "data-label": String(attrs.label ?? "") },
        rows.value.map((row, index) =>
          h(
            "div",
            { class: "el-table-cell-stub", "data-index": index },
            slots.default?.({ row, $index: index }),
          ),
        ),
      );
  },
});

const ElSelectStub = defineComponent({
  name: "ElSelect",
  props: {
    modelValue: {
      type: [String, Number],
      default: "",
    },
  },
  emits: ["update:modelValue", "change"],
  setup(props, { slots }) {
    return () =>
      h(
        "div",
        { class: "el-select-stub", "data-model": String(props.modelValue ?? "") },
        slots.default?.(),
      );
  },
});

const ElOptionGroupStub = defineComponent({
  name: "ElOptionGroup",
  props: {
    label: {
      type: String,
      default: "",
    },
  },
  setup(props, { slots }) {
    return () =>
      h("section", { class: "el-option-group-stub" }, [
        h("h4", { class: "el-option-group-title" }, props.label),
        slots.default?.(),
      ]);
  },
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
  setup(props) {
    return () =>
      h("div", { class: "el-option-stub", "data-value": String(props.value) }, props.label);
  },
});

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button @click="$emit('click')"><slot /></button>`,
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
  template: `<input :value="modelValue" @input="$emit('update:modelValue', $event.target.value)" />`,
});

function flushPromises() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function mountEditor() {
  return mount(CallRelationsEditor, {
    props: {
      resourceId: "app-current",
      resourceType: "service",
    },
    global: {
      stubs: {
        ElTable: ElTableStub,
        ElTableColumn: ElTableColumnStub,
        ElSelect: ElSelectStub,
        ElOptionGroup: ElOptionGroupStub,
        ElOption: ElOptionStub,
        ElButton: ElButtonStub,
        ElInput: ElInputStub,
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

describe("CallRelationsEditor", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("renders target resources with type labels and excludes current resource", async () => {
    __setMockHandler("list_services", (_cmd, args) => {
      expect(args).toEqual({
        params: { page: 1, page_size: 999 },
      });
      return {
        data: [
          { id: "app-current", name: "billing-ui", type: "frontend" },
          { id: "app-b", name: "order-api", type: "backend" },
          { id: "app-gw", name: "edge-gw", type: "gateway" },
        ],
        total: 3,
        page: 1,
        page_size: 999,
      };
    });
    __setMockHandler("list_middlewares", () => ({
      data: [{ id: "mw-1", name: "redis-main", type: "Redis" }],
      total: 1,
      page: 1,
      page_size: 999,
    }));
    __setMockHandler("list_nginx_configs", () => ({
      data: [{ id: "ng-1", name: "traffic-lb" }],
      total: 1,
      page: 1,
      page_size: 999,
    }));
    __setMockHandler("list_call_relations", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 500,
          filters: {
            owner_id: "app-current",
            owner_type: "service",
          },
        },
      });
      return {
        data: [],
        total: 0,
        page: 1,
        page_size: 500,
      };
    });

    const wrapper = mountEditor();
    await flushPromises();

    const addButton = wrapper.findAll("button").find((button) => button.text() === "添加关系");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const renderedText = wrapper.text();
    expect(renderedText).toContain("order-api（后端）");
    expect(renderedText).toContain("edge-gw（网关）");
    expect(renderedText).toContain("redis-main（Redis）");
    expect(renderedText).toContain("traffic-lb（网关）");
    expect(renderedText).not.toContain("billing-ui（前端）");
  });

  it("uses downstream as the default direction for a newly added relation", async () => {
    __setMockHandler("list_services", () => ({
      data: [{ id: "app-b", name: "order-api", type: "backend" }],
      total: 1,
      page: 1,
      page_size: 999,
    }));
    __setMockHandler("list_middlewares", () => ({
      data: [],
      total: 0,
      page: 1,
      page_size: 999,
    }));
    __setMockHandler("list_nginx_configs", () => ({
      data: [],
      total: 0,
      page: 1,
      page_size: 999,
    }));
    __setMockHandler("list_call_relations", () => ({
      data: [],
      total: 0,
      page: 1,
      page_size: 500,
    }));

    const wrapper = mountEditor();
    await flushPromises();

    const addButton = wrapper.findAll("button").find((button) => button.text() === "添加关系");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await flushPromises();

    const directionSelect = wrapper.find('[data-label="方向"] .el-select-stub');
    expect(directionSelect.exists()).toBe(true);
    expect(directionSelect.attributes("data-model")).toBe("downstream");
  });
});
