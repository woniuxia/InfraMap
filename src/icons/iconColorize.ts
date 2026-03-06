const SVG_DATA_URI_PREFIX = "data:image/svg+xml";

const colorizedSvgCache = new Map<string, string>();

function upsertSvgAttribute(openTag: string, name: string, value: string): string {
  const attributePattern = new RegExp(`\\s${name}=("[^"]*"|'[^']*')`, "i");
  if (attributePattern.test(openTag)) {
    return openTag.replace(attributePattern, ` ${name}="${value}"`);
  }

  return openTag.replace(/>$/, ` ${name}="${value}">`);
}

export function colorizeSvgDataUri(iconSrc: string | undefined, color: string): string {
  if (!iconSrc || !color || !iconSrc.startsWith(SVG_DATA_URI_PREFIX)) {
    return iconSrc || "";
  }

  const cacheKey = `${color}::${iconSrc}`;
  const cached = colorizedSvgCache.get(cacheKey);
  if (cached) return cached;

  const dataSeparatorIndex = iconSrc.indexOf(",");
  if (dataSeparatorIndex < 0) return iconSrc;

  const prefix = iconSrc.slice(0, dataSeparatorIndex + 1);
  const encodedSvg = iconSrc.slice(dataSeparatorIndex + 1);

  let svgMarkup = "";
  try {
    svgMarkup = decodeURIComponent(encodedSvg);
  } catch {
    return iconSrc;
  }

  const openTagMatch = svgMarkup.match(/^<svg\b[^>]*>/i);
  if (!openTagMatch) return iconSrc;

  let openTag = openTagMatch[0];
  openTag = upsertSvgAttribute(openTag, "color", color);
  openTag = upsertSvgAttribute(openTag, "fill", color);
  openTag = upsertSvgAttribute(openTag, "stroke", color);

  const colorizedSvg = svgMarkup.replace(/^<svg\b[^>]*>/i, openTag);
  const colorizedUri = `${prefix}${encodeURIComponent(colorizedSvg)}`;
  colorizedSvgCache.set(cacheKey, colorizedUri);
  return colorizedUri;
}

export function clearColorizedSvgCacheForTest() {
  colorizedSvgCache.clear();
}
