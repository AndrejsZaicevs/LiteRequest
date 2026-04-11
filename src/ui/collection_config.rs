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

    // Matrix environment variables for this collection
    pub selected_env_id: Option<String>,
    pub collection_vars: Vec<CollectionVariable>,
    pub vars_dirty: bool,

    // Section expansion state
    pub show_general: bool,
    pub show_auth: bool,
    pub show_vars: bool,
}

impl Default for CollectionConfigState {
    fn default() -> Self {
        Self {
            name: String::new(),
            base_path: String::new(),
            auth: AuthConfig::default(),
            dirty: false,
            selected_env_id: None,
            collection_vars: Vec::new(),
            vars_dirty: false,
            show_general: true,
            show_auth: true,
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
        self.dirty = false;
        self.selected_env_id = None;
        self.collection_vars.clear();
        self.vars_dirty = false;
        // Keep section expansion state across loads
    }

    pub fn to_auth_json(&self) -> Option<String> {
        serde_json::to_string(&self.auth).ok()
    }
}

pub enum ConfigAction {
    None,
    Save,
    /// Load collection variables for this (collection_id, environment_id)
    LoadVars(String, String),
    /// Persist all current collection_vars to the DB
    SaveVars,
    #[allow(dead_code)]
    AddVar(String, String),                // collection_id, environment_id
    /// Add a variable key across ALL environments for a collection
    AddVarAllEnvs(String),                 // collection_id
    DeleteVar(String),                     // variable id
    /// Delete a variable by key across ALL environments
    DeleteVarByKey(String, String),        // collection_id, key
}

pub fn render_collection_config(
    ui: &mut egui::Ui,
    state: &mut CollectionConfigState,
    collection: &Collection,
    environments: &[Environment],
) -> ConfigAction {
    let mut action = ConfigAction::None;

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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if state.dirty || state.vars_dirty {
                        let label = if state.dirty && state.vars_dirty {
                            "Save All"
                        } else if state.dirty {
                            "Save"
                        } else {
                            "Save Variables"
                        };
                        if super::theme::pill_button(ui, label, super::theme::ACCENT) {
                            if state.dirty {
                                action = ConfigAction::Save;
                            }
                            if state.vars_dirty {
                                action = ConfigAction::SaveVars;
                            }
                        }
                        ui.label(
                            egui::RichText::new("●")
                                .size(10.0)
                                .color(egui::Color32::from_rgb(252, 161, 48)),
                        );
                    }
                });
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
                // Save pending edits before switching environments
                // (SaveVars will be returned from the parent function)
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
        let needs_empty = state.collection_vars.is_empty()
            || state
                .collection_vars
                .last()
                .map_or(true, |v| !v.key.is_empty() || !v.value.is_empty());
        if needs_empty {
            let env_id = state.selected_env_id.clone().unwrap_or_default();
            state.collection_vars.push(CollectionVariable {
                id: uuid::Uuid::new_v4().to_string(),
                collection_id: collection.id.clone(),
                environment_id: env_id,
                key: String::new(),
                value: String::new(),
                is_secret: false,
            });
        }

        render_vars_table(ui, state, collection, action);
    }
}

/// Auto-grow KV table for collection variables, matching inspector style.
fn render_vars_table(
    ui: &mut egui::Ui,
    state: &mut CollectionConfigState,
    collection: &Collection,
    action: &mut ConfigAction,
) {
    use egui_extras::{Column, TableBuilder};

    let n_vars = state.collection_vars.len();
    let row_h = 28.0;
    let input_fill = super::theme::SURFACE_0;
    let input_stroke = egui::Stroke::new(1.0, super::theme::BORDER);
    let mut to_delete: Option<usize> = None;
    let mut new_key_entered: Option<String> = None;

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
                    && state.collection_vars[i].key.is_empty()
                    && state.collection_vars[i].value.is_empty();

                body.row(row_h, |mut row| {
                    // Key
                    row.col(|ui| {
                        egui::Frame::NONE
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
                                let old_key = state.collection_vars[i].key.clone();
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(
                                            &mut state.collection_vars[i].key,
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
                                    // If the user typed a key in the empty row, create across all envs
                                    if is_last_empty
                                        && old_key.is_empty()
                                        && !state.collection_vars[i].key.is_empty()
                                    {
                                        new_key_entered =
                                            Some(state.collection_vars[i].key.clone());
                                    }
                                }
                            });
                    });
                    // Value
                    row.col(|ui| {
                        egui::Frame::NONE
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
                                let is_secret = state.collection_vars[i].is_secret;
                                let mut layouter = super::var_highlight::var_text_layouter;
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(
                                            &mut state.collection_vars[i].value,
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
                                .checkbox(&mut state.collection_vars[i].is_secret, "")
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

    // Handle new key entry: create this key in all OTHER environments
    if let Some(key) = new_key_entered {
        // The variable already exists in state for the current env.
        // Insert into DB for current env, then trigger creation in all other envs.
        let var = &state.collection_vars[state.collection_vars.len() - 1];
        let _ = *action; // don't overwrite a pending LoadVars
        *action = ConfigAction::AddVarAllEnvs(collection.id.clone());
        // We save the current var to DB immediately so it persists
        // (the AddVarAllEnvs handler will see the key and create for other envs)
        let _ = key; // key is already set in the var
        let _ = var;
    }

    // Handle deletion: remove from all environments
    if let Some(i) = to_delete {
        let key = state.collection_vars[i].key.clone();
        state.collection_vars.remove(i);
        if !key.is_empty() {
            *action = ConfigAction::DeleteVarByKey(collection.id.clone(), key);
        } else {
            let var_id = state
                .collection_vars
                .get(i)
                .map(|v| v.id.clone())
                .unwrap_or_default();
            if !var_id.is_empty() {
                *action = ConfigAction::DeleteVar(var_id);
            }
        }
    }
}
