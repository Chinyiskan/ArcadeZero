<script lang="ts">
  // Panel de assets (PLAN.md §5, §6.1, §11 Fase 2): tres secciones
  // (Imagenes/Sonidos/Musica), listar/importar/borrar sobre el sketch
  // abierto. Colapsable. Se refresca solo con el evento `assets_changed`
  // (watcher del backend, ver src-tauri/src/assets.rs).
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { t } from "$lib/i18n";
  import {
    ASSET_EXTENSIONS,
    ASSET_KINDS,
    assetKindLabel,
    isPending,
    requestDelete,
    sortAssetNames,
    type AssetKind,
    type PendingDelete,
  } from "./assets";
  import CaretLeftIcon from "phosphor-svelte/lib/CaretLeftIcon";
  import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
  import ImageIcon from "phosphor-svelte/lib/ImageIcon";
  import SpeakerHighIcon from "phosphor-svelte/lib/SpeakerHighIcon";
  import MusicNoteIcon from "phosphor-svelte/lib/MusicNoteIcon";
  import PlusIcon from "phosphor-svelte/lib/PlusIcon";
  import TrashIcon from "phosphor-svelte/lib/TrashIcon";

  const KIND_ICON = { images: ImageIcon, sounds: SpeakerHighIcon, music: MusicNoteIcon };

  let {
    projectPath,
    collapsed = $bindable(false),
    onopenimage,
  }: {
    projectPath: string;
    collapsed?: boolean;
    onopenimage?: (filename: string) => void;
  } = $props();

  let assets = $state<Record<AssetKind, string[]>>({ images: [], sounds: [], music: [] });
  let error = $state<string | null>(null);
  let pending = $state<PendingDelete>(null);

  async function refresh() {
    error = null;
    for (const kind of ASSET_KINDS) {
      try {
        const names = await invoke<string[]>("list_assets", { projectPath, kind });
        assets[kind] = sortAssetNames(names);
      } catch (e) {
        error = String(e);
      }
    }
  }

  async function handleImport(kind: AssetKind) {
    error = null;
    const extensions = ASSET_EXTENSIONS[kind];
    let srcPath: string | null;
    try {
      srcPath = await openDialog({
        title: t("assets.importTitle"),
        multiple: false,
        filters: [{ name: assetKindLabel(kind), extensions }],
      });
    } catch {
      // Sin diálogo nativo disponible en esta plataforma: pedimos la ruta.
      srcPath = window.prompt(t("assets.importTitle"));
    }
    if (!srcPath || Array.isArray(srcPath)) return;
    try {
      await invoke<string>("import_asset", { projectPath, srcPath, kind });
      await refresh();
    } catch (e) {
      error = `${t("assets.importErrorPrefix")}: ${e}`;
    }
  }

  function askDelete(kind: AssetKind, filename: string) {
    pending = requestDelete(kind, filename);
  }

  function cancelDelete() {
    pending = null;
  }

  async function confirmDelete() {
    if (!pending) return;
    const { kind, filename } = pending;
    pending = null;
    try {
      await invoke("delete_asset", { projectPath, kind, filename });
      await refresh();
    } catch (e) {
      error = `${t("assets.deleteErrorPrefix")}: ${e}`;
    }
  }

  onMount(() => {
    refresh();
    let unlisten: UnlistenFn | undefined;
    listen("assets_changed", () => refresh()).then((u) => (unlisten = u));
    return () => unlisten?.();
  });

  $effect(() => {
    projectPath;
    refresh();
  });
</script>

