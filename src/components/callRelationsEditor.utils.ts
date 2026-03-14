import type {
  CallDirection,
  CallRelation,
  CallRelationType,
  ReplaceCallRelationItem,
} from "@/types";

export type EditorCallDirection = CallDirection | "bidirectional";

export interface EditableCallRelationDraft {
  peer_id: string;
  peer_type: "service" | "middleware" | "nginx";
  direction: EditorCallDirection;
  relation_type: CallRelationType;
  description: string;
}

interface MergeAccumulator {
  peer_id: string;
  peer_type: "service" | "middleware" | "nginx";
  relation_type: CallRelationType;
  description: string;
  has_upstream: boolean;
  has_downstream: boolean;
}

function buildMergeKey(relation: CallRelation): string {
  const normalizedDescription = (relation.description ?? "").trim();
  return `${relation.peer_id}|${relation.peer_type}|${relation.relation_type}|${normalizedDescription}`;
}

export function mergeCallRelationsForEditor(
  relations: CallRelation[],
): EditableCallRelationDraft[] {
  const merged = new Map<string, MergeAccumulator>();

  for (const relation of relations) {
    const key = buildMergeKey(relation);
    if (!merged.has(key)) {
      merged.set(key, {
        peer_id: relation.peer_id,
        peer_type: relation.peer_type,
        relation_type: relation.relation_type,
        description: (relation.description ?? "").trim(),
        has_upstream: false,
        has_downstream: false,
      });
    }

    const current = merged.get(key)!;
    if (relation.direction === "upstream") {
      current.has_upstream = true;
    } else if (relation.direction === "downstream") {
      current.has_downstream = true;
    }
  }

  return Array.from(merged.values()).map((item) => ({
    peer_id: item.peer_id,
    peer_type: item.peer_type,
    relation_type: item.relation_type,
    description: item.description,
    direction:
      item.has_upstream && item.has_downstream
        ? "bidirectional"
        : item.has_upstream
          ? "upstream"
          : "downstream",
  }));
}

export function expandEditorDraftsForSave(
  rows: EditableCallRelationDraft[],
): ReplaceCallRelationItem[] {
  const expanded: ReplaceCallRelationItem[] = [];
  for (const row of rows) {
    const base = {
      peer_id: row.peer_id.trim(),
      peer_type: row.peer_type,
      relation_type: row.relation_type,
      description: row.description.trim() || undefined,
    };

    if (row.direction === "bidirectional") {
      expanded.push({ ...base, direction: "upstream" });
      expanded.push({ ...base, direction: "downstream" });
    } else {
      expanded.push({ ...base, direction: row.direction });
    }
  }
  return expanded;
}

export function hasDuplicateExpandedItems(items: ReplaceCallRelationItem[]): boolean {
  const seen = new Set<string>();
  for (const item of items) {
    const key = `${item.peer_id}|${item.peer_type}|${item.direction}|${item.relation_type}`;
    if (seen.has(key)) {
      return true;
    }
    seen.add(key);
  }
  return false;
}
