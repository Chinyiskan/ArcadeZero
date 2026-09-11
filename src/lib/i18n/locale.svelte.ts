// i18n reactivo (PLAN.md §11 Fase 4): `currentLocale` vive en un `$state`
// para que cualquier componente que llame `t()`/`getLocale()` en su markup
// se re-renderice solo con cambiar el locale, sin prop-drilling.
import es from "./es.json";
import en from "./en.json";

export type Locale = "es" | "en";

const DICTIONARIES: Record<Locale, Record<string, string>> = { es, en };

const state = $state<{ locale: Locale }>({ locale: "es" });

export function setLocale(locale: Locale) {
  state.locale = locale;
}

export function getLocale(): Locale {
  return state.locale;
}

export function t(key: string): string {
  return DICTIONARIES[state.locale][key] ?? DICTIONARIES.es[key] ?? key;
}
