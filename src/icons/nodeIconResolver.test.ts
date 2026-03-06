import { afterEach, describe, expect, it } from "vitest";
import { clearRuntimeIconSetsForTest, registerRuntimeIconSet } from "@/icons/iconRegistry";
import {
  resolveApplicationNodeIcon,
  resolveApplicationTypeKey,
  resolveMiddlewareNodeIcon,
} from "@/icons/nodeIconResolver";

const CUSTOM_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><circle cx="10" cy="10" r="8"/></svg>`;

describe("nodeIconResolver", () => {
  afterEach(() => {
    clearRuntimeIconSetsForTest();
  });

  it("normalizes application type keys", () => {
    expect(resolveApplicationTypeKey("frontend")).toBe("frontend");
    expect(resolveApplicationTypeKey("backend")).toBe("backend");
    expect(resolveApplicationTypeKey("batch-job")).toBe("batch_job");
    expect(resolveApplicationTypeKey("micro service")).toBe("microservice");
    expect(resolveApplicationTypeKey("unknown")).toBe("other");
  });

  it("resolves application node icon with fallback support", () => {
    const frontend = resolveApplicationNodeIcon("frontend");
    expect(frontend.iconKey.length).toBeGreaterThan(0);
    expect(frontend.src.startsWith("data:image/svg+xml")).toBe(true);
  });

  it("uses custom middleware icon key when runtime icon exists", () => {
    registerRuntimeIconSet("runtime", {
      custom: CUSTOM_SVG,
    });

    const icon = resolveMiddlewareNodeIcon({
      icon_key: "runtime:custom",
      category: "cache",
      type: "Redis",
    });

    expect(icon.iconKey).toBe("runtime:custom");
    expect(icon.src.startsWith("data:image/svg+xml")).toBe(true);
  });

  it("falls back to middleware type icon when no custom icon exists", () => {
    const icon = resolveMiddlewareNodeIcon({
      category: "cache",
      type: "Redis",
    });

    expect(icon.iconKey).toContain("redis");
    expect(icon.src.startsWith("data:image/svg+xml")).toBe(true);
  });
});
