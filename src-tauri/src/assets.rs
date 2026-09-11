//! `AssetService` (ver PLAN.md §5): listar/importar/borrar assets de un
//! sketch, con renombrado automático y watcher de las 3 subcarpetas.
//!
//! Importar SIEMPRE copia a `images/ sounds/ music/` con un nombre
//! sanitizado (minúsculas, sin acentos/ñ, espacios -> guiones). Borrar
//! SIEMPRE mueve a `<sketch>/.trash/<kind>/` en vez de borrado permanente.
//!
//! ponytail: la papelera es una carpeta `.trash/` local al sketch, no la
//! papelera del SO. Evita sumar el crate `trash` (y sus quirks por
//! plataforma) para algo que una carpeta ya resuelve; si algún día se pide
//! integración real con la papelera del SO, ese es el punto de upgrade.

use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use crate::project::SKETCH_SUBDIRS;

/// Watcher activo del sketch abierto, si hay uno. Reemplazar el `Option`
/// (en vez de acumular watchers) es lo que evita watchers huérfanos: al
/// abrir un proyecto nuevo, el viejo se dropea y notify deja de vigilar.
#[derive(Default)]
pub struct AssetWatcherState(pub Mutex<Option<RecommendedWatcher>>);

fn validate_kind(kind: &str) -> Result<&'static str, String> {
    SKETCH_SUBDIRS
        .iter()
        .find(|&&s| s == kind)
        .copied()
        .ok_or_else(|| format!("Tipo de asset desconocido: {kind}"))
}

fn allowed_extensions(kind: &str) -> Result<&'static [&'static str], String> {
    match kind {
        "images" => Ok(&["png", "gif", "jpg", "jpeg"]),
        "sounds" => Ok(&["wav", "ogg"]),
        "music" => Ok(&["ogg", "mp3"]),
        other => Err(format!("Tipo de asset desconocido: {other}")),
    }
}

/// Minúsculas + sin acentos/ñ/diéresis comunes en español. Cubre lo que un
/// chico va a escribir de verdad; no es una transliteración Unicode
/// completa (evita sumar una dependencia para eso).
fn strip_accents(lower: &str) -> String {
    lower
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

/// Sanea el "stem" (nombre sin extensión) de un archivo: minúsculas, sin
/// acentos, cualquier corrida de caracteres no alfanuméricos -> un guión.
fn sanitize_stem(raw: &str) -> String {
    let normalized = strip_accents(&raw.to_lowercase());
    let mut out = String::new();
    for c in normalized.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("archivo");
    }
    out
}

/// Camino de traversal: un filename que llega desde el frontend nunca debe
/// tener separadores de ruta ni `..`.
fn validate_plain_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
    {
        return Err(format!("Nombre de archivo inválido: {filename}"));
    }
    Ok(())
}

pub fn list_assets_in(sketch_dir: &Path, kind: &str) -> Result<Vec<String>, String> {
    let subdir = validate_kind(kind)?;
    let dir = sketch_dir.join(subdir);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| format!("No se pudo leer {}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    Ok(names)
}

pub fn import_asset_to(sketch_dir: &Path, src_path: &Path, kind: &str) -> Result<String, String> {
    let subdir = validate_kind(kind)?;
    let allowed = allowed_extensions(kind)?;

    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| format!("{} no tiene extensión", src_path.display()))?;
    if !allowed.contains(&ext.as_str()) {
        return Err(format!(
            "Extensión .{ext} no válida para {kind}. Usa: {}",
            allowed.join(", ")
        ));
    }
    if !src_path.is_file() {
        return Err(format!("No existe el archivo {}", src_path.display()));
    }

    let stem_raw = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("archivo");
    let stem = sanitize_stem(stem_raw);

    let dest_dir = sketch_dir.join(subdir);
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    let mut candidate = format!("{stem}.{ext}");
    let mut n = 2;
    while dest_dir.join(&candidate).exists() {
        candidate = format!("{stem}-{n}.{ext}");
        n += 1;
    }

    fs::copy(src_path, dest_dir.join(&candidate))
        .map_err(|e| format!("No se pudo copiar {}: {e}", src_path.display()))?;
    Ok(candidate)
}

