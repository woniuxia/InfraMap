import { ElMessage } from "element-plus";
import { reactive, readonly } from "vue";
import { formatInfraErrorToast } from "@/utils/error";
import type { InfraError } from "@/types/error";

const DEDUPE_WINDOW_MS = 1800;

interface ErrorPresenterState {
  latestError: InfraError | null;
  detailVisible: boolean;
  detailError: InfraError | null;
}

const state = reactive<ErrorPresenterState>({
  latestError: null,
  detailVisible: false,
  detailError: null,
});

const lastShownMap = new Map<string, number>();

function getErrorKey(error: InfraError): string {
  return [error.code, error.command, error.message, error.details ?? ""].join("::");
}

function shouldSuppress(error: InfraError): boolean {
  const now = Date.now();
  const key = getErrorKey(error);
  const lastShown = lastShownMap.get(key);
  if (lastShown && now - lastShown < DEDUPE_WINDOW_MS) {
    return true;
  }

  lastShownMap.set(key, now);
  return false;
}

export function presentInfraError(error: InfraError) {
  state.latestError = error;

  if (!shouldSuppress(error)) {
    ElMessage.error(formatInfraErrorToast(error));
  }
}

export function openInfraErrorDetail(error?: InfraError) {
  state.detailError = error ?? state.latestError;
  state.detailVisible = state.detailError !== null;
}

export function closeInfraErrorDetail() {
  state.detailVisible = false;
}

export function clearInfraError() {
  state.latestError = null;
  state.detailError = null;
  state.detailVisible = false;
}

export function useErrorPresenter() {
  return {
    state: readonly(state),
    presentInfraError,
    openInfraErrorDetail,
    closeInfraErrorDetail,
    clearInfraError,
  };
}
