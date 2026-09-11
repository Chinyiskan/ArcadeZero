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

// API de pgzero (docs/pgzero-cheatsheet.md, verificada contra el vendor en
// runtime/vendored/pgzero/). Inyectada por pgzero como globals de main.py,
// sin import: cubrimos los nombres de alto nivel + snippets de los
// callbacks que pgzero busca por nombre exacto.
const PGZERO_GLOBALS = [
  { label: "Actor", type: "class", detail: "sprite: Actor(\"nombre\", pos)" },
  { label: "screen", type: "variable", detail: "dibujar en la ventana" },
  { label: "keyboard", type: "variable", detail: "estado del teclado" },
  { label: "mouse", type: "variable", detail: "constantes de botones del ratón" },
  { label: "keys", type: "variable", detail: "constantes de teclas (on_key_down)" },
  { label: "keymods", type: "variable", detail: "modificadores (CTRL, SHIFT, ALT...)" },
  { label: "images", type: "variable", detail: "images.<nombre> carga images/<nombre>.png" },
  { label: "sounds", type: "variable", detail: "sounds.<nombre> carga sounds/<nombre>.wav" },
  { label: "music", type: "variable", detail: "reproducir música de fondo" },
  { label: "clock", type: "variable", detail: "temporizadores: schedule, schedule_interval" },
  { label: "animate", type: "function", detail: "animate(obj, pos=..., duration=1.0)" },
  { label: "Rect", type: "class", detail: "Rect((x, y), (ancho, alto))" },
  { label: "exit", type: "function", detail: "termina el juego de forma prolija" },
  { label: "WIDTH", type: "variable", detail: "ancho de la ventana (config de main.py)" },
  { label: "HEIGHT", type: "variable", detail: "alto de la ventana (config de main.py)" },
  { label: "TITLE", type: "variable", detail: "título de la ventana (config de main.py)" },
] satisfies Completion[];

// Miembros de Actor, screen/screen.draw, music y clock — se ofrecen siempre
// (el autocompletado "clasico" de v1 no resuelve el tipo de la expresión
// antes del punto, ver PLAN.md §6.5), sin el "." inicial para que CM6 los
// matchee bien por prefijo al escribir después del punto.
const PGZERO_MEMBERS = [
  // Actor
  "pos", "angle", "opacity", "anchor",
  "topleft", "topright", "bottomleft", "bottomright",
  "midtop", "midbottom", "midleft", "midright", "center",
  "angle_to", "distance_to", "colliderect", "collidepoint",
  // screen / screen.draw
  "clear", "fill", "blit",
  "draw.text", "draw.textbox", "draw.line", "draw.circle",
  "draw.filled_circle", "draw.rect", "draw.filled_rect",
  "draw.polygon", "draw.filled_polygon", "bounds",
  // music
  "play", "play_once", "queue", "stop", "pause", "unpause",
  "fadeout", "set_volume", "get_volume", "is_playing",
  // clock
  "schedule", "schedule_unique", "schedule_interval", "unschedule", "each_tick",
].map((label): Completion => ({ label, type: "property", boost: -1 }));

const PGZERO_SNIPPETS = [
  snippetCompletion("def draw():\n\t${}", {
    label: "def draw",
    type: "function",
    detail: "callback: dibujar cada cuadro (sin parámetros)",
  }),
  snippetCompletion("def update(${dt}):\n\t${}", {
    label: "def update",
    type: "function",
    detail: "callback: actualizar cada cuadro",
  }),
  snippetCompletion("def on_key_down(${key}):\n\t${}", {
    label: "def on_key_down",
    type: "function",
    detail: "callback: al presionar una tecla",
  }),
  snippetCompletion("def on_key_up(${key}):\n\t${}", {
    label: "def on_key_up",
    type: "function",
    detail: "callback: al soltar una tecla",
  }),
  snippetCompletion("def on_mouse_down(${pos}, ${button}):\n\t${}", {
    label: "def on_mouse_down",
    type: "function",
    detail: "callback: al hacer clic",
  }),
  snippetCompletion("def on_mouse_up(${pos}, ${button}):\n\t${}", {
    label: "def on_mouse_up",
    type: "function",
    detail: "callback: al soltar el clic",
  }),
  snippetCompletion("def on_mouse_move(${pos}):\n\t${}", {
    label: "def on_mouse_move",
    type: "function",
    detail: "callback: al mover el ratón",
  }),
  snippetCompletion("def on_music_end():\n\t${}", {
    label: "def on_music_end",
    type: "function",
    detail: "callback: cuando termina la música de fondo",
  }),
  snippetCompletion('${nombre} = Actor("${imagen}", (${x}, ${y}))', {
    label: "Actor(...)",
    type: "class",
    detail: "crear un sprite",
  }),
  snippetCompletion("animate(${objeto}, ${pos}=(${x}, ${y}), duration=${1.0})", {
    label: "animate(...)",
    type: "function",
    detail: "animar un atributo con el tiempo",
  }),
  snippetCompletion("clock.schedule_interval(${funcion}, ${1.0})", {
    label: "clock.schedule_interval(...)",
    type: "function",
    detail: "llamar una función cada N segundos",
  }),
];

export const STATIC_OPTIONS: Completion[] = [
  ...KEYWORDS.map((label): Completion => ({ label, type: "keyword", boost: -1 })),
  ...BUILTINS.map((label): Completion => ({ label, type: "function" })),
  ...SNIPPETS,
  ...PGZERO_GLOBALS,
  ...PGZERO_MEMBERS,
  ...PGZERO_SNIPPETS,
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
