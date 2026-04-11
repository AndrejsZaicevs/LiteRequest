use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use crate::db::Database;
use crate::models::*;
use crate::ui::collection_tree::*;
use crate::ui::collection_config::*;
use crate::ui::request_editor::*;
use crate::ui::response_view::*;
use crate::ui::history_panel;
use crate::ui::environment_panel::*;

/// What the center panel should display
#[derive(Debug, Clone, PartialEq)]
enum CenterView {
    Welcome,
    CollectionConfig(String), // collection_id
    RequestEditor(String),    // request_id
}

/// Result of an async HTTP request
struct HttpResult {
    request_id: String,
    version_id: String,
    result: Result<(ResponseData, u64), String>,
}

pub struct LiteRequestApp {
    db: Database,
    tokio_rt: tokio::runtime::Runtime,

    // Data caches
    collections: Vec<Collection>,
    folders: Vec<Folder>,
    requests: Vec<Request>,
    versions: Vec<RequestVersion>,
    executions: Vec<RequestExecution>,
    environments: Vec<Environment>,
    env_variables: Vec<EnvVariable>,

    // UI state
    tree_state: CollectionTreeState,
    editor_state: RequestEditorState,
    response_state: ResponseViewState,
    env_panel_state: EnvironmentPanelState,
    collection_config_state: CollectionConfigState,
    center_view: CenterView,

    // Currently open request
    current_request: Option<Request>,
    current_execution: Option<RequestExecution>,
    selected_version_id: Option<String>,
    selected_execution_id: Option<String>,

    // Async HTTP channel
    http_tx: mpsc::Sender<HttpResult>,
    http_rx: mpsc::Receiver<HttpResult>,

    // Loading/error/status state
    is_loading: bool,
    error_message: Option<String>,
    status_message: Option<(String, std::time::Instant)>,

    // Resizable request/response split (0.0–1.0, fraction for request)
    split_ratio: f32,
    is_dragging_split: bool,

    // Confirmation modal state
    pending_delete_collection: Option<String>, // collection id awaiting confirmation
}

