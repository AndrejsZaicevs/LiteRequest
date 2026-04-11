use eframe::egui;
use std::collections::HashMap;
use crate::models::*;

#[derive(Debug, Clone, PartialEq)]
enum RenameTarget {
    Request(String),
    Folder(String),
    Collection(String),
}

/// Payload carried during a drag operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DragPayload {
    /// Index into the flat render order so we can show insertion indicators.
    source_idx: usize,
}

pub struct CollectionTreeState {
    pub expanded_collections: std::collections::HashSet<String>,
    pub expanded_folders: std::collections::HashSet<String>,
    pub selected_request_id: Option<String>,
    pub selected_collection_id: Option<String>,
    pub rename_id: Option<String>,
    pub rename_buf: String,
    rename_target: Option<RenameTarget>,
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

/// Pending drop from a container (collection or folder).
/// Contains the DragPayload source_idx and the target container info.
struct PendingDrop {
    source_idx: usize,
    target_collection_id: String,
    target_folder_id: Option<String>,
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

    // Build flat row list for ordered rendering + DnD
    let rows = build_row_list(collections, folders, requests, state);

    // Collect pending drops to resolve after rendering
    let mut pending_drops: Vec<PendingDrop> = Vec::new();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (idx, row) in rows.iter().enumerate() {
                match row {
                    RowItem::Collection(c) => {
                        let (a, drop) = render_collection_row(ui, c, state, idx);
                        merge_action(&mut action, a);
                        if let Some(d) = drop {
                            pending_drops.push(d);
                        }
                    }
                    RowItem::Folder { folder, collection_id, depth } => {
                        let (a, drop) = render_folder_row(ui, folder, collection_id, state, idx, *depth);
                        merge_action(&mut action, a);
                        if let Some(d) = drop {
                            pending_drops.push(d);
                        }
                    }
                    RowItem::Request { request, collection_id, folder_id, depth } => {
                        let a = render_request_item(
                            ui, request, collection_id, folder_id.as_deref(),
                            state, method_map, idx, *depth, &rows,
                        );
                        merge_action(&mut action, a);
                    }
                    RowItem::CollectionEnd(collection_id) => {
                        // Invisible drop zone at the end of a collection
                        let drop_frame = egui::Frame::default().inner_margin(egui::Margin::symmetric(0, 1));
                        let (_, dropped) = ui.dnd_drop_zone::<DragPayload, ()>(drop_frame, |ui| {
                            ui.allocate_space(egui::vec2(ui.available_width(), 2.0));
                        });
                        if let Some(payload) = dropped {
                            pending_drops.push(PendingDrop {
                                source_idx: payload.source_idx,
                                target_collection_id: collection_id.clone(),
                                target_folder_id: None,
                            });
                        }

                        ui.add_space(4.0);
                    }
                }
            }
        });

    // Resolve pending drops using the rows list
    for pd in pending_drops {
        if let Some(RowItem::Request { request, .. }) = rows.get(pd.source_idx) {
            action = TreeAction::MoveRequest(
                request.id.clone(),
                pd.target_collection_id,
                pd.target_folder_id,
            );
            break;
        }
    }

    action
}

fn merge_action(target: &mut TreeAction, source: TreeAction) {
    if !matches!(source, TreeAction::None) {
        *target = source;
    }
}

// ── Flat row model ─────────────────────────────────────────────

#[derive(Clone)]
enum RowItem<'a> {
    Collection(&'a Collection),
    Folder {
        folder: &'a Folder,
        collection_id: String,
        depth: usize,
    },
    Request {
        request: &'a Request,
        collection_id: String,
        folder_id: Option<String>,
        depth: usize,
    },
    /// Marker at end of collection children for bottom drop zone
    CollectionEnd(String),
}

