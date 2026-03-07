import { describe, expect, it } from "vitest";
import type { Application, Host, Middleware, NginxConfig } from "@/types";
import {
  buildApplicationCopyDraft,
  buildCopyName,
  buildHostCopyDraft,
  buildMiddlewareCopyDraft,
  buildNginxCopyDraft,
  formatCopyTimestamp,
  stripSystemFields,
} from "@/utils/resourceCopy";

describe("resourceCopy", () => {
  const fixedDate = new Date(2026, 1, 28, 9, 5, 7);

  it("formats copy timestamp as YYYYMMDDHHmmss", () => {
    expect(formatCopyTimestamp(fixedDate)).toBe("20260228090507");
  });

  it("builds copy name with timestamp suffix", () => {
    expect(buildCopyName("app-core", fixedDate)).toBe("app-core（副本 20260228090507）");
  });

  it("strips system fields while keeping business fields", () => {
    const host: Host = {
      id: "h1",
      hostname: "host-01",
      env: "prod",
      status: "running",
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-02T00:00:00Z",
    };

    const stripped = stripSystemFields(host);

    expect(stripped.hostname).toBe("host-01");
    expect(stripped.id).toBeUndefined();
    expect(stripped.created_at).toBeUndefined();
    expect(stripped.updated_at).toBeUndefined();
  });

  it("builds host copy draft without legacy ip fields", () => {
    const host: Host = {
      id: "h2",
      hostname: "db-host",
      env: "dev",
      status: "running",
      os_type: "Ubuntu 22.04",
      tags: "[\"db\"]",
      created_at: "2026-01-03T00:00:00Z",
      updated_at: "2026-01-03T00:00:00Z",
    };

    const draft = buildHostCopyDraft(host, fixedDate);

    expect(draft.hostname).toBe("db-host（副本 20260228090507）");
    expect(draft.ip_display).toBeUndefined();
    expect(draft.os_type).toBe("Ubuntu 22.04");
    expect(draft.tags).toBe("[\"db\"]");
    expect(draft.id).toBeUndefined();
  });

  it("builds application copy draft with copied name and no system fields", () => {
    const app: Application = {
      id: "a1",
      name: "payment-api",
      type: "backend",
      address: "10.0.1.10",
      port: 8080,
      env: "prod",
      status: "running",
      owners: ["ops", "devops"],
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    };

    const draft = buildApplicationCopyDraft(app, fixedDate);
    expect(draft.name).toBe("payment-api（副本 20260228090507）");
    expect(draft.address).toBe("10.0.1.10");
    expect(draft.owners).toEqual(["ops", "devops"]);
    expect(draft.id).toBeUndefined();
  });

  it("builds middleware copy draft with copied name and preserved fields", () => {
    const middleware: Middleware = {
      id: "m1",
      name: "redis-cache",
      category: "cache",
      type: "Redis",
      address: "10.0.2.20",
      port: 6379,
      env: "test",
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    };

    const draft = buildMiddlewareCopyDraft(middleware, fixedDate);
    expect(draft.name).toBe("redis-cache（副本 20260228090507）");
    expect(draft.category).toBe("cache");
    expect(draft.id).toBeUndefined();
  });

  it("builds nginx config copy draft with copied name and preserved fields", () => {
    const nginxConfig: NginxConfig = {
      id: "n1",
      name: "gateway-main",
      endpoints: [{ host: "10.0.9.1", port: 80 }],
      strategy: "roundrobin",
      env: "prod",
      status: "running",
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    };

    const draft = buildNginxCopyDraft(nginxConfig, fixedDate);
    expect(draft.name).toBe("gateway-main（副本 20260228090507）");
    expect(draft.endpoints).toEqual([{ host: "10.0.9.1", port: 80 }]);
    expect(draft.id).toBeUndefined();
  });
});