pub fn delete_asset_from(sketch_dir: &Path, kind: &str, filename: &str) -> Result<(), String> {
    let subdir = validate_kind(kind)?;
    validate_plain_filename(filename)?;

    let src = sketch_dir.join(subdir).join(filename);
    if !src.is_file() {
        return Err(format!("No existe {filename} en {subdir}"));
    }

    let trash_dir = sketch_dir.join(".trash").join(subdir);
    fs::create_dir_all(&trash_dir).map_err(|e| e.to_string())?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let trashed_name = format!("{ts}-{filename}");

    fs::rename(&src, trash_dir.join(trashed_name))
        .map_err(|e| format!("No se pudo borrar {filename}: {e}"))
}

/// Empieza a vigilar `images/ sounds/ music/` del sketch dado, reemplazando
/// (y por lo tanto deteniendo) cualquier watcher anterior.
pub fn watch_sketch(
    app: AppHandle,
    state: &AssetWatcherState,
    sketch_dir: &Path,
) -> Result<(), String> {
    let app_for_events = app.clone();
    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: notify::Result<Event>| {
            if res.is_ok() {
                let _ = app_for_events.emit("assets_changed", ());
            }
        })
        .map_err(|e| e.to_string())?;

    for sub in SKETCH_SUBDIRS {
        let dir = sketch_dir.join(sub);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        watcher
            .watch(&dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("No se pudo vigilar {}: {e}", dir.display()))?;
    }

    let mut guard = state
        .0
        .lock()
        .map_err(|_| "lock del watcher envenenado".to_string())?;
    *guard = Some(watcher);
    Ok(())
}

// --- comandos Tauri (ver PLAN.md §3.3) ---
//
// ponytail: el contrato original del plan tenía `list_assets(kind)` /
// `import_asset(src, kind)` sin ruta de proyecto, asumiendo estado global
// de "proyecto activo" en Rust. No existe tal estado (save_file/read_file
// ya son así: reciben la ruta explícita desde el frontend, que es quien
// sabe qué sketch está abierto). Se agrega `project_path` explícito a los
// 3 comandos para ser consistente con ese patrón; PLAN.md §3.3 actualizado.

#[tauri::command]
pub fn list_assets(project_path: String, kind: String) -> Result<Vec<String>, String> {
    list_assets_in(Path::new(&project_path), &kind)
}

#[tauri::command]
pub fn import_asset(
    project_path: String,
    src_path: String,
    kind: String,
) -> Result<String, String> {
    import_asset_to(Path::new(&project_path), Path::new(&src_path), &kind)
}

#[tauri::command]
pub fn delete_asset(project_path: String, kind: String, filename: String) -> Result<(), String> {
    delete_asset_from(Path::new(&project_path), &kind, &filename)
}

