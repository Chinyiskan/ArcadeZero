// Look visual del editor (PLAN.md §6.4): usa las variables CSS de
// styles/theme.css, para que Noche / Alto contraste (Fase 4) solo cambien
// las variables, no este archivo.
import { EditorView } from "@codemirror/view";

export const editorTheme = EditorView.theme({
  "&": {
    fontFamily: "var(--az-font-editor)",
    color: "var(--az-color-text)",
    backgroundColor: "var(--az-color-editor-bg)",
    height: "100%",
  },
  ".cm-content": {
    fontFamily: "inherit",
    caretColor: "var(--az-color-accent)",
  },
  ".cm-scroller": {
    fontFamily: "inherit",
    lineHeight: "1.5",
  },
  ".cm-gutters": {
    backgroundColor: "var(--az-color-editor-gutter)",
    color: "var(--az-color-text-muted)",
    border: "none",
  },
  ".cm-activeLine": { backgroundColor: "var(--az-color-editor-active-line)" },
  ".cm-activeLineGutter": { backgroundColor: "var(--az-color-editor-active-line)" },
  "&.cm-focused": {
    outline: "2px solid var(--az-color-focus)",
    outlineOffset: "-1px",
  },
  ".cm-diagnostic-warning": {
    borderLeft: "3px solid var(--az-color-warning)",
  },
  ".cm-tooltip.cm-tooltip-lint": {
    fontFamily: "var(--az-font-ui)",
  },
});

export function fontSizeTheme(size: number) {
  return EditorView.theme({
    "&": { fontSize: `${size}px` },
  });
}
