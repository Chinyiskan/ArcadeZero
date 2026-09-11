<script lang="ts">
  // Toolbar real (PLAN.md §6.2): iconos Phosphor + etiqueta de texto, cero
  // emojis. Un icono por boton, tamano consistente (24px).
  import { t } from "$lib/i18n";
  import FilePlusIcon from "phosphor-svelte/lib/FilePlusIcon";
  import FolderOpenIcon from "phosphor-svelte/lib/FolderOpenIcon";
  import FloppyDiskIcon from "phosphor-svelte/lib/FloppyDiskIcon";
  import PlayIcon from "phosphor-svelte/lib/PlayIcon";
  import StopIcon from "phosphor-svelte/lib/StopIcon";
  import MagnifyingGlassPlusIcon from "phosphor-svelte/lib/MagnifyingGlassPlusIcon";
  import MagnifyingGlassMinusIcon from "phosphor-svelte/lib/MagnifyingGlassMinusIcon";

  let {
    running = false,
    onnew,
    onopen,
    onsave,
    onplay,
    onstop,
    onzoomin,
    onzoomout,
  }: {
    running?: boolean;
    onnew?: () => void;
    onopen?: () => void;
    onsave?: () => void;
    onplay?: () => void;
    onstop?: () => void;
    onzoomin?: () => void;
    onzoomout?: () => void;
  } = $props();

  const ICON_SIZE = 24;
</script>

<div class="toolbar" role="toolbar" aria-label="Barra de herramientas">
  <button class="tbtn" onclick={onnew} title={t("toolbar.new")}>
    <FilePlusIcon size={ICON_SIZE} aria-hidden="true" />
    <span>{t("toolbar.new")}</span>
  </button>
  <button class="tbtn" onclick={onopen} title={t("toolbar.open")}>
    <FolderOpenIcon size={ICON_SIZE} aria-hidden="true" />
    <span>{t("toolbar.open")}</span>
  </button>
  <button class="tbtn" onclick={onsave} title={t("toolbar.save")}>
    <FloppyDiskIcon size={ICON_SIZE} aria-hidden="true" />
    <span>{t("toolbar.save")}</span>
  </button>

  <div class="sep" role="separator"></div>

  <button
    class="tbtn primary"
    onclick={onplay}
    disabled={running}
    title={`${t("toolbar.play")} (F5)`}
  >
    <PlayIcon size={ICON_SIZE} weight="fill" aria-hidden="true" />
    <span>{t("toolbar.play")}</span>
  </button>
  <button
    class="tbtn danger"
    onclick={onstop}
    disabled={!running}
    title={`${t("toolbar.stop")} (F6)`}
  >
    <StopIcon size={ICON_SIZE} weight="fill" aria-hidden="true" />
    <span>{t("toolbar.stop")}</span>
  </button>

  <div class="sep" role="separator"></div>

  <button class="tbtn" onclick={onzoomout} title={`${t("toolbar.zoomOut")} (Ctrl -)`}>
    <MagnifyingGlassMinusIcon size={ICON_SIZE} aria-hidden="true" />
    <span>{t("toolbar.zoomOut")}</span>
  </button>
  <button class="tbtn" onclick={onzoomin} title={`${t("toolbar.zoomIn")} (Ctrl +)`}>
    <MagnifyingGlassPlusIcon size={ICON_SIZE} aria-hidden="true" />
    <span>{t("toolbar.zoomIn")}</span>
  </button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--az-space-2);
    padding: var(--az-space-2) var(--az-space-3);
    background: var(--az-color-panel-bg);
    border-bottom: 1px solid var(--az-color-border);
  }
  .sep {
    width: 1px;
    align-self: stretch;
    background: var(--az-color-border);
    margin: 0 var(--az-space-1);
  }
  .tbtn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    min-width: 64px;
    padding: var(--az-space-1) var(--az-space-2);
    border: 1px solid transparent;
    border-radius: var(--az-radius);
    background: transparent;
    color: var(--az-color-text);
    cursor: pointer;
    font-size: 0.8rem;
  }
  .tbtn:hover:not(:disabled) {
    background: var(--az-color-editor-active-line);
    border-color: var(--az-color-border);
  }
  .tbtn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .primary {
    color: var(--az-color-success);
  }
  .danger {
    color: var(--az-color-danger);
  }
</style>
