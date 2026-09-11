<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import CodeEditor from "$lib/editor/CodeEditor.svelte";
  import Toolbar from "$lib/Toolbar.svelte";
  import StatusBar from "$lib/StatusBar.svelte";
  import Console from "$lib/panels/Console.svelte";
  import AssetsPanel from "$lib/panels/AssetsPanel.svelte";
  import type { ConsoleLine } from "$lib/panels/types";
  import { t } from "$lib/i18n";

  type Settings = { sketches_dir: string | null; theme: string };

  let projectPath = $state<string | null>(null);
  let code = $state("");
  let savedCode = $state("");
  let running = $state(false);
  let fontSize = $state(16);
  let consoleLines = $state<ConsoleLine[]>([]);
  let consoleExpanded = $state(false);
  let runtimeOk = $state<boolean | null>(null);
  let assetsCollapsed = $state(false);
  let settings = $state<Settings>({ sketches_dir: null, theme: "dia" });
  let editor: CodeEditor | undefined = $state();

  const dirty = $derived(code !== savedCode);

  async function loadSettings() {
    try {
      settings = await invoke<Settings>("get_settings");
      document.documentElement.dataset.theme = settings.theme || "dia";
    } catch (e) {
      console.error("get_settings fallo", e);
    }
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
  }

  async function handleNew() {
    const suggestion = settings.sketches_dir
      ? `${settings.sketches_dir}\\mi-juego`
      : "";
    const dest = window.prompt(t("prompt.newProjectFolder"), suggestion);
    if (!dest) return;
    const folder = await invoke<string>("new_project", { template: "en-blanco", dest });
    await openAt(folder);
  }

  async function handleOpen() {
    const dest = window.prompt(t("prompt.openProjectFolder"), projectPath ?? "");
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
    await invoke("run_project", { path: projectPath });
  }

  async function handleStop() {
    await invoke("stop_run");
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
      // TODO(friendly-errors): interceptar lineas con prefijo ##ARCADEZERO##
      // (JSON estructurado, ver PLAN.md §3.2) y mostrarlas como tarjeta de
      // error amigable en vez de texto crudo, cuando exista el evento
      // `run_error` (falta en el backend hoy, ver src-tauri/src/lib.rs).
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
    onnew={handleNew}
    onopen={handleOpen}
    onsave={handleSave}
    onplay={handlePlay}
    onstop={handleStop}
    onzoomin={handleZoomIn}
    onzoomout={handleZoomOut}
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
      <AssetsPanel {projectPath} bind:collapsed={assetsCollapsed} />
      <div class="main">
        <CodeEditor bind:this={editor} bind:value={code} bind:fontSize />
      </div>
    </div>
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
