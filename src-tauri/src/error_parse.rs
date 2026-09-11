//! Parseo de las lineas `##ARCADEZERO##{...}` que emite el excepthook de
//! `runtime/launcher.py` (ver PLAN.md §3.2/§7) + catalogo de errores
//! amigables (dominio friendly-errors). El catalogo vive aca, en Rust, para
//! no tener que sincronizar dos lenguajes (Python solo manda el error crudo).
//!
//! Regla de oro: nunca ocultar el error real. `friendly_es`/`friendly_en`
//! van primero en la UI, pero `message`/`traceback` siempre viajan intactos.

use serde::{Deserialize, Serialize};

/// Prefijo que usa `launcher.py` para marcar una linea de stderr como JSON
/// estructurado en vez de salida normal del juego.
pub const ERROR_PREFIX: &str = "##ARCADEZERO##";

/// Lo que manda `launcher.py` tal cual (ver `_serialize_exception` en
/// `runtime/launcher.py`). `friendly_es`/`friendly_en` de Python son un
/// relleno generico; los recalculamos aca con el catalogo, asi que no hace
/// falta deserializarlos.
#[derive(Debug, Clone, Deserialize)]
struct RawLauncherError {
    #[serde(rename = "type")]
    kind: String,
    message: String,
    file: Option<String>,
    lineno: Option<u32>,
    #[serde(default)]
    traceback: String,
}

/// Error ya parseado y enriquecido, listo para emitirse como evento
/// `run_error` (PLAN.md §3.3) al webview.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructuredError {
    #[serde(rename = "type")]
    pub kind: String,
    pub message: String,
    pub file: Option<String>,
    pub lineno: Option<u32>,
    pub friendly_es: String,
    pub friendly_en: String,
    pub traceback: String,
}

/// Intenta parsear una linea de stderr del subproceso del juego. `None` si
/// la linea no trae el prefijo o el JSON es invalido (en ese caso la linea
/// se trata como stderr normal, ver `run.rs`).
pub fn parse_stderr_line(line: &str) -> Option<StructuredError> {
    let json_part = line.strip_prefix(ERROR_PREFIX)?;
    let raw: RawLauncherError = serde_json::from_str(json_part).ok()?;
    let (friendly_es, friendly_en) = catalog::friendly_text(&raw.kind, &raw.message);
    Some(StructuredError {
        kind: raw.kind,
        message: raw.message,
        file: raw.file,
        lineno: raw.lineno,
        friendly_es,
        friendly_en,
        traceback: raw.traceback,
    })
}

/// Catalogo de errores comunes de principiante (PLAN.md §7). Matching por
/// tipo de excepcion + patrones simples sobre el mensaje (sin `regex`: los
/// mensajes de CPython son suficientemente fijos como para bastar con
/// `contains`/`split`, ver "ladder de simplicidad").
mod catalog {
    /// Nombres de la API de pgzero + builtins comunes (ver
    /// `docs/pgzero-cheatsheet.md`), usados para el fuzzy-match de
    /// `NameError`. No conocemos las variables reales del sketch del alumno
    /// (el JSON de launcher.py no las manda) asi que sugerimos contra esta
    /// lista fija; sigue cubriendo el typo mas comun (escribir mal un
    /// nombre de la API en vez de una variable propia).
    const KNOWN_NAMES: &[&str] = &[
        "draw", "update", "on_key_down", "on_key_up", "on_mouse_down", "on_mouse_up",
        "on_mouse_move", "on_music_end", "screen", "keyboard", "keys", "keymods", "mouse",
        "images", "sounds", "music", "clock", "animate", "Actor", "Rect", "ZRect", "WIDTH",
        "HEIGHT", "TITLE", "ICON", "exit", "print", "range", "len", "input", "str", "int",
        "float", "list", "dict", "tuple", "set", "True", "False", "None",
    ];

