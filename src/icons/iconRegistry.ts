import heroicons from "@iconify-json/heroicons/icons.json";
import { getIconData, iconToSVG } from "@iconify/utils";

interface ParsedIconKey {
  collection: string;
  name: string;
}

const builtinIconifyCollections = new Map<string, IconifyJSON>();
const builtinSvgCollections = new Map<string, Record<string, string>>();
const runtimeSvgCollections = new Map<string, Record<string, string>>();
type IconifyJSON = NonNullable<Parameters<typeof getIconData>[0]>;

const localMiddlewareIconModules = import.meta.glob("../assets/middleware-icons/*.svg", {
  eager: true,
  import: "default",
  query: "?raw",
}) as Record<string, string>;

function encodeSvgDataUri(svgContent: string): string {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svgContent)}`;
}

function upsertSvgRootAttribute(openTag: string, name: string, value: string): string {
  const attributePattern = new RegExp(`\\s${name}=("[^"]*"|'[^']*')`, "i");
  if (attributePattern.test(openTag)) {
    return openTag.replace(attributePattern, ` ${name}="${value}"`);
  }

  return openTag.replace(/>$/, ` ${name}="${value}">`);
}

function normalizeSvgMarkup(raw: string): string {
  const normalized = raw.replace(/\r\n/g, "\n").trim();
  const openTagMatch = normalized.match(/^<svg\b[^>]*>/i);
  if (!openTagMatch) return normalized;

  let openTag = openTagMatch[0];
  openTag = openTag.replace(/\s(?:width|height)=("[^"]*"|'[^']*')/gi, "");
  openTag = upsertSvgRootAttribute(openTag, "preserveAspectRatio", "xMidYMid meet");

  return normalized.replace(/^<svg\b[^>]*>/i, openTag);
}

function parseIconKey(iconKey?: string): ParsedIconKey | null {
  const normalized = (iconKey ?? "").trim();
  if (!normalized) return null;

  const separatorIndex = normalized.indexOf(":");
  if (separatorIndex <= 0 || separatorIndex === normalized.length - 1) {
    return null;
  }

  const collection = normalized.slice(0, separatorIndex).trim();
  const name = normalized.slice(separatorIndex + 1).trim();
  if (!collection || !name) return null;
  return { collection, name };
}

function buildSvgCollectionFromRaw(rawIcons: Record<string, string>): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [name, svg] of Object.entries(rawIcons)) {
    const normalizedName = name.trim();
    if (!normalizedName) continue;
    result[normalizedName] = encodeSvgDataUri(normalizeSvgMarkup(svg));
  }
  return result;
}

function escapeHtmlAttribute(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("\"", "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function resolveIconifyDataUri(collection: IconifyJSON, iconName: string): string | undefined {
  const iconData = getIconData(collection, iconName);
  if (!iconData) return undefined;

  const rendered = iconToSVG(iconData, {
    height: 20,
    width: 20,
  });

  const attrs = Object.entries(rendered.attributes)
    .map(([key, value]) => `${key}="${escapeHtmlAttribute(String(value))}"`)
    .join(" ");
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" ${attrs}>${rendered.body}</svg>`;
  return encodeSvgDataUri(normalizeSvgMarkup(svg));
}

function bootstrapBuiltinCollections() {
  const middlewareIcons: Record<string, string> = {};
  for (const [path, svgContent] of Object.entries(localMiddlewareIconModules)) {
    const fileName = path.split("/").pop() || "";
    const iconName = fileName.replace(/\.svg$/i, "").trim();
    if (!iconName) continue;
    middlewareIcons[iconName] = svgContent;
  }

  builtinSvgCollections.set("local-middleware", buildSvgCollectionFromRaw(middlewareIcons));
  builtinIconifyCollections.set(heroicons.prefix, heroicons as IconifyJSON);
}

bootstrapBuiltinCollections();

export function resolveIconDataUri(iconKey?: string): string | undefined {
  const parsed = parseIconKey(iconKey);
  if (!parsed) return undefined;

  const runtimeCollection = runtimeSvgCollections.get(parsed.collection);
  if (runtimeCollection?.[parsed.name]) {
    return runtimeCollection[parsed.name];
  }

  const svgCollection = builtinSvgCollections.get(parsed.collection);
  if (svgCollection?.[parsed.name]) {
    return svgCollection[parsed.name];
  }

  const iconifyCollection = builtinIconifyCollections.get(parsed.collection);
  if (iconifyCollection) {
    return resolveIconifyDataUri(iconifyCollection, parsed.name);
  }

  return undefined;
}

export function registerRuntimeIconSet(setId: string, icons: Record<string, string>) {
  const normalizedSetId = setId.trim();
  if (!normalizedSetId) return;
  runtimeSvgCollections.set(normalizedSetId, buildSvgCollectionFromRaw(icons));
}

export function unregisterRuntimeIconSet(setId: string) {
  runtimeSvgCollections.delete(setId.trim());
}

export function clearRuntimeIconSetsForTest() {
  runtimeSvgCollections.clear();
}
