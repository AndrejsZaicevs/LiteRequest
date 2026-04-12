use eframe::egui;
use std::collections::HashMap;

use crate::models::*;

// ── Search result types ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SearchResultKind {
    Request,       // request name or URL match
    Collection,    // collection name or base_path match
    Folder,        // folder name
    Header,        // header key/value in a version
    QueryParam,    // query param key/value
    Body,          // request body content
    ResponseBody,  // response body content
    Variable,      // env or collection variable
}

impl SearchResultKind {
    fn label(&self) -> &str {
        match self {
            Self::Request => "Request",
            Self::Collection => "Collection",
            Self::Folder => "Folder",
            Self::Header => "Header",
            Self::QueryParam => "Param",
            Self::Body => "Body",
            Self::ResponseBody => "Response",
            Self::Variable => "Variable",
        }
    }

    fn color(&self) -> egui::Color32 {
        match self {
            Self::Request => super::theme::ACCENT,
            Self::Collection => egui::Color32::from_rgb(168, 85, 247),  // purple
            Self::Folder => egui::Color32::from_rgb(234, 179, 8),      // yellow
            Self::Header => egui::Color32::from_rgb(34, 197, 94),      // green
            Self::QueryParam => egui::Color32::from_rgb(6, 182, 212),  // cyan
            Self::Body => egui::Color32::from_rgb(249, 115, 22),       // orange
            Self::ResponseBody => egui::Color32::from_rgb(236, 72, 153), // pink
            Self::Variable => egui::Color32::from_rgb(139, 92, 246),   // violet
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub kind: SearchResultKind,
    pub title: String,         // primary display text
    pub context: String,       // secondary context (e.g. "GET /api/users")
    pub request_id: Option<String>,
    pub collection_id: Option<String>,
}

// ── Search state ─────────────────────────────────────────────

pub struct GlobalSearchState {
    pub open: bool,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected_idx: usize,
    focus_input: bool,
}

impl Default for GlobalSearchState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            results: Vec::new(),
            selected_idx: 0,
            focus_input: false,
        }
    }
}

impl GlobalSearchState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.query.clear();
            self.results.clear();
            self.selected_idx = 0;
            self.focus_input = true;
        }
    }
}

// ── Search action (returned to app) ─────────────────────────

#[derive(Debug, Clone)]
pub enum SearchAction {
    None,
    NavigateRequest(String),
    NavigateCollection(String),
    Close,
}

// ── Search engine ────────────────────────────────────────────

