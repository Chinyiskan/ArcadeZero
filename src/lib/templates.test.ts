import { describe, it, expect } from "vitest";
import { TEMPLATE_IDS, RECOMMENDED_TEMPLATE, isTemplateId } from "./templates";

describe("templates", () => {
  it("lists the 3 real template folders", () => {
    expect(TEMPLATE_IDS).toEqual(["en-blanco", "mi-primer-sprite", "plataformas-basico"]);
  });

  it("recommended template is one of the known ids", () => {
    expect(isTemplateId(RECOMMENDED_TEMPLATE)).toBe(true);
  });

  it("rejects unknown ids", () => {
    expect(isTemplateId("no-existe")).toBe(false);
  });
});
