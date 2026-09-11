// Ciclo de temas de la toolbar (PLAN.md §6.4): Dia -> Dracula -> One Dark
// Pro -> Alto contraste -> Dia... Logica pura y testeable a proposito (ver
// theme.test.ts).
export const THEMES = ["dia", "dracula", "one-dark-pro", "alto-contraste"] as const;
export type ThemeName = (typeof THEMES)[number];

export function isThemeName(value: string): value is ThemeName {
  return (THEMES as readonly string[]).includes(value);
}

/** Siguiente tema en el ciclo; si el actual es desconocido, vuelve a "dia". */
export function nextTheme(current: string): ThemeName {
  const idx = THEMES.indexOf(current as ThemeName);
  return THEMES[(idx + 1) % THEMES.length];
}

/** Tema inicial cuando no hay `settings.theme` guardado: respeta el SO. */
export function systemDefaultTheme(prefersDark: boolean): ThemeName {
  return prefersDark ? "dracula" : "dia";
}
