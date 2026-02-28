const COMMON_MIDDLEWARE_TYPES: Record<string, string[]> = {
  database: ["MySQL", "PostgreSQL", "MariaDB", "SQL Server", "Oracle", "MongoDB", "ClickHouse"],
  message_queue: ["Kafka", "RabbitMQ", "RocketMQ", "Pulsar", "ActiveMQ", "NSQ"],
  cache: ["Redis", "KeyDB", "Tair", "Couchbase"],
  search_engine: ["Elasticsearch", "OpenSearch", "Solr", "Meilisearch", "Sphinx"],
  config_center: ["Nacos", "Apollo", "Consul", "etcd", "ZooKeeper"],
  other: [],
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
  kafka: 9092,
  rabbitmq: 5672,
  rocketmq: 9876,
  pulsar: 6650,
  activemq: 61616,
  nsq: 4150,
  redis: 6379,
  keydb: 6379,
  tair: 6379,
  couchbase: 8091,
  elasticsearch: 9200,
  opensearch: 9200,
  solr: 8983,
  meilisearch: 7700,
  sphinx: 9312,
  nacos: 8848,
  apollo: 8080,
  consul: 8500,
  etcd: 2379,
  zookeeper: 2181,
};

function normalizeMiddlewareType(type?: string): string {
  return (type ?? "").trim().toLowerCase().replace(/[\s_-]+/g, "");
}

export function getMiddlewareTypeOptions(category?: string, currentType?: string): string[] {
  const defaults = category ? (COMMON_MIDDLEWARE_TYPES[category] ?? []) : [];
  const current = currentType?.trim() ?? "";
  const options = current ? [...defaults, current] : defaults;
  return options.filter((item, index) => item && options.indexOf(item) === index);
}

export function getMiddlewareDefaultPort(type?: string): number | undefined {
  const normalized = normalizeMiddlewareType(type);
  return normalized ? MIDDLEWARE_DEFAULT_PORTS[normalized] : undefined;
}