impl LiteRequestApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::ui::theme::apply_theme(&cc.egui_ctx);

        let db_path = dirs_data_path();
        let db = Database::open(&db_path).expect("Failed to open database");
        let tokio_rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let (http_tx, http_rx) = mpsc::channel();

        let mut app = Self {
            db,
            tokio_rt,
            collections: Vec::new(),
            folders: Vec::new(),
            requests: Vec::new(),
            versions: Vec::new(),
            executions: Vec::new(),
            environments: Vec::new(),
            env_variables: Vec::new(),
            tree_state: CollectionTreeState::default(),
            editor_state: RequestEditorState::default(),
            response_state: ResponseViewState::default(),
            env_panel_state: EnvironmentPanelState::default(),
            collection_config_state: CollectionConfigState::default(),
            center_view: CenterView::Welcome,
            current_request: None,
            current_execution: None,
            selected_version_id: None,
            selected_execution_id: None,
            http_tx,
            http_rx,
            is_loading: false,
            error_message: None,
            status_message: None,
            split_ratio: 0.5,
            is_dragging_split: false,
            pending_delete_collection: None,
        };

        app.refresh_all_data();

        if app.collections.is_empty() {
            let now = chrono::Utc::now().to_rfc3339();
            let collection = Collection {
                id: uuid::Uuid::new_v4().to_string(),
                name: "My Collection".to_string(),
                base_path: String::new(),
                auth_config: None,
                created_at: now.clone(),
                updated_at: now,
            };
            let _ = app.db.insert_collection(&collection);
            app.refresh_all_data();
        }

        app
    }

    fn refresh_all_data(&mut self) {
        self.collections = self.db.list_collections().unwrap_or_default();
        self.environments = self.db.list_environments().unwrap_or_default();
        self.env_variables = self.db.get_active_variables().unwrap_or_default();

        self.folders.clear();
        self.requests.clear();
        for c in &self.collections {
            if let Ok(f) = self.db.list_folders_by_collection(&c.id) {
                self.folders.extend(f);
            }
            if let Ok(r) = self.db.list_requests_by_collection(&c.id) {
                self.requests.extend(r);
            }
        }

        if let Some(req) = &self.current_request {
            self.versions = self.db.list_versions_by_request(&req.id).unwrap_or_default();
            self.executions = self.db.list_executions_by_request(&req.id).unwrap_or_default();
        }
    }

    fn select_collection(&mut self, collection_id: &str) {
        if let Some(collection) = self.collections.iter().find(|c| c.id == collection_id) {
            self.collection_config_state.load_from(collection);
            self.tree_state.selected_collection_id = Some(collection_id.to_string());
            self.tree_state.selected_request_id = None;
            self.current_request = None;
            self.center_view = CenterView::CollectionConfig(collection_id.to_string());
        }
    }

    fn select_request(&mut self, request_id: &str) {
        if let Some(req) = self.requests.iter().find(|r| r.id == request_id).cloned() {
            if let Some(vid) = &req.current_version_id {
                if let Ok(version) = self.db.get_version(vid) {
                    self.editor_state.data = version.data;
                    self.editor_state.dirty = false;
                    self.editor_state.json_error = None;
                    self.selected_version_id = Some(vid.clone());
                }
            } else {
                self.editor_state = RequestEditorState::default();
            }

            self.tree_state.selected_request_id = Some(req.id.clone());
            self.tree_state.selected_collection_id = None;
            self.current_request = Some(req.clone());
            self.center_view = CenterView::RequestEditor(req.id.clone());

            self.versions = self.db.list_versions_by_request(&req.id).unwrap_or_default();
            self.executions = self.db.list_executions_by_request(&req.id).unwrap_or_default();

            self.current_execution = self.executions.first().cloned();
            self.selected_execution_id = self.current_execution.as_ref().map(|e| e.id.clone());
            self.response_state = ResponseViewState::default();
        }
    }

    fn save_version(&mut self) {
        if let Some(req) = &self.current_request {
            let now = chrono::Utc::now().to_rfc3339();
            let version = RequestVersion {
                id: uuid::Uuid::new_v4().to_string(),
                request_id: req.id.clone(),
                data: self.editor_state.data.clone(),
                created_at: now,
            };
            if self.db.insert_version(&version).is_ok() {
                self.selected_version_id = Some(version.id.clone());
                self.versions = self.db.list_versions_by_request(&req.id).unwrap_or_default();
                if let Some(r) = self.requests.iter_mut().find(|r| r.id == req.id) {
                    r.current_version_id = Some(version.id);
                }
            }
            self.editor_state.dirty = false;
        }
    }

    fn send_request(&mut self) {
        self.save_version();

        let Some(req) = &self.current_request else { return };
        let Some(vid) = &self.selected_version_id else { return };

        let data = self.editor_state.data.clone();
        let request_id = req.id.clone();
        let version_id = vid.clone();
        let tx = self.http_tx.clone();

        // Start with global environment variables
        let mut variables: HashMap<String, String> = HashMap::new();
        for v in &self.env_variables {
            variables.insert(v.key.clone(), v.value.clone());
        }

        // Overlay collection-scoped variables (override globals with same key)
        let collection_vars = self
            .db
            .get_active_collection_variables(&req.collection_id)
            .unwrap_or_default();
        for cv in &collection_vars {
            if !cv.key.is_empty() {
                variables.insert(cv.key.clone(), cv.value.clone());
            }
        }

        // Find collection and inject auth headers
        let collection = self.collections.iter().find(|c| c.id == req.collection_id);
        let base_path = collection.map(|c| c.base_path.clone()).unwrap_or_default();
        let auth_config: Option<CollectionAuthConfig> = collection
            .and_then(|c| c.auth_config.as_ref())
            .and_then(|s| serde_json::from_str(s).ok());

        self.is_loading = true;

        self.tokio_rt.spawn(async move {
            let result =
                execute_with_auth(data, &variables, &base_path, auth_config.as_ref()).await;
            let _ = tx.send(HttpResult {
                request_id,
                version_id,
                result,
            });
        });
    }

    fn process_http_results(&mut self) {
        while let Ok(result) = self.http_rx.try_recv() {
            self.is_loading = false;
            match result.result {
                Ok((response, latency_ms)) => {
                    let now = chrono::Utc::now().to_rfc3339();
                    let execution = RequestExecution {
                        id: uuid::Uuid::new_v4().to_string(),
                        version_id: result.version_id,
                        request_id: result.request_id.clone(),
                        response,
                        latency_ms,
                        executed_at: now,
                    };

                    let _ = self.db.insert_execution(&execution);
                    self.current_execution = Some(execution.clone());
                    self.selected_execution_id = Some(execution.id.clone());
                    self.response_state = ResponseViewState::default();

                    self.executions = self
                        .db
                        .list_executions_by_request(&result.request_id)
                        .unwrap_or_default();

                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(e);
                }
            }
        }
    }

    fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), std::time::Instant::now()));
    }
}

