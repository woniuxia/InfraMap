import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ElMessage } from "element-plus";
import { InfraError } from "@/types/error";
import * as errorPresenter from "@/composables/useErrorPresenter";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
  },
}));

function createInfraError(seed: string) {
  return new InfraError(
    {
      code: "INTERNAL_ERROR",
      message: `错误-${seed}`,
      details: `detail-${seed}`,
      command: `cmd-${seed}`,
      retryable: false,
    },
    null,
  );
}

function resetState() {
  const api = errorPresenter as Record<string, unknown>;
  const clearHistory = api.clearInfraErrorHistory;
  const clearCurrent = api.clearInfraError;

  if (typeof clearHistory === "function") {
    clearHistory();
  }

  if (typeof clearCurrent === "function") {
    clearCurrent();
  }
}

describe("useErrorPresenter", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    resetState();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("records full error history and shows collapsed badge after 4 seconds", () => {
    const presenter = errorPresenter.useErrorPresenter() as Record<string, any>;
    const error = createInfraError("one");

    errorPresenter.presentInfraError(error);

    expect(ElMessage.error).toHaveBeenCalledTimes(1);
    expect(presenter.state.history).toHaveLength(1);
    expect(presenter.state.collapsedVisible).toBe(false);

    vi.advanceTimersByTime(4000);

    expect(presenter.state.collapsedVisible).toBe(true);
  });

  it("dedupes toast within 1.8s but still keeps all error records", () => {
    const presenter = errorPresenter.useErrorPresenter() as Record<string, any>;
    const error = createInfraError("dup");

    errorPresenter.presentInfraError(error);
    errorPresenter.presentInfraError(error);

    expect(ElMessage.error).toHaveBeenCalledTimes(1);
    expect(presenter.state.history).toHaveLength(2);
  });

  it("provides clearInfraErrorHistory and fully resets state", () => {
    const presenter = errorPresenter.useErrorPresenter() as Record<string, any>;
    const api = errorPresenter as Record<string, unknown>;
    const clearHistory = api.clearInfraErrorHistory;
    errorPresenter.presentInfraError(createInfraError("reset"));

    expect(typeof clearHistory).toBe("function");
    if (typeof clearHistory === "function") {
      clearHistory();
    }

    expect(presenter.state.history).toHaveLength(0);
    expect(presenter.state.latestError).toBeNull();
    expect(presenter.state.collapsedVisible).toBe(false);
    expect(presenter.state.detailVisible).toBe(false);
  });
});
