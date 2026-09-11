export type ConsoleLine = { kind: "out" | "err"; text: string };

// Forma exacta emitida por el backend en el evento `run_error`
// (src-tauri/src/error_parse.rs::StructuredError). Reemplaza a run_stderr
// solo para las lineas ##ARCADEZERO## (PLAN.md §3.3/§7).
export type StructuredError = {
  type: string;
  message: string;
  file: string | null;
  lineno: number | null;
  friendly_es: string;
  friendly_en: string;
  traceback: string;
};
