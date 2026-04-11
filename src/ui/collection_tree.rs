use eframe::egui;
use std::collections::HashMap;
use crate::models::*;

#[derive(Debug, Clone, PartialEq)]
enum RenameTarget {
    Request(String),
    Folder(String),
    Collection(String),
}

pub struct CollectionTreeState {
    pub expanded_collections: std::collections::HashSet<String>,
    pub expanded_folders: std::collections::HashSet<String>,
    pub selected_request_id: Option<String>,
    pub selected_collection_id: Option<String>,
    pub rename_id: Option<String>,
    pub rename_buf: String,
    rename_target: Option<RenameTarget>,
    drag_request_id: Option<String>,
}

impl Default for CollectionTreeState {
    fn default() -> Self {
        Self {
            expanded_collections: std::collections::HashSet::new(),
            expanded_folders: std::collections::HashSet::new(),
            selected_request_id: None,
            selected_collection_id: None,
            rename_id: None,
            rename_buf: String::new(),
            rename_target: None,
            drag_request_id: None,
        }
    }
}

pub enum TreeAction {
    None,
    SelectRequest(String),
    SelectCollection(String),
    NewCollection,
    NewFolder(String),
    NewRequest(String, Option<String>),
    DeleteCollection(String),
    DeleteFolder(String),
    DeleteRequest(String),
    RenameRequest(String, String),
    RenameFolder(String, String),
    RenameCollection(String, String),
    CloneRequest(String),
    MoveRequest(String, String, Option<String>),
}

pub fn render_collection_tree(
    ui: &mut egui::Ui,
    collections: &[Collection],
    folders: &[Folder],
    requests: &[Request],
    state: &mut CollectionTreeState,
    method_map: &HashMap<String, HttpMethod>,
) -> TreeAction {
    let mut action = TreeAction::None;

    // Header
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Collections")
                .strong()
                .size(15.0)
                .color(super::theme::TEXT_PRIMARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if super::theme::icon_button(ui, egui_phosphor::regular::PLUS, "New collection") {
                action = TreeAction::NewCollection;
            }
        });
    });
    ui.add_space(2.0);
    ui.separator();
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for collection in collections {
                let a = render_collection_row(ui, collection, folders, requests, state, method_map);
                if !matches!(a, TreeAction::None) {
                    action = a;
                }
            }
        });

    action
}

// ── Collection row ──────────────────────────────────────────────

