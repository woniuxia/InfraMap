export const ENV_OPTIONS = [
  { label: "生产", value: "prod" },
  { label: "开发", value: "dev" },
  { label: "测试", value: "test" },
];

export const ENV_LABELS = {
  prod: "生产",
  dev: "开发",
  test: "测试",
} as const;

export const TOPOLOGY_ENV_OPTIONS = [
  { label: "生产", value: "prod" },
  { label: "测试", value: "test" },
  { label: "开发", value: "dev" },
];

export const STATUS_OPTIONS = [
  { label: "运行中", value: "running" },
  { label: "已停止", value: "stopped" },
  { label: "维护中", value: "maintenance" },
];

export const STATUS_LABELS = {
  running: "运行中",
  stopped: "已停止",
  maintenance: "维护中",
} as const;

export const BUSINESS_APPLICATION_STATUS_OPTIONS = [
  { label: "激活", value: "active" },
  { label: "停用", value: "inactive" },
] as const;

export const BUSINESS_APPLICATION_STATUS_LABELS = {
  active: "激活",
  inactive: "停用",
} as const;

function getLabelValue(labelMap: Record<string, string>, value?: string): string {
  if (!value) return "-";
  return labelMap[value] || value;
}

export function getEnvLabel(env?: string): string {
  return getLabelValue(ENV_LABELS, env);
}

export function getStatusLabel(status?: string): string {
  return getLabelValue(STATUS_LABELS, status);
}

export function getBusinessApplicationStatusLabel(status?: string): string {
  return getLabelValue(BUSINESS_APPLICATION_STATUS_LABELS, status);
}

export const DEPLOY_MODE_OPTIONS = [
  { label: "物理机", value: "physical" },
  { label: "虚拟机", value: "vm" },
  { label: "Docker", value: "docker" },
  { label: "Kubernetes", value: "k8s" },
  { label: "Serverless", value: "serverless" },
  { label: "其他", value: "other" },
];