fn build_row_list<'a>(
    collections: &'a [Collection],
    folders: &'a [Folder],
    requests: &'a [Request],
    state: &CollectionTreeState,
) -> Vec<RowItem<'a>> {
    let mut rows = Vec::new();

    for collection in collections {
        rows.push(RowItem::Collection(collection));

        if state.expanded_collections.contains(&collection.id) {
            // Folders at collection root
            let child_folders: Vec<&Folder> = folders
                .iter()
                .filter(|f| f.collection_id == collection.id && f.parent_folder_id.is_none())
                .collect();

            for folder in &child_folders {
                rows.push(RowItem::Folder {
                    folder,
                    collection_id: collection.id.clone(),
                    depth: 1,
                });

                if state.expanded_folders.contains(&folder.id) {
                    let folder_requests: Vec<&Request> = requests
                        .iter()
                        .filter(|r| r.folder_id.as_deref() == Some(&folder.id))
                        .collect();
                    for req in folder_requests {
                        rows.push(RowItem::Request {
                            request: req,
                            collection_id: collection.id.clone(),
                            folder_id: Some(folder.id.clone()),
                            depth: 2,
                        });
                    }
                }
            }

            // Top-level requests (no folder)
            let orphan_requests: Vec<&Request> = requests
                .iter()
                .filter(|r| r.collection_id == collection.id && r.folder_id.is_none())
                .collect();

            for req in orphan_requests {
                rows.push(RowItem::Request {
                    request: req,
                    collection_id: collection.id.clone(),
                    folder_id: None,
                    depth: 1,
                });
            }

            rows.push(RowItem::CollectionEnd(collection.id.clone()));
        }
    }

    rows
}

// ── Collection row ──────────────────────────────────────────────

fn render_collection_row(
    ui: &mut egui::Ui,
    collection: &Collection,
    state: &mut CollectionTreeState,
    _row_idx: usize,
) -> (TreeAction, Option<PendingDrop>) {
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

    // Wrap collection in a drop zone so requests can be dropped onto the collection root
    let drop_frame = egui::Frame::default()
        .fill(frame_fill)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(6, 3));

    let (_, dropped_payload) = ui.dnd_drop_zone::<DragPayload, ()>(drop_frame, |ui| {
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
                let resp = ui.add(
                    egui::Label::new(
                        egui::RichText::new(&collection.name).strong().size(14.0).color(name_color),
                    )
                    .sense(egui::Sense::click()),
                );
                let full_rect = resp.rect.with_max_x(ui.max_rect().right());
                let full_resp = ui.interact(full_rect, resp.id.with("full"), egui::Sense::click());

                if resp.clicked() || full_resp.clicked() {
                    action = TreeAction::SelectCollection(collection.id.clone());
                    state.expanded_collections.insert(collection.id.clone());
                }
                if resp.double_clicked() || full_resp.double_clicked() {
                    start_rename(state, &collection.id, &collection.name, RenameTarget::Collection(collection.id.clone()));
                }
                full_resp.context_menu(|ui| {
                    if ui.button(format!("{} New Request", egui_phosphor::regular::PLUS)).clicked() {
                        action = TreeAction::NewRequest(cid.clone(), None);
                        ui.close();
                    }
                    if ui.button(format!("{} New Folder", egui_phosphor::regular::FOLDER_PLUS)).clicked() {
                        action = TreeAction::NewFolder(cid.clone());
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(format!("{} Rename", egui_phosphor::regular::PENCIL_SIMPLE)).clicked() {
                        start_rename(state, &cid, &cname, RenameTarget::Collection(cid.clone()));
                        ui.close();
                    }
                    if ui.button(format!("{} Delete", egui_phosphor::regular::TRASH)).clicked() {
                        action = TreeAction::DeleteCollection(cid.clone());
                        ui.close();
                    }
                });
            }
        });
    });

    let pending = dropped_payload.map(|p| PendingDrop {
        source_idx: p.source_idx,
        target_collection_id: collection.id.clone(),
        target_folder_id: None,
    });

    (action, pending)
}

// ── Folder row ──────────────────────────────────────────────────

fn render_folder_row(
    ui: &mut egui::Ui,
    folder: &Folder,
    collection_id: &str,
    state: &mut CollectionTreeState,
    _row_idx: usize,
    depth: usize,
) -> (TreeAction, Option<PendingDrop>) {
    let mut action = TreeAction::None;
    let is_expanded = state.expanded_folders.contains(&folder.id);
    let is_renaming = matches!(&state.rename_target, Some(RenameTarget::Folder(id)) if id == &folder.id);

    let fid = folder.id.clone();
    let fname = folder.name.clone();
    let cid_owned = collection_id.to_string();

    let indent = (depth as f32) * 16.0;

    ui.add_space(1.0);

    // Folder is a drop zone
    let drop_frame = egui::Frame::default()
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin { left: indent as i8 + 4, right: 4, top: 2, bottom: 2 });

    let (_, dropped_payload) = ui.dnd_drop_zone::<DragPayload, ()>(drop_frame, |ui| {
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
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(format!("{} Rename", egui_phosphor::regular::PENCIL_SIMPLE)).clicked() {
                        start_rename(state, &fid, &fname, RenameTarget::Folder(fid.clone()));
                        ui.close();
                    }
                    if ui.button(format!("{} Delete", egui_phosphor::regular::TRASH)).clicked() {
                        action = TreeAction::DeleteFolder(fid.clone());
                        ui.close();
                    }
                });
            }
        });
    });

    let pending = dropped_payload.map(|p| PendingDrop {
        source_idx: p.source_idx,
        target_collection_id: collection_id.to_string(),
        target_folder_id: Some(folder.id.clone()),
    });

    (action, pending)
}

