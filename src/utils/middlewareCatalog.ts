import type { Middleware } from "@/types";
import defaultMiddlewareIcon from "@/assets/middleware-icons/default-middleware.svg";
import categoryDatabaseIcon from "@/assets/middleware-icons/category-database.svg";
import categoryMessageQueueIcon from "@/assets/middleware-icons/category-message-queue.svg";
import categoryCacheIcon from "@/assets/middleware-icons/category-cache.svg";
import categorySearchEngineIcon from "@/assets/middleware-icons/category-search-engine.svg";
import categoryConfigCenterIcon from "@/assets/middleware-icons/category-config-center.svg";
import categoryOtherIcon from "@/assets/middleware-icons/category-other.svg";
import mysqlIcon from "@/assets/middleware-icons/mysql.svg";
import postgresqlIcon from "@/assets/middleware-icons/postgresql.svg";
import mariadbIcon from "@/assets/middleware-icons/mariadb.svg";
import sqlserverIcon from "@/assets/middleware-icons/sqlserver.svg";
import oracleIcon from "@/assets/middleware-icons/oracle.svg";
import mongodbIcon from "@/assets/middleware-icons/mongodb.svg";
import clickhouseIcon from "@/assets/middleware-icons/clickhouse.svg";
import kafkaIcon from "@/assets/middleware-icons/kafka.svg";
import rabbitmqIcon from "@/assets/middleware-icons/rabbitmq.svg";
import rocketmqIcon from "@/assets/middleware-icons/rocketmq.svg";
import pulsarIcon from "@/assets/middleware-icons/pulsar.svg";
import activemqIcon from "@/assets/middleware-icons/activemq.svg";
import nsqIcon from "@/assets/middleware-icons/nsq.svg";
import redisIcon from "@/assets/middleware-icons/redis.svg";
import keydbIcon from "@/assets/middleware-icons/keydb.svg";
import tairIcon from "@/assets/middleware-icons/tair.svg";
import couchbaseIcon from "@/assets/middleware-icons/couchbase.svg";
import elasticsearchIcon from "@/assets/middleware-icons/elasticsearch.svg";
import opensearchIcon from "@/assets/middleware-icons/opensearch.svg";
import solrIcon from "@/assets/middleware-icons/solr.svg";
import meilisearchIcon from "@/assets/middleware-icons/meilisearch.svg";
import sphinxIcon from "@/assets/middleware-icons/sphinx.svg";
import nacosIcon from "@/assets/middleware-icons/nacos.svg";
import apolloIcon from "@/assets/middleware-icons/apollo.svg";
import consulIcon from "@/assets/middleware-icons/consul.svg";
import etcdIcon from "@/assets/middleware-icons/etcd.svg";
import zookeeperIcon from "@/assets/middleware-icons/zookeeper.svg";
import memcachedIcon from "@/assets/middleware-icons/memcached.svg";
import tidbIcon from "@/assets/middleware-icons/tidb.svg";
import opengaussIcon from "@/assets/middleware-icons/opengauss.svg";
import dmdatabaseIcon from "@/assets/middleware-icons/dmdatabase.svg";
import kingbaseesIcon from "@/assets/middleware-icons/kingbasees.svg";
import gaussdbIcon from "@/assets/middleware-icons/gaussdb.svg";
import oceanbaseIcon from "@/assets/middleware-icons/oceanbase.svg";
import emqxIcon from "@/assets/middleware-icons/emqx.svg";
import natsIcon from "@/assets/middleware-icons/nats.svg";
import redpandaIcon from "@/assets/middleware-icons/redpanda.svg";
import typesenseIcon from "@/assets/middleware-icons/typesense.svg";
import springcloudconfigIcon from "@/assets/middleware-icons/springcloudconfig.svg";
import minioIcon from "@/assets/middleware-icons/minio.svg";
import apacheapisixIcon from "@/assets/middleware-icons/apacheapisix.svg";
import tongwebIcon from "@/assets/middleware-icons/tongweb.svg";
import goldendbIcon from "@/assets/middleware-icons/goldendb.svg";
import apachedorisIcon from "@/assets/middleware-icons/apachedoris.svg";
import memgraphIcon from "@/assets/middleware-icons/memgraph.svg";
import db2Icon from "@/assets/middleware-icons/db2.svg";

export type MiddlewareCategory = Middleware["category"];

export interface MiddlewareIconMeta {
  key: string;
  src: string;
  alt: string;
  isFallback: boolean;
}

export interface MiddlewareTypeOption {
  label: string;
  value: string;
  icon: MiddlewareIconMeta;
}

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

