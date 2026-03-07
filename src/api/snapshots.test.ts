import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

import { exportSnapshotV2, importSnapshotV2, previewSnapshotV2 } from "@/api/snapshots";

describe("snapshots API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("previewSnapshotV2 should invoke preview_snapshot_v2", async () => {
    const mock = {
      manifest: { format_version: 2, export_time: "", app_version: "0.1.0", schema_version: 21 },
      snapshot_counts: [],
      current_counts: [],
      compatible: true,
      warnings: [],
      total_rows: 0,
    };

    __setMockHandler("preview_snapshot_v2", (_cmd, args) => {
      expect(args).toEqual({ filepath: "E:/tmp/snapshot.json" });
      return mock;
    });

    const result = await previewSnapshotV2("E:/tmp/snapshot.json");
    expect(result).toEqual(mock);
  });

  it("exportSnapshotV2 should invoke export_snapshot_v2", async () => {
    const mock = { job_id: "job-1", filepath: "E:/tmp/snapshot.json", total_rows: 10, table_counts: [] };

    __setMockHandler("export_snapshot_v2", (_cmd, args) => {
      expect(args).toEqual({ filepath: "E:/tmp/snapshot.json" });
      return mock;
    });

    const result = await exportSnapshotV2("E:/tmp/snapshot.json");
    expect(result).toEqual(mock);
  });

  it("importSnapshotV2 should invoke import_snapshot_v2", async () => {
    const mock = {
      job_id: "job-2",
      backup_filename: "backup_pre_import_20260307.db",
      total_rows: 10,
      table_counts: [],
    };

    __setMockHandler("import_snapshot_v2", (_cmd, args) => {
      expect(args).toEqual({ filepath: "E:/tmp/snapshot.json" });
      return mock;
    });

    const result = await importSnapshotV2("E:/tmp/snapshot.json");
    expect(result).toEqual(mock);
  });
});
