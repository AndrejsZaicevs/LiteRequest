use crate::AppState;
use crate::models::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::collections::HashMap;
use tauri::State;

type CmdResult<T> = Result<T, String>;

fn map_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ── Collections ──────────────────────────────────────────────

#[tauri::command]
pub fn list_collections(state: State<AppState>) -> CmdResult<Vec<Collection>> {
    state.db.lock().unwrap().list_collections().map_err(map_err)
}

#[tauri::command]
pub fn insert_collection(state: State<AppState>, collection: Collection) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .insert_collection(&collection)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_collection(state: State<AppState>, collection: Collection) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .update_collection(&collection)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_collection(state: State<AppState>, id: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .delete_collection(&id)
        .map_err(map_err)
}

#[tauri::command]
pub fn rename_collection(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .rename_collection(&id, &name)
        .map_err(map_err)
}

// ── Folders ──────────────────────────────────────────────────

#[tauri::command]
pub fn list_folders(state: State<AppState>, collection_id: String) -> CmdResult<Vec<Folder>> {
    state
        .db
        .lock()
        .unwrap()
        .list_folders_by_collection(&collection_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn insert_folder(state: State<AppState>, folder: Folder) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .insert_folder(&folder)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_folder(state: State<AppState>, id: String) -> CmdResult<()> {
    state.db.lock().unwrap().delete_folder(&id).map_err(map_err)
}

#[tauri::command]
pub fn rename_folder(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .rename_folder(&id, &name)
        .map_err(map_err)
}

#[tauri::command]
pub fn move_folder(
    state: State<AppState>,
    id: String,
    collection_id: String,
    parent_folder_id: Option<String>,
) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .move_folder(&id, &collection_id, parent_folder_id.as_deref())
        .map_err(map_err)
}

// ── Requests ─────────────────────────────────────────────────

#[tauri::command]
pub fn list_requests_by_collection(
    state: State<AppState>,
    collection_id: String,
) -> CmdResult<Vec<Request>> {
    state
        .db
        .lock()
        .unwrap()
        .list_requests_by_collection(&collection_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn list_requests_by_folder(
    state: State<AppState>,
    folder_id: String,
) -> CmdResult<Vec<Request>> {
    state
        .db
        .lock()
        .unwrap()
        .list_requests_by_folder(&folder_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn list_orphan_requests(
    state: State<AppState>,
    collection_id: String,
) -> CmdResult<Vec<Request>> {
    state
        .db
        .lock()
        .unwrap()
        .list_orphan_requests(&collection_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn insert_request(state: State<AppState>, request: Request) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .insert_request(&request)
        .map_err(map_err)
}

#[tauri::command]
pub fn rename_request(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .rename_request(&id, &name)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_request(state: State<AppState>, id: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .delete_request(&id)
        .map_err(map_err)
}

#[tauri::command]
pub fn move_request(
    state: State<AppState>,
    id: String,
    collection_id: String,
    folder_id: Option<String>,
) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .move_request(&id, &collection_id, folder_id.as_deref())
        .map_err(map_err)
}

#[tauri::command]
pub fn reorder_environments(state: State<AppState>, ordered_ids: Vec<String>) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .reorder_environments(&ordered_ids)
        .map_err(map_err)
}

#[tauri::command]
pub fn reorder_requests(state: State<AppState>, ordered_ids: Vec<String>) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .reorder_requests(&ordered_ids)
        .map_err(map_err)
}

#[tauri::command]
pub fn reorder_folders(state: State<AppState>, ordered_ids: Vec<String>) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .reorder_folders(&ordered_ids)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_request_version(
    state: State<AppState>,
    request_id: String,
    version_id: String,
) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .update_request_version(&request_id, &version_id)
        .map_err(map_err)
}

// ── Versions ─────────────────────────────────────────────────

#[tauri::command]
pub fn insert_version(state: State<AppState>, version: RequestVersion) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .insert_version(&version)
        .map_err(map_err)
}

#[tauri::command]
pub fn get_version(state: State<AppState>, id: String) -> CmdResult<RequestVersion> {
    state.db.lock().unwrap().get_version(&id).map_err(map_err)
}

#[tauri::command]
pub fn list_versions(state: State<AppState>, request_id: String) -> CmdResult<Vec<RequestVersion>> {
    state
        .db
        .lock()
        .unwrap()
        .list_versions_by_request(&request_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_version_data(
    state: State<AppState>,
    version_id: String,
    data: RequestData,
    created_at: String,
) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .update_version_data(&version_id, &data, &created_at)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_version(state: State<AppState>, version_id: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .delete_version(&version_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn version_has_executions(state: State<AppState>, version_id: String) -> CmdResult<bool> {
    Ok(state
        .db
        .lock()
        .unwrap()
        .version_has_executions(&version_id))
}

/// Single entry-point: the backend decides whether to update in place or
/// create a new version.  Returns the resulting version.
#[tauri::command]
pub fn save_version(
    state: State<AppState>,
    request_id: String,
    data: RequestData,
) -> CmdResult<RequestVersion> {
    state
        .db
        .lock()
        .unwrap()
        .save_version(&request_id, &data)
        .map_err(map_err)
}

// ── Executions ───────────────────────────────────────────────

#[tauri::command]
pub fn insert_execution(state: State<AppState>, execution: RequestExecution) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .insert_execution(&execution)
        .map_err(map_err)
}

#[tauri::command]
pub fn list_executions(
    state: State<AppState>,
    request_id: String,
) -> CmdResult<Vec<RequestExecution>> {
    state
        .db
        .lock()
        .unwrap()
        .list_executions_by_request(&request_id)
        .map_err(map_err)
}

// ── Environments ─────────────────────────────────────────────

#[tauri::command]
pub fn list_environments(state: State<AppState>) -> CmdResult<Vec<Environment>> {
    state.db.lock().unwrap().list_environments().map_err(map_err)
}

#[tauri::command]
pub fn insert_environment(state: State<AppState>, environment: Environment) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .insert_environment(&environment)
        .map_err(map_err)
}

#[tauri::command]
pub fn set_active_environment(state: State<AppState>, id: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .set_active_environment(&id)
        .map_err(map_err)
}

#[tauri::command]
pub fn rename_environment(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .rename_environment(&id, &name)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_environment(state: State<AppState>, id: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .delete_environment(&id)
        .map_err(map_err)
}

// ── Environment Variables ────────────────────────────────────

#[tauri::command]
pub fn list_env_variables(
    state: State<AppState>,
    environment_id: String,
) -> CmdResult<Vec<EnvVariable>> {
    state
        .db
        .lock()
        .unwrap()
        .list_env_variables(&environment_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn insert_env_variable(state: State<AppState>, variable: EnvVariable) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .insert_env_variable(&variable)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_env_variable(state: State<AppState>, variable: EnvVariable) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .update_env_variable(&variable)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_env_variable(state: State<AppState>, id: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .delete_env_variable(&id)
        .map_err(map_err)
}

#[tauri::command]
pub fn get_active_variables(state: State<AppState>) -> CmdResult<Vec<EnvVariable>> {
    state
        .db
        .lock()
        .unwrap()
        .get_active_variables()
        .map_err(map_err)
}

// ── Env Variable Defs (split model) ─────────────────────────

#[tauri::command]
pub fn list_env_var_defs(state: State<AppState>) -> CmdResult<Vec<EnvVarDef>> {
    state.db.lock().unwrap().list_env_var_defs().map_err(map_err)
}

#[tauri::command]
pub fn insert_env_var_def(state: State<AppState>, def: EnvVarDef) -> CmdResult<()> {
    state.db.lock().unwrap().insert_env_var_def(&def).map_err(map_err)
}

#[tauri::command]
pub fn update_env_var_def_key(state: State<AppState>, def_id: String, key: String) -> CmdResult<()> {
    state.db.lock().unwrap().update_env_var_def_key(&def_id, &key).map_err(map_err)
}

#[tauri::command]
pub fn delete_env_var_def(state: State<AppState>, def_id: String) -> CmdResult<()> {
    state.db.lock().unwrap().delete_env_var_def(&def_id).map_err(map_err)
}

#[tauri::command]
pub fn upsert_env_var_value(
    state: State<AppState>,
    val_id: String,
    def_id: String,
    environment_id: String,
    value: String,
    is_secret: bool,
) -> CmdResult<()> {
    state.db.lock().unwrap()
        .upsert_env_var_value(&val_id, &def_id, &environment_id, &value, is_secret)
        .map_err(map_err)
}

#[tauri::command]
pub fn load_env_var_rows(state: State<AppState>, environment_id: String) -> CmdResult<Vec<VarRow>> {
    state.db.lock().unwrap().load_env_var_rows(&environment_id).map_err(map_err)
}

// ── Collection Variables ─────────────────────────────────────

#[tauri::command]
pub fn insert_var_def(state: State<AppState>, def: VarDef) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .insert_var_def(&def)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_var_def_key(state: State<AppState>, def_id: String, key: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .update_var_def_key(&def_id, &key)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_var_def(state: State<AppState>, def_id: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .delete_var_def(&def_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn list_var_defs(state: State<AppState>, collection_id: String) -> CmdResult<Vec<VarDef>> {
    state
        .db
        .lock()
        .unwrap()
        .list_var_defs(&collection_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn upsert_var_value(
    state: State<AppState>,
    val_id: String,
    def_id: String,
    environment_id: String,
    value: String,
    is_secret: bool,
) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .upsert_var_value(&val_id, &def_id, &environment_id, &value, is_secret)
        .map_err(map_err)
}

#[tauri::command]
pub fn load_var_rows(
    state: State<AppState>,
    collection_id: String,
    environment_id: String,
) -> CmdResult<Vec<VarRow>> {
    state
        .db
        .lock()
        .unwrap()
        .load_var_rows(&collection_id, &environment_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn get_active_collection_variables(
    state: State<AppState>,
    collection_id: String,
) -> CmdResult<Vec<(String, String)>> {
    state
        .db
        .lock()
        .unwrap()
        .get_active_collection_variables(&collection_id)
        .map_err(map_err)
}

// ── App Settings ─────────────────────────────────────────────

#[tauri::command]
pub fn get_app_setting(state: State<AppState>, key: String) -> CmdResult<Option<String>> {
    state
        .db
        .lock()
        .unwrap()
        .get_app_setting(&key)
        .map_err(map_err)
}

#[tauri::command]
pub fn set_app_setting(state: State<AppState>, key: String, value: String) -> CmdResult<()> {
    state
        .db
        .lock()
        .unwrap()
        .set_app_setting(&key, &value)
        .map_err(map_err)
}

// ── HTTP Execution ───────────────────────────────────────────

#[tauri::command]
pub async fn execute_request(
    state: tauri::State<'_, crate::AppState>,
    data: RequestData,
    variables: HashMap<String, String>,
    base_path: String,
    client_certs: Vec<ClientCertEntry>,
) -> CmdResult<(ResponseData, u64)> {
    // Mint a fresh token for this request
    let token = {
        let mut guard = state.cancel_token.lock().unwrap();
        *guard = tokio_util::sync::CancellationToken::new();
        guard.clone()
    };

    tokio::select! {
        result = crate::http::client::execute_request(&data, &variables, &base_path, &client_certs) => {
            result.map_err(map_err)
        }
        _ = token.cancelled() => {
            Err("Request cancelled".into())
        }
    }
}

#[tauri::command]
pub fn cancel_request(
    state: tauri::State<'_, crate::AppState>,
) {
    state.cancel_token.lock().unwrap().cancel();
}

// ── Clipboard ────────────────────────────────────────────────

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> CmdResult<()> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(&text))
        .map_err(|e| e.to_string())
}

// ── cURL ─────────────────────────────────────────────────────

#[tauri::command]
pub fn to_curl(
    data: RequestData,
    variables: HashMap<String, String>,
    base_path: String,
) -> String {
    crate::http::curl::to_curl(&data, &variables, &base_path)
}

#[tauri::command]
pub fn parse_curl(input: String) -> CmdResult<RequestData> {
    crate::http::curl::parse_curl(&input)
}

// ── Interpolation helpers ────────────────────────────────────

#[tauri::command]
pub fn interpolate(input: String, variables: HashMap<String, String>) -> String {
    crate::http::interpolation::interpolate(&input, &variables)
}

#[tauri::command]
pub fn resolve_url(
    base_path: String,
    request_url: String,
    variables: HashMap<String, String>,
) -> String {
    crate::http::interpolation::resolve_url(&base_path, &request_url, &variables)
}

#[tauri::command]
pub fn extract_path_params(url: String) -> Vec<String> {
    crate::http::interpolation::extract_path_params(&url)
}

// ── Maintenance ──────────────────────────────────────────────

#[tauri::command]
pub fn prune_old_executions(state: State<AppState>, days: i64) -> CmdResult<usize> {
    state
        .db
        .lock()
        .unwrap()
        .prune_old_executions(days)
        .map_err(map_err)
}

#[tauri::command]
pub fn get_db_stats(state: State<AppState>) -> CmdResult<DbStats> {
    let stats = state.db.lock().unwrap().get_db_stats().map_err(map_err)?;
    // Supplement with actual file size if available (more accurate than page_count * page_size)
    let file_size = std::fs::metadata(&state.db_path)
        .map(|m| m.len() as i64)
        .unwrap_or(stats.db_size_bytes);
    Ok(DbStats { db_size_bytes: file_size, ..stats })
}

#[tauri::command]
pub fn cleanup_old_data(state: State<AppState>, cutoff_date: String) -> CmdResult<CleanupResult> {
    state
        .db
        .lock()
        .unwrap()
        .cleanup_old_data(&cutoff_date)
        .map_err(map_err)
}

// ── Search ───────────────────────────────────────────────────

#[tauri::command]
pub fn search_all(state: State<AppState>, query: String) -> CmdResult<Vec<crate::models::SearchHit>> {
    state
        .db
        .lock()
        .unwrap()
        .search_all(&query, 80)
        .map_err(map_err)
}

// ── Fingerprint ──────────────────────────────────────────────

#[tauri::command]
pub fn compute_fingerprint(data: RequestData) -> String {
    data.fingerprint()
}

// ── File I/O ─────────────────────────────────────────────────

/// Write `data` to `path`. If `is_base64` is true, decode from base64 first (binary responses).
#[tauri::command]
pub fn save_file(path: String, data: String, is_base64: bool) -> CmdResult<()> {
    if is_base64 {
        let bytes = BASE64.decode(&data).map_err(|e| format!("Failed to decode base64: {e}"))?;
        std::fs::write(&path, bytes).map_err(|e| format!("Failed to write file: {e}"))
    } else {
        std::fs::write(&path, data.as_bytes()).map_err(|e| format!("Failed to write file: {e}"))
    }
}

// ── Import ────────────────────────────────────────────────────

#[tauri::command]
pub fn import_postman_collection(
    state: State<AppState>,
    path: String,
) -> CmdResult<crate::import::postman::ImportSummary> {
    let db = state.db.lock().unwrap();
    crate::import::postman::import_from_path(&path, &db).map_err(|e| e)
}

#[tauri::command]
pub fn export_collection_to_postman(
    state: State<AppState>,
    collection_id: String,
) -> CmdResult<String> {
    let db = state.db.lock().unwrap();
    crate::import::postman::export_collection(&collection_id, &db).map_err(|e| e)
}
