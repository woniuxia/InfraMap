import { describe, expect, it } from "vitest";
import { buildTechStackSuggestions, parseTechStack, techStackToText } from "@/utils/techStack";

describe("techStack utils", () => {
  it("parseTechStack should split by common separators and trim items", () => {
    expect(parseTechStack("Vue 3, TypeScript\uFF1BPinia / Element Plus | Vite")).toEqual([
      "Vue 3",
      "TypeScript",
      "Pinia",
      "Element Plus",
      "Vite",
    ]);
  });

  it("techStackToText should join by comma and space", () => {
    expect(techStackToText(["Vue 3", "TypeScript"])).toBe("Vue 3, TypeScript");
  });

  it("buildTechStackSuggestions should include current selections and de-duplicate", () => {
    const suggestions = buildTechStackSuggestions(
      [
        { tech_stack: "Vue 3, TypeScript, Pinia" },
        { tech_stack: "TypeScript, Rust" },
      ],
      ["Vite", "Rust"]
    );

    expect(suggestions).toEqual(["Vite", "Rust", "Vue 3", "TypeScript", "Pinia"]);
  });
});
