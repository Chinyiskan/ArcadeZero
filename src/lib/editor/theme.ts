// Look visual del editor (PLAN.md §6.4): usa las variables CSS de
// styles/theme.css, para que Dracula / One Dark Pro / Alto contraste solo
// cambien las variables, no este archivo.
import { EditorView } from "@codemirror/view";
import { HighlightStyle } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

// Resaltado de sintaxis atado a las variables --az-syntax-* (theme.css), asi
// que reacciona solo con cambiar `data-theme` en <html>, sin reconfigurar
// CodeMirror (PLAN.md §6.4: 3 temas, mismo resaltado por variables).
export const syntaxColors = HighlightStyle.define([
  { tag: [t.keyword, t.controlKeyword, t.moduleKeyword], color: "var(--az-syntax-keyword)" },
  { tag: [t.string, t.special(t.string)], color: "var(--az-syntax-string)" },
  { tag: [t.comment, t.lineComment, t.blockComment], color: "var(--az-syntax-comment)", fontStyle: "italic" },
  { tag: [t.number, t.bool, t.null], color: "var(--az-syntax-number)" },
  { tag: [t.function(t.variableName), t.function(t.propertyName), t.className], color: "var(--az-syntax-function)" },
  { tag: [t.operator, t.punctuation, t.bracket], color: "var(--az-syntax-operator)" },
  { tag: [t.variableName, t.propertyName, t.definition(t.variableName)], color: "var(--az-syntax-variable)" },
]);

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