{#if collapsed}
  <button
    class="collapsed-strip"
    onclick={() => (collapsed = false)}
    title={t("assets.expand")}
    aria-label={t("assets.expand")}
  >
    <CaretRightIcon size={18} aria-hidden="true" />
  </button>
{:else}
  <aside class="assets-panel" aria-label={t("assets.title")}>
    <div class="panel-header">
      <span>{t("assets.title")}</span>
      <button
        class="icon-btn"
        onclick={() => (collapsed = true)}
        title={t("assets.collapse")}
        aria-label={t("assets.collapse")}
      >
        <CaretLeftIcon size={16} aria-hidden="true" />
      </button>
    </div>

    {#if error}
      <p class="error">{error}</p>
    {/if}

    {#each ASSET_KINDS as kind (kind)}
      {@const Icon = KIND_ICON[kind]}
      <section class="kind-section">
        <h3>
          <Icon size={16} aria-hidden="true" />
          {t(`assets.${kind}`)}
        </h3>
        <ul>
          {#each assets[kind] as filename (filename)}
            <li>
              {#if isPending(pending, kind, filename)}
                <span class="confirm">
                  {t("assets.deleteConfirmPrefix")} "{filename}"?
                  <button class="confirm-yes" onclick={confirmDelete}
                    >{t("assets.deleteYes")}</button
                  >
                  <button class="confirm-no" onclick={cancelDelete}
                    >{t("assets.deleteNo")}</button
                  >
                </span>
              {:else}
                {#if kind === "images"}
                  <button
                    class="filename-btn"
                    onclick={() => onopenimage?.(filename)}
                    title={`${t("assets.openPreview")} ${filename}`}
                  >
                    <span class="filename">{filename}</span>
                  </button>
                {:else}
                  <span class="filename">{filename}</span>
                {/if}
                <button
                  class="icon-btn danger"
                  onclick={() => askDelete(kind, filename)}
                  title={`${t("assets.delete")} ${filename}`}
                  aria-label={`${t("assets.delete")} ${filename}`}
                >
                  <TrashIcon size={14} aria-hidden="true" />
                </button>
              {/if}
            </li>
          {:else}
            <li class="empty">{t("assets.empty")}</li>
          {/each}
        </ul>
        <button class="import-btn" onclick={() => handleImport(kind)}>
          <PlusIcon size={14} aria-hidden="true" />
          {t("assets.import")}
        </button>
      </section>
    {/each}
  </aside>
{/if}

<style>
  .assets-panel {
    width: 220px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: var(--az-space-2);
    padding: var(--az-space-2);
    background: var(--az-color-panel-bg);
    border-right: 1px solid var(--az-color-border);
    overflow-y: auto;
  }
  .collapsed-strip {
    width: 20px;
    flex-shrink: 0;
    background: var(--az-color-panel-bg);
    border: none;
    border-right: 1px solid var(--az-color-border);
    color: var(--az-color-text-muted);
    cursor: pointer;
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-weight: 700;
    padding: 0 var(--az-space-1);
  }
  .kind-section h3 {
    display: flex;
    align-items: center;
    gap: var(--az-space-1);
    font-size: 0.85rem;
    margin: var(--az-space-2) 0 var(--az-space-1);
    color: var(--az-color-text-muted);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--az-space-1);
    font-size: 0.8rem;
    padding: 2px var(--az-space-1);
    border-radius: var(--az-radius);
  }
  li:hover {
    background: var(--az-color-editor-active-line);
  }
  li.empty {
    color: var(--az-color-text-muted);
    font-style: italic;
  }
  .filename {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .filename-btn {
    display: block;
    overflow: hidden;
    min-width: 0;
    flex: 1;
    background: none;
    border: none;
    color: inherit;
    font-family: inherit;
    font-size: inherit;
    text-align: left;
    cursor: pointer;
    padding: 0;
  }
  .filename-btn:hover .filename,
  .filename-btn:focus-visible .filename {
    text-decoration: underline;
  }
  .filename-btn:focus-visible {
    outline: 2px solid var(--az-color-accent);
    outline-offset: 1px;
  }
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--az-color-text-muted);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--az-radius);
  }
  .icon-btn:hover {
    background: var(--az-color-border);
  }
  .icon-btn.danger:hover {
    color: var(--az-color-danger);
  }
  .import-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: var(--az-space-1);
    padding: var(--az-space-1) var(--az-space-2);
    border: 1px dashed var(--az-color-border);
    border-radius: var(--az-radius);
    background: none;
    color: var(--az-color-text);
    cursor: pointer;
    font-size: 0.8rem;
    font-family: inherit;
  }
  .import-btn:hover {
    background: var(--az-color-editor-active-line);
  }
  .confirm {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
    font-size: 0.75rem;
    background: var(--az-color-editor-active-line);
    border-radius: var(--az-radius);
    padding: 4px;
  }
  .confirm button {
    font-family: inherit;
    font-size: 0.75rem;
    border-radius: var(--az-radius);
    border: 1px solid var(--az-color-border);
    cursor: pointer;
    padding: 2px 6px;
  }
  .confirm-yes {
    color: var(--az-color-accent-contrast);
    background: var(--az-color-danger);
    border-color: var(--az-color-danger);
  }
  .confirm-no {
    background: none;
  }
  .error {
    color: var(--az-color-danger);
    font-size: 0.75rem;
  }
</style>
