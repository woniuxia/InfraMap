import { describe, expect, it } from "vitest";
import type { CallRelation } from "@/types";
import {
  expandEditorDraftsForSave,
  hasDuplicateExpandedItems,
  mergeCallRelationsForEditor,
} from "@/components/callRelationsEditor.utils";

function makeRelation(input: Partial<CallRelation>): CallRelation {
  return {
    id: "r-1",
    pair_key: "p-1",
    owner_id: "app-a",
    owner_type: "service",
    peer_id: "app-b",
    peer_type: "service",
    direction: "upstream",
    relation_type: "http_call",
    description: undefined,
    created_at: "",
    updated_at: "",
    ...input,
  };
}

describe("callRelationsEditor utils", () => {
  it("mergeCallRelationsForEditor should collapse paired upstream/downstream into bidirectional", () => {
    const rows = mergeCallRelationsForEditor([
      makeRelation({
        id: "r-1",
        peer_id: "app-b",
        direction: "upstream",
        relation_type: "http_call",
        description: "A-B",
      }),
      makeRelation({
        id: "r-2",
        peer_id: "app-b",
        direction: "downstream",
        relation_type: "http_call",
        description: "A-B",
      }),
      makeRelation({
        id: "r-3",
        peer_id: "app-c",
        direction: "upstream",
        relation_type: "tcp",
      }),
    ]);

    expect(rows).toHaveLength(2);
    expect(rows[0]).toMatchObject({
      peer_id: "app-b",
      direction: "bidirectional",
      relation_type: "http_call",
      description: "A-B",
    });
    expect(rows[1]).toMatchObject({
      peer_id: "app-c",
      direction: "upstream",
      relation_type: "tcp",
    });
  });

  it("expandEditorDraftsForSave should expand bidirectional into two records", () => {
    const items = expandEditorDraftsForSave([
      {
        peer_id: "app-b",
        peer_type: "service",
        direction: "bidirectional",
        relation_type: "http_call",
        description: "AB",
      },
      {
        peer_id: "app-c",
        peer_type: "service",
        direction: "downstream",
        relation_type: "tcp",
        description: "",
      },
    ]);

    expect(items).toEqual([
      {
        peer_id: "app-b",
        peer_type: "service",
        direction: "upstream",
        relation_type: "http_call",
        description: "AB",
      },
      {
        peer_id: "app-b",
        peer_type: "service",
        direction: "downstream",
        relation_type: "http_call",
        description: "AB",
      },
      {
        peer_id: "app-c",
        peer_type: "service",
        direction: "downstream",
        relation_type: "tcp",
        description: undefined,
      },
    ]);
  });

  it("hasDuplicateExpandedItems should detect conflicts after expansion", () => {
    const duplicates = [
      {
        peer_id: "app-b",
        peer_type: "service" as const,
        direction: "upstream" as const,
        relation_type: "http_call" as const,
      },
      {
        peer_id: "app-b",
        peer_type: "service" as const,
        direction: "upstream" as const,
        relation_type: "http_call" as const,
      },
    ];
    expect(hasDuplicateExpandedItems(duplicates)).toBe(true);

    const unique = [
      {
        peer_id: "app-b",
        peer_type: "service" as const,
        direction: "upstream" as const,
        relation_type: "http_call" as const,
      },
      {
        peer_id: "app-b",
        peer_type: "service" as const,
        direction: "downstream" as const,
        relation_type: "http_call" as const,
      },
    ];
    expect(hasDuplicateExpandedItems(unique)).toBe(false);
  });
});
