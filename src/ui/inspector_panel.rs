use eframe::egui;
use std::collections::HashMap;
use crate::models::*;
use super::request_editor::{collapsible_section, count_active_pairs, render_kv_table};
use super::history_panel;

pub struct InspectorState {
    pub show_path_params: bool,
    pub show_params: bool,
    pub show_headers: bool,
    pub show_history: bool,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            show_path_params: true,
            show_params: true,
            show_headers: true,
            show_history: true,
        }
    }
}

pub enum InspectorAction {
    None,
    DataChanged,
    SelectVersion(String),
    SelectExecution(String),
}

pub fn render_inspector(
    ui: &mut egui::Ui,
    data: &mut RequestData,
    dirty: &mut bool,
    path_params: &mut Vec<KeyValuePair>,
    versions: &[RequestVersion],
    executions: &[RequestExecution],
    selected_version_id: Option<&str>,
    selected_execution_id: Option<&str>,
    inspector_state: &mut InspectorState,
    variables: &HashMap<String, String>,
) -> InspectorAction {
    let mut action = InspectorAction::None;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Inspector")
                .strong()
                .size(14.0)
                .color(super::theme::TEXT_PRIMARY),
        );
    });
    ui.separator();
    ui.add_space(2.0);

    egui::ScrollArea::vertical()
        .id_salt("inspector_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // ── PATH PARAMS (only if any exist) ──
            if !path_params.is_empty() {
                let path_count = count_active_pairs(path_params);
                if collapsible_section(ui, "Path Params", path_count, &mut inspector_state.show_path_params) {
                    ui.push_id("path_params_section", |ui| {
                        if render_path_params_table(ui, path_params, variables) {
                            *dirty = true;
                            action = InspectorAction::DataChanged;
                        }
                    });
                    ui.add_space(4.0);
                }
            }

            // ── QUERY PARAMS ──
            let param_count = count_active_pairs(&data.query_params);
            if collapsible_section(ui, "Params", param_count, &mut inspector_state.show_params) {
                ui.push_id("params_section", |ui| {
                    if render_kv_table(ui, &mut data.query_params, "params_table", variables) {
                        *dirty = true;
                        action = InspectorAction::DataChanged;
                    }
                });
                ui.add_space(4.0);
            }

            // ── HEADERS ──
            let header_count = count_active_pairs(&data.headers);
            if collapsible_section(ui, "Headers", header_count, &mut inspector_state.show_headers) {
                ui.push_id("headers_section", |ui| {
                    if render_kv_table(ui, &mut data.headers, "headers_table", variables) {
                        *dirty = true;
                        action = InspectorAction::DataChanged;
                    }
                });
                ui.add_space(4.0);
            }

            // ── HISTORY ──
            if collapsible_section(ui, "History", versions.len() + executions.len(), &mut inspector_state.show_history) {
                // Versions
                if let Some(vid) = history_panel::render_version_history(
                    ui, versions, selected_version_id,
                ) {
                    action = InspectorAction::SelectVersion(vid);
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(2.0);

                // Executions
                if let Some(eid) = history_panel::render_execution_history(
                    ui, executions, selected_execution_id,
                ) {
                    action = InspectorAction::SelectExecution(eid);
                }
            }
        });

    action
}

/// Simplified KV table for path params — key is read-only (derived from URL), only value is editable.
fn render_path_params_table(
    ui: &mut egui::Ui,
    pairs: &mut Vec<KeyValuePair>,
    variables: &HashMap<String, String>,
) -> bool {
    use egui_extras::{TableBuilder, Column};

    let mut changed = false;
    let row_h = 22.0;
    let n_rows = pairs.len();

    TableBuilder::new(ui)
        .id_salt("path_params_table")
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(20.0))               // checkbox
        .column(Column::remainder().at_least(60.0)) // key (read-only)
        .column(Column::remainder().at_least(60.0)) // value (editable)
        .body(|mut body| {
            for i in 0..n_rows {
                body.row(row_h, |mut row| {
                    // Checkbox
                    row.col(|ui| {
                        if ui.checkbox(&mut pairs[i].enabled, "").changed() {
                            changed = true;
                        }
                    });
                    // Key (read-only label styled like the param name)
                    row.col(|ui| {
                        ui.label(
                            egui::RichText::new(format!(":{}", &pairs[i].key))
                                .size(12.0)
                                .color(super::theme::ACCENT)
                                .family(egui::FontFamily::Monospace),
                        );
                    });
                    // Value (editable with variable highlighting)
                    row.col(|ui| {
                        let mut layouter = super::var_highlight::var_text_layouter;
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut pairs[i].value)
                                .desired_width(ui.available_width())
                                .frame(egui::Frame::NONE)
                                .font(egui::TextStyle::Monospace)
                                .layouter(&mut layouter),
                        );
                        if resp.changed() {
                            changed = true;
                        }
                        super::var_highlight::show_variable_tooltip(
                            ui, &resp, &pairs[i].value, variables,
                        );
                    });
                });
            }
        });

    changed
}
