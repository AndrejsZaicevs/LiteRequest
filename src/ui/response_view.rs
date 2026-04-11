use eframe::egui;
use crate::models::*;
use std::collections::HashSet;

pub enum ResponseTab {
    Body,
    Headers,
    Table,
}

pub struct ResponseViewState {
    pub tab: ResponseTab,
    pub json_expanded: HashSet<String>,
    pub search_query: String,
    pub pretty_body: Option<String>,
}

impl Default for ResponseViewState {
    fn default() -> Self {
        Self {
            tab: ResponseTab::Body,
            json_expanded: HashSet::new(),
            search_query: String::new(),
            pretty_body: None,
        }
    }
}

pub fn render_response_view(
    ui: &mut egui::Ui,
    execution: Option<&RequestExecution>,
    state: &mut ResponseViewState,
) {
    // ── Single-line divider: [status info] ... [Body Headers Table] ──
    egui::Frame::default()
        .fill(super::theme::SURFACE_2)
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui: &mut egui::Ui| {
            ui.horizontal(|ui: &mut egui::Ui| {
                // Left side: status info (or placeholder)
                if let Some(exec) = execution {
                    let status_color = super::theme::status_color(exec.response.status);
                    ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("{}", exec.response.status))
                                .strong()
                                .size(12.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(status_color)
                        .rounding(egui::Rounding::same(3.0))
                        .min_size(egui::vec2(0.0, 18.0))
                        .sense(egui::Sense::hover()),
                    );
                    ui.label(
                        egui::RichText::new(&exec.response.status_text)
                            .size(12.0)
                            .color(super::theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new(format!("· {}ms · {}", exec.latency_ms, format_size(exec.response.size_bytes)))
                            .size(12.0)
                            .color(super::theme::TEXT_MUTED),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("No response")
                            .size(12.0)
                            .color(super::theme::TEXT_MUTED)
                            .italics(),
                    );
                }

                // Right side: tabs
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Reversed order because right-to-left
                    for (tab_variant, label) in [
                        (ResponseTab::Table, "Table"),
                        (ResponseTab::Headers, "Headers"),
                        (ResponseTab::Body, "Body"),
                    ] {
                        let is_active = matches!(
                            (&state.tab, &tab_variant),
                            (ResponseTab::Body, ResponseTab::Body)
                                | (ResponseTab::Headers, ResponseTab::Headers)
                                | (ResponseTab::Table, ResponseTab::Table)
                        );
                        let rt = if is_active {
                            egui::RichText::new(label)
                                .strong()
                                .size(12.0)
                                .color(super::theme::ACCENT)
                        } else {
                            egui::RichText::new(label)
                                .size(12.0)
                                .color(super::theme::TEXT_SECONDARY)
                        };

                        let resp = ui.add(egui::Label::new(rt).sense(egui::Sense::click()));
                        if resp.clicked() {
                            state.tab = tab_variant;
                        }
                        if is_active {
                            let rect = resp.rect;
                            ui.painter().line_segment(
                                [rect.left_bottom(), rect.right_bottom()],
                                egui::Stroke::new(2.0, super::theme::ACCENT),
                            );
                        }
                    }
                });
            });
        });

    // ── Response body ──
    let Some(exec) = execution else {
        return;
    };

    ui.add_space(4.0);

    match state.tab {
        ResponseTab::Body => render_body_tab(ui, &exec.response, state),
        ResponseTab::Headers => render_headers_tab(ui, &exec.response),
        ResponseTab::Table => render_table_tab(ui, &exec.response),
    }
}

fn render_body_tab(ui: &mut egui::Ui, response: &ResponseData, state: &mut ResponseViewState) {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response.body) {
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if !state.search_query.is_empty() {
                    let pretty = state.pretty_body.get_or_insert_with(|| {
                        serde_json::to_string_pretty(&json).unwrap_or_default()
                    });
                    for line in pretty.lines() {
                        if line.to_lowercase().contains(&state.search_query.to_lowercase()) {
                            ui.label(egui::RichText::new(line).monospace());
                        }
                    }
                } else {
                    state.pretty_body = None;
                    super::json_tree::json_tree_ui(ui, &json, "", &mut state.json_expanded);
                }
            });
    } else {
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let body = if !state.search_query.is_empty() {
                    response
                        .body
                        .lines()
                        .filter(|l| l.to_lowercase().contains(&state.search_query.to_lowercase()))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    response.body.clone()
                };
                ui.label(egui::RichText::new(&body).monospace());
            });
    }
}

fn render_headers_tab(ui: &mut egui::Ui, response: &ResponseData) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("response_headers_grid")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    for (key, value) in &response.headers {
                        ui.label(
                            egui::RichText::new(key)
                                .color(egui::Color32::from_rgb(156, 220, 254))
                                .monospace(),
                        );
                        ui.label(egui::RichText::new(value).monospace());
                        ui.end_row();
                    }
                });
        });
}

fn render_table_tab(ui: &mut egui::Ui, response: &ResponseData) {
    if let Ok(serde_json::Value::Array(arr)) =
        serde_json::from_str::<serde_json::Value>(&response.body)
    {
        if arr.is_empty() {
            ui.label(
                egui::RichText::new("Empty array")
                    .color(super::theme::TEXT_MUTED)
                    .italics(),
            );
            return;
        }

        let mut columns: Vec<String> = Vec::new();
        for item in &arr {
            if let serde_json::Value::Object(map) = item {
                for key in map.keys() {
                    if !columns.contains(key) {
                        columns.push(key.clone());
                    }
                }
            }
        }

        if columns.is_empty() {
            ui.label(
                egui::RichText::new("Array does not contain objects")
                    .color(super::theme::TEXT_MUTED),
            );
            return;
        }

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("response_table")
                    .num_columns(columns.len())
                    .spacing([14.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for col in &columns {
                            ui.label(
                                egui::RichText::new(col)
                                    .strong()
                                    .color(super::theme::TEXT_SECONDARY),
                            );
                        }
                        ui.end_row();

                        for item in &arr {
                            if let serde_json::Value::Object(map) = item {
                                for col in &columns {
                                    let val = map
                                        .get(col)
                                        .map(|v| match v {
                                            serde_json::Value::String(s) => s.clone(),
                                            other => other.to_string(),
                                        })
                                        .unwrap_or_default();
                                    ui.label(egui::RichText::new(val).monospace());
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
    } else {
        ui.label(
            egui::RichText::new("Response is not a JSON array — table view unavailable.")
                .color(super::theme::TEXT_MUTED)
                .italics(),
        );
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