pub fn perform_search(
    query: &str,
    collections: &[Collection],
    folders: &[Folder],
    requests: &[Request],
    executions: &[RequestExecution],
    environments: &[Environment],
    env_variables: &[EnvVariable],
    all_versions: &HashMap<String, Vec<RequestVersion>>,
) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    if query_lower.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();

    // Search collections
    for c in collections {
        if c.name.to_lowercase().contains(&query_lower)
            || c.base_path.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult {
                kind: SearchResultKind::Collection,
                title: c.name.clone(),
                context: if c.base_path.is_empty() {
                    String::new()
                } else {
                    c.base_path.clone()
                },
                request_id: None,
                collection_id: Some(c.id.clone()),
            });
        }
    }

    // Search folders
    for f in folders {
        if f.name.to_lowercase().contains(&query_lower) {
            let coll_name = collections
                .iter()
                .find(|c| c.id == f.collection_id)
                .map(|c| c.name.as_str())
                .unwrap_or("");
            results.push(SearchResult {
                kind: SearchResultKind::Folder,
                title: f.name.clone(),
                context: format!("in {coll_name}"),
                request_id: None,
                collection_id: Some(f.collection_id.clone()),
            });
        }
    }

    // Search requests (name match)
    for r in requests {
        if r.name.to_lowercase().contains(&query_lower) {
            let coll_name = collections
                .iter()
                .find(|c| c.id == r.collection_id)
                .map(|c| c.name.as_str())
                .unwrap_or("");
            // Get method from current version
            let method_str = all_versions
                .get(&r.id)
                .and_then(|vs| vs.first())
                .map(|v| v.data.method.as_str())
                .unwrap_or("GET");
            results.push(SearchResult {
                kind: SearchResultKind::Request,
                title: r.name.clone(),
                context: format!("{method_str} · {coll_name}"),
                request_id: Some(r.id.clone()),
                collection_id: Some(r.collection_id.clone()),
            });
        }
    }

    // Search versions (URL, headers, query params, body)
    for (request_id, ver_list) in all_versions {
        // Only search latest version per request to avoid noise
        if let Some(v) = ver_list.first() {
            let req = requests.iter().find(|r| r.id == *request_id);
            let req_name = req.map(|r| r.name.as_str()).unwrap_or("Unknown");

            // URL match
            if v.data.url.to_lowercase().contains(&query_lower) {
                // Don't duplicate if request name already matched
                if !req_name.to_lowercase().contains(&query_lower) {
                    results.push(SearchResult {
                        kind: SearchResultKind::Request,
                        title: req_name.to_string(),
                        context: format!("{} {}", v.data.method.as_str(), &v.data.url),
                        request_id: Some(request_id.clone()),
                        collection_id: req.map(|r| r.collection_id.clone()),
                    });
                }
            }

            // Header key/value match
            for h in &v.data.headers {
                if h.key.to_lowercase().contains(&query_lower)
                    || h.value.to_lowercase().contains(&query_lower)
                {
                    results.push(SearchResult {
                        kind: SearchResultKind::Header,
                        title: format!("{}: {}", h.key, h.value),
                        context: req_name.to_string(),
                        request_id: Some(request_id.clone()),
                        collection_id: req.map(|r| r.collection_id.clone()),
                    });
                    break; // one match per request is enough
                }
            }

            // Query param match
            for p in &v.data.query_params {
                if p.key.to_lowercase().contains(&query_lower)
                    || p.value.to_lowercase().contains(&query_lower)
                {
                    results.push(SearchResult {
                        kind: SearchResultKind::QueryParam,
                        title: format!("{}={}", p.key, p.value),
                        context: req_name.to_string(),
                        request_id: Some(request_id.clone()),
                        collection_id: req.map(|r| r.collection_id.clone()),
                    });
                    break;
                }
            }

            // Body match
            if !v.data.body.is_empty() && v.data.body.to_lowercase().contains(&query_lower) {
                let snippet = extract_snippet(&v.data.body, &query_lower, 60);
                results.push(SearchResult {
                    kind: SearchResultKind::Body,
                    title: snippet,
                    context: req_name.to_string(),
                    request_id: Some(request_id.clone()),
                    collection_id: req.map(|r| r.collection_id.clone()),
                });
            }
        }
    }

    // Search execution response bodies (just status text, not full body for perf)
    for e in executions {
        if e.response.status_text.to_lowercase().contains(&query_lower) {
            let req = requests.iter().find(|r| r.id == e.request_id);
            let req_name = req.map(|r| r.name.as_str()).unwrap_or("Unknown");
            results.push(SearchResult {
                kind: SearchResultKind::ResponseBody,
                title: format!("{} {}", e.response.status, e.response.status_text),
                context: req_name.to_string(),
                request_id: Some(e.request_id.clone()),
                collection_id: req.map(|r| r.collection_id.clone()),
            });
        }
    }

    // Search environment variables
    for v in env_variables {
        if v.key.to_lowercase().contains(&query_lower)
            || v.value.to_lowercase().contains(&query_lower)
        {
            let env_name = environments
                .iter()
                .find(|e| e.id == v.environment_id)
                .map(|e| e.name.as_str())
                .unwrap_or("");
            results.push(SearchResult {
                kind: SearchResultKind::Variable,
                title: format!("{} = {}", v.key, v.value),
                context: format!("env: {env_name}"),
                request_id: None,
                collection_id: None,
            });
        }
    }

    // Deduplicate: keep first result per (kind, request_id/collection_id) combo
    let mut seen = std::collections::HashSet::new();
    results.retain(|r| {
        let key = format!(
            "{:?}:{}:{}",
            r.kind,
            r.request_id.as_deref().unwrap_or(""),
            r.collection_id.as_deref().unwrap_or("")
        );
        seen.insert(key)
    });

    // Cap results
    results.truncate(50);
    results
}

