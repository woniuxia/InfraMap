import { afterEach, describe, expect, it } from "vitest";
import { clearColorizedSvgCacheForTest, colorizeSvgDataUri } from "@/icons/iconColorize";

const SVG_URI_PREFIX = "data:image/svg+xml;charset=utf-8,";

function encodeSvg(svg: string): string {
  return `${SVG_URI_PREFIX}${encodeURIComponent(svg)}`;
}

function decodeSvg(uri: string): string {
  expect(uri.startsWith(SVG_URI_PREFIX)).toBe(true);
  return decodeURIComponent(uri.slice(SVG_URI_PREFIX.length));
}

describe("iconColorize", () => {
  afterEach(() => {
    clearColorizedSvgCacheForTest();
  });

  it("tints currentColor svg icons with the provided color", () => {
    const icon = encodeSvg('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" d="M0 0h24v24H0z"/></svg>');

    const tinted = colorizeSvgDataUri(icon, "#5ca3ff");

    expect(decodeSvg(tinted)).toContain('color="#5ca3ff"');
  });

  it("tints fill-less svg paths instead of leaving them black", () => {
    const icon = encodeSvg('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M0 0h24v24H0z"/></svg>');

    const tinted = colorizeSvgDataUri(icon, "#41c58a");

    expect(decodeSvg(tinted)).toContain('fill="#41c58a"');
  });

  it("preserves explicitly colored svg content", () => {
    const icon = encodeSvg('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#00838F" d="M0 0h24v24H0z"/></svg>');

    const tinted = colorizeSvgDataUri(icon, "#5ca3ff");
    const decoded = decodeSvg(tinted);

    expect(decoded).toContain('color="#5ca3ff"');
    expect(decoded).toContain('fill="#00838F"');
  });
});
