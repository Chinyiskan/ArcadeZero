<script lang="ts">
  // Barra de pestañas (PLAN.md §6.1): main.py fija/no-cerrable + una
  // pestaña de solo-lectura por imagen abierta desde el panel de assets.
  // Patrón ARIA "tabs" estándar (role=tablist/tab, flechas para moverse,
  // Home/End a los extremos) — no rompe el resto de navegación por teclado
  // porque solo actúa cuando el foco ya está dentro de la tablist.
  import FileCodeIcon from "phosphor-svelte/lib/FileCodeIcon";
  import ImageIcon from "phosphor-svelte/lib/ImageIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import { t } from "$lib/i18n";
  import type { Tab } from "./tabs";

  let {
    tabs,
    activeId,
    onactivate,
    onclose,
  }: {
    tabs: Tab[];
    activeId: string;
    onactivate: (id: string) => void;
    onclose: (id: string) => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent, index: number) {
    let target = -1;
    if (e.key === "ArrowRight") target = (index + 1) % tabs.length;
    else if (e.key === "ArrowLeft") target = (index - 1 + tabs.length) % tabs.length;
    else if (e.key === "Home") target = 0;
    else if (e.key === "End") target = tabs.length - 1;
    else return;
    e.preventDefault();
    const tab = tabs[target];
    onactivate(tab.id);
    const btn = document.getElementById(`az-tab-${tab.id}`);
    btn?.focus();
  }
</script>

<div class="tabbar" role="tablist" aria-label="Pestañas">
  {#each tabs as tab, index (tab.id)}
    <div class="tab" class:active={tab.id === activeId}>
      <button
        id={`az-tab-${tab.id}`}
        role="tab"
        aria-selected={tab.id === activeId}
        tabindex={tab.id === activeId ? 0 : -1}
        class="tab-label"
        onclick={() => onactivate(tab.id)}
        onkeydown={(e) => handleKeydown(e, index)}
      >
        {#if tab.kind === "main"}
          <FileCodeIcon size={14} aria-hidden="true" />
        {:else}
          <ImageIcon size={14} aria-hidden="true" />
        {/if}
        <span class="label">{tab.label}</span>
      </button>
      {#if tab.kind !== "main"}
        <button
          class="tab-close"
          title={`${t("tabs.closeFor")} ${tab.label}`}
          aria-label={`${t("tabs.closeFor")} ${tab.label}`}
          onclick={() => onclose(tab.id)}
        >
          <XIcon size={12} aria-hidden="true" />
        </button>
      {/if}
    </div>
  {/each}
</div>

<style>
  .tabbar {
    display: flex;
    align-items: stretch;
    background: var(--az-color-panel-bg);
    border-bottom: 1px solid var(--az-color-border);
    overflow-x: auto;
  }
  .tab {
    display: flex;
    align-items: center;
    border-right: 1px solid var(--az-color-border);
    flex-shrink: 0;
  }
  .tab.active {
    background: var(--az-color-bg);
    box-shadow: inset 0 -2px var(--az-color-accent);
  }
  .tab-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: inherit;
    font-size: 0.85rem;
    background: none;
    border: none;
    color: var(--az-color-text);
    cursor: pointer;
    padding: var(--az-space-1) var(--az-space-2);
    white-space: nowrap;
    max-width: 180px;
  }
  .tab-label:focus-visible {
    outline: 2px solid var(--az-color-accent);
    outline-offset: -2px;
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tab-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--az-color-text-muted);
    cursor: pointer;
    padding: 4px;
    margin-right: 4px;
    border-radius: var(--az-radius);
  }
  .tab-close:hover {
    background: var(--az-color-editor-active-line);
    color: var(--az-color-danger);
  }
  .tab-close:focus-visible {
    outline: 2px solid var(--az-color-accent);
  }
</style>
