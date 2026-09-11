import { describe, expect, it } from "vitest";
import { friendlyText } from "./errors";
import type { StructuredError } from "./types";

const err: StructuredError = {
  type: "NameError",
  message: "name 'alein' is not defined",
  file: "main.py",
  lineno: 12,
  friendly_es: "Escribiste alein pero no existe. ¿Quisiste decir alien?",
  friendly_en: "You wrote alein but it doesn't exist. Did you mean alien?",
  traceback: "Traceback...",
};

describe("friendlyText", () => {
  it("usa friendly_es por defecto (es)", () => {
    expect(friendlyText(err, "es")).toBe(err.friendly_es);
  });

  it("usa friendly_en cuando el locale es en", () => {
    expect(friendlyText(err, "en")).toBe(err.friendly_en);
  });
});
