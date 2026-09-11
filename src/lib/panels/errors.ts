// Logica pura de la tarjeta de error amigable (PLAN.md §7). Separada del
// componente Svelte para poder testearla sin montar CM6/DOM.
import type { Locale } from "$lib/i18n";
import type { StructuredError } from "./types";

/** Elige friendly_es o friendly_en segun el idioma activo (default es). */
export function friendlyText(error: StructuredError, locale: Locale): string {
  return locale === "en" ? error.friendly_en : error.friendly_es;
}
