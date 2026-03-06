import type { Middleware } from "@/types";
import { resolveIconDataUri } from "@/icons/iconRegistry";

export type MiddlewareCategory = Middleware["category"];

export interface MiddlewareIconMeta {
  key: string;
  iconKey: string;
  src: string;
  alt: string;
  isFallback: boolean;
}

export interface MiddlewareTypeOption {
  label: string;
  value: string;
  icon: MiddlewareIconMeta;
}

const DEFAULT_ICON_KEY = "local-middleware:default-middleware";

const COMMON_MIDDLEWARE_TYPES: Record<MiddlewareCategory, string[]> = {
  database: [
    "MySQL",
    "PostgreSQL",
    "MariaDB",
    "SQL Server",
    "Oracle",
    "MongoDB",
    "ClickHouse",
    "TiDB",
    "openGauss",
    "Apache Doris",
    "Memgraph",
    "DB2",
    "DM Database",
    "KingbaseES",
    "GaussDB",
    "OceanBase",
  ],
  message_queue: ["Kafka", "RabbitMQ", "RocketMQ", "Pulsar", "ActiveMQ", "NSQ", "EMQX", "NATS", "Redpanda"],
  cache: ["Redis", "KeyDB", "Tair", "Couchbase", "Memcached"],
  search_engine: ["Elasticsearch", "OpenSearch", "Solr", "Meilisearch", "Sphinx", "Typesense"],
  config_center: ["Nacos", "Apollo", "Consul", "etcd", "ZooKeeper", "Spring Cloud Config"],
  other: ["MinIO", "Apache APISIX", "TongWeb", "GoldenDB"],
};

const MIDDLEWARE_CATEGORY_LABELS: Record<MiddlewareCategory, string> = {
  database: "数据库",
  message_queue: "消息队列",
  cache: "缓存",
  search_engine: "搜索引擎",
  config_center: "配置中心",
  other: "其他",
};

const MIDDLEWARE_CATEGORY_ICONS: Record<MiddlewareCategory, { iconKey: string; alt: string }> = {
  database: { iconKey: "local-middleware:category-database", alt: "数据库" },
  message_queue: { iconKey: "local-middleware:category-message-queue", alt: "消息队列" },
  cache: { iconKey: "local-middleware:category-cache", alt: "缓存" },
  search_engine: { iconKey: "local-middleware:category-search-engine", alt: "搜索引擎" },
  config_center: { iconKey: "local-middleware:category-config-center", alt: "配置中心" },
  other: { iconKey: "local-middleware:category-other", alt: "其他" },
};

