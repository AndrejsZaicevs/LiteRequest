use eframe::egui;
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};

use crate::models::request::KeyValuePair;
use crate::models::environment::Environment;

// ── Persisted settings (JSON blobs) ─────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub headers: Vec<KeyValuePair>,
    pub variables: Vec<KeyValuePair>,
}

// ── UI state ────────────────────────────────────────────────────

pub struct AppSettingsState {
    pub headers: Vec<KeyValuePair>,
    pub variables: Vec<KeyValuePair>,
    pub dirty: bool,

    // Environment management
    pub new_env_name: String,

    // Section expansion
    pub show_headers: bool,
    pub show_variables: bool,
    pub show_environments: bool,
}

impl Default for AppSettingsState {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            variables: Vec::new(),
            dirty: false,
            new_env_name: String::new(),
            show_headers: true,
            show_variables: true,
            show_environments: true,
        }
    }
}

impl AppSettingsState {
    pub fn load_from(&mut self, settings: &GlobalSettings) {
        self.headers = settings.headers.clone();
        self.variables = settings.variables.clone();
        self.dirty = false;
    }

    pub fn to_settings(&self) -> GlobalSettings {
        GlobalSettings {
            headers: self
                .headers
                .iter()
                .filter(|h| !h.key.is_empty() || !h.value.is_empty())
                .cloned()
                .collect(),
            variables: self
                .variables
                .iter()
                .filter(|v| !v.key.is_empty() || !v.value.is_empty())
                .cloned()
                .collect(),
        }
    }
}

// ── Actions ─────────────────────────────────────────────────────

pub enum SettingsAction {
    None,
    Save,
    NewEnvironment(String),
    DeleteEnvironment(String),
}

// ── Render ──────────────────────────────────────────────────────

pub fn render_app_settings(
    ui: &mut egui::Ui,
    state: &mut AppSettingsState,
    environments: &[Environment],
) -> SettingsAction {
    let mut action = SettingsAction::None;

    // Auto-save when dirty
    if state.dirty {
        action = SettingsAction::Save;
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
                    egui::RichText::new("Application Settings")
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
            // ── ENVIRONMENTS section ──
            if super::theme::collapsible_header(ui, "ENVIRONMENTS", &mut state.show_environments)
            {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(
                                "Environments switch variable sets across all collections.",
                            )
                            .size(11.0)
                            .color(super::theme::TEXT_MUTED),
                        );
                        ui.add_space(6.0);

                        // Existing environments
                        let mut delete_env: Option<String> = None;
                        for env in environments {
                            ui.horizontal(|ui| {
                                let active_icon = if env.is_active {
                                    egui_phosphor::regular::CHECK_CIRCLE
                                } else {
                                    egui_phosphor::regular::CIRCLE
                                };
                                ui.label(
                                    egui::RichText::new(active_icon)
                                        .size(14.0)
                                        .color(if env.is_active {
                                            super::theme::ACCENT
                                        } else {
                                            super::theme::TEXT_MUTED
                                        }),
                                );
                                ui.label(
                                    egui::RichText::new(&env.name)
                                        .size(12.0)
                                        .color(super::theme::TEXT_PRIMARY),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(egui_phosphor::regular::TRASH)
                                                .size(12.0)
                                                .color(super::theme::TEXT_MUTED),
                                        )
                                        .frame(false),
                                    )
                                    .on_hover_text("Delete environment")
                                    .clicked()
                                {
                                    delete_env = Some(env.id.clone());
                                }
                            });
                        }
                        if let Some(id) = delete_env {
                            action = SettingsAction::DeleteEnvironment(id);
                        }

                        ui.add_space(6.0);

                        // Add new environment
                        ui.horizontal(|ui| {
                            let input_fill = super::theme::SURFACE_0;
                            let input_stroke =
                                egui::Stroke::new(1.0, super::theme::BORDER);
                            egui::Frame::NONE
                                .fill(input_fill)
                                .stroke(input_stroke)
                                .corner_radius(egui::CornerRadius::same(3))
                                .inner_margin(egui::Margin::symmetric(4, 2))
                                .show(ui, |ui| {
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut state.new_env_name)
                                            .desired_width(180.0)
                                            .frame(egui::Frame::NONE)
                                            .font(egui::TextStyle::Monospace)
                                            .hint_text("New environment name"),
                                    );
                                    if resp.lost_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                        && !state.new_env_name.trim().is_empty()
                                    {
                                        action = SettingsAction::NewEnvironment(
                                            state.new_env_name.trim().to_string(),
                                        );
                                        state.new_env_name.clear();
                                    }
                                });
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(egui_phosphor::regular::PLUS)
                                            .size(14.0)
                                            .color(super::theme::ACCENT),
                                    )
                                    .frame(false),
                                )
                                .on_hover_text("Add environment")
                                .clicked()
                                && !state.new_env_name.trim().is_empty()
                            {
                                action = SettingsAction::NewEnvironment(
                                    state.new_env_name.trim().to_string(),
                                );
                                state.new_env_name.clear();
                            }
                        });
                    });
                ui.add_space(4.0);
            }

            // ── GLOBAL HEADERS section ──
            if super::theme::collapsible_header(ui, "DEFAULT HEADERS", &mut state.show_headers) {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(
                                "Auto-included in every request across all collections.",
                            )
                            .size(11.0)
                            .color(super::theme::TEXT_MUTED),
                        );
                        ui.add_space(4.0);
                        if render_kv_table(ui, "global_headers_tbl", &mut state.headers, "Header-Name", "value or {{variable}}") {
                            state.dirty = true;
                        }
                    });
                ui.add_space(4.0);
            }

            // ── GLOBAL VARIABLES section ──
            if super::theme::collapsible_header(ui, "DEFAULT VARIABLES", &mut state.show_variables)
            {
                ui.add_space(4.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(
                                "Available in all collections (overridden by collection/env variables).",
                            )
                            .size(11.0)
                            .color(super::theme::TEXT_MUTED),
                        );
                        ui.add_space(4.0);
                        if render_kv_table(ui, "global_vars_tbl", &mut state.variables, "variable_name", "value") {
                            state.dirty = true;
                        }
                    });
                ui.add_space(4.0);
            }
        });

    action
}

