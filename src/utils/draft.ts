export function generateDraftId(prefix: string): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `${prefix}-${crypto.randomUUID()}`;
  }

  return `${prefix}-draft-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`;
}
