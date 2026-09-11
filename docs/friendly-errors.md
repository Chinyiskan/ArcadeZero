# Catálogo de errores amigables

> Fuente de verdad del texto: `src-tauri/src/error_parse.rs` (`mod catalog`,
> función `friendly_text(kind, message)`). Este documento es la referencia
> legible para QA y para agregar casos nuevos — si lo cambias aquí, cámbialo
> también en el código (y agrega/ajusta el test correspondiente).

## Cómo llega un error a la UI

```
excepción en el juego del alumno
  → sys.excepthook en runtime/launcher.py serializa a JSON
    ##ARCADEZERO##{"type": ..., "message": ..., "file": ..., "lineno": ..., "traceback": ...}
  → RunController (src-tauri/src/run.rs) lee la línea de stderr del subproceso
    → error_parse::parse_stderr_line() la detecta por el prefijo, parsea el JSON
      y le calcula friendly_es/friendly_en con el catálogo (Rust, no Python)
  → se emite el evento `run_error` al webview (en vez de `run_stderr` para esa línea)
  → la UI (pendiente, dominio editor-ux) muestra una tarjeta con el texto
    amigable primero y el traceback original disponible plegado
```

**Regla de oro:** el mensaje amigable nunca reemplaza el error real. `message`
y `traceback` siempre viajan completos en el evento — el catálogo solo
antepone una explicación en español/inglés pensada para un chico de 10–17
años. Si un tipo de excepción no está en el catálogo, cae a un mensaje
genérico decente (nunca "undefined", nunca se rompe el flujo).

## Forma del evento `run_error`

```ts
{
  type: string;          // nombre de la excepción de Python, ej. "NameError"
  message: string;       // str(exc_value), tal cual lo dio Python
  file: string | null;   // ruta del archivo del alumno donde ocurrió (puede ser null)
  lineno: number | null; // línea (puede ser null en algunos SyntaxError muy tempranos)
  friendly_es: string;   // texto amigable en español, calculado en Rust
  friendly_en: string;   // texto amigable en inglés
  traceback: string;     // traceback completo de Python, para el detalle plegable
}
```

## Catálogo

