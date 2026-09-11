// Autocompletado "clasico" (no IA, no LSP): diccionario estatico de Python
// + snippets. Se suma al localCompletionSource/globalCompletion que ya trae
// @codemirror/lang-python (ver PLAN.md §6.5). Ver PythonEditor.svelte.
import { snippetCompletion, type Completion, type CompletionSource } from "@codemirror/autocomplete";

const KEYWORDS = [
  "False", "None", "True", "and", "as", "assert", "async", "await",
  "break", "class", "continue", "def", "del", "elif", "else", "except",
  "finally", "for", "from", "global", "if", "import", "in", "is",
  "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
  "while", "with", "yield",
];

const BUILTINS = [
  "print", "range", "len", "input", "int", "float", "str", "list",
  "dict", "set", "tuple", "abs", "min", "max", "sum", "sorted",
  "enumerate", "zip", "open", "type", "isinstance", "round", "bool",
  "map", "filter", "reversed", "any", "all",
];

const SNIPPETS = [
  snippetCompletion("def ${nombre}(${}):\n\t${}", {
    label: "def",
    type: "keyword",
    detail: "definir función",
  }),
  snippetCompletion("for ${elemento} in ${iterable}:\n\t${}", {
    label: "for",
    type: "keyword",
    detail: "bucle for",
  }),
  snippetCompletion("if ${condicion}:\n\t${}", {
    label: "if",
    type: "keyword",
    detail: "condicional",
  }),
  snippetCompletion("while ${condicion}:\n\t${}", {
    label: "while",
    type: "keyword",
    detail: "bucle while",
  }),
  snippetCompletion("class ${Nombre}:\n\tdef __init__(self):\n\t\t${}", {
    label: "class",
    type: "keyword",
    detail: "definir clase",
  }),
];

const STATIC_OPTIONS: Completion[] = [
  ...KEYWORDS.map((label): Completion => ({ label, type: "keyword", boost: -1 })),
  ...BUILTINS.map((label): Completion => ({ label, type: "function" })),
  ...SNIPPETS,
  // TODO(pgzero-domain): agregar la API de pgzero (Actor, screen, keyboard,
  // sounds, images, music, clock, animate, los callbacks draw()/update()/
  // on_key_down()...) cuando el cheatsheet de pgzero-domain este listo.
];

/** Fuente de completado propia y estatica (keywords + builtins + snippets). */
export const arcadeZeroCompletions: CompletionSource = (context) => {
  const word = context.matchBefore(/\w*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;
  return {
    from: word.from,
    options: STATIC_OPTIONS,
    validFor: /^\w*$/,
  };
};
