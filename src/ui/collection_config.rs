use eframe::egui;
use crate::models::*;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthType {
    None,
    Bearer,
    Basic,
    ApiKey,
}

impl AuthType {
    pub fn as_str(&self) -> &str {
        match self {
            AuthType::None => "No Auth",
            AuthType::Bearer => "Bearer Token",
            AuthType::Basic => "Basic Auth",
            AuthType::ApiKey => "API Key",
        }
    }

    pub fn all() -> &'static [AuthType] {
        &[AuthType::None, AuthType::Bearer, AuthType::Basic, AuthType::ApiKey]
    }
}

/// Parsed auth_config JSON
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthConfig {
    pub auth_type: String,
    pub bearer_token: Option<String>,
    pub basic_username: Option<String>,
    pub basic_password: Option<String>,
    pub api_key_header: Option<String>,
    pub api_key_value: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auth_type: "none".to_string(),
            bearer_token: None,
            basic_username: None,
            basic_password: None,
            api_key_header: None,
            api_key_value: None,
        }
    }
}

impl AuthConfig {
    pub fn get_auth_type(&self) -> AuthType {
        match self.auth_type.as_str() {
            "bearer" => AuthType::Bearer,
            "basic" => AuthType::Basic,
            "api_key" => AuthType::ApiKey,
            _ => AuthType::None,
        }
    }

    pub fn set_auth_type(&mut self, t: &AuthType) {
        self.auth_type = match t {
            AuthType::None => "none",
            AuthType::Bearer => "bearer",
            AuthType::Basic => "basic",
            AuthType::ApiKey => "api_key",
        }
        .to_string();
    }
}

pub struct CollectionConfigState {
    pub name: String,
    pub base_path: String,
    pub auth: AuthConfig,
    pub dirty: bool,

    // Collection-level headers
    pub headers: Vec<KeyValuePair>,

    // Variable definitions + values for current environment
    pub selected_env_id: Option<String>,
    pub var_rows: Vec<VarRow>,
    pub vars_dirty: bool,

    // Section expansion state
    pub show_general: bool,
    pub show_auth: bool,
    pub show_headers: bool,
    pub show_vars: bool,
}

impl Default for CollectionConfigState {
    fn default() -> Self {
        Self {
            name: String::new(),
            base_path: String::new(),
            auth: AuthConfig::default(),
            dirty: false,
            headers: Vec::new(),
            selected_env_id: None,
            var_rows: Vec::new(),
            vars_dirty: false,
            show_general: true,
            show_auth: true,
            show_headers: true,
            show_vars: true,
        }
    }
}

impl CollectionConfigState {
    pub fn load_from(&mut self, collection: &Collection) {
        self.name = collection.name.clone();
        self.base_path = collection.base_path.clone();
        self.auth = collection
            .auth_config
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        self.headers = collection
            .headers_config
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        self.dirty = false;
        self.selected_env_id = None;
        self.var_rows.clear();
        self.vars_dirty = false;
        // Keep section expansion state across loads
    }

    pub fn to_auth_json(&self) -> Option<String> {
        serde_json::to_string(&self.auth).ok()
    }

    pub fn to_headers_json(&self) -> Option<String> {
        // Only serialize non-empty headers
        let non_empty: Vec<&KeyValuePair> = self
            .headers
            .iter()
            .filter(|h| !h.key.is_empty() || !h.value.is_empty())
            .collect();
        if non_empty.is_empty() {
            None
        } else {
            serde_json::to_string(&non_empty).ok()
        }
    }
}

pub enum ConfigAction {
    None,
    Save,
    /// Load variable rows for this (collection_id, environment_id)
    LoadVars(String, String),
    /// Persist current var_rows to the DB
    SaveVars,
    /// Delete a variable definition (cascades to all env values)
    DeleteVarDef(String),
}

