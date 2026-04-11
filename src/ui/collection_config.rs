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
}

impl Default for CollectionConfigState {
    fn default() -> Self {
        Self {
            name: String::new(),
            base_path: String::new(),
            auth: AuthConfig::default(),
            dirty: false,
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
    }

    pub fn to_auth_json(&self) -> Option<String> {
        serde_json::to_string(&self.auth).ok()
    }
}

pub enum ConfigAction {
    None,
    Save,
}

pub fn render_collection_config(
    ui: &mut egui::Ui,
    state: &mut CollectionConfigState,
    collection: &Collection,
) -> ConfigAction {
    let mut action = ConfigAction::None;

    ui.add_space(8.0);

    // Title
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("*")
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

    // General section
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
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut state.base_path)
                            .desired_width(350.0)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("{{protocol}}://{{host}}/v1"),
                    )
                    .changed()
                {
                    state.dirty = true;
                }
                ui.end_row();
            });
    });

    ui.add_space(12.0);

    // Authentication section
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

        // Auth-type-specific fields
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
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut token)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("Bearer token or {{variable}}")
                            .password(true),
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
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut username)
                                    .desired_width(300.0)
                                    .font(egui::TextStyle::Monospace),
                            )
                            .changed()
                        {
                            state.auth.basic_username = Some(username);
                            state.dirty = true;
                        }
                        ui.end_row();

                        let mut password =
                            state.auth.basic_password.clone().unwrap_or_default();
                        ui.label(
                            egui::RichText::new("Password")
                                .strong()
                                .color(super::theme::TEXT_SECONDARY),
                        );
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut password)
                                    .desired_width(300.0)
                                    .font(egui::TextStyle::Monospace)
                                    .password(true),
                            )
                            .changed()
                        {
                            state.auth.basic_password = Some(password);
                            state.dirty = true;
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
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut header_name)
                                    .desired_width(300.0)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text("X-API-Key"),
                            )
                            .changed()
                        {
                            state.auth.api_key_header = Some(header_name);
                            state.dirty = true;
                        }
                        ui.end_row();

                        let mut value =
                            state.auth.api_key_value.clone().unwrap_or_default();
                        ui.label(
                            egui::RichText::new("Value")
                                .strong()
                                .color(super::theme::TEXT_SECONDARY),
                        );
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut value)
                                    .desired_width(300.0)
                                    .font(egui::TextStyle::Monospace)
                                    .password(true)
                                    .hint_text("{{api_key}}"),
                            )
                            .changed()
                        {
                            state.auth.api_key_value = Some(value);
                            state.dirty = true;
                        }
                        ui.end_row();
                    });
            }
        }
    });

    ui.add_space(16.0);

    // Save button
    if state.dirty {
        ui.horizontal(|ui| {
            if super::theme::pill_button(ui, "Save Changes", super::theme::ACCENT) {
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

    action
}
