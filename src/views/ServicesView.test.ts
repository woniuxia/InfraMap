import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, inject, provide } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import ServicesView from "@/views/ServicesView.vue";
import { listServices } from "@/api/services";

const editorState = vi.hoisted(() => ({
  visible: false,
  mode: "",
  initialDraft: {} as Record<string, unknown>,
}));

vi.mock("@/api/services", () => ({
  listServices: vi.fn(async () => ({
    data: [
      {
        id: "svc-1",
        system_id: "sys-1",
        system_name: "订单系统",
        name: "订单服务",
        type: "backend",
        address: "10.0.0.11",
        port: 8080,
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
  deleteService: vi.fn(async () => undefined),
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

const ServiceEditorDialogStub = defineComponent({
  name: "ServiceEditorDialog",
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
      return h("div", { "data-testid": "service-editor-dialog" }, [
        h(
          "button",
          {
            "data-testid": "emit-editor-saved",
            onClick: () => emit("saved", { id: "svc-1", mode: props.mode }),
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
  return mount(ServicesView, {
    global: {
      plugins: [createPinia()],
      stubs: {
        SearchToolbar: SearchToolbarStub,
        ElButton: ElButtonStub,
        ElTable: ElTableStub,
        ElTableColumn: ElTableColumnStub,
        ElTag: PassThroughStub,
        ElPagination: PassThroughStub,
        ServiceEditorDialog: ServiceEditorDialogStub,
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

describe("ServicesView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    editorState.visible = false;
    editorState.mode = "";
    editorState.initialDraft = {};
  });

  it("opens create editor when clicking add button", async () => {
    const wrapper = mountView();
    await flushPromises();

    const addButton = findButtonByText(wrapper, "新增服务");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");

    expect(editorState.visible).toBe(true);
    expect(editorState.mode).toBe("create");
    expect(String(editorState.initialDraft.id || "")).toMatch(/^svc-/);
  });

  it("opens edit editor when clicking edit button", async () => {
    const wrapper = mountView();
    await flushPromises();

    const editButton = findButtonByText(wrapper, "编辑");
    expect(editButton).toBeDefined();
    await editButton!.trigger("click");

    expect(editorState.visible).toBe(true);
    expect(editorState.mode).toBe("edit");
    expect(editorState.initialDraft.id).toBe("svc-1");
  });

  it("does not render owner tags when owners are missing", async () => {
    vi.mocked(listServices).mockResolvedValueOnce({
      data: [
        {
          id: "svc-empty-owners",
          system_id: "sys-1",
          system_name: "系统",
          name: "empty-owners-app",
          type: "backend",
          env: "prod",
          status: "running",
          created_at: "",
          updated_at: "",
        },
      ],
      total: 1,
      page: 1,
      page_size: 20,
    });

    const wrapper = mountView();
    await flushPromises();

    expect(wrapper.text()).not.toContain("alice");
  });

  it("opens edit editor with owners preserved", async () => {
    vi.mocked(listServices).mockResolvedValueOnce({
      data: [
        {
          id: "svc-edit-owners",
          system_id: "sys-1",
          system_name: "系统",
          name: "edit-owners-app",
          type: "backend",
          env: "prod",
          status: "running",
          owners: ["alice", "bob"],
          created_at: "",
          updated_at: "",
        },
      ],
      total: 1,
      page: 1,
      page_size: 20,
    });

    const wrapper = mountView();
    await flushPromises();

    const editButton = findButtonByText(wrapper, "编辑");
    expect(editButton).toBeDefined();
    await editButton!.trigger("click");

    expect(editorState.mode).toBe("edit");
    expect(editorState.initialDraft.owners).toEqual(["alice", "bob"]);
  });

  it("opens copy editor with a new id when clicking copy button", async () => {
    const wrapper = mountView();
    await flushPromises();

    const copyButton = findButtonByText(wrapper, "复制");
    expect(copyButton).toBeDefined();
    await copyButton!.trigger("click");

    expect(editorState.visible).toBe(true);
    expect(editorState.mode).toBe("copy");
    expect(editorState.initialDraft.id).not.toBe("svc-1");
    expect(String(editorState.initialDraft.name || "")).toContain("副本 ");
  });

  it("refreshes list after editor emits saved", async () => {
    const wrapper = mountView();
    await flushPromises();
    expect(vi.mocked(listServices)).toHaveBeenCalledTimes(1);

    const addButton = findButtonByText(wrapper, "新增服务");
    expect(addButton).toBeDefined();
    await addButton!.trigger("click");
    await wrapper.get('[data-testid="emit-editor-saved"]').trigger("click");
    await flushPromises();

    expect(vi.mocked(listServices)).toHaveBeenCalledTimes(2);
  });
});