fn extract_snippet(text: &str, query: &str, max_len: usize) -> String {
    let lower = text.to_lowercase();
    if let Some(pos) = lower.find(query) {
        let start = pos.saturating_sub(max_len / 2);
        let end = (pos + query.len() + max_len / 2).min(text.len());
        // Find safe char boundaries
        let start = text.floor_char_boundary(start);
        let end = text.ceil_char_boundary(end);
        let mut snippet = text[start..end].replace('\n', " ");
        if start > 0 {
            snippet = format!("…{snippet}");
        }
        if end < text.len() {
            snippet = format!("{snippet}…");
        }
        snippet
    } else {
        text.chars().take(max_len).collect()
    }
}

// ── UI rendering ─────────────────────────────────────────────

pub fn render_search_modal(
    ctx: &egui::Context,
    state: &mut GlobalSearchState,
) -> SearchAction {
    if !state.open {
        return SearchAction::None;
    }

    let mut action = SearchAction::None;

    // Semi-transparent background overlay
    let screen_rect = ctx.content_rect();
    let overlay_layer = egui::LayerId::new(egui::Order::Background, egui::Id::new("search_overlay"));
    let painter = ctx.layer_painter(overlay_layer);
    painter.rect_filled(
        screen_rect,
        0.0,
        egui::Color32::from_black_alpha(120),
    );

    // Modal window
    let modal_width = (screen_rect.width() * 0.5).clamp(360.0, 600.0);

    let mut open = true;
    egui::Window::new("search_modal_window")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(modal_width, 0.0)) // auto height
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, screen_rect.height() * 0.15))
        .frame(
            egui::Frame::window(&ctx.global_style())
                .fill(super::theme::SURFACE_1)
                .stroke(egui::Stroke::new(1.0, super::theme::BORDER))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(0)),
        )
        .open(&mut open)
        .show(ctx, |ui| {
            // ── Search input ─────────────────────────────
            ui.add_space(2.0);
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(egui_phosphor::regular::MAGNIFYING_GLASS)
                                .size(18.0)
                                .color(super::theme::TEXT_SECONDARY),
                        );
                        let input = ui.add_sized(
                            egui::vec2(ui.available_width(), super::theme::INPUT_HEIGHT),
                            egui::TextEdit::singleline(&mut state.query)
                                .hint_text("Search requests, collections, headers, bodies…")
                                .frame(egui::Frame::NONE)
                                .font(egui::TextStyle::Body)
                                .text_color(super::theme::TEXT_PRIMARY),
                        );
                        if state.focus_input {
                            input.request_focus();
                            state.focus_input = false;
                        }
                    });
                });

            // Separator
            ui.add(egui::Separator::default().spacing(0.0));

            // ── Handle keyboard navigation ───────────────
            let n = state.results.len();
            if n > 0 {
                ui.input(|i| {
                    if i.key_pressed(egui::Key::ArrowDown) {
                        state.selected_idx = (state.selected_idx + 1).min(n - 1);
                    }
                    if i.key_pressed(egui::Key::ArrowUp) {
                        state.selected_idx = state.selected_idx.saturating_sub(1);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && state.selected_idx < n {
                    let result = &state.results[state.selected_idx];
                    action = result_action(result);
                }
            }

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                action = SearchAction::Close;
            }

            // ── Results list ─────────────────────────────
            if state.results.is_empty() && !state.query.is_empty() {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("No results found")
                                .color(super::theme::TEXT_MUTED)
                                .size(13.0),
                        );
                    });
            } else if !state.results.is_empty() {
                let max_visible = 10;
                let row_height = 36.0;
                let visible_count = state.results.len().min(max_visible);
                let scroll_height = visible_count as f32 * row_height + 8.0;

                egui::ScrollArea::vertical()
                    .max_height(scroll_height)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        for (idx, result) in state.results.iter().enumerate() {
                            let is_selected = idx == state.selected_idx;
                            let clicked = render_search_result(ui, result, is_selected, row_height);
                            if clicked {
                                action = result_action(result);
                            }
                        }
                        ui.add_space(4.0);
                    });
            }

            // ── Footer with keyboard hints ───────────────
            ui.add(egui::Separator::default().spacing(0.0));
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let hint = |ui: &mut egui::Ui, key: &str, desc: &str| {
                            ui.label(
                                egui::RichText::new(key)
                                    .size(10.0)
                                    .color(super::theme::TEXT_MUTED)
                                    .family(egui::FontFamily::Monospace),
                            );
                            ui.label(
                                egui::RichText::new(desc)
                                    .size(10.0)
                                    .color(super::theme::TEXT_MUTED),
                            );
                            ui.add_space(8.0);
                        };
                        hint(ui, "↑↓", "navigate");
                        hint(ui, "↵", "open");
                        hint(ui, "esc", "close");
                    });
                });
        });

    if !open {
        action = SearchAction::Close;
    }

    action
}

