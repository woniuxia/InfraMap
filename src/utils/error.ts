import { InfraError, type InfraErrorCode, type InfraErrorPayload } from "@/types/error";

const STRUCTURED_CODES = new Set<InfraErrorCode>([
  "VALIDATION_ERROR",
  "NOT_FOUND",
  "CONFLICT",
  "DEPENDENCY_CONFLICT",
  "DB_UNAVAILABLE",
  "DB_QUERY_FAILED",
  "IO_ERROR",
  "BACKUP_ERROR",
  "INTERNAL_ERROR",
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isInfraErrorCode(value: unknown): value is InfraErrorCode {
  return typeof value === "string" && STRUCTURED_CODES.has(value as InfraErrorCode);
}

function toPayload(value: Record<string, unknown>, fallbackCommand: string): InfraErrorPayload | null {
  if (!isInfraErrorCode(value.code) || typeof value.message !== "string") {
    return null;
  }

  return {
    code: value.code,
    message: value.message,
    details: typeof value.details === "string" && value.details.length > 0 ? value.details : undefined,
    command: typeof value.command === "string" && value.command.length > 0 ? value.command : fallbackCommand,
    retryable: typeof value.retryable === "boolean" ? value.retryable : false,
  };
}

function fromLegacyMessage(message: string, command: string): InfraErrorPayload {
  if (message.includes("UNIQUE constraint failed: ip_addresses.ip_address, ip_addresses.env")) {
    return {
      code: "CONFLICT",
      message: "保存失败，IP 地址与环境组合已存在，请勿重复创建。",
      details: message,
      command,
      retryable: false,
    };
  }

  if (message.includes("UNIQUE constraint failed")) {
    return {
      code: "CONFLICT",
      message: "保存失败，存在重复数据，请检查后重试。",
      details: message,
      command,
      retryable: false,
    };
  }

  if (message.includes("FOREIGN KEY constraint failed")) {
    return {
      code: "DEPENDENCY_CONFLICT",
      message: "操作失败，存在关联依赖，请先处理引用关系。",
      details: message,
      command,
      retryable: false,
    };
  }

  if (message.includes("Pool error")) {
    return {
      code: "DB_UNAVAILABLE",
      message: "数据库暂时不可用，请稍后重试。",
      details: message,
      command,
      retryable: true,
    };
  }

  return {
    code: "INTERNAL_ERROR",
    message,
    details: message,
    command,
    retryable: false,
  };
}

function parseStructuredFromString(raw: string, fallbackCommand: string): InfraErrorPayload | null {
  const trimmed = raw.trim();
  if (!trimmed.startsWith("{")) {
    return null;
  }

  try {
    const parsed = JSON.parse(trimmed) as unknown;
    if (!isRecord(parsed)) {
      return null;
    }
    return toPayload(parsed, fallbackCommand);
  } catch {
    return null;
  }
}

export function normalizeInvokeError(error: unknown, command: string): InfraError {
  if (error instanceof InfraError) {
    return error;
  }

  if (isRecord(error)) {
    const payload = toPayload(error, command);
    if (payload) {
      return new InfraError(payload, error);
    }

    if (typeof error.message === "string" && error.message.trim().length > 0) {
      const fromMessage = parseStructuredFromString(error.message, command);
      if (fromMessage) {
        return new InfraError(fromMessage, error);
      }
      return new InfraError(fromLegacyMessage(error.message, command), error);
    }
  }

  if (typeof error === "string") {
    const structured = parseStructuredFromString(error, command);
    if (structured) {
      return new InfraError(structured, error);
    }
    return new InfraError(fromLegacyMessage(error, command), error);
  }

  return new InfraError(
    {
      code: "INTERNAL_ERROR",
      message: "发生未知异常，请稍后重试。",
      command,
      retryable: false,
    },
    error,
  );
}

export function formatInfraErrorToast(error: InfraError): string {
  return `${error.message}（${error.code}）`;
}
