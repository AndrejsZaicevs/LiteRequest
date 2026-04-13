pub mod commands;
pub mod db;
pub mod http;
pub mod models;
pub mod utils;

use db::Database;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
}

fn dirs_data_path() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LiteRequest");
    std::fs::create_dir_all(&dir).ok();
    dir.join("literequest.db")
}

pub fn run() {
    let db_path = dirs_data_path();
    let db = Database::open(&db_path).expect("Failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(db),
        })
        .invoke_handler(tauri::generate_handler![
            // Collections
            commands::list_collections,
            commands::insert_collection,
            commands::update_collection,
            commands::delete_collection,
            commands::rename_collection,
            // Folders
            commands::list_folders,
            commands::insert_folder,
            commands::delete_folder,
            commands::rename_folder,
            commands::move_folder,
            // Requests
            commands::list_requests_by_collection,
            commands::list_requests_by_folder,
            commands::list_orphan_requests,
            commands::insert_request,
            commands::rename_request,
            commands::delete_request,
            commands::move_request,
            commands::reorder_requests,
            commands::reorder_folders,
            commands::update_request_version,
            // Versions
            commands::insert_version,
            commands::get_version,
            commands::list_versions,
            commands::update_version_data,
            commands::delete_version,
            commands::version_has_executions,
            // Executions
            commands::insert_execution,
            commands::list_executions,
            // Environments
            commands::list_environments,
            commands::insert_environment,
            commands::set_active_environment,
            commands::rename_environment,
            commands::delete_environment,
            // Env Variables
            commands::list_env_variables,
            commands::insert_env_variable,
            commands::update_env_variable,
            commands::delete_env_variable,
            commands::get_active_variables,
            // Collection Variables
            commands::insert_var_def,
            commands::update_var_def_key,
            commands::delete_var_def,
            commands::list_var_defs,
            commands::upsert_var_value,
            commands::load_var_rows,
            commands::get_active_collection_variables,
            // Settings
            commands::get_app_setting,
            commands::set_app_setting,
            // HTTP
            commands::execute_request,
            // Clipboard
            commands::copy_to_clipboard,
            // cURL
            commands::to_curl,
            commands::parse_curl,
            // Interpolation
            commands::interpolate,
            commands::resolve_url,
            commands::extract_path_params,
            // Maintenance
            commands::prune_old_executions,
            // Search
            commands::search_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
