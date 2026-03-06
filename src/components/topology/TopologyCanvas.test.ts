import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import TopologyCanvas from "@/components/topology/TopologyCanvas.vue";
import type { TopologyGraph } from "@/types";

const { capturedStylesheets } = vi.hoisted(() => ({
  capturedStylesheets: [] as Array<Array<{ selector: string; style: Record<string, unknown> }>>,
}));

vi.mock("cytoscape-dagre", () => ({
  default: vi.fn(),
}));

vi.mock("cytoscape-fcose", () => ({
  default: vi.fn(),
}));

vi.mock("cytoscape-svg", () => ({
  default: vi.fn(),
}));

vi.mock("cytoscape", () => {
  function createCollection(items: any[], onRemove?: () => void) {
    return {
      not(selector: string) {
        if (selector !== ":parent") return createCollection(items, onRemove);
        return createCollection(
          items.filter((item) => typeof item.isParent !== "function" || !item.isParent()),
          onRemove,
        );
      },
      forEach(callback: (item: any) => void) {
        items.forEach(callback);
      },
      map<T>(callback: (item: any) => T) {
        return items.map(callback);
      },
      filter(predicate: ((item: any) => boolean) | string) {
        if (typeof predicate === "string") {
          return createCollection(items, onRemove);
        }
        return createCollection(items.filter(predicate), onRemove);
      },
      removeClass() {
        return this;
      },
      addClass() {
        return this;
      },
      nonempty() {
        return items.length > 0;
      },
      empty() {
        return items.length === 0;
      },
      remove() {
        onRemove?.();
        return this;
      },
    };
  }

  function createNode(item: { data?: Record<string, unknown> }, index: number, parentIds: Set<string>) {
    let position = { x: index * 40, y: index * 20 };
    return {
      id: () => String(item.data?.id || ""),
      isParent: () => parentIds.has(String(item.data?.id || "")),
      position(next?: { x: number; y: number }) {
        if (next) {
          position = next;
          return this;
        }
        return position;
      },
      data(key?: string) {
        if (!key) return item.data || {};
        return item.data?.[key];
      },
    };
  }

  const cytoscapeMock = Object.assign(
    vi.fn((options: { style: Array<{ selector: string; style: Record<string, unknown> }> }) => {
      const state = {
        elements: [] as Array<{ data?: Record<string, unknown> }>,
      };

      const core = {
        on: vi.fn(),
        zoom: vi.fn(() => 1),
        pan: vi.fn(() => ({ x: 0, y: 0 })),
        nodes: vi.fn((selector?: string) => {
          const parentIds = new Set(
            state.elements
              .map((item) => String(item.data?.parent || ""))
              .filter((id) => id.length > 0),
          );
          let nodes = state.elements
            .filter((item) => !item.data?.source && !item.data?.target)
            .map((item, index) => createNode(item, index + 1, parentIds));
          if (selector === ":parent") {
            nodes = nodes.filter((node) => node.isParent());
          }
          return createCollection(nodes);
        }),
        edges: vi.fn(() => createCollection([])),
        elements: vi.fn(() => createCollection(state.elements, () => {
          state.elements = [];
        })),
        batch: vi.fn((callback: () => void) => callback()),
        add: vi.fn((elements: Array<{ data?: Record<string, unknown> }>) => {
          state.elements = elements;
          return createCollection([]);
        }),
        layout: vi.fn(() => {
          let onStop: (() => void) | undefined;
          return {
            one: vi.fn((_event: string, callback: () => void) => {
              onStop = callback;
            }),
            run: vi.fn(() => {
              onStop?.();
            }),
          };
        }),
        style: vi.fn((stylesheet?: Array<{ selector: string; style: Record<string, unknown> }>) => {
          if (stylesheet) {
            capturedStylesheets.push(stylesheet);
          }
          return core;
        }),
        fit: vi.fn(),
        resize: vi.fn(),
        destroy: vi.fn(),
        getElementById: vi.fn(() => createCollection([])),
        center: vi.fn(),
        animate: vi.fn(),
        png: vi.fn(() => "data:image/png;base64,mock"),
        svg: vi.fn(() => "<svg></svg>"),
      };

      capturedStylesheets.push(options.style);
      return core;
    }),
    { use: vi.fn() },
  );

  return {
    default: cytoscapeMock,
  };
});

