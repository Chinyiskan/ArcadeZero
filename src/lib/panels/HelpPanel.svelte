<script lang="ts">
  // Panel de Ayuda (PLAN.md §6.2/§11 Fase 4, atajo F1): cheatsheet de pgzero
  // renderizada desde el mismo markdown que usan `editor-ux`/`pgzero-domain`
  // (docs/pgzero-cheatsheet.md), sin duplicar la API a mano.
  import { t } from "$lib/i18n";
  import { renderMarkdown } from "$lib/markdown";
  import cheatsheet from "../../../docs/pgzero-cheatsheet.md?raw";
  import XIcon from "phosphor-svelte/lib/XIcon";

  let {
    open = false,
    onclose,
    onbrowsetemplates,
  }: {
    open?: boolean;
    onclose?: () => void;
    onbrowsetemplates?: () => void;
  } = $props();

  const html = $derived(renderMarkdown(cheatsheet));
</script>

{#if open}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) onclose?.();
    }}
  >
    <div class="modal" role="dialog" aria-modal="true" aria-label={t("help.title")}>
      <div class="header">
        <h2>{t("help.title")}</h2>
        <div class="actions">
          <button class="link" onclick={onbrowsetemplates}>{t("help.browseTemplates")}</button>
          <button class="icon-btn" onclick={onclose} aria-label={t("settings.close")}>
            <XIcon size={20} aria-hidden="true" />
          </button>
        </div>
      </div>
      <div class="content">{@html html}</div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    max-width: 760px;
    width: 90%;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    background: var(--az-color-panel-bg);
    border: 1px solid var(--az-color-border);
    border-radius: var(--az-radius);
    color: var(--az-color-text);
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--az-space-3);
    border-bottom: 1px solid var(--az-color-border);
  }
  .header h2 {
    margin: 0;
    font-size: 1.1rem;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: var(--az-space-2);
  }
  .link {
    padding: var(--az-space-1) var(--az-space-2);
    border-radius: var(--az-radius);
    border: 1px solid var(--az-color-border);
    background: none;
    color: var(--az-color-text);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.85rem;
  }
  .icon-btn {
    display: flex;
    padding: var(--az-space-1);
    border: none;
    background: none;
    color: var(--az-color-text);
    cursor: pointer;
  }
  .content {
    overflow-y: auto;
    padding: var(--az-space-3) var(--az-space-4);
  }
  .content :global(h1),
  .content :global(h2),
  .content :global(h3) {
    color: var(--az-color-text);
  }
  .content :global(h2) {
    margin-top: var(--az-space-4);
    border-bottom: 1px solid var(--az-color-border);
    padding-bottom: var(--az-space-1);
  }
  .content :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: var(--az-space-2) 0;
    font-size: 0.85rem;
  }
  .content :global(th),
  .content :global(td) {
    text-align: left;
    padding: var(--az-space-1) var(--az-space-2);
    border: 1px solid var(--az-color-border);
    vertical-align: top;
  }
  .content :global(code) {
    font-family: var(--az-font-editor);
    background: var(--az-color-editor-active-line);
    padding: 1px 4px;
    border-radius: 3px;
  }
  .content :global(pre) {
    background: var(--az-color-console-bg);
    color: var(--az-color-console-text);
    padding: var(--az-space-2);
    border-radius: var(--az-radius);
    overflow-x: auto;
  }
  .content :global(pre code) {
    background: none;
    padding: 0;
  }
  .content :global(blockquote) {
    margin: var(--az-space-2) 0;
    padding-left: var(--az-space-3);
    border-left: 3px solid var(--az-color-border);
    color: var(--az-color-text-muted);
  }
</style>
