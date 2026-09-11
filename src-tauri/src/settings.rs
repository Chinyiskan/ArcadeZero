//! Settings minimos en TOML, en el config dir de la app (ver PLAN.md §2).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FILE_NAME: &str = "settings.toml";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub sketches_dir: Option<String>,
    /// `None` = nunca elegido por el usuario: el frontend debe respetar
    /// `prefers-color-scheme` del SO (PLAN.md §6.4) en vez de un default fijo.
    pub theme: Option<String>,
    /// Accesibilidad (PLAN.md §6.4/§11 Fase 4): usar OpenDyslexic en vez de
    /// JetBrains Mono en el editor. `None`/`false` = fuente por defecto.
    pub dyslexic_font: Option<bool>,
    /// Escalado global de UI: "normal" | "grande" | "muy-grande". `None` =
    /// normal (ver `src/lib/theme.ts`).
    pub ui_scale: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sketches_dir: None,
            theme: None,
            dyslexic_font: None,
            ui_scale: None,
        }
    }
}

/// Carga settings desde `config_dir/settings.toml`. Si no existe o esta
/// corrupto, devuelve los valores por defecto (nunca falla el arranque).
pub fn load(config_dir: &Path) -> Settings {
    fs::read_to_string(config_dir.join(FILE_NAME))
        .ok()
        .and_then(|raw| toml::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save(config_dir: &Path, settings: &Settings) -> Result<(), String> {
    fs::create_dir_all(config_dir).map_err(|e| e.to_string())?;
    let raw = toml::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(config_dir.join(FILE_NAME), raw).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("arcadezero_settings_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = temp_dir("missing");
        let settings = load(&dir);
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let dir = temp_dir("roundtrip");
        let settings = Settings {
            sketches_dir: Some("D:/mis-juegos".into()),
            theme: Some("dracula".into()),
            dyslexic_font: Some(true),
            ui_scale: Some("grande".into()),
        };
        save(&dir, &settings).expect("guardar settings");

        let loaded = load(&dir);
        assert_eq!(loaded, settings);

        fs::remove_dir_all(&dir).ok();
    }
}