impl eframe::App for LiteRequestApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.process_http_results();

        if self.is_loading {
            ui.ctx().request_repaint();
        }

        // Clear status message after 3s
        if let Some((_, time)) = &self.status_message {
            if time.elapsed() > std::time::Duration::from_secs(3) {
                self.status_message = None;
            }
        }

        // ── Top bar ──────────────────────────────────────────────
        egui::Panel::top("top_bar")
            .frame(
                egui::Frame::default()
                    .fill(super::theme::SURFACE_1)
                    .stroke(egui::Stroke::new(1.0, super::theme::BORDER))
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("LiteRequest")
                            .strong()
                            .size(18.0)
                            .color(super::theme::ACCENT),
                    );

                    ui.separator();

                    // Environment selector
                    let env_action =
                        render_env_selector(ui, &self.environments, &mut self.env_panel_state);
                    self.handle_env_action(env_action);

                    ui.separator();

                    // Import / Export buttons
                    if ui.button("Import").clicked() {
                        if let Some(path) = rfd_open_file("Import .lreq", &["lreq", "json"]) {
                            match super::import_export::import_from_file(&self.db, &path) {
                                Ok(_id) => {
                                    self.set_status("Collection imported successfully");
                                    self.refresh_all_data();
                                }
                                Err(e) => self.error_message = Some(format!("Import failed: {e}")),
                            }
                        }
                    }

                    if let CenterView::CollectionConfig(ref cid) = self.center_view {
                        let cid = cid.clone();
                        if ui.button("Export").clicked() {
                            if let Some(path) = rfd_save_file("Export .lreq", "collection.lreq") {
                                match super::import_export::export_to_file(&self.db, &cid, &path) {
                                    Ok(()) => self.set_status("Collection exported successfully"),
                                    Err(e) => {
                                        self.error_message = Some(format!("Export failed: {e}"))
                                    }
                                }
                            }
                        }
                    }

                    // Right side: loading / status / error
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(err) = &self.error_message {
                            ui.label(
                                egui::RichText::new(format!("! {err}"))
                                    .color(egui::Color32::from_rgb(249, 62, 62))
                                    .size(12.0),
                            );
                        } else if let Some((msg, _)) = &self.status_message {
                            ui.label(
                                egui::RichText::new(msg)
                                    .color(egui::Color32::from_rgb(73, 204, 144))
                                    .size(12.0),
                            );
                        }
                        if self.is_loading {
                            ui.spinner();
                            ui.label(
                                egui::RichText::new("Sending…")
                                    .size(12.0)
                                    .color(super::theme::TEXT_SECONDARY),
                            );
                        }
                    });
                });
            });

        // ── Left panel: Collection tree ──────────────────────────
        egui::Panel::left("collection_panel")
            .default_size(240.0)
            .min_size(200.0)
            .frame(
                egui::Frame::default()
                    .fill(super::theme::SURFACE_1)
                    .stroke(egui::Stroke::new(1.0, super::theme::BORDER))
                    .inner_margin(egui::Margin::symmetric(8, 6)),
            )
            .show_inside(ui, |ui| {
                let method_map = build_method_map(&self.requests, &self.db);
                let action = render_collection_tree(
                    ui,
                    &self.collections,
                    &self.folders,
                    &self.requests,
                    &mut self.tree_state,
                    &method_map,
                );
                self.handle_tree_action(action);
            });

        // ── Right panel: History (only when editing a request) ───
        if matches!(self.center_view, CenterView::RequestEditor(_)) {
            egui::Panel::right("history_panel")
                .default_size(260.0)
                .min_size(220.0)
                .frame(
                    egui::Frame::default()
                        .fill(super::theme::SURFACE_1)
                        .stroke(egui::Stroke::new(1.0, super::theme::BORDER))
                        .inner_margin(egui::Margin::symmetric(8, 6)),
                )
                .show_inside(ui, |ui| {
                    let available_height = ui.available_height();

                    // Top half: Version history
                    ui.allocate_ui(
                        egui::vec2(ui.available_width(), available_height * 0.5),
                        |ui| {
                            if let Some(vid) = history_panel::render_version_history(
                                ui,
                                &self.versions,
                                self.selected_version_id.as_deref(),
                            ) {
                                self.selected_version_id = Some(vid.clone());
                                if let Ok(version) = self.db.get_version(&vid) {
                                    self.editor_state.data = version.data;
                                    self.editor_state.dirty = false;
                                }
                            }
                        },
                    );

                    ui.separator();

                    // Bottom half: Execution history
                    if let Some(eid) = history_panel::render_execution_history(
                        ui,
                        &self.executions,
                        self.selected_execution_id.as_deref(),
                    ) {
                        self.selected_execution_id = Some(eid.clone());
                        self.current_execution =
                            self.executions.iter().find(|e| e.id == eid).cloned();
                        self.response_state = ResponseViewState::default();
                    }
                });
        }

        // ── Center panel ─────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(super::theme::SURFACE_0)
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show_inside(ui, |ui| {
                match self.center_view.clone() {
                    CenterView::Welcome => {
                        render_welcome(ui);
                    }
                    CenterView::CollectionConfig(cid) => {
                        if let Some(collection) =
                            self.collections.iter().find(|c| c.id == cid).cloned()
                        {
                            let config_action = render_collection_config(
                                ui,
                                &mut self.collection_config_state,
                                &collection,
                                &self.environments,
                            );
                            self.handle_config_action(config_action, &cid);
                        }
                    }
                    CenterView::RequestEditor(_) => {
                        let req = self.current_request.clone();
                        if let Some(req) = req {
                            // Look up collection base path
                            let base_path = self
                                .collections
                                .iter()
                                .find(|c| c.id == req.collection_id)
                                .map(|c| c.base_path.clone())
                                .unwrap_or_default();

                            let available_height = ui.available_height();
                            let editor_height = available_height * self.split_ratio;

                            // Top: request editor
                            ui.allocate_ui(
                                egui::vec2(ui.available_width(), editor_height),
                                |ui| {
                                    let action = render_request_editor(
                                        ui,
                                        &mut self.editor_state,
                                        &req.name,
                                        &base_path,
                                    );
                                    match action {
                                        EditorAction::Send => self.send_request(),
                                        EditorAction::DataChanged | EditorAction::None => {}
                                    }
                                },
                            );

                            // Draggable split handle
                            let handle_rect = ui.allocate_space(egui::vec2(ui.available_width(), 4.0)).1;
                            let handle_resp = ui.interact(handle_rect, ui.id().with("split_handle"), egui::Sense::drag());
                            if handle_resp.dragged() {
                                let delta = handle_resp.drag_delta().y;
                                self.split_ratio = (self.split_ratio + delta / available_height).clamp(0.15, 0.85);
                                self.is_dragging_split = true;
                            }
                            if handle_resp.drag_stopped() {
                                self.is_dragging_split = false;
                            }
                            if handle_resp.hovered() || self.is_dragging_split {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                            }

                            // Bottom: response view (status strip is the divider)
                            render_response_view(
                                ui,
                                self.current_execution.as_ref(),
                                &mut self.response_state,
                            );
                        }
                    }
                }
            });

        // ── Delete collection confirmation modal ─────────────────
        if let Some(ref cid) = self.pending_delete_collection.clone() {
            let cname = self.collections.iter()
                .find(|c| &c.id == cid)
                .map(|c| c.name.as_str())
                .unwrap_or("this collection");
            let mut confirmed = false;
            let mut cancelled = false;
            egui::Window::new("Delete Collection")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label(format!("Delete \"{cname}\" and all its requests?"));
                    ui.label(
                        egui::RichText::new("This cannot be undone.")
                            .color(egui::Color32::from_rgb(249, 62, 62))
                            .size(12.0),
                    );
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if super::theme::pill_button(ui, "Delete", egui::Color32::from_rgb(249, 62, 62)) {
                            confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancelled = true;
                        }
                    });
                    ui.add_space(4.0);
                });
            if confirmed {
                let id = cid.clone();
                self.pending_delete_collection = None;
                let _ = self.db.delete_collection(&id);
                if matches!(&self.center_view, CenterView::CollectionConfig(ref cid2) if *cid2 == id) {
                    self.center_view = CenterView::Welcome;
                }
                if self.current_request.as_ref().map(|r| &r.collection_id) == Some(&id) {
                    self.current_request = None;
                    self.center_view = CenterView::Welcome;
                }
                self.tree_state.selected_request_id = None;
                self.tree_state.selected_collection_id = None;
                self.refresh_all_data();
            } else if cancelled {
                self.pending_delete_collection = None;
            }
        }

        // ── Environment management window ────────────────────────
        let mut env_show = self.env_panel_state.show_panel;
        if env_show {
            let mut env_action = EnvAction::None;
            egui::Window::new("Environments")
                .open(&mut env_show)
                .default_width(550.0)
                .show(ui.ctx(), |ui| {
                    env_action = render_environment_panel(
                        ui,
                        &self.environments,
                        &mut self.env_variables,
                        &mut self.env_panel_state,
                    );
                });
            self.env_panel_state.show_panel = env_show;
            self.handle_env_action(env_action);
        }
    }
}

