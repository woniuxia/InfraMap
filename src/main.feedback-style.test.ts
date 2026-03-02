import { describe, expect, it } from "vitest";
import mainTs from "./main.ts?raw";

describe("main.ts feedback style imports", () => {
  it("loads centralized Element Plus feedback styles", () => {
    expect(mainTs).toContain('import "./styles/element-plus-feedback.css";');
    expect(mainTs).not.toContain('import "element-plus/es/components/message/style/css";');
  });
});
