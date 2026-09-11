// i18n (PLAN.md §6.4/§11 Fase 4): claves ES/EN, ES por defecto. La
// implementacion vive en `locale.svelte.ts` (necesita `$state` para que la
// UI reaccione al cambio de idioma) — este archivo es el punto de entrada
// estable para `import { t } from "$lib/i18n"`.
export { t, setLocale, getLocale, type Locale } from "./locale.svelte";