// ── Action handlers ──────────────────────────────────────────────

impl LiteRequestApp {
    fn save_collection_config(&mut self, collection_id: &str) {
        if let Some(collection) = self.collections.iter().find(|c| c.id == *collection_id) {
            let now = chrono::Utc::now().to_rfc3339();
            let updated = Collection {
                id: collection.id.clone(),
                name: self.collection_config_state.name.clone(),
                base_path: self.collection_config_state.base_path.clone(),
                auth_config: self.collection_config_state.to_auth_json(),
                created_at: collection.created_at.clone(),
                updated_at: now,
            };
            let _ = self.db.update_collection(&updated);
            self.collection_config_state.dirty = false;
            self.set_status("Collection saved");
            self.refresh_all_data();
        }
    }

    fn handle_config_action(&mut self, action: ConfigAction, collection_id: &str) {
        match action {
            ConfigAction::None => {}
            ConfigAction::Save => {
                self.save_collection_config(collection_id);
            }
            ConfigAction::LoadVars(cid, env_id) => {
                self.collection_config_state.collection_vars = self
                    .db
                    .list_collection_variables(&cid, &env_id)
                    .unwrap_or_default();
                self.collection_config_state.vars_dirty = false;
            }
            ConfigAction::SaveVars => {
                for var in &self.collection_config_state.collection_vars {
                    let _ = self.db.update_collection_variable(var);
                }
                self.collection_config_state.vars_dirty = false;
                self.set_status("Variables saved");
            }
            ConfigAction::AddVar(cid, env_id) => {
                let var = CollectionVariable {
                    id: uuid::Uuid::new_v4().to_string(),
                    collection_id: cid,
                    environment_id: env_id,
                    key: String::new(),
                    value: String::new(),
                    is_secret: false,
                };
                let _ = self.db.insert_collection_variable(&var);
                self.collection_config_state.collection_vars.push(var);
            }
            ConfigAction::DeleteVar(var_id) => {
                let _ = self.db.delete_collection_variable(&var_id);
                self.collection_config_state.vars_dirty = false;
            }
        }
    }

