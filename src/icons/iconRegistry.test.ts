import { afterEach, describe, expect, it } from "vitest";
import {
  clearRuntimeIconSetsForTest,
  registerRuntimeIconSet,
  resolveIconDataUri,
  unregisterRuntimeIconSet,
} from "@/icons/iconRegistry";

const FIXED_SIZE_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path d="M0 0h24v24H0z"/></svg>`;

function decodeSvgDataUri(uri: string): string {
  const prefix = "data:image/svg+xml;charset=utf-8,";
  expect(uri.startsWith(prefix)).toBe(true);
  return decodeURIComponent(uri.slice(prefix.length));
}

describe("iconRegistry", () => {
  afterEach(() => {
    clearRuntimeIconSetsForTest();
  });

  it("resolves builtin middleware svg icons", () => {
    const uri = resolveIconDataUri("local-middleware:redis");
    expect(uri).toBeTruthy();
    expect(uri?.startsWith("data:image/svg+xml")).toBe(true);

    const svg = decodeSvgDataUri(String(uri));
    expect(svg).toContain('preserveAspectRatio="xMidYMid meet"');
  });

  it("normalizes builtin heroicons icons for Cytoscape background rendering", () => {
    const candidates = [
      "heroicons:window",
      "heroicons:window-20-solid",
      "heroicons:window-16-solid",
    ];
    const resolved = candidates
      .map((key) => resolveIconDataUri(key))
      .find((value) => Boolean(value));

    expect(resolved).toBeTruthy();

    const svg = decodeSvgDataUri(String(resolved));
    expect(svg).toContain('preserveAspectRatio="xMidYMid meet"');
    expect(svg).not.toMatch(/<svg\b[^>]*\swidth=/i);
    expect(svg).not.toMatch(/<svg\b[^>]*\sheight=/i);
  });

  it("supports runtime icon set registration and cleanup", () => {
    registerRuntimeIconSet("runtime", {
      "custom-node": FIXED_SIZE_SVG,
    });

    const uri = resolveIconDataUri("runtime:custom-node");
    expect(uri).toBeTruthy();
    expect(uri?.startsWith("data:image/svg+xml")).toBe(true);

    const svg = decodeSvgDataUri(String(uri));
    expect(svg).toContain('preserveAspectRatio="xMidYMid meet"');
    expect(svg).not.toMatch(/<svg\b[^>]*\swidth=/i);
    expect(svg).not.toMatch(/<svg\b[^>]*\sheight=/i);

    unregisterRuntimeIconSet("runtime");
    expect(resolveIconDataUri("runtime:custom-node")).toBeUndefined();
  });
});