const graphFixture: TopologyGraph = {
  lanes: [
    { id: "prod", label: "生产", order: 0, node_count: 1, app_count: 1 },
    { id: "test", label: "测试", order: 1, node_count: 0, app_count: 0 },
    { id: "dev", label: "开发", order: 2, node_count: 0, app_count: 0 },
  ],
  nodes: [
    {
      id: "app-prod-1",
      name: "订单服务",
      node_type: "application",
      env: "prod",
      group_kind: "application_service",
      importance: 1,
      status: "running",
      extra: {
        type: "frontend",
      },
    },
  ],
  edges: [],
  legend_stats: {
    env_counts: [
      { env: "prod", count: 1, app_count: 1 },
      { env: "test", count: 0, app_count: 0 },
      { env: "dev", count: 0, app_count: 0 },
    ],
    node_type_counts: [{ kind: "application", count: 1 }],
    edge_type_counts: [],
    application_service_count: 1,
    current_env: "prod",
    external_node_count: 0,
    cross_env_edge_count: 0,
  },
  layout_hints: {
    lane_order: ["prod", "test", "dev"],
    default_collapsed_groups: [],
    high_density_mode: false,
  },
};

function getLatestStylesheet() {
  const stylesheet = capturedStylesheets[capturedStylesheets.length - 1];
  expect(stylesheet).toBeTruthy();
  return stylesheet!;
}

function findStyleBlock(
  stylesheet: Array<{ selector: string; style: Record<string, unknown> }>,
  selector: string,
) {
  const block = stylesheet.find((item) => item.selector === selector);
  expect(block).toBeTruthy();
  return block!;
}

function resolveStyleValue(
  value: unknown,
  data: Record<string, unknown>,
) {
  if (typeof value !== "function") return value;
  return value({
    data(key?: string) {
      if (!key) return data;
      return data[key];
    },
  });
}

function decodeSvgDataUri(uri: string): string {
  const prefix = "data:image/svg+xml;charset=utf-8,";
  expect(uri.startsWith(prefix)).toBe(true);
  return decodeURIComponent(uri.slice(prefix.length));
}

describe("TopologyCanvas", () => {
  beforeEach(() => {
    capturedStylesheets.length = 0;
    vi.stubGlobal("ResizeObserver", class {
      observe() {}
      disconnect() {}
    });
    vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
      callback(0);
      return 1;
    });
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("removes visible node fill while preserving parent group backgrounds", async () => {
    mount(TopologyCanvas, {
      props: {
        graphData: graphFixture,
      },
    });

    await flushPromises();

    const stylesheet = getLatestStylesheet();
    const nodeBlock = findStyleBlock(stylesheet, "node[shape][size][label_font_size]");
    const parentBlock = findStyleBlock(stylesheet, "node:parent");
    const externalBlock = findStyleBlock(stylesheet, 'node[kind = "external"]');
    const backgroundImage = resolveStyleValue(nodeBlock.style["background-image"], {
      node_type: "application",
      app_type_key: "frontend",
      is_external: false,
      status: "running",
      icon_src: "data:image/svg+xml;charset=utf-8,%3Csvg%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%20viewBox%3D%220%200%2024%2024%22%3E%3Cpath%20d%3D%22M0%200h24v24H0z%22/%3E%3C/svg%3E",
    });

    expect(
      resolveStyleValue(nodeBlock.style["background-opacity"], {
        node_type: "application",
        app_type_key: "frontend",
        is_external: false,
        status: "running",
      }),
    ).toBe(0);
    expect(nodeBlock.style["background-image"]).toBeTypeOf("function");
    expect(externalBlock.style["background-color"]).toBeUndefined();
    expect(parentBlock.style["background-color"]).toBeTruthy();
    expect(decodeSvgDataUri(String(backgroundImage))).toContain('color="#5ca3ff"');
  });
});
