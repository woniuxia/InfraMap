export type InfraErrorCode =
  | "VALIDATION_ERROR"
  | "NOT_FOUND"
  | "CONFLICT"
  | "DEPENDENCY_CONFLICT"
  | "DB_UNAVAILABLE"
  | "DB_QUERY_FAILED"
  | "IO_ERROR"
  | "BACKUP_ERROR"
  | "INTERNAL_ERROR";

export interface InfraErrorPayload {
  code: InfraErrorCode;
  message: string;
  details?: string;
  command: string;
  retryable: boolean;
}

export interface TauriInvokeOptions {
  silent?: boolean;
  context?: string;
}

export class InfraError extends Error implements InfraErrorPayload {
  code: InfraErrorCode;
  details?: string;
  command: string;
  retryable: boolean;
  raw: unknown;

  constructor(payload: InfraErrorPayload, raw: unknown) {
    super(payload.message);
    this.name = "InfraError";
    this.code = payload.code;
    this.message = payload.message;
    this.details = payload.details;
    this.command = payload.command;
    this.retryable = payload.retryable;
    this.raw = raw;
  }
}
