//! `ProjectManager` (ver PLAN.md §5): abrir/crear un sketch.
//!
//! Un sketch es una carpeta con `main.py` + `images/ sounds/ music/`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, State};

use crate::assets::{self, AssetWatcherState};

pub const SKETCH_SUBDIRS: [&str; 3] = ["images", "sounds", "music"];

const BLANK_TEMPLATE: &str = "\
# Nuevo juego con pgzero
WIDTH = 800
HEIGHT = 600


def draw():
    screen.clear()
    screen.draw.text(\"Hola, ArcadeZero!\", center=(WIDTH / 2, HEIGHT / 2))


def update():
    pass
";

/// Carpeta `templates/` del repo (armada por `pgzero-domain`, Fase 2).
///
/// Mismo problema que `runtime_dir()` en `run.rs`: en dev vive junto al
/// codigo fuente, pero en un build instalado esa ruta de compilacion no
/// existe en la maquina del usuario. Se empaqueta como resource de Tauri
/// (ver `tauri.conf.json` `bundle.resources`) y hay que preguntarle al
/// `AppHandle` donde quedo en tiempo de ejecucion.
///
/// Override para tests/dev: env var `ARCADEZERO_TEMPLATES_DIR` (mismo patrón
/// que `ARCADEZERO_RUNTIME_DIR` en `run.rs`).
fn templates_dir(app: &AppHandle) -> PathBuf {
    if let Ok(dir) = std::env::var("ARCADEZERO_TEMPLATES_DIR") {
        return PathBuf::from(dir);
    }
    if cfg!(debug_assertions) {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("templates");
    }
    app.path()
        .resource_dir()
        .map(|dir| dir.join("templates"))
        .unwrap_or_else(|_| PathBuf::from("templates"))
}

/// Copia un directorio recursivamente (no hay `fs::copy_dir` en std).
fn copy_dir_recursive(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Crea la estructura de un sketch nuevo en `dest` a partir de `template`
/// (nombre de una carpeta en `templates/`, p.ej. "en-blanco",
/// "mi-primer-sprite", "plataformas-basico").
///
/// Si la plantilla no existe en disco, cae al `BLANK_TEMPLATE` embebido
/// (nunca deja al usuario sin `main.py`).
pub fn create_sketch(template: &str, dest: &Path, templates_root: &Path) -> io::Result<PathBuf> {
    let template_src = templates_root.join(template);
    if template_src.is_dir() {
        copy_dir_recursive(&template_src, dest)?;
    }
    fs::create_dir_all(dest)?;
    for sub in SKETCH_SUBDIRS {
        fs::create_dir_all(dest.join(sub))?;
    }
    let main_py = dest.join("main.py");
    if !main_py.exists() {
        fs::write(&main_py, BLANK_TEMPLATE)?;
    }
    Ok(dest.to_path_buf())
}

/// Valida que `path` sea un sketch abrible (carpeta con `main.py`, o el
/// propio `main.py`). Devuelve la carpeta del sketch.
pub fn validate_sketch(path: &Path) -> Result<PathBuf, String> {
    let main_py = if path.is_dir() {
        path.join("main.py")
    } else {
        path.to_path_buf()
    };

    if main_py.file_name().and_then(|n| n.to_str()) != Some("main.py") {
        return Err(format!(
            "{} no es un sketch de ArcadeZero (se esperaba main.py)",
            path.display()
        ));
    }
    if !main_py.is_file() {
        return Err(format!("No se encontro main.py en {}", path.display()));
    }
    Ok(main_py.parent().unwrap_or(path).to_path_buf())
}

/// ponytail: abrir/crear un sketch es también "el sketch activo cambió",
/// así que aquí mismo se (re)arranca el watcher de assets — reemplaza al
/// anterior si había uno, sin necesidad de un comando `close_project`
/// separado (ver `assets::watch_sketch`).
#[tauri::command]
pub fn new_project(
    app: AppHandle,
    watcher: State<'_, AssetWatcherState>,
    template: String,
    dest: String,
) -> Result<String, String> {
    let templates_root = templates_dir(&app);
    let created = create_sketch(&template, Path::new(&dest), &templates_root)
        .map_err(|e| e.to_string())?;
    assets::watch_sketch(app, &watcher, &created)?;
    Ok(created.display().to_string())
}

#[tauri::command]
pub fn open_project(
    app: AppHandle,
    watcher: State<'_, AssetWatcherState>,
    path: String,
) -> Result<String, String> {
    let sketch_dir = validate_sketch(Path::new(&path))?;
    assets::watch_sketch(app, &watcher, &sketch_dir)?;
    Ok(sketch_dir.display().to_string())
}

#[tauri::command]
pub fn save_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("No se pudo guardar {path}: {e}"))
}

/// Lee el contenido de un archivo de texto (usado para cargar `main.py` en
/// el editor tras `open_project`/`new_project`).
///
/// ponytail: faltaba en el contrato de §3.3 pero es imprescindible para que
/// "Abrir" muestre algo — sin esto el editor no tiene como leer el
/// `main.py` en disco. Agregado minimo, simetrico a `save_file`.
#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("No se pudo leer {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "arcadezero_test_{name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn dev_templates_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("templates")
    }

    #[test]
    fn new_project_creates_expected_structure() {
        let dest = temp_dir("new_project");
        let created =
            create_sketch("en-blanco", &dest, &dev_templates_dir()).expect("crear sketch");

        assert_eq!(created, dest);
        assert!(dest.join("main.py").is_file());
        for sub in SKETCH_SUBDIRS {
            assert!(dest.join(sub).is_dir(), "falta subcarpeta {sub}");
        }

        fs::remove_dir_all(&dest).ok();
    }

    #[test]
    fn open_project_rejects_folder_without_main_py() {
        let dest = temp_dir("open_project_missing");
        fs::create_dir_all(&dest).unwrap();

        let result = validate_sketch(&dest);
        assert!(result.is_err());

        fs::remove_dir_all(&dest).ok();
    }

    #[test]
    fn read_file_returns_written_content() {
        let dest = temp_dir("read_file");
        create_sketch("en-blanco", &dest, &dev_templates_dir()).unwrap();
        let main_py = dest.join("main.py");

        let content = read_file(main_py.display().to_string()).expect("leer main.py");
        assert!(content.contains("Nuevo juego con pgzero"));

        fs::remove_dir_all(&dest).ok();
    }

    #[test]
    fn open_project_accepts_valid_sketch() {
        let dest = temp_dir("open_project_ok");
        create_sketch("en-blanco", &dest, &dev_templates_dir()).unwrap();

        let result = validate_sketch(&dest).expect("sketch valido");
        assert_eq!(result, dest);

        fs::remove_dir_all(&dest).ok();
    }
}