    /// Distancia de Levenshtein clasica (sin dependencia extra: son ~15
    /// lineas y solo la usamos para strings cortos como nombres).
    fn levenshtein(a: &str, b: &str) -> usize {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        let mut curr = vec![0usize; b.len() + 1];
        for i in 1..=a.len() {
            curr[0] = i;
            for j in 1..=b.len() {
                let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
                curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            }
            std::mem::swap(&mut prev, &mut curr);
        }
        prev[b.len()]
    }

    /// Busca el nombre conocido mas parecido a `typo`, si esta lo bastante
    /// cerca para ser un candidato razonable (umbral generoso pero acotado
    /// para nombres cortos como `key`/`up`).
    fn closest_known_name(typo: &str) -> Option<&'static str> {
        let threshold = if typo.len() <= 4 { 1 } else { 2 };
        KNOWN_NAMES
            .iter()
            .map(|&name| (name, levenshtein(typo, name)))
            .filter(|&(_, dist)| dist <= threshold && dist > 0)
            .min_by_key(|&(_, dist)| dist)
            .map(|(name, _)| name)
    }

    /// Extrae el primer nombre entre comillas simples de un mensaje de
    /// excepcion, ej. `name 'alein' is not defined` -> `alein`.
    fn quoted_name(message: &str) -> Option<&str> {
        let start = message.find('\'')? + 1;
        let rest = &message[start..];
        let end = rest.find('\'')?;
        Some(&rest[..end])
    }

    /// Punto de entrada del catalogo: `(tipo, mensaje)` -> `(es, en)`.
    /// Fallback generico decente si el tipo/mensaje no matchea nada — nunca
    /// "undefined", siempre algo constructivo.
    pub fn friendly_text(kind: &str, message: &str) -> (String, String) {
        match kind {
            "NameError" => name_error(message),
            "IndentationError" => indentation_error(message),
            "TabError" => (
                "Esta línea mezcla tabs y espacios para la indentación. Usa siempre espacios \
                 (o siempre tabs, pero no los dos) — 4 espacios por nivel es lo más común en Python."
                    .to_string(),
                "This line mixes tabs and spaces for indentation. Use only spaces (or only \
                 tabs, never both) — 4 spaces per level is the Python convention."
                    .to_string(),
            ),
            "SyntaxError" => syntax_error(message),
            "TypeError" => type_error(message),
            "AttributeError" => attribute_error(message),
            "ModuleNotFoundError" | "ImportError" => module_not_found(message),
            "ZeroDivisionError" => (
                "Tu código intentó dividir un número entre 0, y eso no se puede. Revisa qué \
                 valor tiene el divisor antes de dividir."
                    .to_string(),
                "Your code tried to divide a number by 0, which isn't allowed. Check the value \
                 of the divisor before dividing."
                    .to_string(),
            ),
            "IndexError" => (
                "Intentaste acceder a una posición de una lista que no existe (por ejemplo, \
                 pedir el elemento 5 de una lista de 3). Revisa el número entre corchetes."
                    .to_string(),
                "You tried to access a position in a list that doesn't exist (e.g. asking for \
                 item 5 of a 3-item list). Check the number inside the brackets."
                    .to_string(),
            ),
            "KeyError" => (
                format!(
                    "Buscaste una clave en un diccionario que no existe ({message}). Revisa \
                     cómo la escribiste o si la guardaste antes de usarla."
                ),
                format!(
                    "You looked up a dictionary key that doesn't exist ({message}). Check how \
                     it's spelled, or whether you stored it before using it."
                ),
            ),
            _ if kind == "error" || kind.ends_with(".error") || message.starts_with("Couldn't open") => {
                asset_error(message)
            }
            _ => (
                format!(
                    "Ocurrió un error de tipo {kind}: {message}. Abre el detalle técnico de \
                     abajo para ver exactamente dónde pasó."
                ),
                format!(
                    "An error of type {kind} happened: {message}. Open the technical detail \
                     below to see exactly where it occurred."
                ),
            ),
        }
    }

    fn name_error(message: &str) -> (String, String) {
        let Some(typo) = quoted_name(message) else {
            return (
                format!("Usaste un nombre que no existe: {message}."),
                format!("You used a name that doesn't exist: {message}."),
            );
        };
        match closest_known_name(typo) {
            Some(suggestion) => (
                format!(
                    "Escribiste `{typo}` pero no existe. ¿Quisiste decir `{suggestion}`?"
                ),
                format!(
                    "You wrote `{typo}` but it doesn't exist. Did you mean `{suggestion}`?"
                ),
            ),
            None => (
                format!(
                    "Usaste `{typo}` pero no está definido en ningún lado de tu código. \
                     Revisa que lo hayas escrito antes de usarlo (o que no tenga un error de \
                     tipeo)."
                ),
                format!(
                    "You used `{typo}` but it isn't defined anywhere in your code. Check that \
                     you wrote it earlier (or that it isn't a typo)."
                ),
            ),
        }
    }

    fn indentation_error(message: &str) -> (String, String) {
        if message.contains("unexpected indent") {
            (
                "Esta línea tiene más espacios al principio de los que debería. Alinéala con \
                 la línea de arriba."
                    .to_string(),
                "This line has more leading spaces than it should. Align it with the line above."
                    .to_string(),
            )
        } else if message.contains("expected an indented block") {
            (
                "Después de una línea que termina en `:` necesitas indentar la siguiente \
                 (agregar espacios al principio). Revisa la línea de abajo."
                    .to_string(),
                "After a line ending in `:` you need to indent the next line (add leading \
                 spaces). Check the line below it."
                    .to_string(),
            )
        } else {
            (
                "Esta línea necesita estar alineada con las de arriba. Revisa los espacios al \
                 principio de la línea."
                    .to_string(),
                "This line needs to line up with the ones above it. Check the leading spaces."
                    .to_string(),
            )
        }
    }

    fn syntax_error(message: &str) -> (String, String) {
        if message.contains("expected ':'") {
            (
                "Parece que falta un `:` al final de la línea (por ejemplo, después de `if`, \
                 `for`, `while` o `def`)."
                    .to_string(),
                "Looks like a `:` is missing at the end of the line (e.g. after `if`, `for`, \
                 `while` or `def`)."
                    .to_string(),
            )
        } else if message.contains("unterminated string literal") {
            (
                "Te falta cerrar unas comillas en un texto (string). Revisa que cada `\"` o \
                 `'` que abras tenga su par."
                    .to_string(),
                "You're missing a closing quote on a string. Check that every `\"` or `'` you \
                 open has a matching one."
                    .to_string(),
            )
        } else if message.contains("was never closed") {
            (
                "Abriste un paréntesis, corchete o llave que nunca se cerró. Revisa que cada \
                 `(`, `[` o `{` tenga su cierre."
                    .to_string(),
                "You opened a parenthesis, bracket or brace that was never closed. Check that \
                 every `(`, `[` or `{` has a matching close."
                    .to_string(),
            )
        } else {
            (
                format!(
                    "Hay un error de sintaxis en tu código: {message}. Revisa la línea marcada \
                     con cuidado, algo no está escrito como Python espera."
                ),
                format!(
                    "There's a syntax error in your code: {message}. Check the marked line \
                     carefully — something isn't written the way Python expects."
                ),
            )
        }
    }

    fn type_error(message: &str) -> (String, String) {
        let known_callbacks = ["draw", "update", "on_key_down", "on_key_up", "on_mouse_down"];
        if message.contains("positional argument") {
            if let Some(paren) = message.find("()") {
                let fn_name = &message[..paren];
                if known_callbacks.contains(&fn_name) {
                    return (
                        format!(
                            "La función `{fn_name}()` no debe recibir esos parámetros. Déjala \
                             como pgzero espera (revisa `docs/pgzero-cheatsheet.md`), por \
                             ejemplo `def {fn_name}():`."
                        ),
                        format!(
                            "The `{fn_name}()` function shouldn't take those parameters. Match \
                             the signature pgzero expects (see `docs/pgzero-cheatsheet.md`), \
                             e.g. `def {fn_name}():`."
                        ),
                    );
                }
                return (
                    format!(
                        "Llamaste a `{fn_name}()` con una cantidad de datos (argumentos) \
                         distinta a la que espera. Cuenta los paréntesis y compáralos con \
                         cómo la definiste."
                    ),
                    format!(
                        "You called `{fn_name}()` with a different number of arguments than it \
                         expects. Count the parentheses and compare with how it was defined."
                    ),
                );
            }
        }
        (
            format!(
                "Estás mezclando tipos de datos que no combinan: {message}. Revisa si estás \
                 usando un número donde va texto (o al revés)."
            ),
            format!(
                "You're mixing data types that don't go together: {message}. Check if you're \
                 using a number where text is expected (or the other way around)."
            ),
        )
    }

    fn attribute_error(message: &str) -> (String, String) {
        if message.starts_with("'NoneType' object has no attribute") {
            (
                "Estás usando una variable que vale `None` como si tuviera datos adentro. Es \
                 típico cuando una función no devuelve nada con `return` y usas su resultado."
                    .to_string(),
                "You're using a variable that is `None` as if it had data inside. This usually \
                 happens when a function doesn't `return` anything and you use its result."
                    .to_string(),
            )
        } else {
            (
                format!(
                    "Intentaste usar algo que ese objeto no tiene: {message}. Revisa el nombre \
                     (mayúsculas/minúsculas incluidas) contra `docs/pgzero-cheatsheet.md`."
                ),
                format!(
                    "You tried to use something that object doesn't have: {message}. Check the \
                     spelling (including upper/lowercase) against `docs/pgzero-cheatsheet.md`."
                ),
            )
        }
    }

    fn module_not_found(message: &str) -> (String, String) {
        (
            format!(
                "Tu código intenta importar algo que no está disponible: {message}. ArcadeZero \
                 trae Python + pgzero + pygame-ce; librerías extra no vienen incluidas."
            ),
            format!(
                "Your code tries to import something that isn't available: {message}. \
                 ArcadeZero ships Python + pgzero + pygame-ce; extra libraries aren't included."
            ),
        )
    }

    fn asset_error(message: &str) -> (String, String) {
        if let Some(rest) = message.strip_prefix("Couldn't open ") {
            let path = rest.trim();
            return (
                format!(
                    "No encuentro el archivo `{path}`. Revisa que esté en la carpeta correcta \
                     (`images/`, `sounds/` o `music/`) y en minúsculas, sin espacios ni acentos."
                ),
                format!(
                    "I can't find the file `{path}`. Check that it's in the right folder \
                     (`images/`, `sounds/` or `music/`) and lowercase, with no spaces or accents."
                ),
            );
        }
        (
            format!(
                "pygame no pudo abrir un archivo de tu juego: {message}. Revisa el panel de \
                 assets."
            ),
            format!(
                "pygame couldn't open one of your game's files: {message}. Check the assets \
                 panel."
            ),
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn levenshtein_basic() {
            assert_eq!(levenshtein("alein", "alien"), 2);
            assert_eq!(levenshtein("draw", "draw"), 0);
        }

        #[test]
        fn closest_known_name_finds_typo() {
            assert_eq!(closest_known_name("scren"), Some("screen"));
            assert_eq!(closest_known_name("Acter"), Some("Actor"));
        }

        #[test]
        fn closest_known_name_gives_up_when_too_far() {
            assert_eq!(closest_known_name("xyzxyzxyz"), None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(json: &str) -> String {
        format!("{ERROR_PREFIX}{json}")
    }

    #[test]
    fn ignores_lines_without_prefix() {
        assert_eq!(parse_stderr_line("hola desde print()"), None);
    }

    #[test]
    fn ignores_prefixed_lines_with_invalid_json() {
        assert_eq!(parse_stderr_line(&line("{not json")), None);
    }

    #[test]
    fn parses_name_error_with_fuzzy_suggestion() {
        // "Acter" (typo comun al escribir "Actor" de pgzero) -> match contra
        // el catalogo de nombres conocidos (ver KNOWN_NAMES en catalog).
        let raw = r#"{"type": "NameError", "message": "name 'Acter' is not defined", "file": "C:\\sketch\\main.py", "lineno": 5, "friendly_es": "x", "friendly_en": "y", "traceback": "Traceback...\n"}"#;
        let parsed = parse_stderr_line(&line(raw)).expect("deberia parsear");
        assert_eq!(parsed.kind, "NameError");
        assert_eq!(parsed.lineno, Some(5));
        assert_eq!(parsed.file.as_deref(), Some("C:\\sketch\\main.py"));
        assert!(parsed.friendly_es.contains("Acter"));
        assert!(parsed.friendly_es.contains("Actor"));
        assert!(parsed.friendly_en.contains("Did you mean"));
    }

    #[test]
    fn parses_name_error_without_close_match() {
        let raw = r#"{"type": "NameError", "message": "name 'puntaje_total_del_jugador' is not defined", "file": "main.py", "lineno": 10, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("no está definido"));
        assert!(!parsed.friendly_es.contains("Quisiste decir"));
    }

    #[test]
    fn parses_indentation_error() {
        let raw = r#"{"type": "IndentationError", "message": "unexpected indent", "file": "main.py", "lineno": 8, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("espacios"));
    }

    #[test]
    fn parses_tab_error() {
        let raw = r#"{"type": "TabError", "message": "inconsistent use of tabs and spaces in indentation", "file": "main.py", "lineno": 3, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("tabs"));
    }

    #[test]
    fn parses_syntax_error_missing_colon() {
        let raw = r#"{"type": "SyntaxError", "message": "expected ':'", "file": "main.py", "lineno": 8, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains(':'));
        assert!(parsed.friendly_es.contains("falta"));
    }

    #[test]
    fn parses_asset_not_found() {
        let raw = r#"{"type": "error", "message": "Couldn't open images/Nave.png", "file": "main.py", "lineno": 12, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("images/Nave.png"));
        assert!(parsed.friendly_es.contains("minúsculas"));
    }

    #[test]
    fn parses_type_error_on_draw_callback() {
        let raw = r#"{"type": "TypeError", "message": "draw() takes 0 positional arguments but 1 was given", "file": "main.py", "lineno": 20, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("draw()"));
        assert!(parsed.friendly_es.contains("def draw():"));
    }

    #[test]
    fn parses_attribute_error_none() {
        let raw = r#"{"type": "AttributeError", "message": "'NoneType' object has no attribute 'x'", "file": "main.py", "lineno": 30, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("None"));
    }

    #[test]
    fn parses_zero_division_error() {
        let raw = r#"{"type": "ZeroDivisionError", "message": "division by zero", "file": "main.py", "lineno": 40, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("dividir"));
    }

    #[test]
    fn parses_module_not_found_error() {
        let raw = r#"{"type": "ModuleNotFoundError", "message": "No module named 'numpy2'", "file": "main.py", "lineno": 1, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("importar"));
    }

    #[test]
    fn falls_back_to_generic_message_for_unknown_type() {
        let raw = r#"{"type": "RecursionError", "message": "maximum recursion depth exceeded", "file": "main.py", "lineno": 1, "friendly_es": "", "friendly_en": "", "traceback": "Traceback...\n"}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert!(parsed.friendly_es.contains("RecursionError"));
        assert!(parsed.friendly_en.contains("RecursionError"));
        // el traceback original nunca se pierde, aunque el tipo no este en el catalogo
        assert_eq!(parsed.traceback, "Traceback...\n");
    }

    #[test]
    fn null_lineno_and_file_are_preserved_as_none() {
        // SyntaxError a veces no deja file/lineno si el compile() falla muy temprano.
        let raw = r#"{"type": "SyntaxError", "message": "invalid syntax", "file": null, "lineno": null, "friendly_es": "", "friendly_en": "", "traceback": ""}"#;
        let parsed = parse_stderr_line(&line(raw)).unwrap();
        assert_eq!(parsed.file, None);
        assert_eq!(parsed.lineno, None);
    }
}