fn render_collection_row(
    ui: &mut egui::Ui,
    collection: &Collection,
    folders: &[Folder],
    requests: &[Request],
    state: &mut CollectionTreeState,
    method_map: &HashMap<String, HttpMethod>,
) -> TreeAction {
    let mut action = TreeAction::None;
    let is_expanded = state.expanded_collections.contains(&collection.id);
    let is_selected = state.selected_collection_id.as_deref() == Some(&collection.id);
    let is_renaming = matches!(&state.rename_target, Some(RenameTarget::Collection(id)) if id == &collection.id);

    let frame_fill = if is_selected {
        super::theme::ACCENT.gamma_multiply(0.12)
    } else {
        egui::Color32::TRANSPARENT
    };

    let cid = collection.id.clone();
    let cname = collection.name.clone();

    let frame_resp = egui::Frame::default()
        .fill(frame_fill)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Expand arrow
                let icon = if is_expanded {
                    egui_phosphor::regular::CARET_DOWN
                } else {
                    egui_phosphor::regular::CARET_RIGHT
                };
                if ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new(icon).size(12.0).color(super::theme::TEXT_MUTED),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .clicked()
                {
                    toggle_set(&mut state.expanded_collections, &collection.id);
                }

                // Collection icon
                ui.add(egui::Label::new(
                    egui::RichText::new(egui_phosphor::regular::FOLDER_SIMPLE)
                        .size(14.0)
                        .color(super::theme::ACCENT),
                ));

                if is_renaming {
                    let a = inline_rename_edit(ui, state, &collection.id, RenameType::Collection);
                    if !matches!(a, TreeAction::None) {
                        action = a;
                    }
                } else {
                    let name_color = if is_selected {
                        super::theme::ACCENT
                    } else {
                        super::theme::TEXT_PRIMARY
                    };
                    // Fill remaining width but stay left-aligned
                    let resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new(&collection.name).strong().size(14.0).color(name_color),
                        )
                        .sense(egui::Sense::click()),
                    );
                    // Extend interaction area to full row width
                    let full_rect = resp.rect.with_max_x(ui.max_rect().right());
                    let full_resp = ui.interact(full_rect, resp.id.with("full"), egui::Sense::click());

                    if resp.clicked() || full_resp.clicked() {
                        action = TreeAction::SelectCollection(collection.id.clone());
                        state.expanded_collections.insert(collection.id.clone());
                    }
                    if resp.double_clicked() || full_resp.double_clicked() {
                        start_rename(state, &collection.id, &collection.name, RenameTarget::Collection(collection.id.clone()));
                    }
                    // Context menu on the full row area
                    full_resp.context_menu(|ui| {
                        if ui.button(format!("{} New Request", egui_phosphor::regular::PLUS)).clicked() {
                            action = TreeAction::NewRequest(cid.clone(), None);
                            ui.close_menu();
                        }
                        if ui.button(format!("{} New Folder", egui_phosphor::regular::FOLDER_PLUS)).clicked() {
                            action = TreeAction::NewFolder(cid.clone());
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(format!("{} Rename", egui_phosphor::regular::PENCIL_SIMPLE)).clicked() {
                            start_rename(state, &cid, &cname, RenameTarget::Collection(cid.clone()));
                            ui.close_menu();
                        }
                        if ui.button(format!("{} Delete", egui_phosphor::regular::TRASH)).clicked() {
                            action = TreeAction::DeleteCollection(cid.clone());
                            ui.close_menu();
                        }
                    });
                }
            });
        });

    let frame_response = frame_resp.response;

    // Drop target: collection root (move request here → folder_id = None)
    let is_drop_hover = check_drop_target(ui, &frame_response, state);
    if is_drop_hover {
        highlight_drop_target(ui, &frame_response);
    }
    if let Some(move_action) = handle_drop(ui, &frame_response, state, &collection.id, None) {
        action = move_action;
    }

    // Children
    if is_expanded {
        ui.indent(&collection.id, |ui| {
            // Folders
            let child_folders: Vec<&Folder> = folders
                .iter()
                .filter(|f| f.collection_id == collection.id && f.parent_folder_id.is_none())
                .collect();

            for folder in &child_folders {
                let a = render_folder_row(ui, folder, &collection.id, requests, state, method_map);
                if !matches!(a, TreeAction::None) {
                    action = a;
                }
            }

            // Top-level requests (no folder)
            let orphan_requests: Vec<&Request> = requests
                .iter()
                .filter(|r| r.collection_id == collection.id && r.folder_id.is_none())
                .collect();

            for req in orphan_requests {
                let a = render_request_item(ui, req, &collection.id, state, method_map);
                if !matches!(a, TreeAction::None) {
                    action = a;
                }
            }
        });
    }

    ui.add_space(4.0);
    action
}

// ── Folder row ──────────────────────────────────────────────────