const MIDDLEWARE_TYPE_ICONS: Record<string, { iconKey: string; alt: string }> = {
  mysql: { iconKey: "local-middleware:mysql", alt: "MySQL" },
  postgresql: { iconKey: "local-middleware:postgresql", alt: "PostgreSQL" },
  postgres: { iconKey: "local-middleware:postgresql", alt: "PostgreSQL" },
  mariadb: { iconKey: "local-middleware:mariadb", alt: "MariaDB" },
  sqlserver: { iconKey: "local-middleware:sqlserver", alt: "SQL Server" },
  oracle: { iconKey: "local-middleware:oracle", alt: "Oracle" },
  mongodb: { iconKey: "local-middleware:mongodb", alt: "MongoDB" },
  clickhouse: { iconKey: "local-middleware:clickhouse", alt: "ClickHouse" },
  tidb: { iconKey: "local-middleware:tidb", alt: "TiDB" },
  opengauss: { iconKey: "local-middleware:opengauss", alt: "openGauss" },
  apachedoris: { iconKey: "local-middleware:apachedoris", alt: "Apache Doris" },
  memgraph: { iconKey: "local-middleware:memgraph", alt: "Memgraph" },
  db2: { iconKey: "local-middleware:db2", alt: "DB2" },
  dmdatabase: { iconKey: "local-middleware:dmdatabase", alt: "DM Database" },
  kingbasees: { iconKey: "local-middleware:kingbasees", alt: "KingbaseES" },
  gaussdb: { iconKey: "local-middleware:gaussdb", alt: "GaussDB" },
  oceanbase: { iconKey: "local-middleware:oceanbase", alt: "OceanBase" },
  kafka: { iconKey: "local-middleware:kafka", alt: "Kafka" },
  rabbitmq: { iconKey: "local-middleware:rabbitmq", alt: "RabbitMQ" },
  rocketmq: { iconKey: "local-middleware:rocketmq", alt: "RocketMQ" },
  pulsar: { iconKey: "local-middleware:pulsar", alt: "Pulsar" },
  activemq: { iconKey: "local-middleware:activemq", alt: "ActiveMQ" },
  nsq: { iconKey: "local-middleware:nsq", alt: "NSQ" },
  emqx: { iconKey: "local-middleware:emqx", alt: "EMQX" },
  nats: { iconKey: "local-middleware:nats", alt: "NATS" },
  natsio: { iconKey: "local-middleware:nats", alt: "NATS" },
  redpanda: { iconKey: "local-middleware:redpanda", alt: "Redpanda" },
  redis: { iconKey: "local-middleware:redis", alt: "Redis" },
  keydb: { iconKey: "local-middleware:keydb", alt: "KeyDB" },
  tair: { iconKey: "local-middleware:tair", alt: "Tair" },
  couchbase: { iconKey: "local-middleware:couchbase", alt: "Couchbase" },
  memcached: { iconKey: "local-middleware:memcached", alt: "Memcached" },
  elasticsearch: { iconKey: "local-middleware:elasticsearch", alt: "Elasticsearch" },
  opensearch: { iconKey: "local-middleware:opensearch", alt: "OpenSearch" },
  solr: { iconKey: "local-middleware:solr", alt: "Solr" },
  meilisearch: { iconKey: "local-middleware:meilisearch", alt: "Meilisearch" },
  sphinx: { iconKey: "local-middleware:sphinx", alt: "Sphinx" },
  typesense: { iconKey: "local-middleware:typesense", alt: "Typesense" },
  nacos: { iconKey: "local-middleware:nacos", alt: "Nacos" },
  apollo: { iconKey: "local-middleware:apollo", alt: "Apollo" },
  consul: { iconKey: "local-middleware:consul", alt: "Consul" },
  etcd: { iconKey: "local-middleware:etcd", alt: "etcd" },
  zookeeper: { iconKey: "local-middleware:zookeeper", alt: "ZooKeeper" },
  springcloudconfig: { iconKey: "local-middleware:springcloudconfig", alt: "Spring Cloud Config" },
  minio: { iconKey: "local-middleware:minio", alt: "MinIO" },
  apacheapisix: { iconKey: "local-middleware:apacheapisix", alt: "Apache APISIX" },
  tongweb: { iconKey: "local-middleware:tongweb", alt: "TongWeb" },
  goldendb: { iconKey: "local-middleware:goldendb", alt: "GoldenDB" },
};

const MIDDLEWARE_DEFAULT_PORTS: Record<string, number> = {
  mysql: 3306,
  mariadb: 3306,
  postgresql: 5432,
  postgres: 5432,
  sqlserver: 1433,
  oracle: 1521,
  mongodb: 27017,
  clickhouse: 8123,
  tidb: 4000,
  opengauss: 5432,
  apachedoris: 9030,
  memgraph: 7687,
  db2: 50000,
  dmdatabase: 5236,
  kingbasees: 54321,
  gaussdb: 8000,
  oceanbase: 2881,
  kafka: 9092,
  rabbitmq: 5672,
  rocketmq: 9876,
  pulsar: 6650,
  activemq: 61616,
  nsq: 4150,
  emqx: 1883,
  nats: 4222,
  redpanda: 9092,
  redis: 6379,
  keydb: 6379,
  tair: 6379,
  couchbase: 8091,
  memcached: 11211,
  elasticsearch: 9200,
  opensearch: 9200,
  solr: 8983,
  meilisearch: 7700,
  sphinx: 9312,
  typesense: 8108,
  nacos: 8848,
  apollo: 8080,
  consul: 8500,
  etcd: 2379,
  zookeeper: 2181,
  springcloudconfig: 8888,
  minio: 9000,
  apacheapisix: 9180,
  tongweb: 9060,
  goldendb: 1888,
};