    fn handle_tree_action(&mut self, action: TreeAction) {
        match action {
            TreeAction::None => {}
            TreeAction::SelectCollection(id) => {
                self.select_collection(&id);
            }
            TreeAction::SelectRequest(id) => {
                self.select_request(&id);
            }
            TreeAction::NewCollection => {
                let now = chrono::Utc::now().to_rfc3339();
                let collection = Collection {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "New Collection".to_string(),
                    base_path: String::new(),
                    auth_config: None,
                    created_at: now.clone(),
                    updated_at: now,
                };
                let cid = collection.id.clone();
                let _ = self.db.insert_collection(&collection);
                self.refresh_all_data();
                self.select_collection(&cid);
            }
            TreeAction::NewFolder(collection_id) => {
                let folder = Folder {
                    id: uuid::Uuid::new_v4().to_string(),
                    collection_id,
                    parent_folder_id: None,
                    name: "New Folder".to_string(),
                    path_prefix: String::new(),
                    auth_override: None,
                    sort_order: self.folders.len() as i32,
                };
                let _ = self.db.insert_folder(&folder);
                self.refresh_all_data();
            }
            TreeAction::NewRequest(collection_id, folder_id) => {
                let req = Request {
                    id: uuid::Uuid::new_v4().to_string(),
                    collection_id,
                    folder_id,
                    name: "New Request".to_string(),
                    current_version_id: None,
                    sort_order: self.requests.len() as i32,
                };
                let req_id = req.id.clone();
                let _ = self.db.insert_request(&req);
                self.refresh_all_data();
                self.select_request(&req_id);
            }
            TreeAction::DeleteCollection(id) => {
                // Show confirmation modal instead of deleting immediately
                self.pending_delete_collection = Some(id);
            }
            TreeAction::DeleteFolder(id) => {
                let _ = self.db.delete_folder(&id);
                self.refresh_all_data();
            }
            TreeAction::DeleteRequest(id) => {
                let _ = self.db.delete_request(&id);
                if self.current_request.as_ref().map(|r| &r.id) == Some(&id) {
                    self.current_request = None;
                    self.center_view = CenterView::Welcome;
                    self.tree_state.selected_request_id = None;
                }
                self.refresh_all_data();
            }
            TreeAction::RenameRequest(id, name) => {
                let _ = self.db.rename_request(&id, &name);
                self.refresh_all_data();
                if let Some(req) = &mut self.current_request {
                    if req.id == id {
                        req.name = name;
                    }
                }
            }
            TreeAction::RenameFolder(id, name) => {
                let _ = self.db.rename_folder(&id, &name);
                self.refresh_all_data();
            }
            TreeAction::RenameCollection(id, name) => {
                let _ = self.db.rename_collection(&id, &name);
                self.refresh_all_data();
            }
            TreeAction::CloneRequest(id) => {
                if let Some(src) = self.requests.iter().find(|r| r.id == id).cloned() {
                    let new_req = Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        collection_id: src.collection_id.clone(),
                        folder_id: src.folder_id.clone(),
                        name: format!("{} (copy)", src.name),
                        current_version_id: None,
                        sort_order: self.requests.len() as i32,
                    };
                    let new_req_id = new_req.id.clone();
                    let _ = self.db.insert_request(&new_req);
                    // Copy the latest version data if available
                    if let Some(vid) = &src.current_version_id {
                        if let Ok(ver) = self.db.get_version(vid) {
                            let now = chrono::Utc::now().to_rfc3339();
                            let new_ver = RequestVersion {
                                id: uuid::Uuid::new_v4().to_string(),
                                request_id: new_req_id.clone(),
                                data: ver.data,
                                created_at: now,
                            };
                            let _ = self.db.insert_version(&new_ver);
                        }
                    }
                    self.refresh_all_data();
                    self.select_request(&new_req_id);
                }
            }
            TreeAction::MoveRequest(request_id, collection_id, folder_id) => {
                let _ = self.db.move_request(&request_id, &collection_id, folder_id.as_deref());
                self.refresh_all_data();
            }
        }
    }

    fn handle_env_action(&mut self, action: EnvAction) {
        match action {
            EnvAction::None => {}
            EnvAction::NewEnvironment => {
                let now = chrono::Utc::now().to_rfc3339();
                let env = Environment {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: self.env_panel_state.new_env_name.clone(),
                    is_active: false,
                    created_at: now,
                };
                let _ = self.db.insert_environment(&env);
                self.env_panel_state.new_env_name.clear();
                self.refresh_all_data();
            }
            EnvAction::SelectEnvironment(id) => {
                let _ = self.db.set_active_environment(&id);
                self.refresh_all_data();
            }
            EnvAction::DeleteEnvironment(id) => {
                let _ = self.db.delete_environment(&id);
                self.refresh_all_data();
            }
            EnvAction::AddVariable(env_id) => {
                let var = EnvVariable {
                    id: uuid::Uuid::new_v4().to_string(),
                    environment_id: env_id,
                    key: String::new(),
                    value: String::new(),
                    is_secret: false,
                };
                let _ = self.db.insert_env_variable(&var);
                self.refresh_all_data();
            }
            EnvAction::UpdateVariable(var) => {
                let _ = self.db.update_env_variable(&var);
                self.refresh_all_data();
            }
            EnvAction::DeleteVariable(id) => {
                let _ = self.db.delete_env_variable(&id);
                self.refresh_all_data();
            }
        }
    }
}

