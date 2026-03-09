import { describe, expect, it } from "vitest";
import { generateDraftId } from "@/utils/draft";

describe("generateDraftId", () => {
  it("prefixes the generated uuid with the resource key", () => {
    const id = generateDraftId("app");

    expect(id).toMatch(/^app-[0-9a-f-]{36}$/);
  });

  it("returns different values for consecutive calls", () => {
    const first = generateDraftId("mw");
    const second = generateDraftId("mw");

    expect(first).not.toBe(second);
  });
});