fn render_folder_row(
    ui: &mut egui::Ui,
    folder: &Folder,
    collection_id: &str,
    requests: &[Request],
    state: &mut CollectionTreeState,
    method_map: &HashMap<String, HttpMethod>,
) -> TreeAction {
    let mut action = TreeAction::None;
    let is_expanded = state.expanded_folders.contains(&folder.id);
    let is_renaming = matches!(&state.rename_target, Some(RenameTarget::Folder(id)) if id == &folder.id);

    ui.add_space(1.0);

    let fid = folder.id.clone();
    let fname = folder.name.clone();
    let cid_owned = collection_id.to_string();

    let frame_resp = egui::Frame::default()
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(4, 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let icon = if is_expanded {
                    egui_phosphor::regular::CARET_DOWN
                } else {
                    egui_phosphor::regular::CARET_RIGHT
                };
                let folder_color = egui::Color32::from_rgb(252, 196, 55);
                if ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new(icon).size(12.0).color(folder_color),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .clicked()
                {
                    toggle_set(&mut state.expanded_folders, &folder.id);
                }

                // Folder icon
                let folder_icon = if is_expanded {
                    egui_phosphor::regular::FOLDER_OPEN
                } else {
                    egui_phosphor::regular::FOLDER
                };
                ui.add(egui::Label::new(
                    egui::RichText::new(folder_icon)
                        .size(14.0)
                        .color(folder_color),
                ));

                if is_renaming {
                    let a = inline_rename_edit(ui, state, &folder.id, RenameType::Folder);
                    if !matches!(a, TreeAction::None) {
                        action = a;
                    }
                } else {
                    let resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new(&folder.name)
                                .size(13.0)
                                .color(super::theme::TEXT_PRIMARY),
                        )
                        .sense(egui::Sense::click()),
                    );
                    let full_rect = resp.rect.with_max_x(ui.max_rect().right());
                    let full_resp = ui.interact(full_rect, resp.id.with("full"), egui::Sense::click());

                    if resp.clicked() || full_resp.clicked() {
                        toggle_set(&mut state.expanded_folders, &folder.id);
                    }
                    if resp.double_clicked() || full_resp.double_clicked() {
                        start_rename(state, &folder.id, &folder.name, RenameTarget::Folder(folder.id.clone()));
                    }
                    full_resp.context_menu(|ui| {
                        if ui.button(format!("{} New Request", egui_phosphor::regular::PLUS)).clicked() {
                            action = TreeAction::NewRequest(cid_owned.clone(), Some(fid.clone()));
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(format!("{} Rename", egui_phosphor::regular::PENCIL_SIMPLE)).clicked() {
                            start_rename(state, &fid, &fname, RenameTarget::Folder(fid.clone()));
                            ui.close_menu();
                        }
                        if ui.button(format!("{} Delete", egui_phosphor::regular::TRASH)).clicked() {
                            action = TreeAction::DeleteFolder(fid.clone());
                            ui.close_menu();
                        }
                    });
                }
            });
        });

    let frame_response = frame_resp.response;

    // Drop target: folder
    let is_drop_hover = check_drop_target(ui, &frame_response, state);
    if is_drop_hover {
        highlight_drop_target(ui, &frame_response);
    }
    if let Some(move_action) = handle_drop(ui, &frame_response, state, collection_id, Some(&folder.id)) {
        action = move_action;
    }

    // Folder children
    if is_expanded {
        ui.indent(&folder.id, |ui| {
            let folder_requests: Vec<&Request> = requests
                .iter()
                .filter(|r| r.folder_id.as_deref() == Some(&folder.id))
                .collect();

            for req in folder_requests {
                let a = render_request_item(ui, req, collection_id, state, method_map);
                if !matches!(a, TreeAction::None) {
                    action = a;
                }
            }
        });
    }

    action
}

// ── Request row ─────────────────────────────────────────────────

fn render_request_item(
    ui: &mut egui::Ui,
    req: &Request,
    _collection_id: &str,
    state: &mut CollectionTreeState,
    method_map: &HashMap<String, HttpMethod>,
) -> TreeAction {
    let mut action = TreeAction::None;
    let is_selected = state.selected_request_id.as_deref() == Some(&req.id);
    let is_renaming = matches!(&state.rename_target, Some(RenameTarget::Request(id)) if id == &req.id);
    let is_being_dragged = state.drag_request_id.as_deref() == Some(&req.id);

    let fill = if is_being_dragged {
        super::theme::ACCENT.gamma_multiply(0.08)
    } else if is_selected {
        super::theme::ACCENT.gamma_multiply(0.15)
    } else {
        egui::Color32::TRANSPARENT
    };

    let stroke = if is_selected {
        egui::Stroke::new(1.0, super::theme::ACCENT.gamma_multiply(0.4))
    } else {
        egui::Stroke::NONE
    };

    let rid = req.id.clone();
    let rname = req.name.clone();

    let frame_resp = egui::Frame::default()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Method badge pill
                let method = method_map.get(&req.id);
                render_method_badge(ui, method);

                if is_renaming {
                    let a = inline_rename_edit(ui, state, &req.id, RenameType::Request);
                    if !matches!(a, TreeAction::None) {
                        action = a;
                    }
                } else {
                    let text_color = if is_selected {
                        egui::Color32::WHITE
                    } else {
                        super::theme::TEXT_PRIMARY
                    };

                    let resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new(&req.name).size(13.0).color(text_color),
                        )
                        .sense(egui::Sense::click_and_drag()),
                    );
                    let full_rect = resp.rect.with_max_x(ui.max_rect().right());
                    let full_resp = ui.interact(full_rect, resp.id.with("full"), egui::Sense::click_and_drag());

                    if resp.clicked() || full_resp.clicked() {
                        action = TreeAction::SelectRequest(req.id.clone());
                    }
                    if resp.double_clicked() || full_resp.double_clicked() {
                        start_rename(state, &req.id, &req.name, RenameTarget::Request(req.id.clone()));
                    }
                    if resp.drag_started() || full_resp.drag_started() {
                        state.drag_request_id = Some(req.id.clone());
                    }
                    full_resp.context_menu(|ui| {
                        if ui.button(format!("{} Rename", egui_phosphor::regular::PENCIL_SIMPLE)).clicked() {
                            start_rename(state, &rid, &rname, RenameTarget::Request(rid.clone()));
                            ui.close_menu();
                        }
                        if ui.button(format!("{} Clone", egui_phosphor::regular::COPY)).clicked() {
                            action = TreeAction::CloneRequest(rid.clone());
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(format!("{} Delete", egui_phosphor::regular::TRASH)).clicked() {
                            action = TreeAction::DeleteRequest(rid.clone());
                            ui.close_menu();
                        }
                    });
                }
            });
        });

    ui.add_space(1.0);
    action
}

