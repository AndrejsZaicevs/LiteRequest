use eframe::egui;
use std::collections::HashMap;

/// Color for `{{variable}}` tokens
pub const VAR_COLOR: egui::Color32 = egui::Color32::from_rgb(252, 186, 3); // amber/gold

/// Build a `LayoutJob` that renders normal text in `base_color` and
/// `{{variable}}` references in `VAR_COLOR`.
pub fn highlight_variables_job(
    text: &str,
    base_color: egui::Color32,
    font_id: egui::FontId,
    wrap_width: f32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut normal_start = 0;

    while i < len {
        if i + 1 < len && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Flush normal text before this variable
            if i > normal_start {
                job.append(
                    &text[normal_start..i],
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color: base_color,
                        ..Default::default()
                    },
                );
            }

            // Find closing }}
            if let Some(end_offset) = text[i + 2..].find("}}") {
                let var_end = i + 2 + end_offset + 2;
                job.append(
                    &text[i..var_end],
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color: VAR_COLOR,
                        ..Default::default()
                    },
                );
                i = var_end;
                normal_start = i;
                continue;
            } else {
                // No closing }} — treat as normal text
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    // Flush remaining normal text
    if normal_start < len {
        job.append(
            &text[normal_start..],
            0.0,
            egui::TextFormat {
                font_id,
                color: base_color,
                ..Default::default()
            },
        );
    }

    job
}

/// Layouter function for use with `TextEdit::layouter()`.
/// Highlights `{{variable}}` patterns in amber.
pub fn var_text_layouter(ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32) -> std::sync::Arc<egui::Galley> {
    let font_id = egui::FontId::new(13.0, egui::FontFamily::Monospace);
    let base_color = super::theme::TEXT_PRIMARY;
    let job = highlight_variables_job(text.as_str(), base_color, font_id, wrap_width);
    ui.fonts_mut(|f| f.layout_job(job))
}

/// Show tooltip when hovering over `{{variable}}` in a response rect.
/// Call this after adding a TextEdit, passing its response.
pub fn show_variable_tooltip(
    ui: &egui::Ui,
    response: &egui::Response,
    text: &str,
    variables: &HashMap<String, String>,
) {
    if !response.hovered() {
        return;
    }
    let Some(pointer) = ui.ctx().pointer_hover_pos() else {
        return;
    };
    // Estimate character position from pointer x relative to response rect
    let char_width = 7.8; // approximate monospace char width at 13px
    let x_offset = pointer.x - response.rect.left();
    let char_idx = (x_offset / char_width).max(0.0) as usize;

    // Find if char_idx falls inside a {{variable}}
    if let Some((var_name, _start, _end)) = find_var_at_pos(text, char_idx) {
        if let Some(value) = variables.get(var_name) {
            response.clone().on_hover_ui_at_pointer(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(var_name)
                            .color(VAR_COLOR)
                            .strong()
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new("=")
                            .color(super::theme::TEXT_MUTED)
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new(value)
                            .color(super::theme::TEXT_PRIMARY)
                            .size(12.0)
                            .family(egui::FontFamily::Monospace),
                    );
                });
            });
        } else {
            response.clone().on_hover_ui_at_pointer(|ui| {
                ui.label(
                    egui::RichText::new(format!("{var_name}: undefined"))
                        .color(egui::Color32::from_rgb(249, 62, 62))
                        .size(12.0),
                );
            });
        }
    }
}

/// Find the variable name at a given character index, if any.
fn find_var_at_pos(text: &str, char_idx: usize) -> Option<(&str, usize, usize)> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i + 3 < len {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end_offset) = text[i + 2..].find("}}") {
                let var_start = i;
                let var_end = i + 2 + end_offset + 2;
                if char_idx >= var_start && char_idx < var_end {
                    let name = &text[i + 2..i + 2 + end_offset];
                    return Some((name.trim(), var_start, var_end));
                }
                i = var_end;
                continue;
            }
        }
        i += 1;
    }
    None
}

/// Resolve variables in a string for display purposes.
pub fn resolve_display(input: &str, variables: &HashMap<String, String>) -> String {
    crate::http::interpolation::interpolate(input, variables)
}
