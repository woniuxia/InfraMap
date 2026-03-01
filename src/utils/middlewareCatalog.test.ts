import { describe, expect, it } from "vitest";
import {
  getMiddlewareCategoryLabel,
  getMiddlewareDefaultPort,
  getMiddlewareIcon,
  getMiddlewareIconByType,
  getMiddlewareTypeOptions,
  getMiddlewareTypeOptionsWithIcon,
} from "@/utils/middlewareCatalog";

describe("middlewareCatalog", () => {
  it("returns common database middleware types", () => {
    expect(getMiddlewareTypeOptions("database")).toEqual(
      expect.arrayContaining([
        "MySQL",
        "PostgreSQL",
        "MariaDB",
        "SQL Server",
        "Oracle",
        "MongoDB",
        "ClickHouse",
      ])
    );
  });

  it("includes expanded common and xinchuang middleware types by category", () => {
    expect(getMiddlewareTypeOptions("database")).toEqual(
      expect.arrayContaining([
        "TiDB",
        "openGauss",
        "Apache Doris",
        "DM Database",
        "KingbaseES",
        "GaussDB",
        "OceanBase",
      ])
    );
    expect(getMiddlewareTypeOptions("message_queue")).toEqual(
      expect.arrayContaining(["EMQX", "NATS", "Redpanda"])
    );
    expect(getMiddlewareTypeOptions("cache")).toEqual(expect.arrayContaining(["Memcached"]));
    expect(getMiddlewareTypeOptions("search_engine")).toEqual(expect.arrayContaining(["Typesense"]));
    expect(getMiddlewareTypeOptions("config_center")).toEqual(
      expect.arrayContaining(["Spring Cloud Config"])
    );
    expect(getMiddlewareTypeOptions("other")).toEqual(
      expect.arrayContaining(["MinIO", "Apache APISIX", "TongWeb", "GoldenDB"])
    );
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

  it("returns middleware type options with icon metadata", () => {
    const options = getMiddlewareTypeOptionsWithIcon("cache", "Memcached");
    const tail = options[options.length - 1];
    expect(options[0].value).toBe("Redis");
    expect(options[0].icon.alt).toBe("Redis");
    expect(options[0].icon.isFallback).toBe(false);
    expect(tail?.value).toBe("Memcached");
    expect(tail?.icon.key).toBe("memcached");
    expect(tail?.icon.isFallback).toBe(false);
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

  it("returns default ports for newly added common and xinchuang middleware", () => {
    expect(getMiddlewareDefaultPort("TiDB")).toBe(4000);
    expect(getMiddlewareDefaultPort("openGauss")).toBe(5432);
    expect(getMiddlewareDefaultPort("Apache Doris")).toBe(9030);
    expect(getMiddlewareDefaultPort("EMQX")).toBe(1883);
    expect(getMiddlewareDefaultPort("NATS")).toBe(4222);
    expect(getMiddlewareDefaultPort("Memcached")).toBe(11211);
    expect(getMiddlewareDefaultPort("MinIO")).toBe(9000);
    expect(getMiddlewareDefaultPort("Apache APISIX")).toBe(9180);
    expect(getMiddlewareDefaultPort("TongWeb")).toBe(9060);
    expect(getMiddlewareDefaultPort("KingbaseES")).toBe(54321);
    expect(getMiddlewareDefaultPort("DM Database")).toBe(5236);
  });

  it("supports normalized type aliases and returns undefined for unknown types", () => {
    expect(getMiddlewareDefaultPort("  open-search  ")).toBe(9200);
    expect(getMiddlewareDefaultPort("active_mq")).toBe(61616);
    expect(getMiddlewareDefaultPort("custom")).toBeUndefined();
    expect(getMiddlewareDefaultPort("")).toBeUndefined();
  });

  it("supports xinchuang chinese aliases for default ports", () => {
    expect(getMiddlewareDefaultPort("人大金仓")).toBe(54321);
    expect(getMiddlewareDefaultPort("金仓")).toBe(54321);
    expect(getMiddlewareDefaultPort("达梦")).toBe(5236);
    expect(getMiddlewareDefaultPort("东方通")).toBe(9060);
  });

  it("returns dedicated icon for common middleware types", () => {
    const icon = getMiddlewareIcon("cache", "Redis");
    expect(icon.key).toBe("redis");
    expect(icon.isFallback).toBe(false);
    expect(icon.alt).toBe("Redis");
    expect(icon.src.startsWith("data:image/svg+xml")).toBe(true);
  });

  it("returns type-first icon info and keeps category for fallback", () => {
    const known = getMiddlewareIconByType("Kafka", "message_queue");
    const unknown = getMiddlewareIconByType("CustomBus", "message_queue");
    expect(known.key).toBe("kafka");
    expect(known.isFallback).toBe(false);
    expect(unknown.key).toBe("message_queue");
    expect(unknown.isFallback).toBe(true);
  });

  it("returns dedicated icons for selected newly-added middleware types", () => {
    const memcached = getMiddlewareIconByType("Memcached", "cache");
    const tidb = getMiddlewareIconByType("TiDB", "database");
    const emqx = getMiddlewareIconByType("EMQX", "message_queue");
    const nats = getMiddlewareIconByType("NATS", "message_queue");
    const minio = getMiddlewareIconByType("MinIO", "other");

    expect(memcached.isFallback).toBe(false);
    expect(tidb.isFallback).toBe(false);
    expect(emqx.isFallback).toBe(false);
    expect(nats.isFallback).toBe(false);
    expect(minio.isFallback).toBe(false);
  });

  it("resolves type aliases to the same icon mapping", () => {
    const icon = getMiddlewareIcon("search_engine", "open-search");
    const direct = getMiddlewareIcon("search_engine", "OpenSearch");
    expect(icon.key).toBe("opensearch");
    expect(icon.isFallback).toBe(false);
    expect(icon.alt).toBe("OpenSearch");
    expect(icon.src).toBe(direct.src);
  });

  it("falls back to category icon for unknown middleware type", () => {
    const icon = getMiddlewareIcon("message_queue", "CustomMQ");
    const categoryOnly = getMiddlewareIcon("message_queue", undefined);
    expect(icon.key).toBe("message_queue");
    expect(icon.isFallback).toBe(true);
    expect(icon.alt).toBe("消息队列");
    expect(icon.src).toBe(categoryOnly.src);
  });

  it("falls back to default middleware icon for unknown type and category", () => {
    const icon = getMiddlewareIcon("unknown", "Custom");
    const categoryFallback = getMiddlewareIcon("cache", "Custom");
    expect(icon.key).toBe("middleware");
    expect(icon.isFallback).toBe(true);
    expect(icon.alt).toBe("中间件");
    expect(icon.src.startsWith("data:image/svg+xml")).toBe(true);
    expect(icon.src).not.toBe(categoryFallback.src);
  });

  it("returns stable labels for middleware categories", () => {
    expect(getMiddlewareCategoryLabel("database")).toBe("数据库");
    expect(getMiddlewareCategoryLabel("message_queue")).toBe("消息队列");
    expect(getMiddlewareCategoryLabel("cache")).toBe("缓存");
    expect(getMiddlewareCategoryLabel("search_engine")).toBe("搜索引擎");
    expect(getMiddlewareCategoryLabel("config_center")).toBe("配置中心");
    expect(getMiddlewareCategoryLabel("other")).toBe("其他");
    expect(getMiddlewareCategoryLabel("unknown")).toBe("unknown");
  });
});