| Python dice (`type` / patrón en `message`) | ArcadeZero dice (ES) | ArcadeZero dice (EN) | Cuándo aplica |
|---|---|---|---|
| `NameError: name 'X' is not defined` (con sugerencia) | "Escribiste `X` pero no existe. ¿Quisiste decir `Y`?" | "You wrote `X` but it doesn't exist. Did you mean `Y`?" | `X` está a distancia de edición ≤2 (≤1 si tiene 4 letras o menos) de un nombre conocido de la API de pgzero o un builtin común (ver `KNOWN_NAMES` en el código). No conocemos las variables propias del alumno (el JSON de `launcher.py` no las manda), así que el fuzzy-match es contra esa lista fija — cubre el typo más común: escribir mal `screen`, `Actor`, `keyboard`, etc. |
| `NameError: name 'X' is not defined` (sin sugerencia) | "Usaste `X` pero no está definido en ningún lado de tu código. Revisa que lo hayas escrito antes de usarlo (o que no tenga un error de tipeo)." | "You used `X` but it isn't defined anywhere in your code. Check that you wrote it earlier (or that it isn't a typo)." | Ningún nombre conocido está lo bastante cerca. |
| `IndentationError: unexpected indent` | "Esta línea tiene más espacios al principio de los que debería. Alinéala con la línea de arriba." | "This line has more leading spaces than it should. Align it with the line above." | Indentación de más. |
| `IndentationError: expected an indented block` | "Después de una línea que termina en `:` necesitas indentar la siguiente (agregar espacios al principio). Revisa la línea de abajo." | "After a line ending in `:` you need to indent the next line (add leading spaces). Check the line below it." | Falta indentar tras `if`/`for`/`def`/etc. |
| `IndentationError` (otro mensaje) | "Esta línea necesita estar alineada con las de arriba. Revisa los espacios al principio de la línea." | "This line needs to line up with the ones above it. Check the leading spaces." | Fallback del tipo. |
| `TabError: inconsistent use of tabs...` | "Esta línea mezcla tabs y espacios para la indentación. Usa siempre espacios (o siempre tabs, pero no los dos) — 4 espacios por nivel es lo más común en Python." | "This line mixes tabs and spaces for indentation. Use only spaces (or only tabs, never both) — 4 spaces per level is the Python convention." | Indentación mixta. |
| `SyntaxError: expected ':'` | "Parece que falta un `:` al final de la línea (por ejemplo, después de `if`, `for`, `while` o `def`)." | "Looks like a `:` is missing at the end of the line (e.g. after `if`, `for`, `while` or `def`)." | CPython 3.10+ ya da este mensaje específico — no hace falta heurística propia. |
| `SyntaxError: unterminated string literal` | "Te falta cerrar unas comillas en un texto (string). Revisa que cada `"` o `'` que abras tenga su par." | "You're missing a closing quote on a string. Check that every `\"` or `'` you open has a matching one." | Comillas sin cerrar. |
| `SyntaxError: ... was never closed` | "Abriste un paréntesis, corchete o llave que nunca se cerró. Revisa que cada `(`, `[` o `{` tenga su cierre." | "You opened a parenthesis, bracket or brace that was never closed. Check that every `(`, `[` or `{` has a matching close." | Paréntesis/corchete/llave sin cerrar. |
| `SyntaxError` (otro mensaje) | "Hay un error de sintaxis en tu código: {message}. Revisa la línea marcada con cuidado, algo no está escrito como Python espera." | "There's a syntax error in your code: {message}. Check the marked line carefully — something isn't written the way Python expects." | Fallback del tipo. |
| `TypeError: draw() takes 0 positional arguments but 1 was given` (o `update`/`on_key_down`/`on_key_up`/`on_mouse_down`) | "La función `draw()` no debe recibir esos parámetros. Déjala como pgzero espera (revisa `docs/pgzero-cheatsheet.md`), por ejemplo `def draw():`." | "The `draw()` function shouldn't take those parameters. Match the signature pgzero expects (see `docs/pgzero-cheatsheet.md`), e.g. `def draw():`." | Firma incorrecta de un callback de pgzero — el caso #1 de principiante al copiar `def draw(screen):` de otro tutorial. |
| `TypeError: ...() takes ... positional arguments...` (función propia) | "Llamaste a `f()` con una cantidad de datos (argumentos) distinta a la que espera. Cuenta los paréntesis y compáralos con cómo la definiste." | "You called `f()` with a different number of arguments than it expects. Count the parentheses and compare with how it was defined." | Cualquier otra función con mismatch de argumentos. |
| `TypeError` (otro mensaje) | "Estás mezclando tipos de datos que no combinan: {message}. Revisa si estás usando un número donde va texto (o al revés)." | "You're mixing data types that don't go together: {message}. Check if you're using a number where text is expected (or the other way around)." | Fallback del tipo (ej. `int + str`). |
| `AttributeError: 'NoneType' object has no attribute 'X'` | "Estás usando una variable que vale `None` como si tuviera datos adentro. Es típico cuando una función no devuelve nada con `return` y usas su resultado." | "You're using a variable that is `None` as if it had data inside. This usually happens when a function doesn't `return` anything and you use its result." | Olvido de `return`, causa muy común de `NoneType`. |
| `AttributeError` (otro mensaje) | "Intentaste usar algo que ese objeto no tiene: {message}. Revisa el nombre (mayúsculas/minúsculas incluidas) contra `docs/pgzero-cheatsheet.md`." | "You tried to use something that object doesn't have: {message}. Check the spelling (including upper/lowercase) against `docs/pgzero-cheatsheet.md`." | Ej. `Actor.postion` en vez de `.pos`. |
| `ModuleNotFoundError` / `ImportError` | "Tu código intenta importar algo que no está disponible: {message}. ArcadeZero trae Python + pgzero + pygame-ce; librerías extra no vienen incluidas." | "Your code tries to import something that isn't available: {message}. ArcadeZero ships Python + pgzero + pygame-ce; extra libraries aren't included." | `import` de algo no vendorizado. |
| `ZeroDivisionError: division by zero` | "Tu código intentó dividir un número entre 0, y eso no se puede. Revisa qué valor tiene el divisor antes de dividir." | "Your code tried to divide a number by 0, which isn't allowed. Check the value of the divisor before dividing." | — |
| `IndexError: list index out of range` | "Intentaste acceder a una posición de una lista que no existe (por ejemplo, pedir el elemento 5 de una lista de 3). Revisa el número entre corchetes." | "You tried to access a position in a list that doesn't exist (e.g. asking for item 5 of a 3-item list). Check the number inside the brackets." | — |
| `KeyError: 'X'` | "Buscaste una clave en un diccionario que no existe ('X'). Revisa cómo la escribiste o si la guardaste antes de usarla." | "You looked up a dictionary key that doesn't exist ('X'). Check how it's spelled, or whether you stored it before using it." | — |
| `pygame.error: Couldn't open images/Nave.png` (`type` viene como `"error"`, nombre de clase `pygame.error`) | "No encuentro el archivo `images/Nave.png`. Revisa que esté en la carpeta correcta (`images/`, `sounds/` o `music/`) y en minúsculas, sin espacios ni acentos." | "I can't find the file `images/Nave.png`. Check that it's in the right folder (`images/`, `sounds/` or `music/`) and lowercase, with no spaces or accents." | Asset no encontrado — la causa #1 de frustración con pgzero, ver PLAN.md §5. |
| `pygame.error` (otro mensaje) | "pygame no pudo abrir un archivo de tu juego: {message}. Revisa el panel de assets." | "pygame couldn't open one of your game's files: {message}. Check the assets panel." | Fallback de errores de pygame. |
| Cualquier otro tipo no listado | "Ocurrió un error de tipo {type}: {message}. Abre el detalle técnico de abajo para ver exactamente dónde pasó." | "An error of type {type} happened: {message}. Open the technical detail below to see exactly where it occurred." | Fallback genérico — nunca se rompe, nunca "undefined". |

