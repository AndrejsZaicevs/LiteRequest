use eframe::egui;
use crate::models::*;

// ── URL / size helpers ──────────────────────────────────────────

fn extract_path(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let path = parsed.path();
        if path.is_empty() || path == "/" {
            return url.to_string();
        }
        return path.to_string();
    }
    if let Some(rest) = url.find("://").map(|i| &url[i + 3..]) {
        if let Some(slash) = rest.find('/') {
            return rest[slash..].to_string();
        }
    }
    url.to_string()
}

fn fmt_size(bytes: usize) -> String {
    if bytes == 0 {
        return String::new();
    }
    if bytes < 1024 {
        return format!("{}b", bytes);
    }
    format!("{:.1}KB", bytes as f64 / 1024.0)
}

// ── Time bucketing ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeBucket {
    Today = 0,
    Yesterday = 1,
    LastWeek = 2,
    LastMonth = 3,
    Older = 4,
}

impl TimeBucket {
    fn label(&self) -> &'static str {
        match self {
            TimeBucket::Today => "Today",
            TimeBucket::Yesterday => "Yesterday",
            TimeBucket::LastWeek => "Last 7 Days",
            TimeBucket::LastMonth => "Last 30 Days",
            TimeBucket::Older => "Older",
        }
    }

    fn from_timestamp(ts: &str, now: &chrono::DateTime<chrono::Utc>) -> Self {
        let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) else {
            return TimeBucket::Older;
        };
        let dt_utc = dt.with_timezone(&chrono::Utc);
        let diff = *now - dt_utc;

        if diff.num_hours() < 24 && now.date_naive() == dt_utc.date_naive() {
            TimeBucket::Today
        } else if diff.num_hours() < 48 {
            TimeBucket::Yesterday
        } else if diff.num_days() < 7 {
            TimeBucket::LastWeek
        } else if diff.num_days() < 30 {
            TimeBucket::LastMonth
        } else {
            TimeBucket::Older
        }
    }
}

const ALL_BUCKETS: [TimeBucket; 5] = [
    TimeBucket::Today,
    TimeBucket::Yesterday,
    TimeBucket::LastWeek,
    TimeBucket::LastMonth,
    TimeBucket::Older,
];

// ── Filtered + grouped execution list ───────────────────────────

