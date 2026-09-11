// Resalta indentacion inconsistente (PLAN.md §6.5): tabs mezclados con
// espacios, o indentacion que no es multiplo de 4. Reusa @codemirror/lint
// (ya trae subrayado + tooltip) en vez de inventar decoraciones propias.
import { linter, type Diagnostic } from "@codemirror/lint";
import type { EditorView } from "@codemirror/view";

export type IndentIssue = {
  kind: "mixed" | "non-multiple";
  message: string;
};

/** Pura y testeable: revisa la indentacion de una sola linea. */
export function checkIndentIssue(lineText: string, unit = 4): IndentIssue | null {
  let i = 0;
  let spaces = 0;
  let tabs = 0;
  while (i < lineText.length && (lineText[i] === " " || lineText[i] === "\t")) {
    if (lineText[i] === " ") spaces += 1;
    else tabs += 1;
    i += 1;
  }
  if (i === 0 || i === lineText.length) return null; // sin indentar o linea en blanco

  if (tabs > 0 && spaces > 0) {
    return {
      kind: "mixed",
      message: "Mezclaste tabs y espacios en la indentación. Usa solo espacios (4 por nivel).",
    };
  }
  if (tabs > 0) return null; // tabs puros: no se penaliza en v1

  if (spaces % unit !== 0) {
    return {
      kind: "non-multiple",
      message: `La indentación tiene ${spaces} espacios; usa múltiplos de 4.`,
    };
  }
  return null;
}

export const indentConsistencyLinter = () =>
  linter((view: EditorView) => {
    const diagnostics: Diagnostic[] = [];
    for (let i = 1; i <= view.state.doc.lines; i++) {
      const line = view.state.doc.line(i);
      const issue = checkIndentIssue(line.text);
      if (issue) {
        const leadingLen = line.text.length - line.text.trimStart().length;
        diagnostics.push({
          from: line.from,
          to: line.from + Math.max(leadingLen, 1),
          severity: "warning",
          message: issue.message,
        });
      }
    }
    return diagnostics;
  });