// ── Request row ─────────────────────────────────────────────────

fn render_request_item(
    ui: &mut egui::Ui,
    req: &Request,
    collection_id: &str,
    folder_id: Option<&str>,
    state: &mut CollectionTreeState,
    method_map: &HashMap<String, HttpMethod>,
    row_idx: usize,
    depth: usize,
    rows: &[RowItem],
) -> TreeAction {
    let mut action = TreeAction::None;
    let is_selected = state.selected_request_id.as_deref() == Some(&req.id);
    let is_renaming = matches!(&state.rename_target, Some(RenameTarget::Request(id)) if id == &req.id);

    let fill = if is_selected {
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
    let indent = (depth as f32) * 16.0;

    let item_id = egui::Id::new(("req_drag", &req.id));
    let payload = DragPayload { source_idx: row_idx };

    // Render the request as a drag source
    let drag_resp = ui.dnd_drag_source(item_id, payload, |ui| {
        let frame = egui::Frame::default()
            .fill(fill)
            .stroke(stroke)
            .corner_radius(egui::CornerRadius::same(5))
            .inner_margin(egui::Margin { left: indent as i8 + 8, right: 8, top: 3, bottom: 3 });

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
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
                        .sense(egui::Sense::click()),
                    );
                    let full_rect = resp.rect.with_max_x(ui.max_rect().right());
                    let full_resp = ui.interact(full_rect, resp.id.with("full"), egui::Sense::click());

                    if resp.clicked() || full_resp.clicked() {
                        action = TreeAction::SelectRequest(req.id.clone());
                    }
                    if resp.double_clicked() || full_resp.double_clicked() {
                        start_rename(state, &req.id, &req.name, RenameTarget::Request(req.id.clone()));
                    }
                    full_resp.context_menu(|ui| {
                        if ui.button(format!("{} Rename", egui_phosphor::regular::PENCIL_SIMPLE)).clicked() {
                            start_rename(state, &rid, &rname, RenameTarget::Request(rid.clone()));
                            ui.close();
                        }
                        if ui.button(format!("{} Clone", egui_phosphor::regular::COPY)).clicked() {
                            action = TreeAction::CloneRequest(rid.clone());
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(format!("{} Delete", egui_phosphor::regular::TRASH)).clicked() {
                            action = TreeAction::DeleteRequest(rid.clone());
                            ui.close();
                        }
                    });
                }
            });
        });
    });

    let response = drag_resp.response;

    // Show drop indicator when another request is hovering over this one
    if let (Some(pointer), Some(hovered_payload)) = (
        ui.input(|i| i.pointer.interact_pos()),
        response.dnd_hover_payload::<DragPayload>(),
    ) {
        // Don't show indicator if dragging onto itself
        if hovered_payload.source_idx != row_idx {
            let rect = response.rect;
            let stroke = egui::Stroke::new(2.0, super::theme::ACCENT);

            if pointer.y < rect.center().y {
                // Insert above
                ui.painter().hline(rect.x_range(), rect.top(), stroke);
            } else {
                // Insert below
                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
            }
        }

        // Check for release
        if let Some(dropped) = response.dnd_release_payload::<DragPayload>() {
            if dropped.source_idx != row_idx {
                // Resolve the source request from the flat rows list
                if let Some(RowItem::Request { request: src_req, .. }) = rows.get(dropped.source_idx) {
                    // Drop onto the same container as the target request
                    action = TreeAction::MoveRequest(
                        src_req.id.clone(),
                        collection_id.to_string(),
                        folder_id.map(|s| s.to_string()),
                    );
                }
            }
        }
    }

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

// ── Utility ─────────────────────────────────────────────────────

fn toggle_set(set: &mut std::collections::HashSet<String>, key: &str) {
    if set.contains(key) {
        set.remove(key);
    } else {
        set.insert(key.to_string());
    }
}