// ── Welcome screen ───────────────────────────────────────────────

fn render_welcome(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        ui.label(
            egui::RichText::new("~")
                .size(48.0)
                .color(super::theme::ACCENT),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("LiteRequest")
                .size(36.0)
                .strong()
                .color(super::theme::TEXT_PRIMARY),
        );
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new("Lightweight & Offline API Client")
                .size(16.0)
                .color(super::theme::TEXT_SECONDARY),
        );
        ui.add_space(24.0);
        super::theme::framed_section(ui, |ui| {
            ui.set_max_width(400.0);
            ui.label(
                egui::RichText::new("Getting Started")
                    .strong()
                    .size(15.0)
                    .color(super::theme::TEXT_PRIMARY),
            );
            ui.add_space(6.0);
            for tip in [
                "• Click a collection name to configure its base path & auth",
                "• Click [+r] to create a new request",
                "• Double-click a request name to rename it",
                "• Use {{variables}} in URLs, resolved from the active environment",
                "• Import/Export collections via the top bar",
            ] {
                ui.label(
                    egui::RichText::new(tip)
                        .size(13.0)
                        .color(super::theme::TEXT_SECONDARY),
                );
            }
        });
    });
}

// ── Auth injection helper ────────────────────────────────────────

/// Same shape as CollectionConfig in collection_config.rs but renamed to
/// avoid confusion — used only for deserialization on the send path.
#[derive(serde::Deserialize)]
struct CollectionAuthConfig {
    auth_type: String,
    bearer_token: Option<String>,
    basic_username: Option<String>,
    basic_password: Option<String>,
    api_key_header: Option<String>,
    api_key_value: Option<String>,
}