/// Bytes crudos de un asset, para la previsualización de imágenes en el
/// frontend (PLAN.md §6.1). El frontend arma un `Blob`/`URL.createObjectURL`
/// con esto — evitamos abrir el protocolo `asset:`/su scope de filesystem
/// para carpetas de sketch arbitrarias (el usuario elige la carpeta en
/// runtime, no hay una raíz fija que declarar en `capabilities/`); este
/// comando ya valida `kind`/`filename` igual que `delete_asset`.
///
/// ponytail: sketches de pgzero son sprites pequeños (KB, no MB), así que
/// serializar como `Vec<u8>` (array JSON) es suficiente; si algún día se
/// abre a imágenes grandes, ahí vale la pena el protocolo `asset:` con scope
/// por-sketch registrado dinámicamente.
#[tauri::command]
pub fn read_asset_bytes(project_path: String, kind: String, filename: String) -> Result<Vec<u8>, String> {
    let subdir = validate_kind(&kind)?;
    validate_plain_filename(&filename)?;
    let path = Path::new(&project_path).join(subdir).join(&filename);
    fs::read(&path).map_err(|e| format!("No se pudo leer {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_sketch(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("arcadezero_assets_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for sub in SKETCH_SUBDIRS {
            fs::create_dir_all(dir.join(sub)).unwrap();
        }
        dir
    }

    fn write_src(dir: &Path, name: &str, contents: &[u8]) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn import_renames_accents_spaces_and_uppercase() {
        let sketch = temp_sketch("rename");
        let src_dir = std::env::temp_dir();
        let src = write_src(&src_dir, "Nave Espacial Ñandú.PNG", b"fake png");

        let result = import_asset_to(&sketch, &src, "images").expect("importar");
        assert_eq!(result, "nave-espacial-nandu.png");
        assert!(sketch.join("images").join(&result).is_file());

        fs::remove_file(&src).ok();
        fs::remove_dir_all(&sketch).ok();
    }

    #[test]
    fn import_rejects_invalid_extension() {
        let sketch = temp_sketch("badext");
        let src_dir = std::env::temp_dir();
        let src = write_src(&src_dir, "sonido.mp3", b"fake");

        let result = import_asset_to(&sketch, &src, "sounds");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no válida"));

        fs::remove_file(&src).ok();
        fs::remove_dir_all(&sketch).ok();
    }

    #[test]
    fn import_handles_name_collision_with_suffix() {
        let sketch = temp_sketch("collision");
        let src_dir = std::env::temp_dir();
        let src1 = write_src(&src_dir, "nave.png", b"one");
        let src2 = write_src(&src_dir, "NAVE.png", b"two");

        let first = import_asset_to(&sketch, &src1, "images").unwrap();
        let second = import_asset_to(&sketch, &src2, "images").unwrap();

        assert_eq!(first, "nave.png");
        assert_eq!(second, "nave-2.png");

        fs::remove_file(&src1).ok();
        fs::remove_file(&src2).ok();
        fs::remove_dir_all(&sketch).ok();
    }

    #[test]
    fn delete_moves_to_trash_not_permanent() {
        let sketch = temp_sketch("delete");
        fs::write(sketch.join("images").join("nave.png"), b"data").unwrap();

        delete_asset_from(&sketch, "images", "nave.png").expect("borrar");

        assert!(!sketch.join("images").join("nave.png").exists());
        let trash_dir = sketch.join(".trash").join("images");
        let trashed: Vec<_> = fs::read_dir(&trash_dir).unwrap().collect();
        assert_eq!(trashed.len(), 1);

        fs::remove_dir_all(&sketch).ok();
    }

    #[test]
    fn delete_rejects_path_traversal() {
        let sketch = temp_sketch("traversal");
        let result = delete_asset_from(&sketch, "images", "../../etc/passwd");
        assert!(result.is_err());
        fs::remove_dir_all(&sketch).ok();
    }

    #[test]
    fn read_asset_bytes_returns_file_contents() {
        let sketch = temp_sketch("read-bytes");
        fs::write(sketch.join("images").join("nave.png"), b"fake png bytes").unwrap();

        let bytes = read_asset_bytes(
            sketch.display().to_string(),
            "images".to_string(),
            "nave.png".to_string(),
        )
        .expect("leer bytes");
        assert_eq!(bytes, b"fake png bytes");

        fs::remove_dir_all(&sketch).ok();
    }

    #[test]
    fn read_asset_bytes_rejects_path_traversal() {
        let sketch = temp_sketch("read-bytes-traversal");
        let result = read_asset_bytes(
            sketch.display().to_string(),
            "images".to_string(),
            "../../etc/passwd".to_string(),
        );
        assert!(result.is_err());
        fs::remove_dir_all(&sketch).ok();
    }

    #[test]
    fn list_assets_returns_sorted_filenames() {
        let sketch = temp_sketch("list");
        fs::write(sketch.join("images").join("b.png"), b"1").unwrap();
        fs::write(sketch.join("images").join("a.png"), b"2").unwrap();

        let names = list_assets_in(&sketch, "images").unwrap();
        assert_eq!(names, vec!["a.png", "b.png"]);

        fs::remove_dir_all(&sketch).ok();
    }
}