## Cómo agregar un caso nuevo

1. Identifica el `type` (nombre de la clase de excepción) y, si hace falta
   distinguir submensajes, un patrón simple sobre `message` (`contains`,
   `starts_with` — sin `regex`, ver "ladder de simplicidad" del proyecto).
2. Agrega la rama en `friendly_text()` (`src-tauri/src/error_parse.rs`), con
   texto ES y EN.
3. Agrega un test en el `mod tests` de `error_parse.rs` con una línea real
   `##ARCADEZERO##{...}` de fixture (tipo + mensaje reales de CPython/pygame)
   que confirme el texto esperado.
4. Agrega la fila a la tabla de arriba.
5. Corre `cargo test --lib` en `src-tauri`.

## Pendiente (fuera de esta pasada)

- **Botón "Revisar"** (PLAN.md §8): chequeo estático con `pyflakes`
  vendorizado, antes de ejecutar. No implementado todavía — requiere
  vendorizar `pyflakes` en `runtime/` (dominio `runtime-packager`) y un
  comando Tauri nuevo. El catálogo de errores de este documento es para
  excepciones en *tiempo de ejecución*; pyflakes cubriría avisos estáticos
  (variables sin usar, nombres no definidos) con un tono igual de no
  punitivo.
- **Consumo en la UI**: el evento `run_error` ya se emite (ver
  `src-tauri/src/run.rs`), pero la tarjeta de error + "Ir a la línea" en el
  editor es tarea de `editor-ux` (PLAN.md §7). Ver `TODO(editor-ux)` en
  `src/routes/+page.svelte`.