pub fn render_collection_config(
    ui: &mut egui::Ui,
    state: &mut CollectionConfigState,
    collection: &Collection,
    environments: &[Environment],
) -> ConfigAction {
    let mut action = ConfigAction::None;

    // Auto-save: trigger save actions when dirty
    if state.dirty {
        action = ConfigAction::Save;
    }
    if state.vars_dirty {
        action = ConfigAction::SaveVars;
    }

    // Title bar
    egui::Frame::default()
        .fill(super::theme::SURFACE_2)
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(egui_phosphor::regular::GEAR_SIX)
                        .size(14.0)
                        .color(super::theme::ACCENT),
                );
                ui.label(
                    egui::RichText::new("Collection Settings")
                        .strong()
                        .size(14.0)
                        .color(super::theme::TEXT_PRIMARY),
                );
            });
        });
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // ── GENERAL section ──
            if super::theme::collapsible_header(ui, "GENERAL", &mut state.show_general) {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Name")
                                .size(11.0)
                                .strong()
                                .color(super::theme::TEXT_MUTED),
                        );
                        ui.add_space(2.0);
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut state.name)
                                    .desired_width(f32::INFINITY)
                                    .min_size(egui::vec2(0.0, super::theme::INPUT_HEIGHT))
                                    .margin(egui::Margin::symmetric(8, 6))
                                    .hint_text("Collection name"),
                            )
                            .changed()
                        {
                            state.dirty = true;
                        }

                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new("Base Path")
                                .size(11.0)
                                .strong()
                                .color(super::theme::TEXT_MUTED),
                        );
                        ui.add_space(2.0);
                        {
                            let mut layouter = super::var_highlight::var_text_layouter;
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut state.base_path)
                                        .desired_width(f32::INFINITY)
                                        .min_size(egui::vec2(0.0, super::theme::INPUT_HEIGHT))
                                        .margin(egui::Margin::symmetric(8, 6))
                                        .font(egui::TextStyle::Monospace)
                                        .hint_text("https://{{host}}/v1/{{instance_id}}")
                                        .layouter(&mut layouter),
                                )
                                .changed()
                            {
                                state.dirty = true;
                            }
                        }
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{} Supports {{{{variables}}}} — define them per environment below.",
                                egui_phosphor::regular::INFO,
                            ))
                            .size(11.0)
                            .color(super::theme::TEXT_MUTED),
                        );
                    });
                ui.add_space(4.0);
            }

            // ── AUTHENTICATION section ──
            if super::theme::collapsible_header(ui, "AUTHENTICATION", &mut state.show_auth) {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        // Auth type selector
                        ui.label(
                            egui::RichText::new("Type")
                                .size(11.0)
                                .strong()
                                .color(super::theme::TEXT_MUTED),
                        );
                        ui.add_space(2.0);
                        let current_type = state.auth.get_auth_type();
                        egui::ComboBox::from_id_salt("auth_type_selector")
                            .selected_text(current_type.as_str())
                            .width(ui.available_width().min(250.0))
                            .show_ui(ui, |ui| {
                                for t in AuthType::all() {
                                    if ui
                                        .selectable_label(current_type == *t, t.as_str())
                                        .clicked()
                                    {
                                        state.auth.set_auth_type(t);
                                        state.dirty = true;
                                    }
                                }
                            });

                        ui.add_space(8.0);

                        let auth_type = state.auth.get_auth_type();
                        match auth_type {
                            AuthType::None => {
                                ui.label(
                                    egui::RichText::new("No authentication configured.")
                                        .color(super::theme::TEXT_MUTED)
                                        .italics(),
                                );
                            }
                            AuthType::Bearer => {
                                render_auth_field(
                                    ui,
                                    "Token",
                                    &mut state.auth.bearer_token,
                                    "Bearer token or {{variable}}",
                                    true,
                                    &mut state.dirty,
                                );
                            }
                            AuthType::Basic => {
                                render_auth_field(
                                    ui,
                                    "Username",
                                    &mut state.auth.basic_username,
                                    "",
                                    false,
                                    &mut state.dirty,
                                );
                                ui.add_space(6.0);
                                render_auth_field(
                                    ui,
                                    "Password",
                                    &mut state.auth.basic_password,
                                    "",
                                    true,
                                    &mut state.dirty,
                                );
                            }
                            AuthType::ApiKey => {
                                render_auth_field(
                                    ui,
                                    "Header Name",
                                    &mut state.auth.api_key_header,
                                    "X-API-Key",
                                    false,
                                    &mut state.dirty,
                                );
                                ui.add_space(6.0);
                                render_auth_field(
                                    ui,
                                    "Value",
                                    &mut state.auth.api_key_value,
                                    "{{api_key}}",
                                    true,
                                    &mut state.dirty,
                                );
                            }
                        }
                    });
                ui.add_space(4.0);
            }

            // ── HEADERS section ──
            if super::theme::collapsible_header(ui, "HEADERS", &mut state.show_headers) {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Auto-included in every request in this collection.")
                                .size(11.0)
                                .color(super::theme::TEXT_MUTED),
                        );
                        ui.add_space(4.0);
                        if render_collection_headers_table(ui, &mut state.headers) {
                            state.dirty = true;
                        }
                    });
                ui.add_space(4.0);
            }

            // ── COLLECTION VARIABLES section ──
            if super::theme::collapsible_header(ui, "COLLECTION VARIABLES", &mut state.show_vars) {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        if environments.is_empty() {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} Create an environment first to define collection variables.",
                                    egui_phosphor::regular::WARNING,
                                ))
                                .color(super::theme::TEXT_MUTED)
                                .italics(),
                            );
                        } else {
                            render_variables_section(
                                ui,
                                state,
                                collection,
                                environments,
                                &mut action,
                            );
                        }
                    });
                ui.add_space(4.0);
            }

            // Footer info
            ui.add_space(12.0);
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(12, 0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("ID: {}", collection.id))
                            .size(11.0)
                            .color(super::theme::TEXT_MUTED),
                    );
                    ui.label(
                        egui::RichText::new(format!("Created: {}", collection.created_at))
                            .size(11.0)
                            .color(super::theme::TEXT_MUTED),
                    );
                });
        });

    action
}

