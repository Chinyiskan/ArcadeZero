//! Comando `check_syntax` para el boton "Revisar" (PLAN.md §8): corre
//! pyflakes vendorizado contra `main.py` on-demand (no en cada keystroke).
//! Reusa `python_exe()`/`vendored_dir()` de `run.rs`: mismo interprete
//! embebido, mismo directorio vendorizado que ya trae pgzero.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::run::{python_exe, vendored_dir};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckIssue {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

fn check_script() -> PathBuf {
    vendored_dir().join("_check_pyflakes.py")
}

#[tauri::command]
pub async fn check_syntax(path: String) -> Result<Vec<CheckIssue>, String> {
    let python = python_exe();
    if !python.is_file() {
        return Err("No se encontro el Python embebido.".into());
    }

    let out = Command::new(&python)
        .arg(check_script())
        .arg(&path)
        .output()
        .await
        .map_err(|e| format!("No se pudo correr el chequeo: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("Respuesta inesperada del chequeo: {e}"))
}
