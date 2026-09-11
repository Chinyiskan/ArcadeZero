<script lang="ts">
  // Integracion de CodeMirror 6 para editar main.py (PLAN.md §6.5).
  import { onMount } from "svelte";
  import { Compartment, EditorState } from "@codemirror/state";
  import {
    EditorView,
    keymap,
    lineNumbers,
    highlightActiveLine,
    highlightActiveLineGutter,
  } from "@codemirror/view";
  import {
    defaultKeymap,
    history,
    historyKeymap,
    indentWithTab,
  } from "@codemirror/commands";
  import { python, pythonLanguage } from "@codemirror/lang-python";
  import { bracketMatching, indentUnit, syntaxHighlighting } from "@codemirror/language";
  import {
    autocompletion,
    closeBrackets,
    closeBracketsKeymap,
    completionKeymap,
  } from "@codemirror/autocomplete";
  import { searchKeymap } from "@codemirror/search";
  import { lintKeymap } from "@codemirror/lint";
  import { arcadeZeroCompletions } from "./completions";
  import { indentGuides } from "./indentGuides";
  import { indentConsistencyLinter } from "./indentLint";
  import { editorTheme, fontSizeTheme, syntaxColors } from "./theme";

  let {
    value = $bindable(""),
    fontSize = $bindable(16),
    onchange,
  }: {
    value?: string;
    fontSize?: number;
    onchange?: (value: string) => void;
  } = $props();

  let container: HTMLDivElement;
  let view: EditorView | undefined;
  const fontSizeCompartment = new Compartment();
  let lastKnownValue = value;

  export function focus() {
    view?.focus();
  }

  /** Zoom de fuente del editor (toolbar A+/A-, PLAN.md §6.2). */
  export function zoom(delta: number) {
    fontSize = Math.min(28, Math.max(10, fontSize + delta));
    view?.dispatch({
      effects: fontSizeCompartment.reconfigure(fontSizeTheme(fontSize)),
    });
  }

  /** Para el futuro boton "ir a la linea" de friendly-errors. */
  export function goToLine(line: number) {
    if (!view) return;
    const clamped = Math.max(1, Math.min(line, view.state.doc.lines));
    const lineInfo = view.state.doc.line(clamped);
    view.dispatch({
      selection: { anchor: lineInfo.from },
      effects: EditorView.scrollIntoView(lineInfo.from, { y: "center" }),
    });
    view.focus();
  }

  onMount(() => {
    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLine(),
        highlightActiveLineGutter(),
        history(),
        closeBrackets(),
        bracketMatching(),
        indentUnit.of("    "),
        syntaxHighlighting(syntaxColors, { fallback: true }),
        python(),
        // Fuente propia sumada al localCompletionSource/globalCompletion
        // que ya activa python() por defecto (ver completions.ts).
        pythonLanguage.data.of({ autocomplete: arcadeZeroCompletions }),
        autocompletion({ activateOnTyping: true }),
        indentGuides(),
        indentConsistencyLinter(),
        fontSizeCompartment.of(fontSizeTheme(fontSize)),
        editorTheme,
        keymap.of([
          indentWithTab,
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...historyKeymap,
          ...searchKeymap,
          ...completionKeymap,
          ...lintKeymap,
        ]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const text = update.state.doc.toString();
            lastKnownValue = text;
            value = text;
            onchange?.(text);
          }
        }),
      ],
    });
    view = new EditorView({ state, parent: container });
    return () => view?.destroy();
  });

  // Cambios externos al valor (abrir otro sketch, plantilla nueva) empujan
  // el documento del editor sin pelear con lo que el usuario esta tecleando.
  $effect(() => {
    const external = value;
    if (view && external !== lastKnownValue && external !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: external },
      });
      lastKnownValue = external;
    }
  });
</script>

<div class="cm-container" bind:this={container}></div>

<style>
  .cm-container {
    height: 100%;
    width: 100%;
    overflow: hidden;
  }
  .cm-container :global(.cm-editor) {
    height: 100%;
  }
</style>
