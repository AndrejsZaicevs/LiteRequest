use eframe::egui;
use std::collections::HashMap;
use crate::models::*;
use super::request_editor::count_active_pairs;
use super::history_panel;

pub struct InspectorState {
    pub show_path_params: bool,
    pub show_params: bool,
    pub show_headers: bool,
    pub show_versions: bool,
    pub show_executions: bool,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            show_path_params: true,
            show_params: true,
            show_headers: true,
            show_versions: false,
            show_executions: false,
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

    // Title bar with background
    egui::Frame::default()
        .fill(super::theme::SURFACE_2)
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(egui_phosphor::regular::SLIDERS_HORIZONTAL)
                        .size(14.0)
                        .color(super::theme::ACCENT),
                );
                ui.label(
                    egui::RichText::new("Inspector")
                        .strong()
                        .size(14.0)
                        .color(super::theme::TEXT_PRIMARY),
                );
            });
        });
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .id_salt("inspector_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // ── PATH PARAMS (only if any exist) ──
            if !path_params.is_empty() {
                let path_count = count_active_pairs(path_params);
                if section_header(ui, "PATH PARAMS", path_count, &mut inspector_state.show_path_params) {
                    ui.push_id("path_params_section", |ui| {
                        if render_path_params_table(ui, path_params, variables) {
                            *dirty = true;
                            action = InspectorAction::DataChanged;
                        }
                    });
                    ui.add_space(6.0);
                }
            }

            // ── QUERY PARAMS ──
            let param_count = count_active_pairs(&data.query_params);
            if section_header(ui, "QUERY PARAMS", param_count, &mut inspector_state.show_params) {
                ui.push_id("params_section", |ui| {
                    if render_kv_table(ui, &mut data.query_params, "params_table", variables) {
                        *dirty = true;
                        action = InspectorAction::DataChanged;
                    }
                });
                ui.add_space(6.0);
            }

            // ── HEADERS ──
            let header_count = count_active_pairs(&data.headers);
            if section_header(ui, "HEADERS", header_count, &mut inspector_state.show_headers) {
                ui.push_id("headers_section", |ui| {
                    if render_kv_table(ui, &mut data.headers, "headers_table", variables) {
                        *dirty = true;
                        action = InspectorAction::DataChanged;
                    }
                });
                ui.add_space(6.0);
            }

            // ── VERSIONS ──
            if section_header(ui, "VERSIONS", versions.len(), &mut inspector_state.show_versions) {
                if let Some(vid) = history_panel::render_version_list(
                    ui, versions, selected_version_id,
                ) {
                    action = InspectorAction::SelectVersion(vid);
                }
                ui.add_space(6.0);
            }

            // ── EXECUTIONS ──
            if section_header(ui, "EXECUTIONS", executions.len(), &mut inspector_state.show_executions) {
                if let Some(eid) = history_panel::render_execution_list(
                    ui, executions, selected_execution_id,
                ) {
                    action = InspectorAction::SelectExecution(eid);
                }
                ui.add_space(6.0);
            }
        });

    action
}

/// Prominent section header for the inspector with uppercase label, count badge, and divider line.
fn section_header(ui: &mut egui::Ui, label: &str, count: usize, expanded: &mut bool) -> bool {
    let icon = if *expanded {
        egui_phosphor::regular::CARET_DOWN
    } else {
        egui_phosphor::regular::CARET_RIGHT
    };

    let header_color = if *expanded {
        super::theme::SURFACE_2
    } else {
        egui::Color32::TRANSPARENT
    };

    ui.add_space(1.0);

    let available_w = ui.available_width();
    let resp = ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(icon)
                .size(10.0)
                .color(super::theme::TEXT_MUTED),
        );
        ui.label(
            egui::RichText::new(label)
                .strong()
                .size(12.0)
                .color(if *expanded {
                    super::theme::TEXT_PRIMARY
                } else {
                    super::theme::TEXT_SECONDARY
                }),
        );
        if count > 0 {
            ui.add(
                egui::Button::new(
                    egui::RichText::new(format!("{count}"))
                        .size(10.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                )
                .fill(super::theme::ACCENT.gamma_multiply(0.7))
                .corner_radius(egui::CornerRadius::same(8))
                .min_size(egui::vec2(20.0, 16.0))
                .sense(egui::Sense::hover()),
            );
        }
    });

    // Paint background spanning full width behind the row
    let mut bg_rect = resp.response.rect;
    bg_rect.set_width(available_w);
    ui.painter()
        .rect_filled(bg_rect, 0.0, header_color);

    let click_resp = ui.interact(
        bg_rect,
        ui.id().with(label),
        egui::Sense::click(),
    );
    if click_resp.clicked() {
        *expanded = !*expanded;
    }
    if click_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    *expanded
}

/// KV editor with auto-grow: an empty row is always present at the bottom.
/// When the user types into the last empty row, a new empty row is appended automatically.
/// No "+ Add" button needed.
fn render_kv_table(
    ui: &mut egui::Ui,
    pairs: &mut Vec<KeyValuePair>,
    id: &str,
    variables: &HashMap<String, String>,
) -> bool {
    use egui_extras::{TableBuilder, Column};

    // Ensure there's always one empty row at the end for the user to type into
    let needs_empty = pairs.is_empty()
        || pairs.last().map_or(true, |p| !p.key.is_empty() || !p.value.is_empty());
    if needs_empty {
        pairs.push(KeyValuePair::default());
    }

    let mut changed = false;
    let mut to_remove: Option<usize> = None;
    let row_h = 22.0;
    let n_rows = pairs.len();

    TableBuilder::new(ui)
        .id_salt(id)
        .striped(true)
        .max_scroll_height(n_rows as f32 * row_h + 4.0)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(20.0))           // checkbox
        .column(Column::remainder().at_least(60.0)) // key
        .column(Column::remainder().at_least(60.0)) // value
        .column(Column::exact(20.0))           // remove btn
        .body(|mut body| {
            for i in 0..n_rows {
                let is_last_empty = i == n_rows - 1
                    && pairs[i].key.is_empty()
                    && pairs[i].value.is_empty();

                body.row(row_h, |mut row| {
                    // Checkbox
                    row.col(|ui| {
                        if !is_last_empty {
                            if ui.checkbox(&mut pairs[i].enabled, "").changed() {
                                changed = true;
                            }
                        }
                    });
                    // Key
                    row.col(|ui| {
                        let mut layouter = super::var_highlight::var_text_layouter;
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut pairs[i].key)
                                .desired_width(ui.available_width())
                                .frame(egui::Frame::NONE)
                                .font(egui::TextStyle::Monospace)
                                .layouter(&mut layouter),
                        );
                        if resp.changed() {
                            changed = true;
                        }
                        super::var_highlight::show_variable_tooltip(
                            ui, &resp, &pairs[i].key, variables,
                        );
                    });
                    // Value
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
                    // Remove (hide for the trailing empty row)
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
        pairs.remove(idx);
        changed = true;
    }

    changed
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
        .max_scroll_height(n_rows as f32 * row_h + 4.0)
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
