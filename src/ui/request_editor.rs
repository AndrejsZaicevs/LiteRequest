use eframe::egui;
use crate::models::*;

pub struct RequestEditorState {
    pub data: RequestData,
    pub dirty: bool,
    pub json_error: Option<String>,
    pub show_params: bool,
    pub show_headers: bool,
    pub show_body: bool,
}

impl Default for RequestEditorState {
    fn default() -> Self {
        Self {
            data: RequestData::default(),
            dirty: false,
            json_error: None,
            show_params: true,
            show_headers: true,
            show_body: true,
        }
    }
}

pub enum EditorAction {
    None,
    Send,
    DataChanged,
}

/// Join base_path and url without producing double slashes.
fn join_display_path<'a>(base: &'a str, path: &'a str) -> String {
    if base.is_empty() {
        return path.to_string();
    }
    let base_trimmed = base.trim_end_matches('/');
    if path.is_empty() || path.starts_with('/') {
        format!("{base_trimmed}{path}")
    } else {
        format!("{base_trimmed}/{path}")
    }
}

pub fn render_request_editor(
    ui: &mut egui::Ui,
    state: &mut RequestEditorState,
    request_name: &str,
    base_path: &str,
) -> EditorAction {
    let mut action = EditorAction::None;

    // Request name + dirty indicator
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(request_name)
                .size(18.0)
                .strong()
                .color(super::theme::TEXT_PRIMARY),
        );
        if state.dirty {
            ui.label(
                egui::RichText::new("* unsaved")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(252, 161, 48)),
            );
        }
    });
    ui.add_space(6.0);

    // ── URL bar: [Method] [base_path + path in one frame] [Send] ──
    ui.horizontal(|ui| {
        let method_text = state.data.method.as_str();
        let [r, g, b] = state.data.method.color();
        let method_color = egui::Color32::from_rgb(r, g, b);

        egui::ComboBox::from_id_salt("method_selector")
            .selected_text(
                egui::RichText::new(method_text)
                    .color(method_color)
                    .strong()
                    .size(14.0),
            )
            .width(90.0)
            .show_ui(ui, |ui| {
                for m in HttpMethod::all() {
                    let [mr, mg, mb] = m.color();
                    let label = egui::RichText::new(m.as_str())
                        .color(egui::Color32::from_rgb(mr, mg, mb))
                        .size(14.0);
                    if ui.selectable_label(state.data.method == *m, label).clicked() {
                        state.data.method = m.clone();
                        state.dirty = true;
                    }
                }
            });

        let send_btn_w = 80.0;
        let available_for_url = (ui.available_width() - send_btn_w - 8.0).max(200.0);

        if !base_path.is_empty() {
            // Smart display: trim trailing slash from base if url starts with /
            let display_base = if state.data.url.starts_with('/') {
                base_path.trim_end_matches('/')
            } else if base_path.ends_with('/') {
                base_path
            } else {
                // append implicit separator only visually
                base_path
            };

            egui::Frame::default()
                .fill(super::theme::SURFACE_2)
                .stroke(egui::Stroke::new(1.0, super::theme::BORDER))
                .rounding(egui::Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(6.0, 0.0))
                .show(ui, |ui: &mut egui::Ui| {
                    ui.set_width(available_for_url);
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(
                            egui::RichText::new(display_base)
                                .size(13.0)
                                .color(super::theme::TEXT_MUTED)
                                .family(egui::FontFamily::Monospace),
                        );
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut state.data.url)
                                    .desired_width(ui.available_width())
                                    .frame(false)
                                    .font(egui::TextStyle::Monospace),
                            )
                            .changed()
                        {
                            state.dirty = true;
                        }
                    });
                });
        } else {
            if ui
                .add_sized(
                    egui::vec2(available_for_url, 28.0),
                    egui::TextEdit::singleline(&mut state.data.url)
                        .font(egui::TextStyle::Monospace),
                )
                .changed()
            {
                state.dirty = true;
            }
        }

        if super::theme::pill_button(ui, "Send", super::theme::ACCENT) {
            action = EditorAction::Send;
        }
    });

    ui.add_space(6.0);

    // ── Flat scrollable view: Params, Headers, Body ──
    egui::ScrollArea::vertical()
        .id_salt("editor_flat_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // ── PARAMS ──
            let param_count = count_active_pairs(&state.data.query_params);
            if collapsible_section(ui, "Params", param_count, &mut state.show_params) {
                ui.push_id("params_section", |ui| {
                    if render_kv_table(ui, &mut state.data.query_params, "params_table") {
                        state.dirty = true;
                        action = EditorAction::DataChanged;
                    }
                });
                ui.add_space(4.0);
            }

            // ── HEADERS ──
            let header_count = count_active_pairs(&state.data.headers);
            if collapsible_section(ui, "Headers", header_count, &mut state.show_headers) {
                ui.push_id("headers_section", |ui| {
                    if render_kv_table(ui, &mut state.data.headers, "headers_table") {
                        state.dirty = true;
                        action = EditorAction::DataChanged;
                    }
                });
                ui.add_space(4.0);
            }

            // ── BODY ──
            let body_count = if state.data.body_type != BodyType::None { 1 } else { 0 };
            if collapsible_section(ui, "Body", body_count, &mut state.show_body) {
                ui.horizontal(|ui| {
                    for bt in BodyType::all() {
                        let is_active = state.data.body_type == *bt;
                        let rt = if is_active {
                            egui::RichText::new(bt.as_str())
                                .strong()
                                .size(13.0)
                                .color(super::theme::ACCENT)
                        } else {
                            egui::RichText::new(bt.as_str())
                                .size(13.0)
                                .color(super::theme::TEXT_SECONDARY)
                        };
                        if ui
                            .add(
                                egui::Button::new(rt)
                                    .fill(if is_active {
                                        super::theme::ACCENT.gamma_multiply(0.15)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    })
                                    .stroke(if is_active {
                                        egui::Stroke::new(
                                            1.0,
                                            super::theme::ACCENT.gamma_multiply(0.4),
                                        )
                                    } else {
                                        egui::Stroke::NONE
                                    })
                                    .rounding(egui::Rounding::same(4.0)),
                            )
                            .clicked()
                        {
                            state.data.body_type = bt.clone();
                            state.dirty = true;
                            action = EditorAction::DataChanged;
                        }
                    }
                });

                if state.data.body_type != BodyType::None {
                    ui.add_space(4.0);

                    if state.data.body_type == BodyType::Json {
                        if let Some(err) = &state.json_error {
                            ui.label(
                                egui::RichText::new(format!("! {err}"))
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(249, 62, 62)),
                            );
                        } else if !state.data.body.is_empty() {
                            ui.label(
                                egui::RichText::new("Valid JSON")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(73, 204, 144)),
                            );
                        }
                    }

                    let response = ui.add(
                        egui::TextEdit::multiline(&mut state.data.body)
                            .desired_width(f32::INFINITY)
                            .desired_rows(6)
                            .font(egui::TextStyle::Monospace)
                            .code_editor(),
                    );

                    if response.changed() {
                        state.dirty = true;
                        action = EditorAction::DataChanged;
                        if state.data.body_type == BodyType::Json
                            && !state.data.body.is_empty()
                        {
                            match serde_json::from_str::<serde_json::Value>(&state.data.body) {
                                Ok(_) => state.json_error = None,
                                Err(e) => state.json_error = Some(e.to_string()),
                            }
                        } else {
                            state.json_error = None;
                        }
                    }
                }
            }
        });

    action
}

