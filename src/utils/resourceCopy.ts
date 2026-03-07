import type { Application, Host, Middleware, NginxConfig } from "@/types";

const COPY_DEFAULT_NAME = "未命名";

function pad2(value: number): string {
  return String(value).padStart(2, "0");
}

export function formatCopyTimestamp(date: Date = new Date()): string {
  const year = date.getFullYear();
  const month = pad2(date.getMonth() + 1);
  const day = pad2(date.getDate());
  const hour = pad2(date.getHours());
  const minute = pad2(date.getMinutes());
  const second = pad2(date.getSeconds());
  return `${year}${month}${day}${hour}${minute}${second}`;
}

export function buildCopyName(base: string, date: Date = new Date()): string {
  const normalized = base.trim() || COPY_DEFAULT_NAME;
  return `${normalized}（副本 ${formatCopyTimestamp(date)}）`;
}

export function stripSystemFields<T extends object>(row: T): Partial<T> {
  const source = row as Record<string, unknown>;
  const { id, created_at, updated_at, ...rest } = source;
  void id;
  void created_at;
  void updated_at;
  return rest as Partial<T>;
}

export function buildHostCopyDraft(row: Host, date: Date = new Date()): Partial<Host> {
  const draft = stripSystemFields(row) as Partial<Host>;
  delete draft.ip_display;
  return {
    ...draft,
    hostname: buildCopyName(row.hostname, date),
  };
}

export function buildApplicationCopyDraft(
  row: Application,
  date: Date = new Date()
): Partial<Application> {
  return {
    ...stripSystemFields(row),
    name: buildCopyName(row.name, date),
  };
}

export function buildMiddlewareCopyDraft(
  row: Middleware,
  date: Date = new Date()
): Partial<Middleware> {
  return {
    ...stripSystemFields(row),
    name: buildCopyName(row.name, date),
  };
}

export function buildNginxCopyDraft(
  row: NginxConfig,
  date: Date = new Date()
): Partial<NginxConfig> {
  return {
    ...stripSystemFields(row),
    name: buildCopyName(row.name, date),
  };
}
