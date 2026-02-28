import { describe, expect, it } from "vitest";
import { COMMON_DISK_OPTIONS_GB, COMMON_RAM_OPTIONS_GB } from "@/views/hostsHardwareOptions";

describe("hosts hardware options", () => {
  it("uses the expected common RAM options in GB", () => {
    expect(COMMON_RAM_OPTIONS_GB).toEqual([4, 8, 16, 32, 64, 128]);
  });

  it("uses the expected common disk options in GB", () => {
    expect(COMMON_DISK_OPTIONS_GB).toEqual([128, 256, 512, 1024, 2048]);
  });
});