fn render_search_result(
    ui: &mut egui::Ui,
    result: &SearchResult,
    is_selected: bool,
    row_height: f32,
) -> bool {
    let available_width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(available_width, row_height),
        egui::Sense::click(),
    );

    if !ui.is_rect_visible(rect) {
        return false;
    }

    let painter = ui.painter();

    // Hover / selection highlight
    let bg = if is_selected {
        super::theme::SURFACE_2
    } else if response.hovered() {
        egui::Color32::from_rgba_premultiplied(42, 42, 48, 140)
    } else {
        egui::Color32::TRANSPARENT
    };
    painter.rect_filled(rect.shrink2(egui::vec2(4.0, 1.0)), 4.0, bg);

    let inner = rect.shrink2(egui::vec2(12.0, 0.0));

    // Kind badge
    let badge_text = result.kind.label();
    let badge_color = result.kind.color();
    let badge_font = egui::FontId::new(9.0, egui::FontFamily::Proportional);
    let badge_galley = painter.layout_no_wrap(badge_text.to_string(), badge_font, badge_color);
    let badge_width = badge_galley.size().x + 8.0;
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(inner.left(), inner.center().y - 8.0),
        egui::vec2(badge_width, 16.0),
    );
    painter.rect_filled(
        badge_rect,
        3.0,
        badge_color.gamma_multiply(0.15),
    );
    painter.galley(
        egui::pos2(badge_rect.left() + 4.0, badge_rect.top() + 2.0),
        badge_galley,
        badge_color,
    );

    // Title
    let title_left = badge_rect.right() + 8.0;
    let title_font = egui::FontId::new(13.0, egui::FontFamily::Proportional);
    let max_title_width = inner.right() - title_left - 8.0;
    let title_galley = painter.layout(
        result.title.clone(),
        title_font,
        super::theme::TEXT_PRIMARY,
        max_title_width.max(50.0),
    );
    painter.galley(
        egui::pos2(title_left, inner.center().y - title_galley.size().y * 0.5 - if result.context.is_empty() { 0.0 } else { 5.0 }),
        title_galley,
        super::theme::TEXT_PRIMARY,
    );

    // Context (below title)
    if !result.context.is_empty() {
        let ctx_font = egui::FontId::new(10.5, egui::FontFamily::Proportional);
        let ctx_galley = painter.layout(
            result.context.clone(),
            ctx_font,
            super::theme::TEXT_MUTED,
            max_title_width.max(50.0),
        );
        painter.galley(
            egui::pos2(title_left, inner.center().y + 3.0),
            ctx_galley,
            super::theme::TEXT_MUTED,
        );
    }

    // Scroll selected item into view
    if is_selected {
        response.scroll_to_me(Some(egui::Align::Center));
    }

    response.clicked()
}

fn result_action(result: &SearchResult) -> SearchAction {
    if let Some(ref rid) = result.request_id {
        SearchAction::NavigateRequest(rid.clone())
    } else if let Some(ref cid) = result.collection_id {
        SearchAction::NavigateCollection(cid.clone())
    } else {
        SearchAction::Close
    }
}