async fn execute_with_auth(
    mut data: RequestData,
    variables: &HashMap<String, String>,
    base_path: &str,
    auth: Option<&CollectionAuthConfig>,
) -> Result<(ResponseData, u64), String> {
    // Inject auth headers from collection config
    if let Some(auth) = auth {
        match auth.auth_type.as_str() {
            "bearer" => {
                if let Some(token) = &auth.bearer_token {
                    let token = crate::http::interpolation::interpolate(token, variables);
                    data.headers.push(KeyValuePair {
                        key: "Authorization".to_string(),
                        value: format!("Bearer {token}"),
                        enabled: true,
                    });
                }
            }
            "basic" => {
                let user = auth
                    .basic_username
                    .as_deref()
                    .unwrap_or_default();
                let pass = auth
                    .basic_password
                    .as_deref()
                    .unwrap_or_default();
                let user = crate::http::interpolation::interpolate(user, variables);
                let pass = crate::http::interpolation::interpolate(pass, variables);
                let encoded = base64_encode(&format!("{user}:{pass}"));
                data.headers.push(KeyValuePair {
                    key: "Authorization".to_string(),
                    value: format!("Basic {encoded}"),
                    enabled: true,
                });
            }
            "api_key" => {
                if let (Some(header), Some(value)) = (&auth.api_key_header, &auth.api_key_value) {
                    let header = crate::http::interpolation::interpolate(header, variables);
                    let value = crate::http::interpolation::interpolate(value, variables);
                    data.headers.push(KeyValuePair {
                        key: header,
                        value,
                        enabled: true,
                    });
                }
            }
            _ => {}
        }
    }

    crate::http::client::execute_request(&data, variables, base_path).await
}

