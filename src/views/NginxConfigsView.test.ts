import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import NginxConfigsView from "@/views/NginxConfigsView.vue";
import { listNginxConfigs } from "@/api/nginx-configs";

const editorState = vi.hoisted(() => ({
  visible: false,
  mode: "",
  initialDraft: {} as Record<string, unknown>,
}));

vi.mock("@/api/nginx-configs", () => ({
  listNginxConfigs: vi.fn(async () => ({
    data: [
      {
        id: "ng-1",
        name: "网关入口",
        endpoints: [{ host: "10.0.0.8", port: 80 }],
        strategy: "roundrobin",
        env: "prod",
        status: "running",
        created_at: "",
        updated_at: "",
      },
    ],
    total: 1,
    page: 1,
    page_size: 20,
  })),
  deleteNginxConfig: vi.fn(async () => undefined),
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
            slots.default ? slots.default({ row, $index: index }) : String(row[props.prop] ?? ""),
          ),
        ),
      );
  },
});

const NginxConfigEditorDialogStub = defineComponent({
  name: "NginxConfigEditorDialog",
  props: {
    modelValue: {
      type: Boolean,
      default: false,
    },
    mode: {
      type: String,
      default: "create",
    },
    initialDraft: {
      type: Object,
      default: () => ({}),
    },
  },
  emits: ["saved", "update:modelValue"],
  setup(props, { emit }) {
    return () => {
      editorState.visible = props.modelValue;
      editorState.mode = props.mode;
      editorState.initialDraft = props.initialDraft as Record<string, unknown>;
      return h("div", { "data-testid": "nginx-editor-dialog" }, [
        h(
          "button",
          {
            "data-testid": "emit-editor-saved",
            onClick: () => emit("saved", { id: "ng-1", mode: props.mode }),
          },
          "emit-editor-saved",
        ),
      ]);
    };
  },
});

const PassThroughStub = defineComponent({
  template: `<div><slot /></div>`,
});

function mountView() {
  return mount(NginxConfigsView, {
    global: {
      stubs: {
        SearchToolbar: SearchToolbarStub,
        ElButton: ElButtonStub,
        ElTable: ElTableStub,
        ElTableColumn: ElTableColumnStub,
        ElTag: PassThroughStub,
        ElPagination: PassThroughStub,
        NginxConfigEditorDialog: NginxConfigEditorDialogStub,
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

describe("NginxConfigsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    editorState.visible = false;
    editorState.mode = "";
    editorState.initialDraft = {};
  });

  it("opens create editor when clicking add button", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增配置");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");

    expect(editorState.visible).toBe(true);
    expect(editorState.mode).toBe("create");
    expect(editorState.initialDraft.status).toBe("running");
  });

  it("opens edit editor when clicking edit button", async () => {
    const wrapper = mountView();
    await flushPromises();

    const editButton = findButtonByText(wrapper, "编辑");
    expect(editButton).toBeDefined();
    await editButton!.trigger("click");

    expect(editorState.visible).toBe(true);
    expect(editorState.mode).toBe("edit");
    expect(editorState.initialDraft.id).toBe("ng-1");
  });

  it("opens copy editor with copied draft when clicking copy button", async () => {
    const wrapper = mountView();
    await flushPromises();

    const copyButton = findButtonByText(wrapper, "复制");
    expect(copyButton).toBeDefined();
    await copyButton!.trigger("click");

    expect(editorState.visible).toBe(true);
    expect(editorState.mode).toBe("copy");
    expect(editorState.initialDraft.id).toBeUndefined();
    expect(String(editorState.initialDraft.name || "")).toContain("副本 ");
  });

  it("refreshes list after editor emits saved", async () => {
    const wrapper = mountView();
    await flushPromises();
    expect(vi.mocked(listNginxConfigs)).toHaveBeenCalledTimes(1);

    const addButton = findButtonByText(wrapper, "新增配置");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await wrapper.get('[data-testid="emit-editor-saved"]').trigger("click");
    await flushPromises();

    expect(vi.mocked(listNginxConfigs)).toHaveBeenCalledTimes(2);
  });
});
