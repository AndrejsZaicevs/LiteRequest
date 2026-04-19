use crate::AppState;
use crate::error::LiteRequestError;
use crate::models::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::collections::HashMap;
use tauri::State;

type CmdResult<T> = Result<T, LiteRequestError>;

fn map_err(e: impl std::fmt::Display) -> LiteRequestError {
    LiteRequestError::Internal(e.to_string())
}

fn db<'a>(state: &'a State<'a, AppState>) -> CmdResult<std::sync::MutexGuard<'a, crate::db::Database>> {
    state.db.lock().map_err(|e| LiteRequestError::LockPoisoned(e.to_string()))
}

// ── Collections ──────────────────────────────────────────────

#[tauri::command]
pub fn list_collections(state: State<AppState>) -> CmdResult<Vec<Collection>> {
    db(&state)?.list_collections().map_err(map_err)
}

#[tauri::command]
pub fn insert_collection(state: State<AppState>, collection: Collection) -> CmdResult<()> {
    db(&state)?
        .insert_collection(&collection)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_collection(state: State<AppState>, collection: Collection) -> CmdResult<()> {
    db(&state)?
        .update_collection(&collection)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_collection(state: State<AppState>, id: String) -> CmdResult<()> {
    db(&state)?
        .delete_collection(&id)
        .map_err(map_err)
}

#[tauri::command]
pub fn rename_collection(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    db(&state)?
        .rename_collection(&id, &name)
        .map_err(map_err)
}

// ── Folders ──────────────────────────────────────────────────

#[tauri::command]
pub fn list_folders(state: State<AppState>, collection_id: String) -> CmdResult<Vec<Folder>> {
    db(&state)?
        .list_folders_by_collection(&collection_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn insert_folder(state: State<AppState>, folder: Folder) -> CmdResult<()> {
    db(&state)?
        .insert_folder(&folder)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_folder(state: State<AppState>, id: String) -> CmdResult<()> {
    db(&state)?.delete_folder(&id).map_err(map_err)
}

#[tauri::command]
pub fn rename_folder(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    db(&state)?
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
    db(&state)?
        .move_folder(&id, &collection_id, parent_folder_id.as_deref())
        .map_err(map_err)
}

// ── Requests ─────────────────────────────────────────────────

#[tauri::command]
pub fn list_requests_by_collection(
    state: State<AppState>,
    collection_id: String,
) -> CmdResult<Vec<Request>> {
    db(&state)?
        .list_requests_by_collection(&collection_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn list_requests_by_folder(
    state: State<AppState>,
    folder_id: String,
) -> CmdResult<Vec<Request>> {
    db(&state)?
        .list_requests_by_folder(&folder_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn list_orphan_requests(
    state: State<AppState>,
    collection_id: String,
) -> CmdResult<Vec<Request>> {
    db(&state)?
        .list_orphan_requests(&collection_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn insert_request(state: State<AppState>, request: Request) -> CmdResult<()> {
    db(&state)?
        .insert_request(&request)
        .map_err(map_err)
}

#[tauri::command]
pub fn rename_request(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    db(&state)?
        .rename_request(&id, &name)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_request(state: State<AppState>, id: String) -> CmdResult<()> {
    db(&state)?
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
    db(&state)?
        .move_request(&id, &collection_id, folder_id.as_deref())
        .map_err(map_err)
}

#[tauri::command]
pub fn reorder_environments(state: State<AppState>, ordered_ids: Vec<String>) -> CmdResult<()> {
    db(&state)?
        .reorder_environments(&ordered_ids)
        .map_err(map_err)
}

#[tauri::command]
pub fn reorder_requests(state: State<AppState>, ordered_ids: Vec<String>) -> CmdResult<()> {
    db(&state)?
        .reorder_requests(&ordered_ids)
        .map_err(map_err)
}

#[tauri::command]
pub fn reorder_folders(state: State<AppState>, ordered_ids: Vec<String>) -> CmdResult<()> {
    db(&state)?
        .reorder_folders(&ordered_ids)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_request_version(
    state: State<AppState>,
    request_id: String,
    version_id: String,
) -> CmdResult<()> {
    db(&state)?
        .update_request_version(&request_id, &version_id)
        .map_err(map_err)
}

// ── Versions ─────────────────────────────────────────────────

#[tauri::command]
pub fn insert_version(state: State<AppState>, version: RequestVersion) -> CmdResult<()> {
    db(&state)?
        .insert_version(&version)
        .map_err(map_err)
}

#[tauri::command]
pub fn get_version(state: State<AppState>, id: String) -> CmdResult<RequestVersion> {
    db(&state)?.get_version(&id).map_err(map_err)
}

#[tauri::command]
pub fn list_versions(state: State<AppState>, request_id: String) -> CmdResult<Vec<RequestVersion>> {
    db(&state)?
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
    db(&state)?
        .update_version_data(&version_id, &data, &created_at)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_version(state: State<AppState>, version_id: String) -> CmdResult<()> {
    db(&state)?
        .delete_version(&version_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn version_has_executions(state: State<AppState>, version_id: String) -> CmdResult<bool> {
    Ok(db(&state)?
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
    db(&state)?
        .save_version(&request_id, &data)
        .map_err(map_err)
}

// ── Executions ───────────────────────────────────────────────

#[tauri::command]
pub fn insert_execution(state: State<AppState>, execution: RequestExecution) -> CmdResult<()> {
    db(&state)?
        .insert_execution(&execution)
        .map_err(map_err)
}

#[tauri::command]
pub fn list_executions(
    state: State<AppState>,
    request_id: String,
) -> CmdResult<Vec<RequestExecution>> {
    db(&state)?
        .list_executions_by_request(&request_id)
        .map_err(map_err)
}

// ── Environments ─────────────────────────────────────────────

#[tauri::command]
pub fn list_environments(state: State<AppState>) -> CmdResult<Vec<Environment>> {
    db(&state)?.list_environments().map_err(map_err)
}

#[tauri::command]
pub fn insert_environment(state: State<AppState>, environment: Environment) -> CmdResult<()> {
    db(&state)?
        .insert_environment(&environment)
        .map_err(map_err)
}

#[tauri::command]
pub fn set_active_environment(state: State<AppState>, id: String) -> CmdResult<()> {
    db(&state)?
        .set_active_environment(&id)
        .map_err(map_err)
}

#[tauri::command]
pub fn rename_environment(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    db(&state)?
        .rename_environment(&id, &name)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_environment(state: State<AppState>, id: String) -> CmdResult<()> {
    db(&state)?
        .delete_environment(&id)
        .map_err(map_err)
}

// ── Environment Variables ────────────────────────────────────

#[tauri::command]
pub fn list_env_variables(
    state: State<AppState>,
    environment_id: String,
) -> CmdResult<Vec<EnvVariable>> {
    db(&state)?
        .list_env_variables(&environment_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn insert_env_variable(state: State<AppState>, variable: EnvVariable) -> CmdResult<()> {
    db(&state)?
        .insert_env_variable(&variable)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_env_variable(state: State<AppState>, variable: EnvVariable) -> CmdResult<()> {
    db(&state)?
        .update_env_variable(&variable)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_env_variable(state: State<AppState>, id: String) -> CmdResult<()> {
    db(&state)?
        .delete_env_variable(&id)
        .map_err(map_err)
}

#[tauri::command]
pub fn get_active_variables(state: State<AppState>) -> CmdResult<Vec<EnvVariable>> {
    db(&state)?
        .get_active_variables()
        .map_err(map_err)
}

// ── Env Variable Defs (split model) ─────────────────────────

#[tauri::command]
pub fn list_env_var_defs(state: State<AppState>) -> CmdResult<Vec<EnvVarDef>> {
    db(&state)?.list_env_var_defs().map_err(map_err)
}

#[tauri::command]
pub fn insert_env_var_def(state: State<AppState>, def: EnvVarDef) -> CmdResult<()> {
    db(&state)?.insert_env_var_def(&def).map_err(map_err)
}

#[tauri::command]
pub fn update_env_var_def_key(state: State<AppState>, def_id: String, key: String) -> CmdResult<()> {
    db(&state)?.update_env_var_def_key(&def_id, &key).map_err(map_err)
}

#[tauri::command]
pub fn delete_env_var_def(state: State<AppState>, def_id: String) -> CmdResult<()> {
    db(&state)?.delete_env_var_def(&def_id).map_err(map_err)
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
    db(&state)?.load_env_var_rows(&environment_id).map_err(map_err)
}

// ── Collection Variables ─────────────────────────────────────

#[tauri::command]
pub fn insert_var_def(state: State<AppState>, def: VarDef) -> CmdResult<()> {
    db(&state)?
        .insert_var_def(&def)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_var_def_key(state: State<AppState>, def_id: String, key: String) -> CmdResult<()> {
    db(&state)?
        .update_var_def_key(&def_id, &key)
        .map_err(map_err)
}

#[tauri::command]
pub fn delete_var_def(state: State<AppState>, def_id: String) -> CmdResult<()> {
    db(&state)?
        .delete_var_def(&def_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn list_var_defs(state: State<AppState>, collection_id: String) -> CmdResult<Vec<VarDef>> {
    db(&state)?
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
    db(&state)?
        .upsert_var_value(&val_id, &def_id, &environment_id, &value, is_secret)
        .map_err(map_err)
}

#[tauri::command]
pub fn update_var_def_type(state: State<AppState>, def_id: String, var_type: String) -> CmdResult<()> {
    db(&state)?
        .update_var_def_type(&def_id, &var_type)
        .map_err(map_err)
}

#[tauri::command]
pub fn load_operative_var_rows(
    state: State<AppState>,
    collection_id: String,
    environment_id: String,
) -> CmdResult<Vec<VarRow>> {
    db(&state)?
        .load_operative_var_rows(&collection_id, &environment_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn load_var_rows(
    state: State<AppState>,
    collection_id: String,
    environment_id: String,
) -> CmdResult<Vec<VarRow>> {
    db(&state)?
        .load_var_rows(&collection_id, &environment_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn get_active_collection_variables(
    state: State<AppState>,
    collection_id: String,
) -> CmdResult<Vec<(String, String)>> {
    db(&state)?
        .get_active_collection_variables(&collection_id)
        .map_err(map_err)
}

// ── App Settings ─────────────────────────────────────────────

#[tauri::command]
pub fn get_app_setting(state: State<AppState>, key: String) -> CmdResult<Option<String>> {
    db(&state)?
        .get_app_setting(&key)
        .map_err(map_err)
}

#[tauri::command]
pub fn set_app_setting(state: State<AppState>, key: String, value: String) -> CmdResult<()> {
    db(&state)?
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
        let mut guard = state.cancel_token.lock().map_err(|e| LiteRequestError::LockPoisoned(e.to_string()))?;
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
    if let Ok(token) = state.cancel_token.lock() { token.cancel(); }
}

// ── Clipboard ────────────────────────────────────────────────

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> CmdResult<()> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(&text))
        .map_err(map_err)
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
    crate::http::curl::parse_curl(&input).map_err(map_err)
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
    db(&state)?
        .prune_old_executions(days)
        .map_err(map_err)
}

#[tauri::command]
pub fn get_db_stats(state: State<AppState>) -> CmdResult<DbStats> {
    let stats = db(&state)?.get_db_stats().map_err(map_err)?;
    // Supplement with actual file size if available (more accurate than page_count * page_size)
    let file_size = std::fs::metadata(&state.db_path)
        .map(|m| m.len() as i64)
        .unwrap_or(stats.db_size_bytes);
    Ok(DbStats { db_size_bytes: file_size, ..stats })
}

#[tauri::command]
pub fn cleanup_old_data(state: State<AppState>, cutoff_date: String) -> CmdResult<CleanupResult> {
    db(&state)?
        .cleanup_old_data(&cutoff_date)
        .map_err(map_err)
}

// ── Search ───────────────────────────────────────────────────

#[tauri::command]
pub fn search_all(state: State<AppState>, query: String) -> CmdResult<Vec<crate::models::SearchHit>> {
    db(&state)?
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
        std::fs::write(&path, bytes).map_err(map_err)
    } else {
        std::fs::write(&path, data.as_bytes()).map_err(map_err)
    }
}

// ── Import ────────────────────────────────────────────────────

#[tauri::command]
pub fn import_postman_collection(
    state: State<AppState>,
    path: String,
) -> CmdResult<crate::import::postman::ImportSummary> {
    let db = db(&state)?;
    crate::import::postman::import_from_path(&path, &db).map_err(map_err)
}

#[tauri::command]
pub fn export_collection_to_postman(
    state: State<AppState>,
    collection_id: String,
) -> CmdResult<String> {
    let db = db(&state)?;
    crate::import::postman::export_collection(&collection_id, &db).map_err(map_err)
}

// ── Trash ─────────────────────────────────────────────────────

#[tauri::command]
pub fn list_trash(state: State<AppState>) -> CmdResult<Vec<crate::models::TrashedItem>> {
    db(&state)?.list_trash().map_err(map_err)
}

#[tauri::command]
pub fn restore_item(state: State<AppState>, item_type: String, id: String) -> CmdResult<()> {
    db(&state)?.restore_item(&item_type, &id).map_err(map_err)
}

#[tauri::command]
pub fn purge_item(state: State<AppState>, item_type: String, id: String) -> CmdResult<()> {
    db(&state)?.purge_item(&item_type, &id).map_err(map_err)
}

#[tauri::command]
pub fn empty_trash(state: State<AppState>) -> CmdResult<()> {
    db(&state)?.empty_trash().map_err(map_err)
}

// ── Clone ─────────────────────────────────────────────────────

#[tauri::command]
pub fn clone_request(state: State<AppState>, id: String) -> CmdResult<String> {
    db(&state)?.clone_request(&id).map_err(map_err)
}

#[tauri::command]
pub fn clone_folder(state: State<AppState>, id: String) -> CmdResult<String> {
    db(&state)?.clone_folder(&id).map_err(map_err)
}

// ── Scripts ──────────────────────────────────────────────────

#[tauri::command]
pub fn list_scripts_by_collection(state: State<AppState>, collection_id: String) -> CmdResult<Vec<Script>> {
    db(&state)?.list_scripts_by_collection(&collection_id).map_err(map_err)
}

#[tauri::command]
pub fn list_scripts_by_folder(state: State<AppState>, folder_id: String) -> CmdResult<Vec<Script>> {
    db(&state)?.list_scripts_by_folder(&folder_id).map_err(map_err)
}

#[tauri::command]
pub fn insert_script(state: State<AppState>, script: Script) -> CmdResult<()> {
    db(&state)?.insert_script(&script).map_err(map_err)
}

#[tauri::command]
pub fn get_script(state: State<AppState>, id: String) -> CmdResult<Script> {
    db(&state)?.get_script(&id).map_err(map_err)
}

#[tauri::command]
pub fn rename_script(state: State<AppState>, id: String, name: String) -> CmdResult<()> {
    db(&state)?.rename_script(&id, &name).map_err(map_err)
}

#[tauri::command]
pub fn delete_script(state: State<AppState>, id: String) -> CmdResult<()> {
    db(&state)?.delete_script(&id).map_err(map_err)
}

#[tauri::command]
pub fn move_script(state: State<AppState>, id: String, collection_id: String, folder_id: Option<String>) -> CmdResult<()> {
    db(&state)?.move_script(&id, &collection_id, folder_id.as_deref()).map_err(map_err)
}

// ── Script Versions ──────────────────────────────────────────

#[tauri::command]
pub fn get_script_version(state: State<AppState>, id: String) -> CmdResult<ScriptVersion> {
    db(&state)?.get_script_version(&id).map_err(map_err)
}

#[tauri::command]
pub fn list_script_versions(state: State<AppState>, script_id: String) -> CmdResult<Vec<ScriptVersion>> {
    db(&state)?.list_script_versions(&script_id).map_err(map_err)
}

#[tauri::command]
pub fn save_script_version(
    state: State<AppState>,
    script_id: String,
    content_ts: String,
    content_js: String,
) -> CmdResult<ScriptVersion> {
    db(&state)?.save_script_version(&script_id, &content_ts, &content_js).map_err(map_err)
}

#[tauri::command]
pub fn script_version_has_runs(state: State<AppState>, version_id: String) -> CmdResult<bool> {
    Ok(db(&state)?.script_version_has_runs(&version_id))
}

// ── Script Runs ──────────────────────────────────────────────

#[tauri::command]
pub fn list_script_runs(state: State<AppState>, script_id: String) -> CmdResult<Vec<ScriptRun>> {
    db(&state)?.list_script_runs(&script_id).map_err(map_err)
}

#[tauri::command]
pub fn list_script_runs_by_request(state: State<AppState>, request_id: String) -> CmdResult<Vec<ScriptRun>> {
    db(&state)?.list_script_runs_by_request(&request_id).map_err(map_err)
}

// ── Post-Script ──────────────────────────────────────────────

#[tauri::command]
pub fn get_post_script(state: State<AppState>, request_id: String) -> CmdResult<String> {
    db(&state)?.get_post_script(&request_id).map_err(map_err)
}

#[tauri::command]
pub fn set_post_script(state: State<AppState>, request_id: String, script: String) -> CmdResult<()> {
    db(&state)?.set_post_script(&request_id, &script).map_err(map_err)
}

#[tauri::command]
pub fn apply_script_variables(
    state: State<AppState>,
    collection_id: String,
    environment_id: String,
    variables: std::collections::HashMap<String, String>,
) -> CmdResult<()> {
    db(&state)?.apply_script_variables(&collection_id, &environment_id, &variables).map_err(map_err)
}

// ── Script Execution ─────────────────────────────────────────

#[tauri::command]
pub async fn run_post_script(
    state: State<'_, AppState>,
    request_id: String,
    execution_id: String,
    request_data: RequestData,
    response_data: ResponseData,
    latency_ms: u64,
    variables: HashMap<String, String>,
    environment: String,
    script_js: String,
) -> CmdResult<ScriptResult> {
    let post_script = db(&state)?.get_post_script(&request_id).map_err(map_err)?;
    if post_script.is_empty() && script_js.is_empty() {
        return Ok(ScriptResult {
            status: "success".to_string(),
            logs: vec![],
            variables_set: HashMap::new(),
            error: None,
            duration_ms: 0,
            transformed_response: None,
        });
    }

    let js_code = if script_js.is_empty() { post_script.clone() } else { script_js };
    let script_source = post_script;

    // Run QuickJS on a blocking thread (not Send-safe)
    let result = tokio::task::spawn_blocking(move || {
        use crate::scripting::context::{PostExecContext, ScriptSideEffects, SharedEffects};
        use std::sync::Arc;

        let effects: SharedEffects = Arc::new(std::sync::Mutex::new(ScriptSideEffects::default()));
        let start = std::time::Instant::now();

        let engine = crate::scripting::runtime::ScriptEngine::new()?;
        let ctx = engine.create_context()?;

        let effects_clone = effects.clone();
        let post_ctx = PostExecContext {
            request: request_data,
            response: response_data,
            latency_ms,
            variables,
            environment,
        };

        let exec_result = ctx.with(|js_ctx| {
            use rquickjs::CatchResultExt;
            crate::scripting::context::inject_post_exec_globals(&js_ctx, &post_ctx, effects_clone)?;
            js_ctx.eval::<(), String>(js_code)
                .catch(&js_ctx)
                .map_err(|e| format!("{e}"))
        });

        let duration_ms = start.elapsed().as_millis() as u64;
        let eff = effects.lock().unwrap_or_else(|e| e.into_inner());

        let (status, error) = match exec_result {
            Ok(()) => ("success".to_string(), None),
            Err(e) => ("error".to_string(), Some(e)),
        };

        Ok::<(ScriptResult, ScriptRun), String>((
            ScriptResult {
                status: status.clone(),
                logs: eff.logs.clone(),
                variables_set: eff.variables_set.clone(),
                error: error.clone(),
                duration_ms,
                transformed_response: eff.transformed_response.clone(),
            },
            ScriptRun {
                id: uuid::Uuid::new_v4().to_string(),
                script_id: None,
                version_id: None,
                request_id: Some(request_id),
                execution_id: Some(execution_id),
                status,
                logs: serde_json::to_string(&eff.logs).unwrap_or_else(|_| "[]".to_string()),
                variables_set: serde_json::to_string(&eff.variables_set).unwrap_or_else(|_| "{}".to_string()),
                script_source,
                error,
                duration_ms,
                executed_at: chrono::Utc::now().to_rfc3339(),
            },
        ))
    }).await.map_err(|e| LiteRequestError::Internal(format!("Script task failed: {e}")))?
      .map_err(map_err)?;

    let _ = db(&state)?.insert_script_run(&result.1);
    Ok(result.0)
}

#[tauri::command]
pub async fn run_script(
    state: State<'_, AppState>,
    script_id: String,
    content_js: String,
    variables: HashMap<String, String>,
    environment: String,
) -> CmdResult<ScriptResult> {
    if content_js.is_empty() {
        return Ok(ScriptResult {
            status: "success".to_string(),
            logs: vec![],
            variables_set: HashMap::new(),
            error: None,
            duration_ms: 0,
            transformed_response: None,
        });
    }

    let script = db(&state)?.get_script(&script_id).map_err(map_err)?;
    let version_id = script.current_version_id.clone();
    let js_for_log = content_js.clone();

    let result = tokio::task::spawn_blocking(move || {
        use crate::scripting::context::{StandaloneContext, ScriptSideEffects, SharedEffects};
        use std::sync::Arc;

        let effects: SharedEffects = Arc::new(std::sync::Mutex::new(ScriptSideEffects::default()));
        let start = std::time::Instant::now();

        let engine = crate::scripting::runtime::ScriptEngine::new()?;
        let ctx = engine.create_context()?;

        let effects_clone = effects.clone();
        let standalone_ctx = StandaloneContext { variables, environment };

        let exec_result = ctx.with(|js_ctx| {
            use rquickjs::CatchResultExt;
            crate::scripting::context::inject_standalone_globals(&js_ctx, &standalone_ctx, effects_clone)?;
            crate::scripting::bridge::install_sleep_bridge(&js_ctx)?;
            js_ctx.eval::<(), String>(content_js)
                .catch(&js_ctx)
                .map_err(|e| format!("{e}"))
        });

        let duration_ms = start.elapsed().as_millis() as u64;
        let eff = effects.lock().unwrap_or_else(|e| e.into_inner());

        let (status, error) = match exec_result {
            Ok(()) => ("success".to_string(), None),
            Err(e) => ("error".to_string(), Some(e)),
        };

        Ok::<(ScriptResult, ScriptRun), String>((
            ScriptResult {
                status: status.clone(),
                logs: eff.logs.clone(),
                variables_set: eff.variables_set.clone(),
                error: error.clone(),
                duration_ms,
                transformed_response: eff.transformed_response.clone(),
            },
            ScriptRun {
                id: uuid::Uuid::new_v4().to_string(),
                script_id: Some(script_id),
                version_id,
                request_id: None,
                execution_id: None,
                status,
                logs: serde_json::to_string(&eff.logs).unwrap_or_else(|_| "[]".to_string()),
                variables_set: serde_json::to_string(&eff.variables_set).unwrap_or_else(|_| "{}".to_string()),
                script_source: js_for_log,
                error,
                duration_ms,
                executed_at: chrono::Utc::now().to_rfc3339(),
            },
        ))
    }).await.map_err(|e| LiteRequestError::Internal(format!("Script task failed: {e}")))?
      .map_err(map_err)?;

    let _ = db(&state)?.insert_script_run(&result.1);
    Ok(result.0)
}

// ── Type Generation ──────────────────────────────────────────

#[tauri::command]
pub fn generate_script_types(state: State<AppState>) -> CmdResult<String> {
    let db_guard = db(&state)?;
    crate::scripting::typegen::generate_all_types(&db_guard).map_err(map_err)
}