// ── Method badge pill ───────────────────────────────────────────

fn render_method_badge(ui: &mut egui::Ui, method: Option<&HttpMethod>) {
    let (label, color) = match method {
        Some(m) => {
            let [r, g, b] = m.color();
            (m.as_str(), egui::Color32::from_rgb(r, g, b))
        }
        None => ("GET", egui::Color32::from_rgb(97, 175, 254)),
    };

    let short = match label {
        "DELETE" => "DEL",
        "OPTIONS" => "OPT",
        "PATCH" => "PAT",
        other => other,
    };

    let (badge_rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 16.0), egui::Sense::hover());
    ui.painter().rect_filled(
        badge_rect,
        egui::CornerRadius::same(3),
        color.gamma_multiply(0.18),
    );
    ui.painter().text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        short,
        egui::FontId::new(9.0, egui::FontFamily::Proportional),
        color,
    );
}

// ── Inline rename ───────────────────────────────────────────────

enum RenameType {
    Request,
    Folder,
    Collection,
}

fn start_rename(state: &mut CollectionTreeState, id: &str, current_name: &str, target: RenameTarget) {
    state.rename_id = Some(id.to_string());
    state.rename_buf = current_name.to_string();
    state.rename_target = Some(target);
}

fn inline_rename_edit(
    ui: &mut egui::Ui,
    state: &mut CollectionTreeState,
    id: &str,
    rename_type: RenameType,
) -> TreeAction {
    let mut action = TreeAction::None;
    let resp = ui.add(
        egui::TextEdit::singleline(&mut state.rename_buf)
            .desired_width(140.0)
            .font(egui::TextStyle::Body),
    );
    if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        let new_name = state.rename_buf.clone();
        let item_id = id.to_string();
        state.rename_id = None;
        state.rename_target = None;
        if !new_name.is_empty() {
            action = match rename_type {
                RenameType::Request => TreeAction::RenameRequest(item_id, new_name),
                RenameType::Folder => TreeAction::RenameFolder(item_id, new_name),
                RenameType::Collection => TreeAction::RenameCollection(item_id, new_name),
            };
        }
    }
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.rename_id = None;
        state.rename_target = None;
    }
    resp.request_focus();
    action
}

// ── Drag & drop helpers ─────────────────────────────────────────

fn check_drop_target(
    _ui: &mut egui::Ui,
    resp: &egui::Response,
    state: &CollectionTreeState,
) -> bool {
    resp.hovered() && state.drag_request_id.is_some()
}

fn highlight_drop_target(ui: &mut egui::Ui, resp: &egui::Response) {
    let rect = resp.rect;
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(4),
        egui::Stroke::new(2.0, super::theme::ACCENT),
        egui::StrokeKind::Outside,
    );
}

fn handle_drop(
    ui: &mut egui::Ui,
    resp: &egui::Response,
    state: &mut CollectionTreeState,
    collection_id: &str,
    folder_id: Option<&str>,
) -> Option<TreeAction> {
    if state.drag_request_id.is_none() {
        return None;
    }
    if !resp.hovered() {
        return None;
    }
    if ui.input(|i| i.pointer.any_released()) {
        if let Some(drag_id) = state.drag_request_id.take() {
            return Some(TreeAction::MoveRequest(
                drag_id,
                collection_id.to_string(),
                folder_id.map(|s| s.to_string()),
            ));
        }
    }
    None
}

// ── Utility ─────────────────────────────────────────────────────

fn toggle_set(set: &mut std::collections::HashSet<String>, key: &str) {
    if set.contains(key) {
        set.remove(key);
    } else {
        set.insert(key.to_string());
    }
}