/// Collapsible section header. Returns true if section is expanded.
fn collapsible_section(ui: &mut egui::Ui, label: &str, count: usize, expanded: &mut bool) -> bool {
    let icon = if *expanded { "v" } else { ">" };
    let count_text = if count > 0 {
        format!(" ({count})")
    } else {
        String::new()
    };

    ui.horizontal(|ui| {
        let resp = ui.add(
            egui::Label::new(
                egui::RichText::new(format!("{icon}  {label}{count_text}"))
                    .strong()
                    .size(13.0)
                    .color(if *expanded {
                        super::theme::TEXT_PRIMARY
                    } else {
                        super::theme::TEXT_SECONDARY
                    }),
            )
            .sense(egui::Sense::click()),
        );
        if resp.clicked() {
            *expanded = !*expanded;
        }

        let rect = resp.rect;
        let line_y = rect.center().y;
        ui.painter().line_segment(
            [
                egui::pos2(rect.right() + 8.0, line_y),
                egui::pos2(ui.available_rect_before_wrap().right(), line_y),
            ],
            egui::Stroke::new(1.0, super::theme::BORDER),
        );
    });

    ui.add_space(2.0);
    *expanded
}

fn count_active_pairs(pairs: &[KeyValuePair]) -> usize {
    pairs
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .count()
}

/// KV editor using egui_extras table for clean column alignment.
fn render_kv_table(ui: &mut egui::Ui, pairs: &mut Vec<KeyValuePair>, id: &str) -> bool {
    use egui_extras::{TableBuilder, Column};

    let mut changed = false;
    let mut to_remove: Option<usize> = None;
    let row_h = 22.0;
    let n_rows = pairs.len();

    TableBuilder::new(ui)
        .id_salt(id)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(20.0))           // checkbox
        .column(Column::remainder().at_least(80.0)) // key
        .column(Column::remainder().at_least(80.0)) // value
        .column(Column::exact(20.0))           // remove btn
        .body(|mut body| {
            for i in 0..n_rows {
                body.row(row_h, |mut row| {
                    // Checkbox
                    row.col(|ui| {
                        if ui.checkbox(&mut pairs[i].enabled, "").changed() {
                            changed = true;
                        }
                    });
                    // Key
                    row.col(|ui| {
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut pairs[i].key)
                                    .desired_width(ui.available_width())
                                    .frame(false)
                                    .font(egui::TextStyle::Monospace),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
                    // Value
                    row.col(|ui| {
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut pairs[i].value)
                                    .desired_width(ui.available_width())
                                    .frame(false)
                                    .font(egui::TextStyle::Monospace),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
                    // Remove
                    row.col(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("x")
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
                    });
                });
            }
        });

    if let Some(idx) = to_remove {
        pairs.remove(idx);
        changed = true;
    }

    // "+ Add" button
    if ui
        .add(
            egui::Button::new(
                egui::RichText::new("+ Add")
                    .size(12.0)
                    .color(super::theme::TEXT_SECONDARY),
            )
            .frame(false),
        )
        .clicked()
    {
        pairs.push(KeyValuePair::default());
        changed = true;
    }

    changed
}
