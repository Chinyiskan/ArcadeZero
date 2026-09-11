// Guias de indentacion "rainbow" (PLAN.md §6.5).
//
// Se evaluo primero @replit/codemirror-indentation-markers: solo pinta UN
// color de marcador (claro/oscuro), no un color por nivel de anidamiento
// como pide el plan (estilo indent-rainbow de VS Code). Por eso va el
// fallback propio con Decoration.line que describe el plan: pocas lineas,
// sin dependencia extra, coloreando por `nivel % N` via CSS custom
// properties (--az-indent-0..N) definidas en styles/theme.css.
import { RangeSetBuilder } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";

const PALETTE_SIZE = 6;

/**
 * Cuenta cuantos niveles de indentacion de `unit` espacios tiene una linea.
 * Pura y testeable a proposito (ver editor.test.ts).
 */
export function indentLevel(lineText: string, unit = 4): number {
  let spaces = 0;
  for (const ch of lineText) {
    if (ch === " ") spaces += 1;
    else if (ch === "\t") spaces += unit; // aproxima un tab a un nivel
    else break;
  }
  return Math.floor(spaces / unit);
}

/** Construye el `style` inline con N lineas verticales de 1px, una por nivel. */
export function guideStyle(level: number, unit = 4): string {
  if (level <= 0) return "";
  const layers: string[] = [];
  const positions: string[] = [];
  const sizes: string[] = [];
  for (let i = 0; i < level; i++) {
    const color = `var(--az-indent-${i % PALETTE_SIZE})`;
    layers.push(`linear-gradient(${color}, ${color})`);
    positions.push(`${i * unit}ch 0`);
    sizes.push("var(--az-indent-width, 1px) 100%");
  }
  return (
    `background-image:${layers.join(",")};` +
    `background-repeat:no-repeat;` +
    `background-position:${positions.join(",")};` +
    `background-size:${sizes.join(",")};`
  );
}

function buildDecorations(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  for (const { from, to } of view.visibleRanges) {
    let pos = from;
    while (pos <= to) {
      const line = view.state.doc.lineAt(pos);
      const level = indentLevel(line.text);
      if (level > 0) {
        builder.add(
          line.from,
          line.from,
          Decoration.line({ attributes: { style: guideStyle(level) } }),
        );
      }
      pos = line.to + 1;
    }
  }
  return builder.finish();
}

export function indentGuides() {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = buildDecorations(view);
      }
      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged || update.geometryChanged) {
          this.decorations = buildDecorations(update.view);
        }
      }
    },
    { decorations: (v) => v.decorations },
  );
}
