interface TechStackSource {
  tech_stack?: string | null;
}

function normalizeTechStacks(items: string[]): string[] {
  const unique = new Set<string>();
  for (const item of items) {
    const value = item.trim();
    if (!value || unique.has(value)) continue;
    unique.add(value);
  }
  return Array.from(unique);
}

export function parseTechStack(value?: string | null): string[] {
  if (!value) return [];
  const parts = value.split(/[,\uFF0C;\uFF1B|/]/);
  return normalizeTechStacks(parts);
}

export function techStackToText(list: string[]): string {
  return normalizeTechStacks(list).join(", ");
}

export function buildTechStackSuggestions(
  sources: TechStackSource[],
  currentSelections: string[] = []
): string[] {
  const options: string[] = [...currentSelections];
  for (const source of sources) {
    options.push(...parseTechStack(source.tech_stack));
  }
  return normalizeTechStacks(options);
}
