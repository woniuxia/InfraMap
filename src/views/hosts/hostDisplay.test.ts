import { describe, expect, it } from "vitest";
import type { Host } from "@/types";
import {
  buildHardwareSummary,
  parseHostTags,
  summarizeIpDisplay,
} from "@/views/hosts/hostDisplay";

describe("hostDisplay", () => {
  it("parseHostTags should trim, deduplicate and ignore invalid values", () => {
    const result = parseHostTags(`[" core ","edge","core","",123]`);
    expect(result).toEqual(["core", "edge"]);
  });

  it("summarizeIpDisplay should return compact summary for multiple ips", () => {
    const result = summarizeIpDisplay("10.0.0.1, 10.0.0.2,10.0.0.3");
    expect(result.primary).toBe("10.0.0.1");
    expect(result.extraCount).toBe(2);
    expect(result.all).toEqual(["10.0.0.1", "10.0.0.2", "10.0.0.3"]);
  });

  it("buildHardwareSummary should join available hardware fields", () => {
    const host: Partial<Host> = {
      cpu_model: "Xeon Gold",
      cpu_cores: 16,
      cpu_threads: 32,
      cpu_freq: "2.6",
      ram_gb: 64,
      disk_gb: 2048,
    };
    expect(buildHardwareSummary(host)).toBe("Xeon Gold / 16C32T / 2.6GHz / 64GB RAM / 2048GB Disk");
  });

  it("buildHardwareSummary should fallback to placeholder when all fields are empty", () => {
    expect(buildHardwareSummary({})).toBe("-");
  });
});
