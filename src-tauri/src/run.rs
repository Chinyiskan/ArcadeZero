//! `RunController` (ver PLAN.md §3.1/§3.2): lanza `launcher.py` sobre el
//! Python embebido como subproceso async y transmite su stdout/stderr al
//! webview como eventos. Nunca bloquea el hilo del IDE.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::error_parse::parse_stderr_line;

/// Estado compartido en Tauri: el proceso del juego en ejecucion, si hay uno.
/// Un solo run a la vez (igual que Mu: un `main.py`, un juego corriendo).
#[derive(Default, Clone)]
pub struct RunState(pub Arc<Mutex<Option<Child>>>);

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStatus {
    pub python_found: bool,
    pub python_path: String,
    pub pgzero_ok: bool,
    pub detail: String,
}

/// Resuelve la carpeta del runtime embebido armado por `build_runtime.py`.
///
/// En dev, `runtime/dist/windows-x64` vive junto al codigo fuente (relativo
/// a `CARGO_MANIFEST_DIR`, resuelto en compilacion). En un build instalado
/// esa ruta no existe en la maquina del usuario: el runtime se empaqueta
/// como resource de Tauri (ver `tauri.conf.json` `bundle.resources`) y hay
/// que preguntarle al `AppHandle` donde quedo en tiempo de ejecucion.
///
/// ponytail: hardcodea windows-x64 (unico target de Fase 0-1); agregar
/// deteccion de plataforma cuando Fase 5 arme runtimes de mac/linux.
/// Override para tests/dev: env var `ARCADEZERO_RUNTIME_DIR`.
fn runtime_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("ARCADEZERO_RUNTIME_DIR") {
        return Ok(PathBuf::from(dir));
    }
    if cfg!(debug_assertions) {
        return Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("runtime")
            .join("dist")
            .join("windows-x64"));
    }
    app.path()
        .resource_dir()
        .map(|dir| dir.join("runtime"))
        .map_err(|e| format!("No se pudo resolver la carpeta de recursos de la app: {e}"))
}

pub(crate) fn python_exe(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(runtime_dir(app)?.join("python").join("python.exe"))
}

fn launcher_py(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(runtime_dir(app)?.join("launcher.py"))
}

pub(crate) fn vendored_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(runtime_dir(app)?.join("vendored"))
}

/// Valida que `path` sea una carpeta de sketch abrible (tiene `main.py`).
/// Separado de `run_project` para poder testearlo sin un `AppHandle`.
fn validate_sketch_dir(path: &Path) -> Result<PathBuf, String> {
    if !path.join("main.py").is_file() {
        return Err(format!("No se encontro main.py en {}", path.display()));
    }
    Ok(path.to_path_buf())
}

async fn stream_lines<R>(app: AppHandle, reader: R, event: &'static str)
where
    R: AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                // Lineas ##ARCADEZERO##{...} del excepthook de launcher.py
                // (PLAN.md §3.2/§7): se parsean y salen como `run_error`
                // estructurado en vez de texto crudo, sea stdout o stderr.
                match parse_stderr_line(&line) {
                    Some(structured) => {
                        let _ = app.emit("run_error", structured);
                    }
                    None => {
                        let _ = app.emit(event, line);
                    }
                }
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }
}

#[tauri::command]
pub async fn run_project(
    app: AppHandle,
    state: State<'_, RunState>,
    path: String,
) -> Result<(), String> {
    let sketch_dir = validate_sketch_dir(Path::new(&path))?;

    let python = python_exe(&app)?;
    if !python.is_file() {
        return Err(format!(
            "No se encontro el Python embebido en {}",
            python.display()
        ));
    }

    // Solo un juego corriendo a la vez: si habia uno, lo matamos primero.
    {
        let mut guard = state.0.lock().await;
        if let Some(mut old) = guard.take() {
            let _ = old.start_kill();
        }
    }

    let mut child = Command::new(&python)
        .arg(launcher_py(&app)?)
        .arg(&sketch_dir)
        .current_dir(&sketch_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("No se pudo lanzar el juego: {e}"))?;

    let stdout = child.stdout.take().expect("stdout fue pedido con piped()");
    let stderr = child.stderr.take().expect("stderr fue pedido con piped()");

    *state.0.lock().await = Some(child);
    let _ = app.emit("run_start", ());

    tokio::spawn(stream_lines(app.clone(), stdout, "run_stdout"));
    tokio::spawn(stream_lines(app.clone(), stderr, "run_stderr"));

    let state_for_wait = state.0.clone();
    let app_for_exit = app.clone();
    tokio::spawn(async move {
        let code = wait_for_exit(state_for_wait).await;
        let _ = app_for_exit.emit("run_exit", code);
    });

    Ok(())
}

