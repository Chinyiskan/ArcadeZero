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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(run::RunState::default())
        .invoke_handler(tauri::generate_handler![
            project::open_project,
            project::new_project,
            project::save_file,
            project::read_file,
            run::run_project,
            run::stop_run,
            run::runtime_status,
            get_settings,
            set_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