/// Render a labeled auth field (full-width, with variable highlighting).
fn render_auth_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut Option<String>,
    hint: &str,
    password: bool,
    dirty: &mut bool,
) {
    ui.label(
        egui::RichText::new(label)
            .size(11.0)
            .strong()
            .color(super::theme::TEXT_MUTED),
    );
    ui.add_space(2.0);
    let mut text = value.clone().unwrap_or_default();
    let mut layouter = super::var_highlight::var_text_layouter;
    let mut edit = egui::TextEdit::singleline(&mut text)
        .desired_width(f32::INFINITY)
        .min_size(egui::vec2(0.0, super::theme::INPUT_HEIGHT))
        .margin(egui::Margin::symmetric(8, 6))
        .font(egui::TextStyle::Monospace)
        .layouter(&mut layouter);
    if !hint.is_empty() {
        edit = edit.hint_text(hint);
    }
    if password {
        edit = edit.password(true);
    }
    if ui.add(edit).changed() {
        *value = Some(text);
        *dirty = true;
    }
}

/// Render the variables sub-section with environment tabs and auto-grow KV table.
fn render_variables_section(
    ui: &mut egui::Ui,
    state: &mut CollectionConfigState,
    collection: &Collection,
    environments: &[Environment],
    action: &mut ConfigAction,
) {
    // Environment tab bar
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "{} Environment:",
                egui_phosphor::regular::GLOBE_HEMISPHERE_WEST
            ))
            .size(12.0)
            .color(super::theme::TEXT_SECONDARY),
        );

        for env in environments {
            let is_selected = state.selected_env_id.as_deref() == Some(&env.id);
            let label = if env.is_active {
                format!("{} (active)", env.name)
            } else {
                env.name.clone()
            };

            let btn = egui::Button::new(
                egui::RichText::new(&label).size(12.0).color(
                    if is_selected {
                        egui::Color32::WHITE
                    } else {
                        super::theme::TEXT_SECONDARY
                    },
                ),
            )
            .fill(if is_selected {
                super::theme::ACCENT.gamma_multiply(0.3)
            } else {
                egui::Color32::TRANSPARENT
            })
            .stroke(if is_selected {
                egui::Stroke::new(1.0, super::theme::ACCENT)
            } else {
                egui::Stroke::new(1.0, super::theme::SURFACE_2)
            })
            .corner_radius(egui::CornerRadius::same(4));

            if ui.add(btn).clicked() && !is_selected {
                state.selected_env_id = Some(env.id.clone());
                *action =
                    ConfigAction::LoadVars(collection.id.clone(), env.id.clone());
            }
        }
    });

    ui.add_space(6.0);

    // Auto-select first environment if none selected
    if state.selected_env_id.is_none() {
        let auto_env = environments
            .iter()
            .find(|e| e.is_active)
            .or_else(|| environments.first());
        if let Some(env) = auto_env {
            state.selected_env_id = Some(env.id.clone());
            *action =
                ConfigAction::LoadVars(collection.id.clone(), env.id.clone());
        }
    }

    if state.selected_env_id.is_some() {
        // Auto-grow: ensure there's always one empty row at the end
        let needs_empty = state.var_rows.is_empty()
            || state
                .var_rows
                .last()
                .map_or(true, |v| !v.key.is_empty() || !v.value.is_empty());
        if needs_empty {
            state.var_rows.push(VarRow {
                def_id: uuid::Uuid::new_v4().to_string(),
                key: String::new(),
                value: String::new(),
                is_secret: false,
                value_id: None,
            });
        }

        render_vars_table(ui, state, action);
    }
}

