import { describe, expect, it } from "vitest";
import { indentLevel, guideStyle } from "./indentGuides";
import { checkIndentIssue } from "./indentLint";
import { STATIC_OPTIONS } from "./completions";

describe("indentLevel", () => {
  it("cuenta niveles de 4 espacios", () => {
    expect(indentLevel("")).toBe(0);
    expect(indentLevel("    x = 1")).toBe(1);
    expect(indentLevel("        x = 1")).toBe(2);
    expect(indentLevel("  x = 1")).toBe(0); // 2 espacios no llegan a un nivel
  });

  it("aproxima un tab a un nivel", () => {
    expect(indentLevel("\tx = 1")).toBe(1);
  });
});

describe("guideStyle", () => {
  it("no genera estilo para nivel 0", () => {
    expect(guideStyle(0)).toBe("");
  });

  it("genera una capa de gradiente por nivel", () => {
    const style = guideStyle(2);
    expect(style).toContain("--az-indent-0");
    expect(style).toContain("--az-indent-1");
    expect(style.match(/linear-gradient/g)?.length).toBe(2);
  });

  it("cicla la paleta cada 6 niveles", () => {
    const style = guideStyle(7);
    // nivel 6 (indice 6) deberia reusar --az-indent-0 (color aparece 2 veces
    // por capa: linear-gradient(color, color))
    expect(style.split("--az-indent-0").length - 1).toBe(4);
  });
});

describe("checkIndentIssue", () => {
  it("no marca lineas sin indentar", () => {
    expect(checkIndentIssue("x = 1")).toBeNull();
  });

  it("no marca lineas en blanco", () => {
    expect(checkIndentIssue("    ")).toBeNull();
  });

  it("marca tabs mezclados con espacios", () => {
    const issue = checkIndentIssue(" \tx = 1");
    expect(issue?.kind).toBe("mixed");
  });

  it("marca indentacion que no es multiplo de 4", () => {
    const issue = checkIndentIssue("   x = 1");
    expect(issue?.kind).toBe("non-multiple");
  });

  it("acepta multiplos de 4", () => {
    expect(checkIndentIssue("    x = 1")).toBeNull();
    expect(checkIndentIssue("        x = 1")).toBeNull();
  });

  it("no penaliza tabs puros en v1", () => {
    expect(checkIndentIssue("\tx = 1")).toBeNull();
  });
});

describe("STATIC_OPTIONS (autocompletado pgzero)", () => {
  const labels = STATIC_OPTIONS.map((o) => o.label);

  it("incluye la API de alto nivel del cheatsheet de pgzero", () => {
    for (const name of ["Actor", "screen", "keyboard", "sounds", "music", "clock", "animate"]) {
      expect(labels).toContain(name);
    }
  });

  it("incluye snippets para los callbacks que pgzero busca por nombre", () => {
    for (const name of ["def draw", "def update", "def on_key_down"]) {
      expect(labels).toContain(name);
    }
  });

  it("no duplica los symbolos de pgzero agregados", () => {
    const pgzeroLabels = labels.filter(
      (l) => !["def", "for", "if", "while", "class"].includes(l),
    );
    expect(new Set(pgzeroLabels).size).toBe(pgzeroLabels.length);
  });
});
