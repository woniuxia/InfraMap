import { reactive, readonly } from "vue";
import { formatInfraErrorToast } from "@/utils/error";
import { showErrorMessage } from "@/utils/message";
import type { InfraError } from "@/types/error";

const DEDUPE_WINDOW_MS = 1800;

interface ErrorPresenterState {
  latestError: InfraError | null;
  history: InfraError[];
  collapsedVisible: boolean;
  activeErrorIndex: number;
  detailVisible: boolean;
  detailError: InfraError | null;
}

const state = reactive<ErrorPresenterState>({
  latestError: null,
  history: [],
  collapsedVisible: false,
  activeErrorIndex: 0,
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
  state.history.unshift(error);
  state.latestError = error;
  state.activeErrorIndex = 0;
  state.collapsedVisible = !state.detailVisible && state.history.length > 0;
  if (state.detailVisible) {
    state.detailError = error;
  }

  if (!shouldSuppress(error)) {
    showErrorMessage(formatInfraErrorToast(error));
  }
}

export function openInfraErrorDetail(errorOrIndex?: InfraError | number) {
  if (typeof errorOrIndex === "number") {
    if (state.history.length === 0) {
      state.detailError = null;
      state.detailVisible = false;
      return;
    }

    const index = Math.min(Math.max(errorOrIndex, 0), state.history.length - 1);
    state.activeErrorIndex = index;
    state.detailError = state.history[index] ?? null;
  } else if (errorOrIndex) {
    state.detailError = errorOrIndex;
    const index = state.history.findIndex((item) => item === errorOrIndex);
    if (index >= 0) {
      state.activeErrorIndex = index;
    }
  } else if (state.history.length > 0) {
    const index = Math.min(Math.max(state.activeErrorIndex, 0), state.history.length - 1);
    state.activeErrorIndex = index;
    state.detailError = state.history[index] ?? null;
  } else {
    state.detailError = state.latestError;
  }

  state.detailVisible = state.detailError !== null;
  if (state.detailVisible) {
    state.collapsedVisible = false;
  }
}

export function setActiveError(index: number) {
  openInfraErrorDetail(index);
}

export function closeInfraErrorDetail() {
  state.detailVisible = false;
  state.collapsedVisible = state.history.length > 0;
}

export function clearInfraErrorHistory() {
  lastShownMap.clear();
  state.latestError = null;
  state.history = [];
  state.collapsedVisible = false;
  state.activeErrorIndex = 0;
  state.detailError = null;
  state.detailVisible = false;
}

export function clearInfraError() {
  clearInfraErrorHistory();
}

export function useErrorPresenter() {
  return {
    state: readonly(state),
    presentInfraError,
    openInfraErrorDetail,
    setActiveError,
    closeInfraErrorDetail,
    clearInfraErrorHistory,
    clearInfraError,
  };
}
