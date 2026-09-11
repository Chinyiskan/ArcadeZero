import { describe, expect, it } from "vitest";
import { nextTheme, systemDefaultTheme, isThemeName, THEMES } from "./theme";

describe("nextTheme", () => {
  it("cicla dia -> dracula -> one-dark-pro -> dia", () => {
    expect(nextTheme("dia")).toBe("dracula");
    expect(nextTheme("dracula")).toBe("one-dark-pro");
    expect(nextTheme("one-dark-pro")).toBe("dia");
  });

  it("vuelve a dia si el tema actual es desconocido", () => {
    expect(nextTheme("no-existe")).toBe("dia");
  });
});

describe("systemDefaultTheme", () => {
  it("usa dracula si el SO prefiere oscuro", () => {
    expect(systemDefaultTheme(true)).toBe("dracula");
  });

  it("usa dia si el SO prefiere claro", () => {
    expect(systemDefaultTheme(false)).toBe("dia");
  });
});

describe("isThemeName / THEMES", () => {
  it("valida solo los 3 temas soportados", () => {
    for (const name of THEMES) expect(isThemeName(name)).toBe(true);
    expect(isThemeName("noche")).toBe(false);
  });
});
