import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import {
  getStorageProfile,
  updateStoragePath,
  restartApp,
} from "@/api/settings";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("settings storage API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("getStorageProfile should call get_storage_profile command", async () => {
    __setMockHandler("get_storage_profile", () => ({
      active_root_path: "D:/InfraMapData",
      db_path: "D:/InfraMapData/inframap.db",
      backup_dir: "D:/InfraMapData/backups",
      is_default_path: false,
    }));

    const profile = await getStorageProfile();
    expect(profile.active_root_path).toBe("D:/InfraMapData");
    expect(profile.is_default_path).toBe(false);
  });

  it("updateStoragePath should pass data object", async () => {
    __setMockHandler("update_storage_path", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          root_path: "D:/InfraMapData",
        },
      });
      return {
        restart_required: true,
        migrated: true,
      };
    });

    const result = await updateStoragePath({ root_path: "D:/InfraMapData" });
    expect(result.restart_required).toBe(true);
    expect(result.migrated).toBe(true);
  });

  it("restartApp should call restart_app command", async () => {
    let called = false;
    __setMockHandler("restart_app", () => {
      called = true;
      return undefined;
    });

    await restartApp();
    expect(called).toBe(true);
  });
});
