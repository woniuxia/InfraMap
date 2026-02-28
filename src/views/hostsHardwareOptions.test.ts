import { describe, expect, it } from "vitest";
import {
  COMMON_CPU_CORES_OPTIONS,
  COMMON_CPU_FREQ_OPTIONS,
  COMMON_CPU_THREADS_OPTIONS,
  COMMON_DISK_OPTIONS_GB,
  COMMON_RAM_OPTIONS_GB,
  normalizeCpuFreqValue,
  normalizePositiveIntegerValue,
} from "@/views/hostsHardwareOptions";

describe("hosts hardware options", () => {
  it("uses the expected common RAM options in GB", () => {
    expect(COMMON_RAM_OPTIONS_GB).toEqual([4, 8, 16, 32, 64, 128]);
  });

  it("uses the expected common disk options in GB", () => {
    expect(COMMON_DISK_OPTIONS_GB).toEqual([128, 256, 512, 1024, 2048]);
  });

  it("uses the expected common CPU core options", () => {
    expect(COMMON_CPU_CORES_OPTIONS).toEqual([2, 4, 8, 16, 32, 64]);
  });

  it("uses the expected common CPU thread options", () => {
    expect(COMMON_CPU_THREADS_OPTIONS).toEqual([4, 8, 16, 32, 64, 128]);
  });

  it("uses the expected common CPU frequency options", () => {
    expect(COMMON_CPU_FREQ_OPTIONS).toEqual([
      "2.0",
      "2.2",
      "2.4",
      "2.6",
      "2.8",
      "3.0",
    ]);
  });

  it("normalizes positive integer values from number and string", () => {
    expect(normalizePositiveIntegerValue(16)).toBe(16);
    expect(normalizePositiveIntegerValue("32")).toBe(32);
    expect(normalizePositiveIntegerValue(" 64 ")).toBe(64);
  });

  it("rejects invalid positive integer values", () => {
    expect(normalizePositiveIntegerValue(undefined)).toBeUndefined();
    expect(normalizePositiveIntegerValue("")).toBeUndefined();
    expect(normalizePositiveIntegerValue("abc")).toBeUndefined();
    expect(normalizePositiveIntegerValue(0)).toBeUndefined();
    expect(normalizePositiveIntegerValue("0")).toBeUndefined();
    expect(normalizePositiveIntegerValue(-1)).toBeUndefined();
    expect(normalizePositiveIntegerValue("2.5")).toBeUndefined();
  });

  it("normalizes CPU frequency text values", () => {
    expect(normalizeCpuFreqValue("2.4 GHz")).toBe("2.4");
    expect(normalizeCpuFreqValue("2.4ghz")).toBe("2.4");
    expect(normalizeCpuFreqValue(" 3.2GHz ")).toBe("3.2");
    expect(normalizeCpuFreqValue("3.5")).toBe("3.5");
    expect(normalizeCpuFreqValue("")).toBeUndefined();
    expect(normalizeCpuFreqValue("   ")).toBeUndefined();
    expect(normalizeCpuFreqValue(undefined)).toBeUndefined();
  });
});
