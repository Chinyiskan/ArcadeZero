<script lang="ts">
  // Tarjeta de error amigable (PLAN.md §7): prioriza friendly_es/en, nunca
  // oculta el mensaje/traceback original (solo lo hace secundario). Vive
  // justo encima de la Consola (ver src/routes/+page.svelte).
  import { t, getLocale } from "$lib/i18n";
  import { friendlyText } from "./errors";
  import type { StructuredError } from "./types";
  import WarningIcon from "phosphor-svelte/lib/WarningIcon";

  let {
    error,
    ongotoline,
  }: {
    error: StructuredError;
    ongotoline?: (line: number) => void;
  } = $props();

  const friendly = $derived(friendlyText(error, getLocale()));
</script>

<section class="error-card" role="alert">
  <div class="header">
    <WarningIcon size={20} weight="fill" aria-hidden="true" />
    <h3>{t("error.title")}</h3>
  </div>
  <p class="friendly">{friendly}</p>
  <p class="original">
    <span class="label">{t("error.originalLabel")}:</span>
    {error.type}: {error.message}
  </p>
  {#if error.lineno !== null}
    <button class="goto" onclick={() => ongotoline?.(error.lineno as number)}>
      {t("error.goToLinePrefix")} {error.lineno}
    </button>
  {/if}
  <details>
    <summary>{t("error.detailsSummary")}</summary>
    <pre>{error.traceback}</pre>
  </details>
</section>

<style>
  .error-card {
    background: var(--az-color-panel-bg);
    border: 1px solid var(--az-color-danger);
    border-left: 4px solid var(--az-color-danger);
    border-radius: var(--az-radius);
    padding: var(--az-space-3);
    margin: var(--az-space-2) var(--az-space-3) 0;
    color: var(--az-color-text);
  }
  .header {
    display: flex;
    align-items: center;
    gap: var(--az-space-2);
    color: var(--az-color-danger);
  }
  .header h3 {
    margin: 0;
    font-size: 1rem;
  }
  .friendly {
    margin: var(--az-space-2) 0;
    font-size: 1rem;
    font-weight: 500;
  }
  .original {
    margin: 0 0 var(--az-space-2);
    color: var(--az-color-text-muted);
    font-size: 0.85rem;
    font-family: var(--az-font-editor);
  }
  .label {
    font-weight: 700;
  }
  .goto {
    display: inline-block;
    margin-bottom: var(--az-space-2);
    padding: var(--az-space-1) var(--az-space-3);
    border-radius: var(--az-radius);
    border: 1px solid var(--az-color-accent);
    background: var(--az-color-accent);
    color: var(--az-color-accent-contrast);
    cursor: pointer;
    font-family: inherit;
    font-weight: 700;
  }
  .goto:hover {
    opacity: 0.9;
  }
  details {
    color: var(--az-color-text-muted);
    font-size: 0.85rem;
  }
  summary {
    cursor: pointer;
  }
  pre {
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--az-font-editor);
    font-size: 0.8rem;
    background: var(--az-color-console-bg);
    color: var(--az-color-console-text);
    padding: var(--az-space-2);
    border-radius: var(--az-radius);
    margin-top: var(--az-space-1);
  }
</style>
