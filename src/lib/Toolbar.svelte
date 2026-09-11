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
  import PaletteIcon from "phosphor-svelte/lib/PaletteIcon";
  import GearSixIcon from "phosphor-svelte/lib/GearSixIcon";

  const THEME_LABELS: Record<string, string> = {
    dia: "Día",
    dracula: "Dracula",
    "one-dark-pro": "One Dark Pro",
    "alto-contraste": "Alto contraste",
  };

  let {
    running = false,
    theme = "dia",
    dyslexicFont = false,
    uiScale = "normal",
    onnew,
    onopen,
    onsave,
    onplay,
    onstop,
    onzoomin,
    onzoomout,
    ontheme,
    ondyslexicfont,
    onuiscale,
  }: {
    running?: boolean;
    theme?: string;
    dyslexicFont?: boolean;
    uiScale?: string;
    onnew?: () => void;
    onopen?: () => void;
    onsave?: () => void;
    onplay?: () => void;
    onstop?: () => void;
    onzoomin?: () => void;
    onzoomout?: () => void;
    ontheme?: () => void;
    ondyslexicfont?: (value: boolean) => void;
    onuiscale?: (value: string) => void;
  } = $props();

  const ICON_SIZE = 24;
  let settingsOpen = $state(false);
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

  <div class="sep" role="separator"></div>

  <button
    class="tbtn"
    onclick={ontheme}
    title={`${t("toolbar.theme")}: ${THEME_LABELS[theme] ?? theme}`}
  >
    <PaletteIcon size={ICON_SIZE} aria-hidden="true" />
    <span>{THEME_LABELS[theme] ?? theme}</span>
  </button>

  <div class="sep" role="separator"></div>

  <div class="settings-wrap">
    <button
      class="tbtn"
      onclick={() => (settingsOpen = !settingsOpen)}
      title={t("toolbar.settings")}
      aria-expanded={settingsOpen}
      aria-haspopup="true"
    >
      <GearSixIcon size={ICON_SIZE} aria-hidden="true" />
      <span>{t("toolbar.settings")}</span>
    </button>
    {#if settingsOpen}
      <div class="settings-popover" role="dialog" aria-label={t("settings.title")}>
        <h3>{t("settings.title")}</h3>
        <label class="row">
          <input
            type="checkbox"
            checked={dyslexicFont}
            onchange={(e) => ondyslexicfont?.(e.currentTarget.checked)}
          />
          {t("settings.dyslexicFont")}
        </label>
        <label class="row">
          {t("settings.uiScale")}
          <select value={uiScale} onchange={(e) => onuiscale?.(e.currentTarget.value)}>
            <option value="normal">{t("settings.uiScaleNormal")}</option>
            <option value="grande">{t("settings.uiScaleLarge")}</option>
            <option value="muy-grande">{t("settings.uiScaleExtraLarge")}</option>
          </select>
        </label>
        <button class="close-btn" onclick={() => (settingsOpen = false)}>
          {t("settings.close")}
        </button>
      </div>
    {/if}
  </div>
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
  .settings-wrap {
    position: relative;
  }
  .settings-popover {
    position: absolute;
    top: calc(100% + var(--az-space-1));
    right: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: var(--az-space-2);
    min-width: 240px;
    padding: var(--az-space-3);
    background: var(--az-color-panel-bg);
    border: 1px solid var(--az-color-border);
    border-radius: var(--az-radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }
  .settings-popover h3 {
    margin: 0;
    font-size: 0.9rem;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--az-space-2);
    font-size: 0.85rem;
  }
  .row select {
    font-family: inherit;
    font-size: 0.85rem;
    padding: 2px 4px;
    border-radius: var(--az-radius);
    border: 1px solid var(--az-color-border);
    background: var(--az-color-panel-bg);
    color: var(--az-color-text);
  }
  .close-btn {
    align-self: flex-end;
    padding: var(--az-space-1) var(--az-space-2);
    border-radius: var(--az-radius);
    border: 1px solid var(--az-color-border);
    background: none;
    color: var(--az-color-text);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.8rem;
  }
</style>
