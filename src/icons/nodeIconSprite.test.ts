import { afterEach, describe, expect, it } from "vitest";
import {
  buildNodeIconSpriteDataUri,
  clearNodeIconSpriteCacheForTest,
} from "@/icons/nodeIconSprite";

const SVG_URI_PREFIX = "data:image/svg+xml;charset=utf-8,";

function encodeSvg(svg: string): string {
  return `${SVG_URI_PREFIX}${encodeURIComponent(svg)}`;
}

function decodeSvg(uri: string): string {
  expect(uri.startsWith(SVG_URI_PREFIX)).toBe(true);
  return decodeURIComponent(uri.slice(SVG_URI_PREFIX.length));
}

describe("nodeIconSprite", () => {
  afterEach(() => {
    clearNodeIconSpriteCacheForTest();
  });

  it("builds a fixed 48x48 sprite for svg icons that already have viewBox", () => {
    const icon = encodeSvg(
      '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><path d="M2 2h16v16H2z"/></svg>',
    );

    const sprite = buildNodeIconSpriteDataUri(icon, "#5ca3ff");
    const decoded = decodeSvg(sprite);

    expect(decoded).toContain('width="48"');
    expect(decoded).toContain('height="48"');
    expect(decoded).toContain('viewBox="0 0 48 48"');
    expect(decoded).toContain(
      '<svg x="8" y="8" width="32" height="32" viewBox="0 0 20 20" preserveAspectRatio="xMidYMid meet" color="#5ca3ff" fill="#5ca3ff" stroke="#5ca3ff">',
    );
    expect(decoded).toContain('<path d="M2 2h16v16H2z"/>');
  });

  it("derives viewBox from width and height when source svg has no viewBox", () => {
    const icon = encodeSvg(
      '<?xml version="1.0" encoding="UTF-8"?><!-- icon --><svg xmlns="http://www.w3.org/2000/svg" width="18" height="30"><circle cx="9" cy="15" r="8"/></svg>',
    );

    const sprite = buildNodeIconSpriteDataUri(icon, "#41c58a");
    const decoded = decodeSvg(sprite);

    expect(decoded).toContain('viewBox="0 0 18 30"');
    expect(decoded).toContain('preserveAspectRatio="xMidYMid meet"');
    expect(decoded).toContain('color="#41c58a"');
  });

  it("returns non-svg data uris as-is", () => {
    const pngUri = "data:image/png;base64,AAAA";

    expect(buildNodeIconSpriteDataUri(pngUri, "#5ca3ff")).toBe(pngUri);
  });
});
