import { invoke } from "@tauri-apps/api/core";
import { presentInfraError } from "@/composables/useErrorPresenter";
import { normalizeInvokeError } from "@/utils/error";
import type { TauriInvokeOptions } from "@/types/error";

export async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
  options?: TauriInvokeOptions,
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    const infraError = normalizeInvokeError(error, cmd);

    if (!options?.silent) {
      presentInfraError(infraError);
    }

    throw infraError;
  }
}
