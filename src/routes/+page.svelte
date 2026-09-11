<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import CodeEditor from "$lib/editor/CodeEditor.svelte";
  import Toolbar from "$lib/Toolbar.svelte";
  import StatusBar from "$lib/StatusBar.svelte";
  import Console from "$lib/panels/Console.svelte";
  import AssetsPanel from "$lib/panels/AssetsPanel.svelte";
  import ErrorCard from "$lib/panels/ErrorCard.svelte";
  import HelpPanel from "$lib/panels/HelpPanel.svelte";
  import TemplatePicker from "$lib/panels/TemplatePicker.svelte";
  import TabBar from "$lib/TabBar.svelte";
  import ImagePreview from "$lib/panels/ImagePreview.svelte";
  import type { ConsoleLine, StructuredError } from "$lib/panels/types";
  import { t, setLocale, type Locale } from "$lib/i18n";
  import { nextTheme, systemDefaultTheme, type ThemeName } from "$lib/theme";
  import type { TemplateId } from "$lib/templates";
  import { activateTab, closeTab, initialTabs, openImageTab, type Tab } from "$lib/tabs";

  type Settings = {
    sketches_dir: string | null;
    theme: string | null;
    dyslexic_font: boolean | null;
    ui_scale: string | null;
    locale: string | null;
  };

  let projectPath = $state<string | null>(null);
  let code = $state("");
  let savedCode = $state("");
  let running = $state(false);
  let fontSize = $state(16);
  let consoleLines = $state<ConsoleLine[]>([]);
  let consoleExpanded = $state(false);
  let runError = $state<StructuredError | null>(null);
  let runtimeOk = $state<boolean | null>(null);
  let assetsCollapsed = $state(false);
  let settings = $state<Settings>({
    sketches_dir: null,
    theme: null,
    dyslexic_font: null,
    ui_scale: null,
    locale: null,
  });
  let theme = $state<ThemeName>("dia");
  let dyslexicFont = $state(false);
  let uiScale = $state("normal");
  let locale = $state<Locale>("es");
  let helpOpen = $state(false);
  let templatePickerOpen = $state(false);
  let editor: CodeEditor | undefined = $state();
  let tabs = $state<Tab[]>(initialTabs());
  let activeTabId = $state("main");

  const dirty = $derived(code !== savedCode);
  const activeTab = $derived(tabs.find((tab) => tab.id === activeTabId) ?? tabs[0]);

  function handleOpenImage(filename: string) {
    const result = openImageTab(tabs, filename);
    tabs = result.tabs;
    activeTabId = result.activeId;
  }

  function handleActivateTab(id: string) {
    activeTabId = activateTab(tabs, id);
  }

  function handleCloseTab(id: string) {
    const result = closeTab(tabs, activeTabId, id);
    tabs = result.tabs;
    activeTabId = result.activeId;
  }

  async function loadSettings() {
    try {
      settings = await invoke<Settings>("get_settings");
      // Sin tema guardado todavia: respeta prefers-color-scheme del SO
      // (PLAN.md §6.4). No se persiste hasta que el usuario cicle manual.
      const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
      theme = (settings.theme as ThemeName) || systemDefaultTheme(prefersDark);
      document.documentElement.dataset.theme = theme;
      dyslexicFont = settings.dyslexic_font ?? false;
      uiScale = settings.ui_scale ?? "normal";
      locale = (settings.locale as Locale) || "es";
      setLocale(locale);
      applyAccessibilitySettings();
    } catch (e) {
      console.error("get_settings fallo", e);
    }
  }

  function applyAccessibilitySettings() {
    document.documentElement.dataset.font = dyslexicFont ? "opendyslexic" : "";
    document.documentElement.dataset.uiScale = uiScale;
  }

  async function persistSettings(patch: Partial<Settings>) {
    settings = { ...settings, ...patch };
    try {
      await invoke("set_settings", { patch: settings });
    } catch (e) {
      console.error("set_settings fallo", e);
    }
  }

  async function handleTheme() {
    theme = nextTheme(theme);
    document.documentElement.dataset.theme = theme;
    await persistSettings({ theme });
  }

  async function handleDyslexicFont(value: boolean) {
    dyslexicFont = value;
    applyAccessibilitySettings();
    await persistSettings({ dyslexic_font: value });
  }

  async function handleUiScale(value: string) {
    uiScale = value;
    applyAccessibilitySettings();
    await persistSettings({ ui_scale: value });
  }

  async function handleLocale(value: string) {
    locale = value as Locale;
    setLocale(locale);
    await persistSettings({ locale: value });
  }

  async function refreshRuntimeStatus() {
    try {
      const status = await invoke<{ pgzero_ok: boolean }>("runtime_status");
      runtimeOk = status.pgzero_ok;
    } catch {
      runtimeOk = false;
    }
  }

  async function openAt(path: string) {
    const folder = await invoke<string>("open_project", { path });
    const content = await invoke<string>("read_file", { path: `${folder}\\main.py` });
    projectPath = folder;
    code = content;
    savedCode = content;
    tabs = initialTabs();
    activeTabId = "main";
  }

  // ponytail: selector de carpeta con dialogo nativo (tauri-plugin-dialog);
  // si la plataforma no lo soporta, cae a prompt de texto como red de
  // seguridad minima (mismo patron que AssetsPanel.svelte).
  async function pickDirectory(title: string, defaultPath?: string): Promise<string | null> {
    try {
      const picked = await openDialog({ title, directory: true, defaultPath });
      return typeof picked === "string" ? picked : null;
    } catch {
      return window.prompt(title, defaultPath ?? "");
    }
  }

  function handleNew() {
    templatePickerOpen = true;
  }

  async function handleTemplatePicked(templateId: TemplateId) {
    templatePickerOpen = false;
    const parent = await pickDirectory(t("prompt.newProjectParent"), settings.sketches_dir ?? undefined);
    if (!parent) return;
    const name = window.prompt(t("prompt.newProjectName"), "mi-juego");
    if (!name) return;
    const dest = `${parent}\\${name}`;
    const folder = await invoke<string>("new_project", { template: templateId, dest });
    await openAt(folder);
  }

  async function handleOpen() {
    const dest = await pickDirectory(t("prompt.openProjectFolder"), projectPath ?? undefined);
    if (!dest) return;
    await openAt(dest);
  }

  async function handleSave() {
    if (!projectPath) return;
    await invoke("save_file", { path: `${projectPath}\\main.py`, content: code });
    savedCode = code;
  }

  async function handlePlay() {
    if (!projectPath) return;
    if (dirty) await handleSave();
    consoleLines = [];
    consoleExpanded = true;
    runError = null;
    await invoke("run_project", { path: projectPath });
  }

  async function handleStop() {
    await invoke("stop_run");
  }

  function handleGoToLine(line: number) {
    editor?.goToLine(line);
  }

  function handleZoomIn() {
    editor?.zoom(1);
  }
  function handleZoomOut() {
    editor?.zoom(-1);
  }

  function handleKeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key === "s") {
      e.preventDefault();
      handleSave();
    } else if (mod && (e.key === "+" || e.key === "=")) {
      e.preventDefault();
      handleZoomIn();
    } else if (mod && e.key === "-") {
      e.preventDefault();
      handleZoomOut();
    } else if (mod && e.key.toLowerCase() === "o") {
      e.preventDefault();
      handleOpen();
    } else if (mod && e.key.toLowerCase() === "n") {
      e.preventDefault();
      handleNew();
    } else if (e.key === "F5") {
      e.preventDefault();
      handlePlay();
    } else if (e.key === "F6") {
      e.preventDefault();
      handleStop();
    } else if (e.key === "F1") {
      e.preventDefault();
      helpOpen = true;
    } else if (e.key === "Escape") {
      if (helpOpen) helpOpen = false;
      else if (templatePickerOpen) templatePickerOpen = false;
    }
  }

  onMount(() => {
    loadSettings();
    refreshRuntimeStatus();

    const unlisten: UnlistenFn[] = [];
    listen("run_start", () => {
      running = true;
    }).then((u) => unlisten.push(u));
    listen<string>("run_stdout", (e) => {
      consoleLines = [...consoleLines, { kind: "out", text: e.payload }];
      consoleExpanded = true;
    }).then((u) => unlisten.push(u));
    listen<string>("run_stderr", (e) => {
      consoleLines = [...consoleLines, { kind: "err", text: e.payload }];
      consoleExpanded = true;
    }).then((u) => unlisten.push(u));
    listen<StructuredError>("run_error", (e) => {
      runError = e.payload;
      consoleExpanded = true;
    }).then((u) => unlisten.push(u));
    listen<number | null>("run_exit", () => {
      running = false;
    }).then((u) => unlisten.push(u));

    window.addEventListener("keydown", handleKeydown);
    return () => {
      unlisten.forEach((u) => u());
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<div class="app">
  <Toolbar
    {running}
    {theme}
    {dyslexicFont}
    {uiScale}
    {locale}
    onnew={handleNew}
    onopen={handleOpen}
    onsave={handleSave}
    onplay={handlePlay}
    onstop={handleStop}
    onzoomin={handleZoomIn}
    onzoomout={handleZoomOut}
    ontheme={handleTheme}
    ondyslexicfont={handleDyslexicFont}
    onuiscale={handleUiScale}
    onlocale={handleLocale}
    onhelp={() => (helpOpen = true)}
  />

  <TemplatePicker
    open={templatePickerOpen}
    onpick={handleTemplatePicked}
    onclose={() => (templatePickerOpen = false)}
  />
  <HelpPanel
    open={helpOpen}
    onclose={() => (helpOpen = false)}
    onbrowsetemplates={() => {
      helpOpen = false;
      templatePickerOpen = true;
    }}
  />

  {#if !projectPath}
    <div class="onboarding">
      <div class="card">
        <h1>{t("onboarding.title")}</h1>
        <p>{t("onboarding.body")}</p>
        <div class="actions">
          <button class="primary" onclick={handleNew}>{t("onboarding.newBlank")}</button>
          <button onclick={handleOpen}>{t("onboarding.openExisting")}</button>
        </div>
      </div>
    </div>
  {:else}
    <div class="banner">{t("banner.firstRun")}</div>
    <div class="workspace">
      <AssetsPanel {projectPath} bind:collapsed={assetsCollapsed} onopenimage={handleOpenImage} />
      <div class="main">
        <TabBar {tabs} activeId={activeTabId} onactivate={handleActivateTab} onclose={handleCloseTab} />
        <div class="main-body">
          <div class="editor-slot" style:display={activeTab?.kind === "main" ? "block" : "none"}>
            <CodeEditor bind:this={editor} bind:value={code} bind:fontSize />
          </div>
          {#if activeTab && activeTab.kind === "image" && projectPath}
            <ImagePreview {projectPath} filename={activeTab.filename} />
          {/if}
        </div>
      </div>
    </div>
    {#if runError}
      <ErrorCard error={runError} ongotoline={handleGoToLine} />
    {/if}
    <Console bind:lines={consoleLines} bind:expanded={consoleExpanded} />
    <StatusBar fileName="main.py" saved={!dirty} {runtimeOk} />
  {/if}
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .workspace {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .main {
    flex: 1;
    min-height: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .main-body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-areas: "stack";
    grid-template-columns: 1fr;
    grid-template-rows: 1fr;
  }
  .main-body > :global(*) {
    grid-area: stack;
    min-height: 0;
    min-width: 0;
  }
  .editor-slot {
    height: 100%;
  }
  .banner {
    padding: var(--az-space-1) var(--az-space-3);
    background: var(--az-color-editor-active-line);
    color: var(--az-color-text-muted);
    font-size: 0.85rem;
  }
  .onboarding {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .card {
    max-width: 420px;
    text-align: center;
    padding: var(--az-space-4);
    border: 1px solid var(--az-color-border);
    border-radius: var(--az-radius);
    background: var(--az-color-panel-bg);
  }
  .actions {
    display: flex;
    gap: var(--az-space-2);
    justify-content: center;
    margin-top: var(--az-space-3);
  }
  .actions button {
    padding: var(--az-space-2) var(--az-space-3);
    border-radius: var(--az-radius);
    border: 1px solid var(--az-color-border);
    cursor: pointer;
    font-family: inherit;
  }
  .actions .primary {
    background: var(--az-color-accent);
    color: var(--az-color-accent-contrast);
    border-color: var(--az-color-accent);
  }
</style>
