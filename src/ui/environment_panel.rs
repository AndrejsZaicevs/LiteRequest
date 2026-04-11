use eframe::egui;
use crate::models::*;

pub enum EnvAction {
    None,
    NewEnvironment,
    SelectEnvironment(String),
    DeleteEnvironment(String),
    AddVariable(String),           // environment_id
    UpdateVariable(EnvVariable),
    DeleteVariable(String),        // variable_id
}

pub struct EnvironmentPanelState {
    pub show_panel: bool,
    pub selected_env_id: Option<String>,
    pub new_env_name: String,
}

impl Default for EnvironmentPanelState {
    fn default() -> Self {
        Self {
            show_panel: false,
            selected_env_id: None,
            new_env_name: String::new(),
        }
    }
}

pub fn render_env_selector(
    ui: &mut egui::Ui,
    environments: &[Environment],
    _state: &mut EnvironmentPanelState,
) -> EnvAction {
    let mut action = EnvAction::None;

    // "None" chip — active when no environment is selected
    let none_active = !environments.iter().any(|e| e.is_active);
    let chip = env_chip(ui, "None", none_active);
    if chip.clicked() {
        // Deselect all — send empty id
        action = EnvAction::SelectEnvironment(String::new());
    }

    for env in environments {
        let chip = env_chip(ui, &env.name, env.is_active);
        if chip.clicked() {
            action = EnvAction::SelectEnvironment(env.id.clone());
        }
    }

    action
}

fn env_chip(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    let (fill, text_color, stroke) = if active {
        (
            super::theme::ACCENT.gamma_multiply(0.20),
            super::theme::ACCENT,
            egui::Stroke::new(1.0, super::theme::ACCENT.gamma_multiply(0.5)),
        )
    } else {
        (
            egui::Color32::TRANSPARENT,
            super::theme::TEXT_SECONDARY,
            egui::Stroke::new(1.0, super::theme::BORDER),
        )
    };

    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .size(12.0)
                .color(text_color),
        )
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(12))
        .min_size(egui::vec2(0.0, 22.0)),
    )
}

pub fn render_environment_panel(
    ui: &mut egui::Ui,
    environments: &[Environment],
    variables: &mut Vec<EnvVariable>,
    state: &mut EnvironmentPanelState,
) -> EnvAction {
    let mut action = EnvAction::None;

    // New environment
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.new_env_name);
        if ui.button("Add Environment").clicked() && !state.new_env_name.is_empty() {
            action = EnvAction::NewEnvironment;
        }
    });

    ui.separator();

    // Environment list
    for env in environments {
        ui.horizontal(|ui| {
            let is_selected = state.selected_env_id.as_deref() == Some(&env.id);
            if ui.selectable_label(is_selected, &env.name).clicked() {
                state.selected_env_id = Some(env.id.clone());
            }
            if ui.small_button(egui_phosphor::regular::X).clicked() {
                action = EnvAction::DeleteEnvironment(env.id.clone());
            }
        });
    }

    ui.separator();

    // Variables for selected environment
    if let Some(env_id) = &state.selected_env_id {
        ui.heading("Variables");

        let mut var_action = EnvAction::None;
        let mut to_delete = None;

        egui::Grid::new("env_vars_grid")
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Key").strong());
                ui.label(egui::RichText::new("Value").strong());
                ui.label(egui::RichText::new("Secret").strong());
                ui.label("");
                ui.end_row();

                for (i, var) in variables.iter_mut().enumerate() {
                    let mut changed = false;

                    let key_resp = ui.add(
                        egui::TextEdit::singleline(&mut var.key)
                            .desired_width(120.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    changed |= key_resp.changed();

                    if var.is_secret {
                        let val_resp = ui.add(
                            egui::TextEdit::singleline(&mut var.value)
                                .desired_width(200.0)
                                .password(true)
                                .font(egui::TextStyle::Monospace),
                        );
                        changed |= val_resp.changed();
                    } else {
                        let val_resp = ui.add(
                            egui::TextEdit::singleline(&mut var.value)
                                .desired_width(200.0)
                                .font(egui::TextStyle::Monospace),
                        );
                        changed |= val_resp.changed();
                    }

                    if ui.checkbox(&mut var.is_secret, "secret").changed() {
                        changed = true;
                    }

                    if ui.small_button(egui_phosphor::regular::X).clicked() {
                        to_delete = Some(i);
                    }

                    if changed {
                        var_action = EnvAction::UpdateVariable(var.clone());
                    }

                    ui.end_row();
                }
            });

        if let Some(idx) = to_delete {
            if idx < variables.len() {
                let id = variables[idx].id.clone();
                variables.remove(idx);
                action = EnvAction::DeleteVariable(id);
            }
        }

        if let EnvAction::UpdateVariable(_) = &var_action {
            action = var_action;
        }

        if ui.button("+ Add Variable").clicked() {
            action = EnvAction::AddVariable(env_id.clone());
        }
    }

    action
}