fn base64_encode(input: &str) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = base64_writer(&mut buf);
        let _ = encoder.write_all(input.as_bytes());
    }
    String::from_utf8(buf).unwrap_or_default()
}

/// Minimal base64 encoder (no extra dependency)
fn base64_writer(out: &mut Vec<u8>) -> Base64Writer<'_> {
    Base64Writer { out, buf: [0; 3], len: 0 }
}

struct Base64Writer<'a> {
    out: &'a mut Vec<u8>,
    buf: [u8; 3],
    len: usize,
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

impl<'a> std::io::Write for Base64Writer<'a> {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        for &b in data {
            self.buf[self.len] = b;
            self.len += 1;
            if self.len == 3 {
                self.out.push(B64[(self.buf[0] >> 2) as usize]);
                self.out.push(B64[(((self.buf[0] & 3) << 4) | (self.buf[1] >> 4)) as usize]);
                self.out.push(B64[(((self.buf[1] & 0xf) << 2) | (self.buf[2] >> 6)) as usize]);
                self.out.push(B64[(self.buf[2] & 0x3f) as usize]);
                self.len = 0;
            }
        }
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.len == 1 {
            self.out.push(B64[(self.buf[0] >> 2) as usize]);
            self.out.push(B64[((self.buf[0] & 3) << 4) as usize]);
            self.out.push(b'=');
            self.out.push(b'=');
        } else if self.len == 2 {
            self.out.push(B64[(self.buf[0] >> 2) as usize]);
            self.out.push(B64[(((self.buf[0] & 3) << 4) | (self.buf[1] >> 4)) as usize]);
            self.out.push(B64[((self.buf[1] & 0xf) << 2) as usize]);
            self.out.push(b'=');
        }
        self.len = 0;
        Ok(())
    }
}

impl<'a> Drop for Base64Writer<'a> {
    fn drop(&mut self) {
        let _ = std::io::Write::flush(self);
    }
}

// ── File dialogs (simple stdin/stdout fallback) ──────────────────

fn rfd_open_file(_title: &str, _extensions: &[&str]) -> Option<PathBuf> {
    // Try native file dialog via rfd if available, else use simple CLI prompt
    #[cfg(feature = "rfd")]
    {
        rfd::FileDialog::new()
            .set_title(_title)
            .add_filter("LiteRequest", _extensions)
            .pick_file()
    }
    #[cfg(not(feature = "rfd"))]
    {
        // Fallback: use environment variable or hardcoded path for now
        // In a real app, you'd use a text input dialog in the UI
        None
    }
}

fn rfd_save_file(_title: &str, _default_name: &str) -> Option<PathBuf> {
    #[cfg(feature = "rfd")]
    {
        rfd::FileDialog::new()
            .set_title(_title)
            .set_file_name(_default_name)
            .save_file()
    }
    #[cfg(not(feature = "rfd"))]
    {
        None
    }
}

// ── Build method map for tree badges ─────────────────────────────

fn build_method_map(requests: &[Request], db: &crate::db::Database) -> HashMap<String, HttpMethod> {
    let mut map = HashMap::new();
    for req in requests {
        if let Some(vid) = &req.current_version_id {
            if let Ok(ver) = db.get_version(vid) {
                map.insert(req.id.clone(), ver.data.method.clone());
            }
        }
    }
    map
}

// ── Data directory ───────────────────────────────────────────────

fn dirs_data_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("lite_request");
    std::fs::create_dir_all(&path).ok();
    path.push("lite_request.db");
    path
}

mod dirs {
    use std::path::PathBuf;

    pub fn data_local_dir() -> Option<PathBuf> {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".local/share"))
            })
    }
}
