import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import type { Host } from "@/types";
import { useHostViewModel } from "@/views/hosts/useHostViewModel";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
  ElMessageBox: {
    confirm: vi.fn(),
  },
}));

function createHost(overrides: Partial<Host> = {}): Host {
  return {
    id: "host-1",
    hostname: "web-prod-01",
    env: "prod",
    status: "running",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

function setupMockHandlers() {
  __setMockHandler("list_ip_addresses", () => ({ data: [], total: 0 }));
  __setMockHandler("list_taxonomy_terms", () => []);
  __setMockHandler("list_host_ip_bindings", () => []);
}

function setupTaxonomyOrderHandlers() {
  __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
    if (!args) {
      return [];
    }

    if (args.fieldKey === "tags") {
      expect(args.sortBy).toBe("recent");
      return ["core", "edge"];
    }

    if (args.fieldKey === "cpu_model") {
      expect(args.sortBy).toBe("recent");
      return ["Intel Xeon Gold 6258R"];
    }

    if (args.fieldKey === "os_type") {
      expect(args.sortBy).toBe("count");
      return ["Ubuntu 22.04", "Windows Server 2022", "openEuler"];
    }

    return [];
  });
}

describe("useHostViewModel", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    __clearMockHandlers();
    vi.clearAllMocks();
    setupMockHandlers();
  });

  it("fills default hardware fields when opening add dialog", async () => {
    const viewModel = useHostViewModel();

    await viewModel.openAdd();

    expect(viewModel.editingHost.value).toMatchObject({
      id: "",
      hostname: "",
      env: "prod",
      status: "running",
      cpu_cores: 8,
      cpu_threads: 16,
      cpu_freq: "2.4",
      ram_gb: 16,
      disk_gb: 512,
    });
  });

  it("keeps existing hardware values when editing", async () => {
    const viewModel = useHostViewModel();
    const host = createHost({
      cpu_cores: 6,
      cpu_threads: 12,
      cpu_freq: "3.6",
      ram_gb: 24,
      disk_gb: 768,
    });

    await viewModel.openEdit(host);

    expect(viewModel.editingHost.value).toMatchObject({
      id: host.id,
      hostname: host.hostname,
      env: host.env,
      status: host.status,
      cpu_cores: 6,
      cpu_threads: 12,
      cpu_freq: "3.6",
      ram_gb: 24,
      disk_gb: 768,
    });
  });

  it("does not backfill defaults when copying a host without hardware fields", async () => {
    const viewModel = useHostViewModel();
    const host = createHost({
      hostname: "edge-node",
      cpu_cores: undefined,
      cpu_threads: undefined,
      cpu_freq: undefined,
      ram_gb: undefined,
      disk_gb: undefined,
    });

    await viewModel.openCopy(host);

    expect(viewModel.editingHost.value.cpu_cores).toBeUndefined();
    expect(viewModel.editingHost.value.cpu_threads).toBeUndefined();
    expect(viewModel.editingHost.value.cpu_freq).toBeUndefined();
    expect(viewModel.editingHost.value.ram_gb).toBeUndefined();
    expect(viewModel.editingHost.value.disk_gb).toBeUndefined();
  });

  it("loads os filter options by count and keeps them at the front of form suggestions", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      if (!args) {
        return [];
      }

      if (args.fieldKey === "os_type" && args.sortBy === "count") {
        return ["Ubuntu 22.04", "CentOS 7"];
      }
      return [];
    });

    const viewModel = useHostViewModel();

    await viewModel.openAdd();

    const osField = viewModel.toolbarFields.value.find((field) => field.key === "os_type");
    expect((osField?.options ?? []).map((option) => option.value)).toEqual([
      "Ubuntu 22.04",
      "CentOS 7",
    ]);
    expect(
      viewModel.formOsSuggestionOptions.value.slice(0, 2).map((option) => option.value),
    ).toEqual(["Ubuntu 22.04", "CentOS 7"]);
    expect(
      viewModel.formOsSuggestionOptions.value.findIndex((option) => option.value === "CentOS 8"),
    ).toBeGreaterThan(1);
  });

  it("keeps the current os value ahead of unused defaults when editing", async () => {
    __setMockHandler("list_taxonomy_terms", (_cmd, args) => {
      if (!args) {
        return [];
      }

      if (args.fieldKey === "os_type" && args.sortBy === "count") {
        return ["Ubuntu 22.04", "CentOS 7"];
      }
      return [];
    });

    const viewModel = useHostViewModel();

    await viewModel.openEdit(
      createHost({
        os_type: "Custom Linux 1.0",
      }),
    );

    const optionValues = viewModel.formOsSuggestionOptions.value.map((option) => option.value);
    expect(optionValues.slice(0, 3)).toEqual(["Ubuntu 22.04", "CentOS 7", "Custom Linux 1.0"]);
    expect(optionValues.indexOf("Custom Linux 1.0")).toBeLessThan(optionValues.indexOf("CentOS 8"));
  });

  it("should expose os_type and cpu_model filter fields for toolbar", () => {
    const viewModel = useHostViewModel();
    const keys = viewModel.toolbarFields.value.map((field) => field.key);

    expect(keys).toContain("os_type");
    expect(keys).toContain("cpu_model");
  });

  it("orders host os filter options by usage count after loading taxonomy", async () => {
    setupTaxonomyOrderHandlers();
    const viewModel = useHostViewModel();

    await viewModel.openAdd();

    const osField = viewModel.toolbarFields.value.find((field) => field.key === "os_type");
    expect(osField?.options).toEqual([
      { label: "Ubuntu 22.04", value: "Ubuntu 22.04" },
      { label: "Windows Server 2022", value: "Windows Server 2022" },
      { label: "openEuler", value: "openEuler" },
    ]);
  });

  it("orders host os suggestions by usage count and appends unused defaults", async () => {
    setupTaxonomyOrderHandlers();
    const viewModel = useHostViewModel();

    await viewModel.openAdd();

    expect(viewModel.formOsSuggestionOptions.value.slice(0, 5)).toEqual([
      { label: "Ubuntu 22.04", value: "Ubuntu 22.04" },
      { label: "Windows Server 2022", value: "Windows Server 2022" },
      { label: "openEuler", value: "openEuler" },
      { label: "CentOS 7", value: "CentOS 7" },
      { label: "CentOS 8", value: "CentOS 8" },
    ]);
  });

  it("keeps current os value in suggestions when it is not counted or predefined", async () => {
    setupTaxonomyOrderHandlers();
    const viewModel = useHostViewModel();

    await viewModel.openEdit(createHost({ os_type: "Arch Linux" }));

    expect(viewModel.formOsSuggestionOptions.value.slice(0, 4)).toEqual([
      { label: "Ubuntu 22.04", value: "Ubuntu 22.04" },
      { label: "Windows Server 2022", value: "Windows Server 2022" },
      { label: "openEuler", value: "openEuler" },
      { label: "Arch Linux", value: "Arch Linux" },
    ]);
  });
});
