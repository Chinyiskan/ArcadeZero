// i18n minimo (PLAN.md §6.4): claves desde el dia 1, ES por defecto.
// El selector de idioma completo (UI, persistencia en settings) es Fase 4;
// hoy solo exponemos `t(key)` reactivo a un locale en memoria.
import es from "./es.json";
import en from "./en.json";

export type Locale = "es" | "en";

const DICTIONARIES: Record<Locale, Record<string, string>> = { es, en };

let currentLocale: Locale = "es";

export function setLocale(locale: Locale) {
  currentLocale = locale;
}

export function getLocale(): Locale {
  return currentLocale;
}

export function t(key: string): string {
  return DICTIONARIES[currentLocale][key] ?? DICTIONARIES.es[key] ?? key;
}