pub fn render_execution_list_filtered(
    ui: &mut egui::Ui,
    executions: &[RequestExecution],
    selected_execution_id: Option<&str>,
    selected_version_id: Option<&str>,
    environments: &[Environment],
    filter_version: &mut bool,
    filter_env: &mut bool,
    time_expanded: &mut [bool; 5],
    groups_initialized: &mut bool,
) -> Option<String> {
    if executions.is_empty() {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("No executions yet — hit Send")
                .size(11.0)
                .color(super::theme::TEXT_MUTED)
                .italics(),
        );
        return None;
    }

    // Filter toolbar
    ui.add_space(2.0);
    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(egui_phosphor::regular::FUNNEL)
                        .size(12.0)
                        .color(super::theme::TEXT_MUTED),
                );
                ui.checkbox(filter_version, "");
                ui.label(
                    egui::RichText::new("This version")
                        .size(11.0)
                        .color(if *filter_version {
                            super::theme::TEXT_PRIMARY
                        } else {
                            super::theme::TEXT_MUTED
                        }),
                );
                ui.add_space(6.0);
                ui.checkbox(filter_env, "");
                ui.label(
                    egui::RichText::new("Active env")
                        .size(11.0)
                        .color(if *filter_env {
                            super::theme::TEXT_PRIMARY
                        } else {
                            super::theme::TEXT_MUTED
                        }),
                );
            });
        });
    ui.add_space(4.0);

    // Apply filters
    let active_env_id: Option<String> = if *filter_env {
        environments.iter().find(|e| e.is_active).map(|e| e.id.clone())
    } else {
        None
    };

    let filtered: Vec<&RequestExecution> = executions
        .iter()
        .filter(|e| {
            if *filter_version {
                if let Some(vid) = selected_version_id {
                    if e.version_id != vid {
                        return false;
                    }
                }
            }
            if let Some(ref env_id) = active_env_id {
                if !e.environment_id.is_empty() && e.environment_id != *env_id {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered.is_empty() {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("No executions match filters")
                .size(11.0)
                .color(super::theme::TEXT_MUTED)
                .italics(),
        );
        return None;
    }

    // Group by time bucket
    let now = chrono::Utc::now();
    let mut buckets: [Vec<&RequestExecution>; 5] = Default::default();
    for e in &filtered {
        let b = TimeBucket::from_timestamp(&e.executed_at, &now);
        buckets[b as usize].push(e);
    }

    // Auto-initialize: expand only the first non-empty bucket
    if !*groups_initialized {
        *groups_initialized = true;
        let mut found_first = false;
        for (i, bucket) in buckets.iter().enumerate() {
            time_expanded[i] = !bucket.is_empty() && !found_first;
            if !bucket.is_empty() && !found_first {
                found_first = true;
            }
        }
    }

    let mut selected = None;
    let env_name_map: std::collections::HashMap<String, String> = environments
        .iter()
        .map(|e| (e.id.clone(), e.name.clone()))
        .collect();

    for bucket_type in &ALL_BUCKETS {
        let idx = *bucket_type as usize;
        let items = &buckets[idx];
        if items.is_empty() {
            continue;
        }

        // Mini-collapsible sub-header for the time group
        let expanded = &mut time_expanded[idx];
        if time_group_header(ui, bucket_type.label(), items.len(), expanded) {
            for exec in items {
                if let Some(eid) = render_single_execution(
                    ui,
                    exec,
                    selected_execution_id,
                    &env_name_map,
                ) {
                    selected = Some(eid);
                }
            }
            ui.add_space(2.0);
        }
    }

    selected
}

/// Small sub-header for time groups inside the execution list.
fn time_group_header(
    ui: &mut egui::Ui,
    label: &str,
    count: usize,
    expanded: &mut bool,
) -> bool {
    let icon = if *expanded {
        egui_phosphor::regular::CARET_DOWN
    } else {
        egui_phosphor::regular::CARET_RIGHT
    };

    let resp = ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(icon)
                .size(10.0)
                .color(super::theme::TEXT_MUTED),
        );
        ui.label(
            egui::RichText::new(label)
                .size(11.0)
                .strong()
                .color(super::theme::TEXT_SECONDARY),
        );
        ui.label(
            egui::RichText::new(format!("({})", count))
                .size(10.0)
                .color(super::theme::TEXT_MUTED),
        );
    });

    let click = ui.interact(
        resp.response.rect,
        ui.id().with(("time_group", label)),
        egui::Sense::click(),
    );
    if click.clicked() {
        *expanded = !*expanded;
    }
    if click.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    *expanded
}

/// Render a single execution row, showing env name if available.
fn render_single_execution(
    ui: &mut egui::Ui,
    exec: &RequestExecution,
    selected_execution_id: Option<&str>,
    env_name_map: &std::collections::HashMap<String, String>,
) -> Option<String> {
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
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format_timestamp(&exec.executed_at))
                        .size(10.0)
                        .color(super::theme::TEXT_MUTED),
                );
                if !exec.environment_id.is_empty() {
                    if let Some(name) = env_name_map.get(&exec.environment_id) {
                        ui.label(
                            egui::RichText::new(format!("• {name}"))
                                .size(10.0)
                                .color(super::theme::ACCENT.gamma_multiply(0.7)),
                        );
                    }
                }
            });
        });

    let click_resp = ui.interact(
        frame_resp.response.rect,
        ui.id().with(("exec_click", &exec.id)),
        egui::Sense::click(),
    );
    if click_resp.clicked() {
        return Some(exec.id.clone());
    }
    None
}

// ── Original functions (still used by standalone panels) ────────