const MIDDLEWARE_CATEGORY_ICONS: Record<MiddlewareCategory, string> = {
  database: categoryDatabaseIcon,
  message_queue: categoryMessageQueueIcon,
  cache: categoryCacheIcon,
  search_engine: categorySearchEngineIcon,
  config_center: categoryConfigCenterIcon,
  other: categoryOtherIcon,
};

const MIDDLEWARE_TYPE_ICONS: Record<string, { src: string; alt: string }> = {
  mysql: { src: mysqlIcon, alt: "MySQL" },
  postgresql: { src: postgresqlIcon, alt: "PostgreSQL" },
  postgres: { src: postgresqlIcon, alt: "PostgreSQL" },
  mariadb: { src: mariadbIcon, alt: "MariaDB" },
  sqlserver: { src: sqlserverIcon, alt: "SQL Server" },
  oracle: { src: oracleIcon, alt: "Oracle" },
  mongodb: { src: mongodbIcon, alt: "MongoDB" },
  clickhouse: { src: clickhouseIcon, alt: "ClickHouse" },
  tidb: { src: tidbIcon, alt: "TiDB" },
  opengauss: { src: opengaussIcon, alt: "openGauss" },
  apachedoris: { src: apachedorisIcon, alt: "Apache Doris" },
  memgraph: { src: memgraphIcon, alt: "Memgraph" },
  db2: { src: db2Icon, alt: "DB2" },
  dmdatabase: { src: dmdatabaseIcon, alt: "DM Database" },
  kingbasees: { src: kingbaseesIcon, alt: "KingbaseES" },
  gaussdb: { src: gaussdbIcon, alt: "GaussDB" },
  oceanbase: { src: oceanbaseIcon, alt: "OceanBase" },
  kafka: { src: kafkaIcon, alt: "Kafka" },
  rabbitmq: { src: rabbitmqIcon, alt: "RabbitMQ" },
  rocketmq: { src: rocketmqIcon, alt: "RocketMQ" },
  pulsar: { src: pulsarIcon, alt: "Pulsar" },
  activemq: { src: activemqIcon, alt: "ActiveMQ" },
  nsq: { src: nsqIcon, alt: "NSQ" },
  emqx: { src: emqxIcon, alt: "EMQX" },
  nats: { src: natsIcon, alt: "NATS" },
  natsio: { src: natsIcon, alt: "NATS" },
  redpanda: { src: redpandaIcon, alt: "Redpanda" },
  redis: { src: redisIcon, alt: "Redis" },
  keydb: { src: keydbIcon, alt: "KeyDB" },
  tair: { src: tairIcon, alt: "Tair" },
  couchbase: { src: couchbaseIcon, alt: "Couchbase" },
  memcached: { src: memcachedIcon, alt: "Memcached" },
  elasticsearch: { src: elasticsearchIcon, alt: "Elasticsearch" },
  opensearch: { src: opensearchIcon, alt: "OpenSearch" },
  solr: { src: solrIcon, alt: "Solr" },
  meilisearch: { src: meilisearchIcon, alt: "Meilisearch" },
  sphinx: { src: sphinxIcon, alt: "Sphinx" },
  typesense: { src: typesenseIcon, alt: "Typesense" },
  nacos: { src: nacosIcon, alt: "Nacos" },
  apollo: { src: apolloIcon, alt: "Apollo" },
  consul: { src: consulIcon, alt: "Consul" },
  etcd: { src: etcdIcon, alt: "etcd" },
  zookeeper: { src: zookeeperIcon, alt: "ZooKeeper" },
  springcloudconfig: { src: springcloudconfigIcon, alt: "Spring Cloud Config" },
  minio: { src: minioIcon, alt: "MinIO" },
  apacheapisix: { src: apacheapisixIcon, alt: "Apache APISIX" },
  tongweb: { src: tongwebIcon, alt: "TongWeb" },
  goldendb: { src: goldendbIcon, alt: "GoldenDB" },
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
      return {
        key: normalizedType,
        src: typeIcon.src,
        alt: typeIcon.alt,
        isFallback: false,
      };
    }
  }

  const normalizedCategory = normalizeMiddlewareCategory(category);
  if (normalizedCategory) {
    return {
      key: normalizedCategory,
      src: MIDDLEWARE_CATEGORY_ICONS[normalizedCategory],
      alt: MIDDLEWARE_CATEGORY_LABELS[normalizedCategory],
      isFallback: true,
    };
  }

  return {
    key: "middleware",
    src: defaultMiddlewareIcon,
    alt: "中间件",
    isFallback: true,
  };
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
