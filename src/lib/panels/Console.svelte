<script lang="ts">
  // Consola colapsable (PLAN.md §6.1): stdout/stderr en vivo, se expande
  // sola cuando hay salida. La tarjeta de error amigable (§7) ya tiene su
  // evento listo en el backend: `run_error` (src-tauri/src/run.rs +
  // error_parse.rs), con forma { type, message, file, lineno, friendly_es,
  // friendly_en, traceback } — ver el TODO(editor-ux) en
  // src/routes/+page.svelte para el detalle. Falta consumirlo y renderizar
  // la tarjeta aca (o en un componente nuevo); hoy este panel solo muestra
  // las lineas crudas de run_stdout/run_stderr (que ya no incluyen las
  // lineas ##ARCADEZERO##, interceptadas antes en Rust).
  import { t } from "$lib/i18n";
  import type { ConsoleLine } from "./types";
  import CaretDownIcon from "phosphor-svelte/lib/CaretDownIcon";
  import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";

  let {
    lines = $bindable([]),
    expanded = $bindable(false),
  }: {
    lines?: ConsoleLine[];
    expanded?: boolean;
  } = $props();

  let logEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    lines.length; // depender de la longitud para re-scrollear
    if (logEl) logEl.scrollTop = logEl.scrollHeight;
  });
</script>

<section class="console" class:collapsed={!expanded}>
  <button
    class="console-header"
    onclick={() => (expanded = !expanded)}
    aria-expanded={expanded}
  >
    {#if expanded}
      <CaretDownIcon size={16} aria-hidden="true" />
    {:else}
      <CaretRightIcon size={16} aria-hidden="true" />
    {/if}
    {t("console.title")}
    {#if lines.length > 0}<span class="count">{lines.length}</span>{/if}
  </button>
  {#if expanded}
    <div class="console-log" bind:this={logEl}>
      {#if lines.length === 0}
        <p class="empty">{t("console.empty")}</p>
      {/if}
      {#each lines as line}
        <pre class={line.kind === "err" ? "err" : "out"}>{line.text}</pre>
      {/each}
    </div>
  {/if}
</section>

<style>
  .console {
    display: flex;
    flex-direction: column;
    background: var(--az-color-console-bg);
    color: var(--az-color-console-text);
    border-top: 1px solid var(--az-color-border);
    max-height: 28vh;
  }
  .console.collapsed {
    max-height: auto;
  }
  .console-header {
    display: flex;
    align-items: center;
    gap: var(--az-space-2);
    width: 100%;
    background: none;
    border: none;
    color: inherit;
    font-family: var(--az-font-ui);
    font-weight: 700;
    text-align: left;
    padding: var(--az-space-2) var(--az-space-3);
    cursor: pointer;
  }
  .count {
    margin-left: auto;
    background: var(--az-color-accent);
    color: var(--az-color-accent-contrast);
    border-radius: 999px;
    padding: 0 var(--az-space-2);
    font-size: 0.75rem;
  }
  .console-log {
    overflow-y: auto;
    padding: 0 var(--az-space-3) var(--az-space-2);
  }
  pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--az-font-editor);
    font-size: 0.9rem;
    line-height: 1.4;
  }
  .out {
    color: var(--az-color-console-text);
  }
  .err {
    color: var(--az-color-console-err);
  }
  .empty {
    color: #888;
    font-style: italic;
  }
</style>
