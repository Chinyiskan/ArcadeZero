mod assets;
mod check;
mod error_parse;
mod project;
mod run;
mod settings;

use tauri::Manager;

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<settings::Settings, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(settings::load(&dir))
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, patch: settings::Settings) -> Result<(), String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    settings::save(&dir, &patch)
}

/// Instala un panic hook que deja rastro en disco antes de que la app GUI
/// desaparezca sin avisar (ver auditoria: sin esto, un panic en un binario
/// sin consola adjunta es indiagnosticable a distancia — el icono
/// simplemente parpadea y se cierra).
fn install_crash_log_hook() {
    let log_path = std::env::temp_dir().join("arcadezero_crash.log");
    std::panic::set_hook(Box::new(move |info| {
        let msg = format!("{} - ArcadeZero crasheo: {info}\n", unix_timestamp());
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, msg.as_bytes()));
    }));
}

/// Timestamp minimo sin traer una dependencia de fecha solo para esto.
fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    install_crash_log_hook();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(run::RunState::default())
        .manage(assets::AssetWatcherState::default())
        .invoke_handler(tauri::generate_handler![
            project::open_project,
            project::new_project,
            project::save_file,
            project::read_file,
            run::run_project,
            run::stop_run,
            run::runtime_status,
            check::check_syntax,
            assets::list_assets,
            assets::import_asset,
            assets::delete_asset,
            assets::read_asset_bytes,
            get_settings,
            set_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