/// Content-only version list (no title header — used inside inspector's own section header).
pub fn render_version_list(
    ui: &mut egui::Ui,
    versions: &[RequestVersion],
    selected_version_id: Option<&str>,
    time_expanded: &mut [bool; 5],
    groups_initialized: &mut bool,
) -> Option<String> {
    if versions.is_empty() {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("No versions yet — start editing")
                .size(11.0)
                .color(super::theme::TEXT_MUTED)
                .italics(),
        );
        return None;
    }

    // Group by time bucket
    let now = chrono::Utc::now();
    let mut buckets: [Vec<&RequestVersion>; 5] = Default::default();
    for v in versions {
        let b = TimeBucket::from_timestamp(&v.created_at, &now);
        buckets[b as usize].push(v);
    }

    // Auto-initialize: expand only first non-empty bucket
    if !*groups_initialized {
        *groups_initialized = true;
        let mut found_first = false;
        for (i, bucket) in buckets.iter().enumerate() {
            time_expanded[i] = !bucket.is_empty() && !found_first;
            if !bucket.is_empty() && !found_first {
                found_first = true;
            }
        }
    }

    let mut selected = None;

    for bucket_type in &ALL_BUCKETS {
        let idx = *bucket_type as usize;
        let items = &buckets[idx];
        if items.is_empty() {
            continue;
        }

        let expanded = &mut time_expanded[idx];
        if time_group_header(ui, bucket_type.label(), items.len(), expanded) {
            for version in items {
                if let Some(vid) = render_single_version(ui, version, selected_version_id) {
                    selected = Some(vid);
                }
            }
            ui.add_space(2.0);
        }
    }

    selected
}

fn render_single_version(
    ui: &mut egui::Ui,
    version: &RequestVersion,
    selected_version_id: Option<&str>,
) -> Option<String> {
    let is_selected = selected_version_id == Some(&version.id);
    let [r, g, b] = version.data.method.color();
    let method_color = egui::Color32::from_rgb(r, g, b);

    let fill = if is_selected {
        super::theme::ACCENT.gamma_multiply(0.12)
    } else {
        egui::Color32::TRANSPARENT
    };

    let path = extract_path(&version.data.url);
    let body_size = fmt_size(version.data.body.len());

    let frame_resp = egui::Frame::default()
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(ui, |ui: &mut egui::Ui| {
            ui.set_width(ui.available_width());
            // First line: method badge + truncated path
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.add(
                    egui::Button::new(
                        egui::RichText::new(version.data.method.as_str())
                            .strong()
                            .size(10.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(method_color)
                    .corner_radius(egui::CornerRadius::same(3))
                    .sense(egui::Sense::hover()),
                );
                ui.label(
                    egui::RichText::new(truncate_str(&path, 22))
                        .size(11.0)
                        .color(super::theme::TEXT_SECONDARY),
                );
            });
            // Second line: timestamp + optional body size
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format_timestamp(&version.created_at))
                        .size(10.0)
                        .color(super::theme::TEXT_MUTED),
                );
                if !body_size.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("· {}", body_size))
                            .size(10.0)
                            .color(super::theme::TEXT_MUTED),
                    );
                }
            });
        });

    let click_resp = ui.interact(
        frame_resp.response.rect,
        ui.id().with(("version_click", &version.id)),
        egui::Sense::click(),
    );
    if click_resp.clicked() {
        return Some(version.id.clone());
    }
    None
}

/// Content-only execution list (no title header — used inside inspector's own section header).
pub fn render_execution_list(
    ui: &mut egui::Ui,
    executions: &[RequestExecution],
    selected_execution_id: Option<&str>,
) -> Option<String> {
    let mut selected = None;

    if executions.is_empty() {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("No executions yet — hit Send")
                .size(11.0)
                .color(super::theme::TEXT_MUTED)
                .italics(),
        );
        return None;
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

    selected
}

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
                if let Some(vid) = render_single_version(ui, version, selected_version_id) {
                    selected = Some(vid);
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
        let local = dt.with_timezone(&chrono::Local);
        local.format("%H:%M:%S").to_string()
    } else {
        truncate_str(ts, 19)
    }
}