const MIDDLEWARE_TYPE_ALIASES: Record<string, string> = {
  dameng: "dmdatabase",
  达梦: "dmdatabase",
  dmdb: "dmdatabase",
  kingbase: "kingbasees",
  人大金仓: "kingbasees",
  金仓: "kingbasees",
  tongtech: "tongweb",
  东方通: "tongweb",
  natsdotio: "nats",
};

function normalizeMiddlewareType(type?: string): string {
  const normalized = (type ?? "").trim().toLowerCase().replace(/[\s_-]+/g, "");
  if (!normalized) return "";
  return MIDDLEWARE_TYPE_ALIASES[normalized] ?? normalized;
}

function normalizeMiddlewareCategory(category?: string): MiddlewareCategory | undefined {
  const normalized = (category ?? "").trim();
  if (!normalized) return undefined;
  if (normalized in MIDDLEWARE_CATEGORY_LABELS) {
    return normalized as MiddlewareCategory;
  }
  return undefined;
}

function buildResolvedIcon(
  key: string,
  iconKey: string,
  alt: string,
  isFallback: boolean,
): MiddlewareIconMeta | null {
  const src = resolveIconDataUri(iconKey);
  if (!src) return null;
  return {
    key,
    iconKey,
    src,
    alt,
    isFallback,
  };
}

function resolveDefaultIcon(): MiddlewareIconMeta {
  const src = resolveIconDataUri(DEFAULT_ICON_KEY) || "";
  return {
    key: "middleware",
    iconKey: DEFAULT_ICON_KEY,
    src,
    alt: "中间件",
    isFallback: true,
  };
}

export const MIDDLEWARE_CATEGORY_OPTIONS = (
  Object.entries(MIDDLEWARE_CATEGORY_LABELS) as Array<[MiddlewareCategory, string]>
).map(([value, label]) => ({ value, label }));

export function getMiddlewareCategoryLabel(category: string): string {
  const normalized = normalizeMiddlewareCategory(category);
  return normalized ? MIDDLEWARE_CATEGORY_LABELS[normalized] : category;
}

export function getMiddlewareTypeOptions(category?: string, currentType?: string): string[] {
  const normalized = normalizeMiddlewareCategory(category);
  const defaults = normalized ? COMMON_MIDDLEWARE_TYPES[normalized] : [];
  const current = currentType?.trim() ?? "";
  const options = current ? [...defaults, current] : defaults;
  return options.filter((item, index) => item && options.indexOf(item) === index);
}

export function getMiddlewareIconByType(type?: string, category?: string): MiddlewareIconMeta {
  const normalizedType = normalizeMiddlewareType(type);
  if (normalizedType) {
    const typeIcon = MIDDLEWARE_TYPE_ICONS[normalizedType];
    if (typeIcon) {
      const resolved = buildResolvedIcon(normalizedType, typeIcon.iconKey, typeIcon.alt, false);
      if (resolved) return resolved;
    }
  }

  const normalizedCategory = normalizeMiddlewareCategory(category);
  if (normalizedCategory) {
    const categoryIcon = MIDDLEWARE_CATEGORY_ICONS[normalizedCategory];
    const resolved = buildResolvedIcon(
      normalizedCategory,
      categoryIcon.iconKey,
      categoryIcon.alt,
      true,
    );
    if (resolved) return resolved;
  }

  return resolveDefaultIcon();
}

export function getMiddlewareIcon(category?: string, type?: string): MiddlewareIconMeta {
  return getMiddlewareIconByType(type, category);
}

export function getMiddlewareTypeOptionsWithIcon(
  category?: string,
  currentType?: string
): MiddlewareTypeOption[] {
  return getMiddlewareTypeOptions(category, currentType).map((type) => ({
    label: type,
    value: type,
    icon: getMiddlewareIconByType(type, category),
  }));
}

export function getMiddlewareDefaultPort(type?: string): number | undefined {
  const normalized = normalizeMiddlewareType(type);
  return normalized ? MIDDLEWARE_DEFAULT_PORTS[normalized] : undefined;
}
