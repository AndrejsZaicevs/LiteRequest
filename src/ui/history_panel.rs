use eframe::egui;
use crate::models::*;

pub fn render_version_history(
    ui: &mut egui::Ui,
    versions: &[RequestVersion],
    selected_version_id: Option<&str>,
) -> Option<String> {
    let mut selected = None;

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Versions")
                .strong()
                .size(14.0)
                .color(super::theme::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(format!("({})", versions.len()))
                .size(12.0)
                .color(super::theme::TEXT_MUTED),
        );
    });
    ui.label(
        egui::RichText::new("How the request was built")
            .size(11.0)
            .color(super::theme::TEXT_MUTED),
    );
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .id_salt("version_history_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui: &mut egui::Ui| {
            if versions.is_empty() {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("No versions yet -- start editing")
                        .color(super::theme::TEXT_MUTED)
                        .italics(),
                );
                return;
            }

            for (i, version) in versions.iter().enumerate() {
                let is_selected = selected_version_id == Some(&version.id);
                let [r, g, b] = version.data.method.color();

                let fill = if is_selected {
                    super::theme::ACCENT.gamma_multiply(0.12)
                } else {
                    egui::Color32::TRANSPARENT
                };

                let frame_resp = egui::Frame::default()
                    .fill(fill)
                    .corner_radius(egui::CornerRadius::same(5))
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label(
                                egui::RichText::new(format!("v{}", versions.len() - i))
                                    .strong()
                                    .size(11.0)
                                    .color(super::theme::TEXT_MUTED),
                            );
                            ui.label(
                                egui::RichText::new(version.data.method.as_str())
                                    .strong()
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(r, g, b)),
                            );
                            ui.label(
                                egui::RichText::new(truncate_str(&version.data.url, 22))
                                    .size(11.0)
                                    .color(super::theme::TEXT_SECONDARY),
                            );
                        });
                        ui.label(
                            egui::RichText::new(format_timestamp(&version.created_at))
                                .size(10.0)
                                .color(super::theme::TEXT_MUTED),
                        );
                    });

                // Make the entire frame area clickable
                let click_resp = ui.interact(
                    frame_resp.response.rect,
                    ui.id().with(("version_click", &version.id)),
                    egui::Sense::click(),
                );
                if click_resp.clicked() {
                    selected = Some(version.id.clone());
                }

                if i < versions.len() - 1 {
                    ui.add_space(1.0);
                }
            }
        });

    selected
}

pub fn render_execution_history(
    ui: &mut egui::Ui,
    executions: &[RequestExecution],
    selected_execution_id: Option<&str>,
) -> Option<String> {
    let mut selected = None;

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Executions")
                .strong()
                .size(14.0)
                .color(super::theme::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(format!("({})", executions.len()))
                .size(12.0)
                .color(super::theme::TEXT_MUTED),
        );
    });
    ui.label(
        egui::RichText::new("What the server returned")
            .size(11.0)
            .color(super::theme::TEXT_MUTED),
    );
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .id_salt("execution_history_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui: &mut egui::Ui| {
            if executions.is_empty() {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("No executions yet -- hit Send")
                        .color(super::theme::TEXT_MUTED)
                        .italics(),
                );
                return;
            }

            for (i, exec) in executions.iter().enumerate() {
                let is_selected = selected_execution_id == Some(&exec.id);
                let status_color = super::theme::status_color(exec.response.status);

                let fill = if is_selected {
                    super::theme::ACCENT.gamma_multiply(0.12)
                } else {
                    egui::Color32::TRANSPARENT
                };

                let frame_resp = egui::Frame::default()
                    .fill(fill)
                    .corner_radius(egui::CornerRadius::same(5))
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{}", exec.response.status))
                                        .strong()
                                        .size(11.0)
                                        .color(egui::Color32::WHITE),
                                )
                                .fill(status_color)
                                .corner_radius(egui::CornerRadius::same(3))
                                .sense(egui::Sense::hover()),
                            );
                            ui.label(
                                egui::RichText::new(&exec.response.status_text)
                                    .size(12.0)
                                    .color(super::theme::TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(format!("{}ms", exec.latency_ms))
                                    .size(11.0)
                                    .color(super::theme::TEXT_MUTED),
                            );
                        });
                        ui.label(
                            egui::RichText::new(format_timestamp(&exec.executed_at))
                                .size(10.0)
                                .color(super::theme::TEXT_MUTED),
                        );
                    });

                // Make the entire frame area clickable
                let click_resp = ui.interact(
                    frame_resp.response.rect,
                    ui.id().with(("exec_click", &exec.id)),
                    egui::Sense::click(),
                );
                if click_resp.clicked() {
                    selected = Some(exec.id.clone());
                }

                if i < executions.len() - 1 {
                    ui.add_space(1.0);
                }
            }
        });

    selected
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

fn format_timestamp(ts: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        dt.format("%H:%M:%S").to_string()
    } else {
        truncate_str(ts, 19)
    }
}
