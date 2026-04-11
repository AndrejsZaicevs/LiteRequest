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

    // Execution filters
    pub exec_filter_version: bool,
    pub exec_filter_env: bool,

    // Time-group expansion state for executions (indexed by TimeBucket ordinal)
    pub exec_time_expanded: [bool; 5],
    /// Set to true once after switching request so we auto-expand first group
    pub exec_groups_initialized: bool,

    // Time-group expansion state for versions
    pub version_time_expanded: [bool; 5],
    pub version_groups_initialized: bool,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            show_path_params: true,
            show_params: true,
            show_headers: true,
            show_versions: false,
            show_executions: false,
            exec_filter_version: false,
            exec_filter_env: false,
            exec_time_expanded: [true, false, false, false, false],
            exec_groups_initialized: false,
            version_time_expanded: [true, false, false, false, false],
            version_groups_initialized: false,
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
    versions: &[RequestVersion],
    executions: &[RequestExecution],
    selected_version_id: Option<&str>,
    selected_execution_id: Option<&str>,
    inspector_state: &mut InspectorState,
    variables: &HashMap<String, String>,
    environments: &[crate::models::Environment],
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
            if !data.path_params.is_empty() {
                let path_count = count_active_pairs(&data.path_params);
                if section_header(ui, "PATH PARAMS", path_count, &mut inspector_state.show_path_params) {
                    ui.push_id("path_params_section", |ui| {
                        if render_path_params_table(ui, &mut data.path_params, variables) {
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
                    &mut inspector_state.version_time_expanded,
                    &mut inspector_state.version_groups_initialized,
                ) {
                    action = InspectorAction::SelectVersion(vid);
                }
                ui.add_space(6.0);
            }

            // ── EXECUTIONS ──
            if section_header(ui, "EXECUTIONS", executions.len(), &mut inspector_state.show_executions) {
                if let Some(eid) = history_panel::render_execution_list_filtered(
                    ui,
                    executions,
                    selected_execution_id,
                    selected_version_id,
                    environments,
                    &mut inspector_state.exec_filter_version,
                    &mut inspector_state.exec_filter_env,
                    &mut inspector_state.exec_time_expanded,
                    &mut inspector_state.exec_groups_initialized,
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
    let start_y = ui.cursor().min.y;

    // Reserve a shape slot FIRST so the background is drawn behind the text
    let bg_idx = ui.painter().add(egui::Shape::Noop);

    ui.add_space(5.0); // top padding for taller header

    let resp = ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(icon)
                .size(12.0)
                .color(super::theme::TEXT_MUTED),
        );
        ui.label(
            egui::RichText::new(label)
                .strong()
                .size(13.0)
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

    ui.add_space(5.0); // bottom padding for taller header

    let end_y = ui.cursor().min.y;

    // Fill the reserved slot with the full background rect
    let bg_rect = egui::Rect::from_min_size(
        egui::pos2(resp.response.rect.min.x, start_y),
        egui::vec2(available_w, end_y - start_y),
    );
    ui.painter().set(bg_idx, egui::Shape::rect_filled(bg_rect, 0.0, header_color));

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
    let row_h = 28.0;
    let n_rows = pairs.len();
    let input_fill = super::theme::SURFACE_0;
    let input_stroke = egui::Stroke::new(1.0, super::theme::BORDER);

    TableBuilder::new(ui)
        .id_salt(id)
        .striped(false)
        .max_scroll_height(n_rows as f32 * row_h + 4.0)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(22.0))           // checkbox
        .column(Column::remainder().at_least(60.0)) // key
        .column(Column::remainder().at_least(60.0)) // value
        .column(Column::exact(22.0))           // remove btn
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
                        egui::Frame::none()
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
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
                    });
                    // Value
                    row.col(|ui| {
                        egui::Frame::none()
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
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
    let row_h = 28.0;
    let n_rows = pairs.len();
    let input_fill = super::theme::SURFACE_0;
    let input_stroke = egui::Stroke::new(1.0, super::theme::BORDER);

    TableBuilder::new(ui)
        .id_salt("path_params_table")
        .striped(false)
        .max_scroll_height(n_rows as f32 * row_h + 4.0)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(22.0))               // checkbox
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
                    // Key: styled monospace label (no editable frame)
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
                        egui::Frame::none()
                            .fill(input_fill)
                            .stroke(input_stroke)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(4, 2))
                            .show(ui, |ui| {
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
                });
            }
        });

    changed
}