/// Espera a que el proceso guardado en `state` termine, sin bloquear a
/// `stop_run` (que solo necesita el lock un instante para pedir el kill).
///
/// ponytail: poll cada 50ms en vez de un wait() que retenga el lock;
/// simple y suficiente para un solo proceso hijo. Si algun dia hace falta
/// notificacion instantanea de salida, cambiar a un canal oneshot por run.
async fn wait_for_exit(state: Arc<Mutex<Option<Child>>>) -> Option<i32> {
    loop {
        let mut guard = state.lock().await;
        match guard.as_mut() {
            None => return None,
            Some(child) => match child.try_wait() {
                Ok(Some(status)) => {
                    *guard = None;
                    return status.code();
                }
                Ok(None) => {}
                Err(_) => {
                    *guard = None;
                    return None;
                }
            },
        }
        drop(guard);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tauri::command]
pub async fn stop_run(state: State<'_, RunState>) -> Result<(), String> {
    let mut guard = state.0.lock().await;
    if let Some(child) = guard.as_mut() {
        child
            .start_kill()
            .map_err(|e| format!("No se pudo detener el juego: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn runtime_status(app: AppHandle) -> RuntimeStatus {
    let python = match python_exe(&app) {
        Ok(p) => p,
        Err(e) => {
            return RuntimeStatus {
                python_found: false,
                python_path: String::new(),
                pgzero_ok: false,
                detail: e,
            };
        }
    };
    if !python.is_file() {
        return RuntimeStatus {
            python_found: false,
            python_path: python.display().to_string(),
            pgzero_ok: false,
            detail: "Python embebido no encontrado. Corre runtime/build_runtime.py.".into(),
        };
    }

    let vendored = match vendored_dir(&app) {
        Ok(v) => v,
        Err(e) => {
            return RuntimeStatus {
                python_found: true,
                python_path: python.display().to_string(),
                pgzero_ok: false,
                detail: e,
            };
        }
    };
    let check = format!(
        "import sys; sys.path.insert(0, r'{}'); import pgzero; import pygame",
        vendored.display()
    );
    match Command::new(&python).arg("-c").arg(check).output().await {
        Ok(out) if out.status.success() => RuntimeStatus {
            python_found: true,
            python_path: python.display().to_string(),
            pgzero_ok: true,
            detail: "ok".into(),
        },
        Ok(out) => RuntimeStatus {
            python_found: true,
            python_path: python.display().to_string(),
            pgzero_ok: false,
            detail: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => RuntimeStatus {
            python_found: true,
            python_path: python.display().to_string(),
            pgzero_ok: false,
            detail: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_sketch(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("arcadezero_run_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn validate_sketch_dir_errors_without_main_py() {
        let dir = temp_sketch("no_main");
        let result = validate_sketch_dir(&dir);
        assert!(result.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn validate_sketch_dir_ok_with_main_py() {
        let dir = temp_sketch("with_main");
        fs::write(dir.join("main.py"), "def draw():\n    pass\n").unwrap();
        let result = validate_sketch_dir(&dir).unwrap();
        assert_eq!(result, dir);
        fs::remove_dir_all(&dir).ok();
    }

    /// Camino feliz de verdad: spawnea un proceso real (cmd /C, disponible
    /// en cualquier Windows sin depender del runtime Python), lo guarda en
    /// el mismo tipo de estado que usa run_project, y confirma que
    /// start_kill() lo termina limpio y wait_for_exit() lo detecta.
    #[tokio::test]
    async fn spawn_and_kill_process_via_run_state() {
        let mut child = Command::new("cmd")
            .args(["/C", "timeout /T 30"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("spawn cmd de prueba");

        assert!(child.id().is_some());

        let state = Arc::new(Mutex::new(None));
        {
            let mut guard = state.lock().await;
            child.start_kill().expect("pedir kill");
            *guard = Some(child);
        }

        let code = wait_for_exit(state.clone()).await;
        // Un proceso matado en Windows sale con codigo != 0, no None; lo
        // importante es que wait_for_exit no se cuelga y limpia el estado.
        assert!(state.lock().await.is_none());
        let _ = code;
    }
}
