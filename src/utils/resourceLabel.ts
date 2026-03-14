const applicationTypeLabelMap: Record<string, string> = {
  frontend: "前端",
  backend: "后端",
  gateway: "网关",
  batch_job: "批处理",
  microservice: "微服务",
  other: "其他",
};

export interface RelationTargetLabelInput {
  resourceType: "service" | "middleware" | "nginx";
  name: string;
  appType?: string;
  middlewareType?: string;
}

function normalizeText(value?: string): string {
  return (value ?? "").trim();
}

export function getApplicationTypeLabel(type?: string): string {
  const normalizedType = normalizeText(type);
  if (!normalizedType) {
    return "应用";
  }
  return applicationTypeLabelMap[normalizedType] ?? normalizedType;
}

export function formatRelationTargetLabel(input: RelationTargetLabelInput): string {
  const name = normalizeText(input.name);
  const fallbackName = name.length > 0 ? name : "-";

  if (input.resourceType === "service") {
    const typeLabel = getApplicationTypeLabel(input.appType);
    return `${fallbackName}（${typeLabel}）`;
  }

  if (input.resourceType === "middleware") {
    const middlewareType = normalizeText(input.middlewareType);
    const typeLabel = middlewareType.length > 0 ? middlewareType : "中间件";
    return `${fallbackName}（${typeLabel}）`;
  }

  return `${fallbackName}（网关）`;
}
