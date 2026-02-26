import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";

export async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    const message = typeof error === "string" ? error : "Unknown error";
    ElMessage.error(message);
    throw new Error(message);
  }
}
