import { defineComponent, h, inject, provide } from "vue";
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import type { Host } from "@/types";
import HostsListTable from "@/views/hosts/HostsListTable.vue";

interface TableContext {
  data: Host[];
  rowKey: string;
  expandRowKeys: string[];
}

const tableContextKey = Symbol("hostsTableContext");

const ElTableStub = defineComponent({
  name: "ElTable",
  props: {
    data: {
      type: Array,
      default: () => [],
    },
    rowKey: {
      type: String,
      default: "id",
    },
    expandRowKeys: {
      type: Array,
      default: () => [],
    },
  },
  setup(props, { slots }) {
    provide<TableContext>(tableContextKey, {
      data: props.data as Host[],
      rowKey: props.rowKey,
      expandRowKeys: props.expandRowKeys as string[],
    });

    return () =>
      h("div", { "data-testid": "el-table" }, [
        h("div", { "data-testid": "expand-row-keys" }, JSON.stringify(props.expandRowKeys)),
        slots.default ? slots.default() : [],
      ]);
  },
});

const ElTableColumnStub = defineComponent({
  name: "ElTableColumn",
  props: {
    prop: {
      type: String,
      default: "",
    },
    label: {
      type: String,
      default: "",
    },
    type: {
      type: String,
      default: "",
    },
  },
  setup(props, { slots }) {
    const tableContext = inject<TableContext>(tableContextKey);
    return () => {
      const rows = tableContext?.data || [];
      const rowKeyField = tableContext?.rowKey || "id";
      const expandedRowKeys = tableContext?.expandRowKeys || [];
      const columnId = props.type || props.prop || props.label || "column";

      return h(
        "div",
        { "data-testid": `column-${columnId}` },
        rows.map((row, index) => {
          const rowKeyValue = String(row[rowKeyField as keyof Host] ?? "");
          if (props.type === "expand") {
            return h(
              "div",
              { "data-testid": `expand-cell-${index}` },
              expandedRowKeys.includes(rowKeyValue) && slots.default ? slots.default({ row, $index: index }) : [],
            );
          }

          return h(
            "div",
            { "data-testid": `cell-${columnId}-${index}` },
            slots.default ? slots.default({ row, $index: index }) : String(row[props.prop as keyof Host] ?? ""),
          );
        }),
      );
    };
  },
});

const ElButtonStub = defineComponent({
  name: "ElButton",
  emits: ["click"],
  template: `<button v-bind="$attrs" @click="$emit('click')"><slot /></button>`,
});

const ElTagStub = defineComponent({
  name: "ElTag",
  template: `<span><slot /></span>`,
});

const ElPaginationStub = defineComponent({
  name: "ElPagination",
  template: `<div data-testid="el-pagination"></div>`,
});

const hosts: Host[] = [
  {
    id: "host-1",
    hostname: "web-prod-01",
    ip_display: "10.0.0.1, 10.0.0.2,10.0.0.3",
    env: "prod",
    status: "running",
    created_at: "2026-03-01T00:00:00.000Z",
    updated_at: "2026-03-01T00:00:00.000Z",
  },
];

function mountTable() {
  return mount(HostsListTable, {
    props: {
      data: hosts,
      loading: false,
      total: 1,
      page: 1,
      pageSize: 20,
    },
    global: {
      stubs: {
        ElTable: ElTableStub,
        ElTableColumn: ElTableColumnStub,
        ElButton: ElButtonStub,
        ElTag: ElTagStub,
        ElPagination: ElPaginationStub,
      },
      directives: {
        loading: () => undefined,
      },
    },
  });
}

describe("HostsListTable", () => {
  it("renders all ips in a comma-separated list", () => {
    const wrapper = mountTable();
    const ipCell = wrapper.get('[data-testid="cell-IP地址-0"]');
    expect(ipCell.text()).toContain("10.0.0.1, 10.0.0.2, 10.0.0.3");
  });

  it("toggles expanded row when hostname cell is clicked", async () => {
    const wrapper = mountTable();
    const expandState = () => wrapper.get('[data-testid="expand-row-keys"]').text();
    const hostToggle = () => wrapper.get(".host-name-toggle");

    expect(expandState()).toBe("[]");
    expect(hostToggle().attributes("aria-expanded")).toBe("false");

    await hostToggle().trigger("click");
    expect(expandState()).toContain("host-1");
    expect(hostToggle().attributes("aria-expanded")).toBe("true");

    const table = wrapper.getComponent(ElTableStub);
    table.vm.$emit("expand-change", hosts[0], []);
    await wrapper.vm.$nextTick();
    expect(expandState()).toBe("[]");
    expect(hostToggle().attributes("aria-expanded")).toBe("false");
  });
});
