import { beforeEach, describe, expect, it, vi } from "vitest";
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

describe("useHostViewModel", () => {
  beforeEach(() => {
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
});
