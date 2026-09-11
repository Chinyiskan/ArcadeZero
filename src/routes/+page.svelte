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
  import type { ConsoleLine } from "$lib/panels/types";
  import { t } from "$lib/i18n";
  import { nextTheme, systemDefaultTheme, type ThemeName } from "$lib/theme";

  type Settings = { sketches_dir: string | null; theme: string | null };

  let projectPath = $state<string | null>(null);
  let code = $state("");
  let savedCode = $state("");
  let running = $state(false);
  let fontSize = $state(16);
  let consoleLines = $state<ConsoleLine[]>([]);
  let consoleExpanded = $state(false);
  let runtimeOk = $state<boolean | null>(null);
  let assetsCollapsed = $state(false);
  let settings = $state<Settings>({ sketches_dir: null, theme: null });
  let theme = $state<ThemeName>("dia");
  let editor: CodeEditor | undefined = $state();

  const dirty = $derived(code !== savedCode);

  async function loadSettings() {
    try {
      settings = await invoke<Settings>("get_settings");
      // Sin tema guardado todavia: respeta prefers-color-scheme del SO
      // (PLAN.md §6.4). No se persiste hasta que el usuario cicle manual.
      const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
      theme = (settings.theme as ThemeName) || systemDefaultTheme(prefersDark);
      document.documentElement.dataset.theme = theme;
    } catch (e) {
      console.error("get_settings fallo", e);
    }
  }

  async function handleTheme() {
    theme = nextTheme(theme);
    document.documentElement.dataset.theme = theme;
    settings = { ...settings, theme };
    try {
      await invoke("set_settings", { patch: settings });
    } catch (e) {
      console.error("set_settings fallo", e);
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

  async function handleNew() {
    const parent = await pickDirectory(t("prompt.newProjectParent"), settings.sketches_dir ?? undefined);
    if (!parent) return;
    const name = window.prompt(t("prompt.newProjectName"), "mi-juego");
    if (!name) return;
    const dest = `${parent}\\${name}`;
    const folder = await invoke<string>("new_project", { template: "en-blanco", dest });
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
    }).then((u) => unlisten.push(u));
    // TODO(editor-ux): el backend ya emite `run_error` (src-tauri/src/run.rs
    // + error_parse.rs) para las lineas ##ARCADEZERO## que antes llegaban
    // como run_stderr crudo. Falta consumirlo aca: tarjeta de error +
    // "Ir a la linea" (PLAN.md §7). Forma exacta del payload (serde
    // `StructuredError`, ver src-tauri/src/error_parse.rs):
    //   {
    //     type: string;        // ej. "NameError", "SyntaxError", "error" (pygame.error)
    //     message: string;     // mensaje crudo de Python, nunca se oculta
    //     file: string | null;
    //     lineno: number | null;
    //     friendly_es: string; // texto amigable ya calculado en Rust (catalogo)
    //     friendly_en: string;
    //     traceback: string;   // traceback completo, para el detalle plegable
    //   }
    // listen<StructuredError>("run_error", (e) => { ... })
    // Las lineas ##ARCADEZERO## YA NO llegan por run_stderr (run.rs las
    // intercepta antes de emitir), asi que el TODO anterior de parsear el
    // prefijo aca quedo obsoleto.
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
    onnew={handleNew}
    onopen={handleOpen}
    onsave={handleSave}
    onplay={handlePlay}
    onstop={handleStop}
    onzoomin={handleZoomIn}
    onzoomout={handleZoomOut}
    ontheme={handleTheme}
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
