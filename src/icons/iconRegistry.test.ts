import { afterEach, describe, expect, it } from "vitest";
import {
  clearRuntimeIconSetsForTest,
  registerRuntimeIconSet,
  resolveIconDataUri,
  unregisterRuntimeIconSet,
} from "@/icons/iconRegistry";

const SIMPLE_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M0 0h24v24H0z"/></svg>`;

describe("iconRegistry", () => {
  afterEach(() => {
    clearRuntimeIconSetsForTest();
  });

  it("resolves builtin middleware svg icons", () => {
    const uri = resolveIconDataUri("local-middleware:redis");
    expect(uri).toBeTruthy();
    expect(uri?.startsWith("data:image/svg+xml")).toBe(true);
  });

  it("resolves at least one builtin heroicons icon", () => {
    const candidates = [
      "heroicons:window",
      "heroicons:window-20-solid",
      "heroicons:window-16-solid",
    ];
    const found = candidates.some((key) => Boolean(resolveIconDataUri(key)));
    expect(found).toBe(true);
  });

  it("supports runtime icon set registration and cleanup", () => {
    registerRuntimeIconSet("runtime", {
      "custom-node": SIMPLE_SVG,
    });

    const uri = resolveIconDataUri("runtime:custom-node");
    expect(uri).toBeTruthy();
    expect(uri?.startsWith("data:image/svg+xml")).toBe(true);

    unregisterRuntimeIconSet("runtime");
    expect(resolveIconDataUri("runtime:custom-node")).toBeUndefined();
  });
});
