const SVG_DATA_URI_PREFIX = "data:image/svg+xml";
const FALLBACK_VIEW_BOX = "0 0 24 24";
const SPRITE_SIZE = 48;
const ICON_SLOT_OFFSET = 8;
const ICON_SLOT_SIZE = 32;

const nodeIconSpriteCache = new Map<string, string>();

function readSvgAttribute(openTag: string, attribute: string): string | undefined {
  const attributePattern = new RegExp(`\\b${attribute}=("[^"]*"|'[^']*')`, "i");
  const matched = openTag.match(attributePattern);
  if (!matched?.[1]) return undefined;
  return matched[1].slice(1, -1).trim();
}

function parsePositiveDimension(value: string | undefined): number | undefined {
  if (!value) return undefined;
  const numeric = Number.parseFloat(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return undefined;
  return numeric;
}

function deriveViewBox(openTag: string): string {
  const viewBox = readSvgAttribute(openTag, "viewBox");
  if (viewBox) return viewBox;

  const width = parsePositiveDimension(readSvgAttribute(openTag, "width"));
  const height = parsePositiveDimension(readSvgAttribute(openTag, "height"));
  if (width && height) return `0 0 ${width} ${height}`;

  return FALLBACK_VIEW_BOX;
}

function escapeAttribute(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function stripLeadingSvgPreamble(svgMarkup: string): string {
  return svgMarkup
    .replace(/^\uFEFF/, "")
    .replace(/^\s*(?:(?:<\?xml[\s\S]*?\?>)|(?:<!DOCTYPE[\s\S]*?>)|(?:<!--[\s\S]*?-->))*\s*/i, "")
    .trim();
}

function extractRootSvgParts(svgMarkup: string): { openTag: string; innerContent: string } | null {
  const normalized = stripLeadingSvgPreamble(svgMarkup);
  const openTagMatch = normalized.match(/<svg\b[^>]*>/i);
  if (!openTagMatch || typeof openTagMatch.index !== "number") return null;

  const openTag = openTagMatch[0];
  const contentStart = openTagMatch.index + openTag.length;

  if (openTag.endsWith("/>")) {
    return {
      openTag,
      innerContent: "",
    };
  }

  const closeTagIndex = normalized.toLowerCase().lastIndexOf("</svg>");
  if (closeTagIndex < contentStart) return null;

  return {
    openTag,
    innerContent: normalized.slice(contentStart, closeTagIndex).trim(),
  };
}

export function buildNodeIconSpriteDataUri(iconSrc: string | undefined, color: string): string {
  if (!iconSrc || !iconSrc.startsWith(SVG_DATA_URI_PREFIX)) {
    return iconSrc || "";
  }

  const cacheKey = `${color}::${iconSrc}`;
  const cached = nodeIconSpriteCache.get(cacheKey);
  if (cached) return cached;

  const dataSeparatorIndex = iconSrc.indexOf(",");
  if (dataSeparatorIndex < 0) return iconSrc;

  const prefix = iconSrc.slice(0, dataSeparatorIndex + 1);
  const encodedSvg = iconSrc.slice(dataSeparatorIndex + 1);

  let decodedSvg = "";
  try {
    decodedSvg = decodeURIComponent(encodedSvg);
  } catch {
    return iconSrc;
  }

  const svgParts = extractRootSvgParts(decodedSvg);
  if (!svgParts) return iconSrc;

  const viewBox = deriveViewBox(svgParts.openTag);
  const escapedViewBox = escapeAttribute(viewBox);
  const escapedColor = escapeAttribute(color);

  const spriteSvg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="${SPRITE_SIZE}" height="${SPRITE_SIZE}" viewBox="0 0 ${SPRITE_SIZE} ${SPRITE_SIZE}">` +
    `<svg x="${ICON_SLOT_OFFSET}" y="${ICON_SLOT_OFFSET}" width="${ICON_SLOT_SIZE}" height="${ICON_SLOT_SIZE}" viewBox="${escapedViewBox}" preserveAspectRatio="xMidYMid meet" color="${escapedColor}" fill="${escapedColor}" stroke="${escapedColor}">` +
    `${svgParts.innerContent}</svg></svg>`;

  const spriteUri = `${prefix}${encodeURIComponent(spriteSvg)}`;
  nodeIconSpriteCache.set(cacheKey, spriteUri);
  return spriteUri;
}

export function clearNodeIconSpriteCacheForTest() {
  nodeIconSpriteCache.clear();
}