/// Auto-grow KV table for collection variables using the split def/value model.
fn render_vars_table(
    ui: &mut egui::Ui,
    state: &mut CollectionConfigState,
    action: &mut ConfigAction,
) {
    use egui_extras::{Column, TableBuilder};

    let n_vars = state.var_rows.len();
    let row_h = 28.0;
    let input_fill = super::theme::SURFACE_0;
    let input_stroke = egui::Stroke::new(1.0, super::theme::BORDER);
    let mut to_delete: Option<usize> = None;

    TableBuilder::new(ui)
        .id_salt("collection_vars_table")
        .striped(false)
        .max_scroll_height(n_vars as f32 * row_h + 4.0)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(80.0)) // key
        .column(Column::remainder().at_least(80.0)) // value
        .column(Column::exact(24.0)) // secret checkbox
        .column(Column::exact(22.0)) // delete btn
        .body(|mut body| {
            for i in 0..n_vars {
                let is_last_empty = i == n_vars - 1
                    && state.var_rows[i].key.is_empty()
                    && state.var_rows[i].value.is_empty();

                body.row(row_h, |mut row| {
                    // Key (shared across all environments)
                    row.col(|ui| {
                        egui::Frame::NONE
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(
                                            &mut state.var_rows[i].key,
                                        )
                                        .desired_width(ui.available_width())
                                        .frame(egui::Frame::NONE)
                                        .font(egui::TextStyle::Monospace)
                                        .hint_text(if is_last_empty {
                                            "new key..."
                                        } else {
                                            ""
                                        }),
                                    )
                                    .changed()
                                {
                                    state.vars_dirty = true;
                                }
                            });
                    });
                    // Value (per-environment)
                    row.col(|ui| {
                        egui::Frame::NONE
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
                                let is_secret = state.var_rows[i].is_secret;
                                let mut layouter = super::var_highlight::var_text_layouter;
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(
                                            &mut state.var_rows[i].value,
                                        )
                                        .desired_width(ui.available_width())
                                        .frame(egui::Frame::NONE)
                                        .font(egui::TextStyle::Monospace)
                                        .password(is_secret)
                                        .layouter(&mut layouter)
                                        .hint_text(if is_last_empty {
                                            "value..."
                                        } else {
                                            ""
                                        }),
                                    )
                                    .changed()
                                {
                                    state.vars_dirty = true;
                                }
                            });
                    });
                    // Secret
                    row.col(|ui| {
                        if !is_last_empty {
                            if ui
                                .checkbox(&mut state.var_rows[i].is_secret, "")
                                .on_hover_text("Secret")
                                .changed()
                            {
                                state.vars_dirty = true;
                            }
                        }
                    });
                    // Delete
                    row.col(|ui| {
                        if !is_last_empty {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(egui_phosphor::regular::X)
                                            .size(11.0)
                                            .color(super::theme::TEXT_MUTED),
                                    )
                                    .frame(false)
                                    .min_size(egui::vec2(16.0, 16.0)),
                                )
                                .on_hover_text("Remove from all environments")
                                .clicked()
                            {
                                to_delete = Some(i);
                            }
                        }
                    });
                });
            }
        });

    if let Some(i) = to_delete {
        let def_id = state.var_rows[i].def_id.clone();
        state.var_rows.remove(i);
        *action = ConfigAction::DeleteVarDef(def_id);
    }
}

