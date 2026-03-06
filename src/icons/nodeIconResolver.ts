import { resolveIconDataUri } from "@/icons/iconRegistry";
import { getMiddlewareIconByType } from "@/utils/middlewareCatalog";

export type ApplicationTypeKey =
  | "frontend"
  | "backend"
  | "gateway"
  | "batch_job"
  | "microservice"
  | "other";

export interface ResolvedNodeIcon {
  iconKey: string;
  src: string;
  alt: string;
}

const APPLICATION_ICON_ALTS: Record<ApplicationTypeKey, string> = {
  frontend: "前端应用",
  backend: "后端应用",
  gateway: "网关应用",
  batch_job: "批处理应用",
  microservice: "微服务应用",
  other: "应用服务",
};

const APPLICATION_ICON_CANDIDATES: Record<ApplicationTypeKey, string[]> = {
  frontend: ["heroicons:window", "heroicons:window-20-solid", "heroicons:window-16-solid"],
  backend: ["heroicons:server-stack", "heroicons:server-stack-20-solid", "heroicons:server-stack-16-solid"],
  gateway: [
    "heroicons:arrows-right-left",
    "heroicons:arrows-right-left-20-solid",
    "heroicons:arrows-right-left-16-solid",
  ],
  batch_job: ["heroicons:clock", "heroicons:clock-20-solid", "heroicons:clock-16-solid"],
  microservice: ["heroicons:squares-2x2", "heroicons:squares-2x2-20-solid", "heroicons:squares-2x2-16-solid"],
  other: ["heroicons:cube", "heroicons:cube-20-solid", "heroicons:cube-16-solid"],
};

const DEFAULT_ICON_KEY = "local-middleware:default-middleware";
const APPLICATION_TYPE_ALIASES: Record<string, ApplicationTypeKey> = {
  frontend: "frontend",
  backend: "backend",
  gateway: "gateway",
  batchjob: "batch_job",
  batch_job: "batch_job",
  microservice: "microservice",
  micro_service: "microservice",
  other: "other",
};

function extractStringValue(value: unknown): string | undefined {
  return typeof value === "string" ? value.trim() : undefined;
}

function resolveIconUriFromCandidates(candidates: string[]): { iconKey: string; src: string } | null {
  for (const iconKey of candidates) {
    const src = resolveIconDataUri(iconKey);
    if (src) {
      return { iconKey, src };
    }
  }
  return null;
}

function resolveDefaultIconUri(): { iconKey: string; src: string } {
  const fallback = resolveIconDataUri(DEFAULT_ICON_KEY);
  if (fallback) {
    return { iconKey: DEFAULT_ICON_KEY, src: fallback };
  }
  return {
    iconKey: DEFAULT_ICON_KEY,
    src: "",
  };
}

export function resolveApplicationTypeKey(rawType?: string): ApplicationTypeKey {
  const normalized = (rawType ?? "").trim().toLowerCase().replace(/[\s-]+/g, "_");
  if (!normalized) return "other";
  return APPLICATION_TYPE_ALIASES[normalized] ?? "other";
}

export function resolveApplicationNodeIcon(rawType?: string, customIconKey?: string): ResolvedNodeIcon {
  const explicitIconKey = extractStringValue(customIconKey);
  if (explicitIconKey) {
    const explicitIcon = resolveIconDataUri(explicitIconKey);
    if (explicitIcon) {
      return {
        iconKey: explicitIconKey,
        src: explicitIcon,
        alt: "应用服务",
      };
    }
  }

  const appTypeKey = resolveApplicationTypeKey(rawType);
  const match = resolveIconUriFromCandidates(APPLICATION_ICON_CANDIDATES[appTypeKey]);
  if (match) {
    return {
      iconKey: match.iconKey,
      src: match.src,
      alt: APPLICATION_ICON_ALTS[appTypeKey],
    };
  }

  const fallback = resolveDefaultIconUri();
  return {
    iconKey: fallback.iconKey,
    src: fallback.src,
    alt: APPLICATION_ICON_ALTS[appTypeKey],
  };
}

export function resolveMiddlewareNodeIcon(extra?: Record<string, unknown>): ResolvedNodeIcon {
  const customIconKey = extractStringValue(extra?.icon_key);
  if (customIconKey) {
    const icon = resolveIconDataUri(customIconKey);
    if (icon) {
      return {
        iconKey: customIconKey,
        src: icon,
        alt: "中间件",
      };
    }
  }

  const category = extractStringValue(extra?.category);
  const type = extractStringValue(extra?.type);
  const icon = getMiddlewareIconByType(type, category);

  return {
    iconKey: icon.iconKey || icon.key,
    src: icon.src,
    alt: icon.alt,
  };
}
