//! `ProjectManager` (ver PLAN.md §5): abrir/crear un sketch.
//!
//! Un sketch es una carpeta con `main.py` + `images/ sounds/ music/`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

/// Crea la estructura de un sketch nuevo en `dest`.
///
/// ponytail: hoy solo existe la plantilla "en-blanco"; el resto del catalogo
/// (mi-primer-sprite, plataformas-basico) llega en Fase 2 con `templates/`.
pub fn create_sketch(_template: &str, dest: &Path) -> io::Result<PathBuf> {
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

#[tauri::command]
pub fn new_project(template: String, dest: String) -> Result<String, String> {
    create_sketch(&template, Path::new(&dest))
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_project(path: String) -> Result<String, String> {
    validate_sketch(Path::new(&path)).map(|p| p.display().to_string())
}

#[tauri::command]
pub fn save_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("No se pudo guardar {path}: {e}"))
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

    #[test]
    fn new_project_creates_expected_structure() {
        let dest = temp_dir("new_project");
        let created = create_sketch("en-blanco", &dest).expect("crear sketch");

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
    fn open_project_accepts_valid_sketch() {
        let dest = temp_dir("open_project_ok");
        create_sketch("en-blanco", &dest).unwrap();

        let result = validate_sketch(&dest).expect("sketch valido");
        assert_eq!(result, dest);

        fs::remove_dir_all(&dest).ok();
    }
}