/// Auto-grow KV table for collection-level headers, matching inspector style.
/// Returns true if any change was made.
fn render_collection_headers_table(
    ui: &mut egui::Ui,
    headers: &mut Vec<KeyValuePair>,
) -> bool {
    use egui_extras::{Column, TableBuilder};

    // Ensure there's always one empty row at the end
    let needs_empty = headers.is_empty()
        || headers
            .last()
            .map_or(true, |h| !h.key.is_empty() || !h.value.is_empty());
    if needs_empty {
        headers.push(KeyValuePair::default());
    }

    let mut changed = false;
    let mut to_remove: Option<usize> = None;
    let n = headers.len();
    let row_h = 28.0;
    let input_fill = super::theme::SURFACE_0;
    let input_stroke = egui::Stroke::new(1.0, super::theme::BORDER);

    TableBuilder::new(ui)
        .id_salt("collection_headers_table")
        .striped(false)
        .max_scroll_height(n as f32 * row_h + 4.0)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(22.0))
        .column(Column::remainder().at_least(80.0))
        .column(Column::remainder().at_least(80.0))
        .column(Column::exact(22.0))
        .body(|mut body| {
            for i in 0..n {
                let is_last_empty = i == n - 1
                    && headers[i].key.is_empty()
                    && headers[i].value.is_empty();

                body.row(row_h, |mut row| {
                    row.col(|ui| {
                        if !is_last_empty {
                            if ui.checkbox(&mut headers[i].enabled, "").changed() {
                                changed = true;
                            }
                        }
                    });
                    row.col(|ui| {
                        egui::Frame::NONE
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
                                let mut layouter = super::var_highlight::var_text_layouter;
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut headers[i].key)
                                            .desired_width(ui.available_width())
                                            .frame(egui::Frame::NONE)
                                            .font(egui::TextStyle::Monospace)
                                            .hint_text(if is_last_empty { "Header-Name" } else { "" })
                                            .layouter(&mut layouter),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                    });
                    row.col(|ui| {
                        egui::Frame::NONE
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
                                let mut layouter = super::var_highlight::var_text_layouter;
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut headers[i].value)
                                            .desired_width(ui.available_width())
                                            .frame(egui::Frame::NONE)
                                            .font(egui::TextStyle::Monospace)
                                            .hint_text(if is_last_empty { "value or {{variable}}" } else { "" })
                                            .layouter(&mut layouter),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                    });
                    row.col(|ui| {
                        if !is_last_empty {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(egui_phosphor::regular::X)
                                            .size(11.0)
                                            .color(super::theme::TEXT_MUTED),
                                    )
                                    .frame(false)
                                    .min_size(egui::vec2(16.0, 16.0)),
                                )
                                .on_hover_text("Remove")
                                .clicked()
                            {
                                to_remove = Some(i);
                            }
                        }
                    });
                });
            }
        });

    if let Some(idx) = to_remove {
        headers.remove(idx);
        changed = true;
    }

    changed
}