import { describe, expect, it } from "vitest";
import { getMiddlewareDefaultPort, getMiddlewareTypeOptions } from "@/utils/middlewareCatalog";

describe("middlewareCatalog", () => {
  it("returns common database middleware types", () => {
    expect(getMiddlewareTypeOptions("database")).toEqual([
      "MySQL",
      "PostgreSQL",
      "MariaDB",
      "SQL Server",
      "Oracle",
      "MongoDB",
      "ClickHouse",
    ]);
  });

  it("returns category defaults and appends current type when missing", () => {
    expect(getMiddlewareTypeOptions("cache", "Memcached")).toEqual([
      "Redis",
      "KeyDB",
      "Tair",
      "Couchbase",
      "Memcached",
    ]);
  });

  it("returns only deduplicated current type for unknown categories", () => {
    expect(getMiddlewareTypeOptions("unknown", "Custom")).toEqual(["Custom"]);
    expect(getMiddlewareTypeOptions("unknown", "   ")).toEqual([]);
  });

  it("returns default ports for known middleware types", () => {
    expect(getMiddlewareDefaultPort("MySQL")).toBe(3306);
    expect(getMiddlewareDefaultPort("postgres")).toBe(5432);
    expect(getMiddlewareDefaultPort("SQL Server")).toBe(1433);
    expect(getMiddlewareDefaultPort("Redis")).toBe(6379);
    expect(getMiddlewareDefaultPort("RabbitMQ")).toBe(5672);
    expect(getMiddlewareDefaultPort("ZooKeeper")).toBe(2181);
  });

  it("supports normalized type aliases and returns undefined for unknown types", () => {
    expect(getMiddlewareDefaultPort("  open-search  ")).toBe(9200);
    expect(getMiddlewareDefaultPort("active_mq")).toBe(61616);
    expect(getMiddlewareDefaultPort("custom")).toBeUndefined();
    expect(getMiddlewareDefaultPort("")).toBeUndefined();
  });
});
