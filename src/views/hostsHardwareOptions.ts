export const COMMON_CPU_CORES_OPTIONS = [2, 4, 8, 16, 32, 64];
export const COMMON_CPU_THREADS_OPTIONS = [4, 8, 16, 32, 64, 128];
export const COMMON_CPU_FREQ_OPTIONS = ["2.0", "2.2", "2.4", "2.6", "2.8", "3.0"];
export const COMMON_RAM_OPTIONS_GB = [4, 8, 16, 32, 64, 128];
export const COMMON_DISK_OPTIONS_GB = [128, 256, 512, 1024, 2048];
export const DEFAULT_HOST_HARDWARE = {
  cpu_cores: 8,
  cpu_threads: 16,
  cpu_freq: "2.4",
  ram_gb: 16,
  disk_gb: 512,
};

export function normalizePositiveIntegerValue(value: unknown): number | undefined {
  if (value === undefined || value === null || value === "") {
    return undefined;
  }

  const normalized = typeof value === "number" ? value : Number(String(value).trim());
  if (!Number.isInteger(normalized) || normalized <= 0) {
    return undefined;
  }

  return normalized;
}

export function normalizeCpuFreqValue(value: unknown): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }

  const normalized = value.trim();
  if (!normalized) {
    return undefined;
  }

  return normalized.replace(/\s*ghz\s*$/i, "").trim() || undefined;
}
