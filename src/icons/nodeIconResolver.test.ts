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

  it("resolves frontend application icon to window", () => {
    const frontend = resolveApplicationNodeIcon("frontend");
    expect(frontend.iconKey).toContain("window");
    expect(frontend.src.startsWith("data:image/svg+xml")).toBe(true);
  });

  it("resolves backend application icon to server", () => {
    const backend = resolveApplicationNodeIcon("backend");
    expect(backend.iconKey).toContain("server-stack");
    expect(backend.src.startsWith("data:image/svg+xml")).toBe(true);
  });

  it("resolves gateway and unknown application icons to server", () => {
    const gateway = resolveApplicationNodeIcon("gateway");
    const unknown = resolveApplicationNodeIcon("unknown");
    expect(gateway.iconKey).toContain("server-stack");
    expect(unknown.iconKey).toContain("server-stack");
  });

  it("uses custom application icon key when runtime icon exists", () => {
    registerRuntimeIconSet("runtime", {
      custom: CUSTOM_SVG,
    });

    const icon = resolveApplicationNodeIcon("frontend", "runtime:custom");

    expect(icon.iconKey).toBe("runtime:custom");
    expect(icon.src.startsWith("data:image/svg+xml")).toBe(true);
  });

  it("always uses middleware type icon even when custom icon key exists", () => {
    registerRuntimeIconSet("runtime", {
      custom: CUSTOM_SVG,
    });

    const icon = resolveMiddlewareNodeIcon({
      icon_key: "runtime:custom",
      category: "cache",
      type: "Redis",
    });

    expect(icon.iconKey).toContain("redis");
    expect(icon.iconKey).not.toBe("runtime:custom");
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
