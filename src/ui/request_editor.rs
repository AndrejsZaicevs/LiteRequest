use eframe::egui;
use std::collections::HashMap;
use crate::models::*;

pub struct RequestEditorState {
    pub data: RequestData,
    pub dirty: bool,
    pub json_error: Option<String>,
    pub show_curl_import: bool,
    pub curl_import_buf: String,
    pub curl_import_error: Option<String>,
}

impl Default for RequestEditorState {
    fn default() -> Self {
        Self {
            data: RequestData::default(),
            dirty: false,
            json_error: None,
            show_curl_import: false,
            curl_import_buf: String::new(),
            curl_import_error: None,
        }
    }
}

pub enum EditorAction {
    None,
    Send,
    DataChanged,
    UrlCommitted,
    CopyCurl,
    ImportCurl(RequestData),
}

pub fn render_request_editor(
    ui: &mut egui::Ui,
    state: &mut RequestEditorState,
    request_name: &str,
    base_path: &str,
    variables: &HashMap<String, String>,
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

    // ── URL bar: [Method] [resolved_base + path in one frame] [Send] ──
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

        let send_btn_w = 140.0; // Send + cURL menu
        let available_for_url = (ui.available_width() - send_btn_w - 8.0).max(200.0);

        if !base_path.is_empty() {
            // Resolve variables in the base path for display
            let resolved_base = super::var_highlight::resolve_display(base_path, variables);
            let display_base = if state.data.url.starts_with('/') {
                resolved_base.trim_end_matches('/').to_string()
            } else if resolved_base.ends_with('/') {
                resolved_base.clone()
            } else {
                resolved_base.clone()
            };

            egui::Frame::default()
                .fill(super::theme::SURFACE_2)
                .stroke(egui::Stroke::new(1.0, super::theme::BORDER))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(6, 0))
                .show(ui, |ui: &mut egui::Ui| {
                    ui.set_width(available_for_url);
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(
                            egui::RichText::new(&display_base)
                                .size(13.0)
                                .color(super::theme::TEXT_MUTED)
                                .family(egui::FontFamily::Monospace),
                        );
                        let mut layouter = super::var_highlight::var_text_layouter;
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut state.data.url)
                                .desired_width(ui.available_width())
                                .frame(egui::Frame::NONE)
                                .font(egui::TextStyle::Monospace)
                                .layouter(&mut layouter),
                        );
                        if resp.changed() {
                            state.dirty = true;
                            action = EditorAction::DataChanged;
                        }
                        if resp.lost_focus() {
                            action = EditorAction::UrlCommitted;
                        }
                        super::var_highlight::show_variable_tooltip(
                            ui, &resp, &state.data.url, variables,
                        );
                    });
                });
        } else {
            let mut layouter = super::var_highlight::var_text_layouter;
            let resp = ui.add_sized(
                egui::vec2(available_for_url, 28.0),
                egui::TextEdit::singleline(&mut state.data.url)
                    .font(egui::TextStyle::Monospace)
                    .layouter(&mut layouter),
            );
            if resp.changed() {
                state.dirty = true;
                action = EditorAction::DataChanged;
            }
            if resp.lost_focus() {
                action = EditorAction::UrlCommitted;
            }
            super::var_highlight::show_variable_tooltip(
                ui, &resp, &state.data.url, variables,
            );
        }

        if super::theme::pill_button(ui, "Send", super::theme::ACCENT) {
            action = EditorAction::Send;
        }

        // cURL dropdown menu
        let menu_btn = ui.add(
            egui::Button::new(
                egui::RichText::new("⋮")
                    .size(16.0)
                    .color(super::theme::TEXT_SECONDARY),
            )
            .fill(super::theme::SURFACE_1)
            .stroke(egui::Stroke::new(1.0, super::theme::BORDER))
            .corner_radius(egui::CornerRadius::same(4)),
        );
        if menu_btn.clicked() {
            ui.memory_mut(|m| m.toggle_popup(menu_btn.id));
        }
        egui::popup_below_widget(ui, menu_btn.id, &menu_btn, egui::PopupCloseBehavior::CloseOnClick, |ui| {
            ui.set_min_width(160.0);
            if ui.button("📋 Copy as cURL").clicked() {
                action = EditorAction::CopyCurl;
            }
            if ui.button("📥 Import from cURL").clicked() {
                state.show_curl_import = !state.show_curl_import;
                state.curl_import_buf.clear();
                state.curl_import_error = None;
            }
        });
    });

    // ── cURL import panel (collapsible) ──
    if state.show_curl_import {
        ui.add_space(4.0);
        egui::Frame::default()
            .fill(super::theme::SURFACE_1)
            .stroke(egui::Stroke::new(1.0, super::theme::BORDER))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("PASTE CURL COMMAND")
                            .strong()
                            .size(11.0)
                            .color(super::theme::TEXT_SECONDARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✕").clicked() {
                            state.show_curl_import = false;
                        }
                    });
                });
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::multiline(&mut state.curl_import_buf)
                        .desired_width(f32::INFINITY)
                        .desired_rows(4)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("curl https://api.example.com ..."),
                );
                if let Some(err) = &state.curl_import_error {
                    ui.label(
                        egui::RichText::new(err)
                            .size(11.0)
                            .color(egui::Color32::from_rgb(249, 62, 62)),
                    );
                }
                ui.add_space(4.0);
                if super::theme::pill_button(ui, "Import", super::theme::ACCENT) {
                    match crate::http::curl::parse_curl(&state.curl_import_buf) {
                        Ok(data) => {
                            state.show_curl_import = false;
                            state.curl_import_error = None;
                            action = EditorAction::ImportCurl(data);
                        }
                        Err(e) => {
                            state.curl_import_error = Some(e);
                        }
                    }
                }
            });
    }

    ui.add_space(6.0);

    // ── Flat scrollable view: Body ──
    // ── Body type selector row ──
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("BODY")
                .strong()
                .size(11.0)
                .color(super::theme::TEXT_SECONDARY),
        );
        ui.add_space(8.0);
        for bt in BodyType::all() {
            let is_active = state.data.body_type == *bt;
            let rt = if is_active {
                egui::RichText::new(bt.as_str())
                    .strong()
                    .size(12.0)
                    .color(super::theme::ACCENT)
            } else {
                egui::RichText::new(bt.as_str())
                    .size(12.0)
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
                        .corner_radius(egui::CornerRadius::same(4)),
                )
                .clicked()
            {
                state.data.body_type = bt.clone();
                state.dirty = true;
                action = EditorAction::DataChanged;
            }
        }

        // JSON validation indicator inline
        if state.data.body_type == BodyType::Json {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(err) = &state.json_error {
                    ui.label(
                        egui::RichText::new(format!("! {err}"))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(249, 62, 62)),
                    );
                } else if !state.data.body.is_empty() {
                    ui.label(
                        egui::RichText::new("✓ Valid JSON")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(73, 204, 144)),
                    );
                }
            });
        }
    });

    // ── Body editor filling all remaining space ──
    if state.data.body_type != BodyType::None {
        let remaining = ui.available_height();
        let vars_clone = variables.clone();

        if state.data.body_type == BodyType::Json {
            let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
                ui.ctx(),
                ui.style(),
            );
            let mut layouter =
                move |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                    let text = buf.as_str();
                    let mut job = egui_extras::syntax_highlighting::highlight(
                        ui.ctx(),
                        ui.style(),
                        &theme,
                        text,
                        "json",
                    );
                    patch_variable_colors(text, &mut job);
                    job.wrap.max_width = wrap_width;
                    ui.fonts_mut(|f| f.layout_job(job))
                };

            let response = ui.add_sized(
                egui::vec2(ui.available_width(), remaining),
                egui::TextEdit::multiline(&mut state.data.body)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .margin(egui::Margin::same(2))
                    .layouter(&mut layouter),
            );

            if response.changed() {
                state.dirty = true;
                action = EditorAction::DataChanged;
                if !state.data.body.is_empty() {
                    match serde_json::from_str::<serde_json::Value>(&state.data.body) {
                        Ok(_) => state.json_error = None,
                        Err(e) => state.json_error = Some(e.to_string()),
                    }
                } else {
                    state.json_error = None;
                }
            }
            super::var_highlight::show_variable_tooltip(
                ui, &response, &state.data.body, &vars_clone,
            );
        } else {
            let mut layouter = super::var_highlight::var_text_layouter;
            let response = ui.add_sized(
                egui::vec2(ui.available_width(), remaining),
                egui::TextEdit::multiline(&mut state.data.body)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .margin(egui::Margin::same(2))
                    .layouter(&mut layouter),
            );
            if response.changed() {
                state.dirty = true;
                action = EditorAction::DataChanged;
            }
            super::var_highlight::show_variable_tooltip(
                ui, &response, &state.data.body, &vars_clone,
            );
        }
    }

    action
}

/// Patch an existing LayoutJob to color `{{variable}}` spans with VAR_COLOR.
fn patch_variable_colors(text: &str, job: &mut egui::text::LayoutJob) {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i + 3 < len {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end_offset) = text[i + 2..].find("}}") {
                let var_start = i;
                let var_end = i + 2 + end_offset + 2;
                // Find and update all sections that overlap with this range
                for section in &mut job.sections {
                    let s_start = section.byte_range.start;
                    let s_end = section.byte_range.end;
                    // If section is fully within the variable range, color it
                    if s_start >= var_start && s_end <= var_end {
                        section.format.color = super::var_highlight::VAR_COLOR;
                    }
                    // If section partially overlaps, we can't split it easily,
                    // so just color it if majority overlaps
                    else if s_start < var_end && s_end > var_start {
                        let overlap = s_end.min(var_end) - s_start.max(var_start);
                        if overlap > (s_end - s_start) / 2 {
                            section.format.color = super::var_highlight::VAR_COLOR;
                        }
                    }
                }
                i = var_end;
                continue;
            }
        }
        i += 1;
    }
}

pub(crate) fn count_active_pairs(pairs: &[KeyValuePair]) -> usize {
    pairs
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .count()
}