/// Reusable auto-grow KV table with checkbox, key, value, delete button.
fn render_kv_table(
    ui: &mut egui::Ui,
    id_salt: &str,
    items: &mut Vec<KeyValuePair>,
    key_hint: &str,
    value_hint: &str,
) -> bool {
    // Ensure trailing empty row
    let needs_empty = items.is_empty()
        || items
            .last()
            .map_or(true, |h| !h.key.is_empty() || !h.value.is_empty());
    if needs_empty {
        items.push(KeyValuePair::default());
    }

    let mut changed = false;
    let mut to_remove: Option<usize> = None;
    let n = items.len();
    let row_h = 28.0;
    let input_fill = super::theme::SURFACE_0;
    let input_stroke = egui::Stroke::new(1.0, super::theme::BORDER);

    TableBuilder::new(ui)
        .id_salt(id_salt)
        .striped(false)
        .max_scroll_height(n as f32 * row_h + 4.0)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(22.0))
        .column(Column::remainder().at_least(80.0))
        .column(Column::remainder().at_least(80.0))
        .column(Column::exact(22.0))
        .body(|mut body| {
            for i in 0..n {
                let is_last_empty =
                    i == n - 1 && items[i].key.is_empty() && items[i].value.is_empty();

                body.row(row_h, |mut row| {
                    // Checkbox
                    row.col(|ui| {
                        if !is_last_empty {
                            if ui.checkbox(&mut items[i].enabled, "").changed() {
                                changed = true;
                            }
                        }
                    });
                    // Key
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
                                        egui::TextEdit::singleline(&mut items[i].key)
                                            .desired_width(ui.available_width())
                                            .frame(egui::Frame::NONE)
                                            .font(egui::TextStyle::Monospace)
                                            .hint_text(if is_last_empty { key_hint } else { "" })
                                            .layouter(&mut layouter),
                                    )
                                    .changed()
                                {
                                    changed = true;
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
                                let mut layouter = super::var_highlight::var_text_layouter;
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut items[i].value)
                                            .desired_width(ui.available_width())
                                            .frame(egui::Frame::NONE)
                                            .font(egui::TextStyle::Monospace)
                                            .hint_text(if is_last_empty { value_hint } else { "" })
                                            .layouter(&mut layouter),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                    });
                    // Remove
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
        items.remove(idx);
        changed = true;
    }

    changed
}
