import type { Host } from "@/types";

type TagType = "primary" | "success" | "warning" | "info" | "danger";

export function parseHostTags(raw?: string): string[] {
  if (!raw) {
    return [];
  }
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [];
    }
    return Array.from(
      new Set(
        parsed
          .filter((item: unknown) => typeof item === "string")
          .map((item: string) => item.trim())
          .filter((item) => item.length > 0)
      )
    );
  } catch {
    return [];
  }
}

export function splitIpDisplay(ipDisplay?: string): string[] {
  if (!ipDisplay) {
    return [];
  }
  return ipDisplay
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

export function summarizeIpDisplay(ipDisplay?: string): {
  primary: string;
  extraCount: number;
  all: string[];
} {
  const all = splitIpDisplay(ipDisplay);
  if (all.length === 0) {
    return { primary: "-", extraCount: 0, all: [] };
  }
  return {
    primary: all[0],
    extraCount: Math.max(0, all.length - 1),
    all,
  };
}

function buildCpuSummary(host: Partial<Host>): string[] {
  const segments: string[] = [];
  if (host.cpu_model?.trim()) {
    segments.push(host.cpu_model.trim());
  }

  if (host.cpu_cores || host.cpu_threads) {
    const cores = host.cpu_cores ? `${host.cpu_cores}C` : "-";
    const threads = host.cpu_threads ? `${host.cpu_threads}T` : "-";
    segments.push(`${cores}${threads}`);
  }

  if (host.cpu_freq?.trim()) {
    segments.push(`${host.cpu_freq.trim()}GHz`);
  }
  return segments;
}

export function buildHardwareSummary(host: Partial<Host>): string {
  const segments = [...buildCpuSummary(host)];
  if (host.ram_gb) {
    segments.push(`${host.ram_gb}GB RAM`);
  }
  if (host.disk_gb) {
    segments.push(`${host.disk_gb}GB Disk`);
  }
  return segments.length > 0 ? segments.join(" / ") : "-";
}

export function envLabel(env: string): string {
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env;
}

export function envTagType(env: string): TagType {
  const map: Record<string, TagType> = {
    prod: "danger",
    dev: "info",
    test: "warning",
  };
  return map[env] || "info";
}

export function statusLabel(status: string): string {
  return ({ running: "运行中", stopped: "已停止", maintenance: "维护中" } as Record<string, string>)[status] || status;
}

export function statusTagType(status: string): TagType {
  const map: Record<string, TagType> = {
    running: "success",
    stopped: "danger",
    maintenance: "warning",
  };
  return map[status] || "info";
}
