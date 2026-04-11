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
    AddVar(String, String),                // collection_id, environment_id
    DeleteVar(String),                     // variable id
}

pub fn render_collection_config(
    ui: &mut egui::Ui,
    state: &mut CollectionConfigState,
    collection: &Collection,
    environments: &[Environment],
) -> ConfigAction {
    let mut action = ConfigAction::None;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);

            // Title
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(egui_phosphor::regular::GEAR_SIX)
                        .size(22.0)
                        .color(super::theme::ACCENT),
                );
                ui.label(
                    egui::RichText::new("Collection Settings")
                        .size(20.0)
                        .strong()
                        .color(super::theme::TEXT_PRIMARY),
                );
            });

            ui.add_space(12.0);

            // ── General section ──────────────────────────────────
            super::theme::framed_section(ui, |ui| {
                ui.label(
                    egui::RichText::new("GENERAL")
                        .size(11.0)
                        .strong()
                        .color(super::theme::TEXT_SECONDARY),
                );
                ui.add_space(6.0);

                egui::Grid::new("collection_general_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Name")
                                .strong()
                                .color(super::theme::TEXT_SECONDARY),
                        );
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut state.name)
                                    .desired_width(350.0)
                                    .hint_text("Collection name"),
                            )
                            .changed()
                        {
                            state.dirty = true;
                        }
                        ui.end_row();

                        ui.label(
                            egui::RichText::new("Base Path")
                                .strong()
                                .color(super::theme::TEXT_SECONDARY),
                        );
                        {
                            let mut layouter = super::var_highlight::var_text_layouter;
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut state.base_path)
                                        .desired_width(350.0)
                                        .font(egui::TextStyle::Monospace)
                                        .hint_text("https://{{host}}/v1/{{instance_id}}")
                                        .layouter(&mut layouter),
                                )
                                .changed()
                            {
                                state.dirty = true;
                            }
                        }
                        ui.end_row();
                    });

                // Show hint about variable support
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} Base path supports {{{{variables}}}} — define them per environment below.",
                        egui_phosphor::regular::INFO,
                    ))
                    .size(11.0)
                    .color(super::theme::TEXT_MUTED),
                );
            });

            ui.add_space(12.0);

            // ── Authentication section ───────────────────────────
            super::theme::framed_section(ui, |ui| {
                ui.label(
                    egui::RichText::new("AUTHENTICATION")
                        .size(11.0)
                        .strong()
                        .color(super::theme::TEXT_SECONDARY),
                );
                ui.add_space(6.0);

                ui.label(
                    egui::RichText::new("Requests in this collection will inherit these auth settings.")
                        .size(12.0)
                        .color(super::theme::TEXT_MUTED),
                );
                ui.add_space(8.0);

                // Auth type selector
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Type")
                            .strong()
                            .color(super::theme::TEXT_SECONDARY),
                    );
                    let current_type = state.auth.get_auth_type();
                    egui::ComboBox::from_id_salt("auth_type_selector")
                        .selected_text(current_type.as_str())
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            for t in AuthType::all() {
                                if ui.selectable_label(current_type == *t, t.as_str()).clicked() {
                                    state.auth.set_auth_type(t);
                                    state.dirty = true;
                                }
                            }
                        });
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
                        let mut token = state.auth.bearer_token.clone().unwrap_or_default();
                        ui.label(
                            egui::RichText::new("Token")
                                .strong()
                                .color(super::theme::TEXT_SECONDARY),
                        );
                        let mut layouter = super::var_highlight::var_text_layouter;
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut token)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text("Bearer token or {{variable}}")
                                    .password(true)
                                    .layouter(&mut layouter),
                            )
                            .changed()
                        {
                            state.auth.bearer_token = Some(token);
                            state.dirty = true;
                        }
                    }
                    AuthType::Basic => {
                        egui::Grid::new("basic_auth_grid")
                            .num_columns(2)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                let mut username =
                                    state.auth.basic_username.clone().unwrap_or_default();
                                ui.label(
                                    egui::RichText::new("Username")
                                        .strong()
                                        .color(super::theme::TEXT_SECONDARY),
                                );
                                {
                                    let mut layouter = super::var_highlight::var_text_layouter;
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut username)
                                                .desired_width(300.0)
                                                .font(egui::TextStyle::Monospace)
                                                .layouter(&mut layouter),
                                        )
                                        .changed()
                                    {
                                        state.auth.basic_username = Some(username);
                                        state.dirty = true;
                                    }
                                }
                                ui.end_row();

                                let mut password =
                                    state.auth.basic_password.clone().unwrap_or_default();
                                ui.label(
                                    egui::RichText::new("Password")
                                        .strong()
                                        .color(super::theme::TEXT_SECONDARY),
                                );
                                {
                                    let mut layouter = super::var_highlight::var_text_layouter;
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut password)
                                                .desired_width(300.0)
                                                .font(egui::TextStyle::Monospace)
                                                .password(true)
                                                .layouter(&mut layouter),
                                        )
                                        .changed()
                                    {
                                        state.auth.basic_password = Some(password);
                                        state.dirty = true;
                                    }
                                }
                                ui.end_row();
                            });
                    }
                    AuthType::ApiKey => {
                        egui::Grid::new("apikey_auth_grid")
                            .num_columns(2)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                let mut header_name =
                                    state.auth.api_key_header.clone().unwrap_or_default();
                                ui.label(
                                    egui::RichText::new("Header Name")
                                        .strong()
                                        .color(super::theme::TEXT_SECONDARY),
                                );
                                {
                                    let mut layouter = super::var_highlight::var_text_layouter;
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut header_name)
                                                .desired_width(300.0)
                                                .font(egui::TextStyle::Monospace)
                                                .hint_text("X-API-Key")
                                                .layouter(&mut layouter),
                                        )
                                        .changed()
                                    {
                                        state.auth.api_key_header = Some(header_name);
                                        state.dirty = true;
                                    }
                                }
                                ui.end_row();

                                let mut value =
                                    state.auth.api_key_value.clone().unwrap_or_default();
                                ui.label(
                                    egui::RichText::new("Value")
                                        .strong()
                                        .color(super::theme::TEXT_SECONDARY),
                                );
                                {
                                    let mut layouter = super::var_highlight::var_text_layouter;
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut value)
                                                .desired_width(300.0)
                                                .font(egui::TextStyle::Monospace)
                                                .password(true)
                                                .hint_text("{{api_key}}")
                                                .layouter(&mut layouter),
                                        )
                                        .changed()
                                    {
                                        state.auth.api_key_value = Some(value);
                                        state.dirty = true;
                                    }
                                }
                                ui.end_row();
                            });
                    }
                }
            });

            ui.add_space(12.0);

            // ── Collection Variables (Matrix model) ──────────────
            super::theme::framed_section(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("COLLECTION VARIABLES")
                            .size(11.0)
                            .strong()
                            .color(super::theme::TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("(per environment)")
                            .size(11.0)
                            .color(super::theme::TEXT_MUTED),
                    );
                });
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "Scoped to this collection. Override global environment variables with the same key.",
                    )
                    .size(12.0)
                    .color(super::theme::TEXT_MUTED),
                );
                ui.add_space(8.0);

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
                    // Environment tab bar
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{} Environment:", egui_phosphor::regular::GLOBE_HEMISPHERE_WEST))
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
                                // Save any pending var edits before switching
                                if state.vars_dirty {
                                    action = ConfigAction::SaveVars;
                                }
                                state.selected_env_id = Some(env.id.clone());
                                action = ConfigAction::LoadVars(
                                    collection.id.clone(),
                                    env.id.clone(),
                                );
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Auto-select first environment if none selected
                    if state.selected_env_id.is_none() {
                        // Prefer active environment
                        let auto_env = environments
                            .iter()
                            .find(|e| e.is_active)
                            .or_else(|| environments.first());
                        if let Some(env) = auto_env {
                            state.selected_env_id = Some(env.id.clone());
                            action = ConfigAction::LoadVars(
                                collection.id.clone(),
                                env.id.clone(),
                            );
                        }
                    }

                    if let Some(ref env_id) = state.selected_env_id.clone() {
                        let env_name = environments
                            .iter()
                            .find(|e| e.id == *env_id)
                            .map(|e| e.name.as_str())
                            .unwrap_or("Unknown");

                        // Variable table using egui_extras TableBuilder
                        if state.collection_vars.is_empty() {
                            ui.label(
                                egui::RichText::new(format!(
                                    "No variables defined for \"{}\" environment. Click + to add one.",
                                    env_name,
                                ))
                                .size(12.0)
                                .color(super::theme::TEXT_MUTED)
                                .italics(),
                            );
                        } else {
                            use egui_extras::{TableBuilder, Column};

                            let n_vars = state.collection_vars.len();
                            let row_h = 22.0;
                            let mut to_delete: Option<usize> = None;

                            TableBuilder::new(ui)
                                .id_salt("collection_vars_table")
                                .striped(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::remainder().at_least(80.0)) // key
                                .column(Column::remainder().at_least(80.0)) // value
                                .column(Column::exact(24.0))               // secret checkbox
                                .column(Column::exact(20.0))               // delete btn
                                .header(18.0, |mut header| {
                                    header.col(|ui| {
                                        ui.label(
                                            egui::RichText::new("Key")
                                                .size(11.0)
                                                .strong()
                                                .color(super::theme::TEXT_MUTED),
                                        );
                                    });
                                    header.col(|ui| {
                                        ui.label(
                                            egui::RichText::new("Value")
                                                .size(11.0)
                                                .strong()
                                                .color(super::theme::TEXT_MUTED),
                                        );
                                    });
                                    header.col(|ui| {
                                        ui.label(
                                            egui::RichText::new(egui_phosphor::regular::EYE_SLASH)
                                                .size(12.0)
                                                .color(super::theme::TEXT_MUTED),
                                        ).on_hover_text("Secret");
                                    });
                                    header.col(|_| {});
                                })
                                .body(|mut body| {
                                    for i in 0..n_vars {
                                        body.row(row_h, |mut row| {
                                            row.col(|ui| {
                                                if ui.add(
                                                    egui::TextEdit::singleline(&mut state.collection_vars[i].key)
                                                        .desired_width(ui.available_width())
                                                        .frame(egui::Frame::NONE)
                                                        .font(egui::TextStyle::Monospace),
                                                ).changed() {
                                                    state.vars_dirty = true;
                                                }
                                            });
                                            row.col(|ui| {
                                                let is_secret = state.collection_vars[i].is_secret;
                                                let mut layouter = super::var_highlight::var_text_layouter;
                                                if ui.add(
                                                    egui::TextEdit::singleline(&mut state.collection_vars[i].value)
                                                        .desired_width(ui.available_width())
                                                        .frame(egui::Frame::NONE)
                                                        .font(egui::TextStyle::Monospace)
                                                        .password(is_secret)
                                                        .layouter(&mut layouter),
                                                ).changed() {
                                                    state.vars_dirty = true;
                                                }
                                            });
                                            row.col(|ui| {
                                                if ui.checkbox(&mut state.collection_vars[i].is_secret, "").changed() {
                                                    state.vars_dirty = true;
                                                }
                                            });
                                            row.col(|ui| {
                                                if ui.add(
                                                    egui::Button::new(
                                                        egui::RichText::new(egui_phosphor::regular::TRASH)
                                                            .size(11.0)
                                                            .color(super::theme::TEXT_MUTED),
                                                    )
                                                    .frame(false)
                                                    .min_size(egui::vec2(16.0, 16.0)),
                                                ).on_hover_text("Remove").clicked() {
                                                    to_delete = Some(i);
                                                }
                                            });
                                        });
                                    }
                                });

                            if let Some(i) = to_delete {
                                let var_id = state.collection_vars[i].id.clone();
                                state.collection_vars.remove(i);
                                action = ConfigAction::DeleteVar(var_id);
                            }
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{} Add Variable",
                                            egui_phosphor::regular::PLUS,
                                        ))
                                        .size(12.0),
                                    )
                                    .corner_radius(egui::CornerRadius::same(4)),
                                )
                                .clicked()
                            {
                                action = ConfigAction::AddVar(
                                    collection.id.clone(),
                                    env_id.clone(),
                                );
                            }
                            if state.vars_dirty {
                                ui.add_space(8.0);
                                if super::theme::pill_button(ui, "Save Variables", super::theme::ACCENT) {
                                    action = ConfigAction::SaveVars;
                                }
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("unsaved")
                                        .color(egui::Color32::from_rgb(252, 161, 48))
                                        .italics()
                                        .size(11.0),
                                );
                            }
                        });
                    }
                }
            });

            ui.add_space(16.0);

            // Save button for collection config (name, base_path, auth)
            if state.dirty {
                ui.horizontal(|ui| {
                    if super::theme::pill_button(ui, "Save Collection", super::theme::ACCENT) {
                        action = ConfigAction::Save;
                    }
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Unsaved changes")
                            .color(egui::Color32::from_rgb(252, 161, 48))
                            .italics()
                            .size(12.0),
                    );
                });
            }

            // Info
            ui.add_space(16.0);
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

    action
}
