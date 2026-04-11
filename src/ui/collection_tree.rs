use eframe::egui;
use std::collections::HashMap;
use crate::models::*;

#[derive(Debug, Clone, PartialEq)]
enum RenameTarget {
    Request(String),
    Folder(String),
    Collection(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DragKind {
    Request,
    Folder,
}

/// Payload carried during a drag operation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DragPayload {
    /// Index into the flat render order so we can show insertion indicators.
    source_idx: usize,
    /// Display name for the floating drag preview.
    label: String,
    kind: DragKind,
    item_id: String,
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

#[derive(Debug, Clone)]
pub enum DropPosition {
    End,
    Before(String),
    After(String),
}

pub enum TreeAction {
    None,
    SelectRequest(String),
    SelectCollection(String),
    NewCollection,
    NewFolder(String, Option<String>),
    NewRequest(String, Option<String>),
    DeleteCollection(String),
    DeleteFolder(String),
    DeleteRequest(String),
    RenameRequest(String, String),
    RenameFolder(String, String),
    RenameCollection(String, String),
    CloneRequest(String),
    MoveRequest(String, String, Option<String>, DropPosition),
    MoveFolder(String, String, Option<String>),
}

/// Pending drop from a container (collection or folder).
/// Contains the DragPayload source info and the target container info.
struct PendingDrop {
    #[allow(dead_code)]
    source_idx: usize,
    source_kind: DragKind,
    source_item_id: String,
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
                        let (a, drop) = render_folder_row(ui, folder, collection_id, state, idx, *depth, folders);
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
                        // Invisible drop zone at the end of a collection (manual detection)
                        let (space_rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 4.0),
                            egui::Sense::hover(),
                        );
                        let drop_id = egui::Id::new(("coll_end_drop", collection_id));
                        let drop_resp = ui.interact(space_rect, drop_id, egui::Sense::hover());

                        if let Some(payload) = drop_resp.dnd_release_payload::<DragPayload>() {
                            pending_drops.push(PendingDrop {
                                source_idx: payload.source_idx,
                                source_kind: payload.kind.clone(),
                                source_item_id: payload.item_id.clone(),
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
        match pd.source_kind {
            DragKind::Request => {
                action = TreeAction::MoveRequest(
                    pd.source_item_id,
                    pd.target_collection_id,
                    pd.target_folder_id,
                    DropPosition::End,
                );
                break;
            }
            DragKind::Folder => {
                // Anti-cycle: don't allow dropping a folder into itself or its descendants
                if let Some(target_fid) = &pd.target_folder_id {
                    if pd.source_item_id == *target_fid
                        || is_descendant_of(folders, target_fid, &pd.source_item_id)
                    {
                        continue;
                    }
                }
                action = TreeAction::MoveFolder(
                    pd.source_item_id,
                    pd.target_collection_id,
                    pd.target_folder_id,
                );
                break;
            }
        }
    }

    // Floating drag preview near cursor
    if let Some(payload) = egui::DragAndDrop::payload::<DragPayload>(ui.ctx()) {
        if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) {
            let layer = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drag_preview"));
            let painter = ui.ctx().layer_painter(layer);
            let pos = pointer + egui::vec2(14.0, 14.0);
            let font = egui::FontId::new(12.0, egui::FontFamily::Proportional);
            let bg = egui::Frame::default()
                .fill(super::theme::SURFACE_1)
                .stroke(egui::Stroke::new(1.0, super::theme::ACCENT))
                .corner_radius(egui::CornerRadius::same(4))
                .inner_margin(egui::Margin::symmetric(6, 3));
            let galley = painter.layout_no_wrap(payload.label.clone(), font, super::theme::TEXT_PRIMARY);
            let text_size = galley.size();
            let rect = egui::Rect::from_min_size(pos, text_size + egui::vec2(12.0, 6.0));
            painter.add(bg.paint(rect));
            painter.galley(pos + egui::vec2(6.0, 3.0), galley, super::theme::TEXT_PRIMARY);
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
            let coll_folders: Vec<&Folder> = folders
                .iter()
                .filter(|f| f.collection_id == collection.id)
                .collect();

            // Recursively add folders starting from root (parent_folder_id == None)
            add_folder_children(
                &mut rows,
                &coll_folders,
                requests,
                &collection.id,
                None,
                1,
                state,
            );

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

fn add_folder_children<'a>(
    rows: &mut Vec<RowItem<'a>>,
    all_coll_folders: &[&'a Folder],
    requests: &'a [Request],
    collection_id: &str,
    parent_folder_id: Option<&str>,
    depth: usize,
    state: &CollectionTreeState,
) {
    let child_folders: Vec<&&Folder> = all_coll_folders
        .iter()
        .filter(|f| f.parent_folder_id.as_deref() == parent_folder_id)
        .collect();

    for folder in child_folders {
        rows.push(RowItem::Folder {
            folder,
            collection_id: collection_id.to_string(),
            depth,
        });

        if state.expanded_folders.contains(&folder.id) {
            // Recurse into sub-folders
            add_folder_children(
                rows,
                all_coll_folders,
                requests,
                collection_id,
                Some(&folder.id),
                depth + 1,
                state,
            );

            // Requests in this folder
            let folder_requests: Vec<&Request> = requests
                .iter()
                .filter(|r| r.folder_id.as_deref() == Some(&*folder.id))
                .collect();
            for req in folder_requests {
                rows.push(RowItem::Request {
                    request: req,
                    collection_id: collection_id.to_string(),
                    folder_id: Some(folder.id.clone()),
                    depth: depth + 1,
                });
            }
        }
    }
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

    // Normal frame (no dnd_drop_zone — we detect drops manually to avoid the outline artifact)
    let frame = egui::Frame::default()
        .fill(frame_fill)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(6, 3));

    let frame_resp = frame.show(ui, |ui| {
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
                        action = TreeAction::NewFolder(cid.clone(), None);
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

    // Manual drop detection on the frame area (hover-only sense won't steal clicks)
    let drop_id = egui::Id::new(("coll_drop", &collection.id));
    let drop_resp = ui.interact(frame_resp.response.rect, drop_id, egui::Sense::hover());

    let mut pending = None;
    if let Some(payload) = drop_resp.dnd_release_payload::<DragPayload>() {
        pending = Some(PendingDrop {
            source_idx: payload.source_idx,
            source_kind: payload.kind.clone(),
            source_item_id: payload.item_id.clone(),
            target_collection_id: collection.id.clone(),
            target_folder_id: None,
        });
    }

    // Subtle highlight when a drag payload hovers over this collection
    if drop_resp.dnd_hover_payload::<DragPayload>().is_some() {
        ui.painter().rect_stroke(
            frame_resp.response.rect,
            egui::CornerRadius::same(6),
            egui::Stroke::new(1.5, super::theme::ACCENT),
            egui::StrokeKind::Outside,
        );
    }

    (action, pending)
}

// ── Folder row ──────────────────────────────────────────────────

fn render_folder_row(
    ui: &mut egui::Ui,
    folder: &Folder,
    collection_id: &str,
    state: &mut CollectionTreeState,
    row_idx: usize,
    depth: usize,
    all_folders: &[Folder],
) -> (TreeAction, Option<PendingDrop>) {
    let mut action = TreeAction::None;
    let is_expanded = state.expanded_folders.contains(&folder.id);
    let is_renaming = matches!(&state.rename_target, Some(RenameTarget::Folder(id)) if id == &folder.id);

    let fid = folder.id.clone();
    let fname = folder.name.clone();
    let cid_owned = collection_id.to_string();

    let indent = (depth as f32) * 16.0;

    // Dim the folder if it is currently being dragged
    let is_being_dragged = egui::DragAndDrop::payload::<DragPayload>(ui.ctx())
        .map_or(false, |p| p.source_idx == row_idx);

    let fill = if is_being_dragged {
        super::theme::ACCENT.gamma_multiply(0.06)
    } else {
        egui::Color32::TRANSPARENT
    };

    ui.add_space(1.0);

    let frame = egui::Frame::default()
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin { left: indent as i8 + 4, right: 4, top: 2, bottom: 2 });

    let frame_resp = frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let icon = if is_expanded {
                egui_phosphor::regular::CARET_DOWN
            } else {
                egui_phosphor::regular::CARET_RIGHT
            };
            let folder_color = egui::Color32::from_rgb(252, 196, 55);

            let text_color = if is_being_dragged {
                super::theme::TEXT_MUTED
            } else {
                folder_color
            };

            ui.add(egui::Label::new(
                egui::RichText::new(icon).size(12.0).color(text_color),
            ));

            let folder_icon = if is_expanded {
                egui_phosphor::regular::FOLDER_OPEN
            } else {
                egui_phosphor::regular::FOLDER
            };
            ui.add(egui::Label::new(
                egui::RichText::new(folder_icon)
                    .size(14.0)
                    .color(text_color),
            ));

            if is_renaming {
                let a = inline_rename_edit(ui, state, &folder.id, RenameType::Folder);
                if !matches!(a, TreeAction::None) {
                    action = a;
                }
            } else {
                let name_color = if is_being_dragged {
                    super::theme::TEXT_MUTED
                } else {
                    super::theme::TEXT_PRIMARY
                };
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&folder.name)
                            .size(13.0)
                            .color(name_color),
                    ),
                );
            }
        });
    });

    // Single interaction for click + drag on the frame area (only when not renaming)
    if !is_renaming {
        let item_id = egui::Id::new(("folder_item", &folder.id));
        let resp = ui.interact(frame_resp.response.rect, item_id, egui::Sense::click_and_drag());

        let payload = DragPayload {
            source_idx: row_idx,
            label: folder.name.clone(),
            kind: DragKind::Folder,
            item_id: folder.id.clone(),
        };
        resp.dnd_set_drag_payload(payload);

        if resp.clicked() {
            toggle_set(&mut state.expanded_folders, &folder.id);
        }
        if resp.double_clicked() {
            start_rename(state, &folder.id, &folder.name, RenameTarget::Folder(folder.id.clone()));
        }
        resp.context_menu(|ui| {
            if ui.button(format!("{} New Request", egui_phosphor::regular::PLUS)).clicked() {
                action = TreeAction::NewRequest(cid_owned.clone(), Some(fid.clone()));
                ui.close();
            }
            if ui.button(format!("{} New Folder", egui_phosphor::regular::FOLDER_PLUS)).clicked() {
                action = TreeAction::NewFolder(cid_owned.clone(), Some(fid.clone()));
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

    // Manual drop detection on the folder area
    let drop_id = egui::Id::new(("folder_drop", &folder.id));
    let drop_resp = ui.interact(frame_resp.response.rect, drop_id, egui::Sense::hover());

    let mut pending = None;
    if let Some(payload) = drop_resp.dnd_release_payload::<DragPayload>() {
        // Anti-cycle check for folder-into-folder drops
        let allow = match payload.kind {
            DragKind::Folder => {
                payload.item_id != folder.id
                    && !is_descendant_of(all_folders, &folder.id, &payload.item_id)
            }
            DragKind::Request => true,
        };
        if allow {
            pending = Some(PendingDrop {
                source_idx: payload.source_idx,
                source_kind: payload.kind.clone(),
                source_item_id: payload.item_id.clone(),
                target_collection_id: collection_id.to_string(),
                target_folder_id: Some(folder.id.clone()),
            });
        }
    }

    // Subtle highlight when a drag payload hovers over this folder
    if let Some(hovered) = drop_resp.dnd_hover_payload::<DragPayload>() {
        let show_highlight = match hovered.kind {
            DragKind::Folder => {
                hovered.item_id != folder.id
                    && !is_descendant_of(all_folders, &folder.id, &hovered.item_id)
            }
            DragKind::Request => true,
        };
        if show_highlight {
            ui.painter().rect_stroke(
                frame_resp.response.rect,
                egui::CornerRadius::same(4),
                egui::Stroke::new(1.5, super::theme::ACCENT),
                egui::StrokeKind::Outside,
            );
        }
    }

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
    _rows: &[RowItem],
) -> TreeAction {
    let mut action = TreeAction::None;
    let is_selected = state.selected_request_id.as_deref() == Some(&req.id);
    let is_renaming = matches!(&state.rename_target, Some(RenameTarget::Request(id)) if id == &req.id);

    // Dim the item if it is currently being dragged
    let is_being_dragged = egui::DragAndDrop::payload::<DragPayload>(ui.ctx())
        .map_or(false, |p| p.source_idx == row_idx);

    let fill = if is_being_dragged {
        super::theme::ACCENT.gamma_multiply(0.06)
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
    let indent = (depth as f32) * 16.0;

    let payload = DragPayload {
        source_idx: row_idx,
        label: req.name.clone(),
        kind: DragKind::Request,
        item_id: req.id.clone(),
    };

    // Render frame normally (no dnd_drag_source — we handle drag via click_and_drag sense)
    let frame = egui::Frame::default()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin { left: indent as i8 + 8, right: 8, top: 3, bottom: 3 });

    let frame_resp = frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let method = method_map.get(&req.id);
            render_method_badge(ui, method);

            if is_renaming {
                let a = inline_rename_edit(ui, state, &req.id, RenameType::Request);
                if !matches!(a, TreeAction::None) {
                    action = a;
                }
            } else {
                let text_color = if is_being_dragged {
                    super::theme::TEXT_MUTED
                } else if is_selected {
                    egui::Color32::WHITE
                } else {
                    super::theme::TEXT_PRIMARY
                };

                // Label without click sense — all interactions handled by full_resp below
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&req.name).size(13.0).color(text_color),
                    ),
                );
            }
        });
    });

    // Single interaction for click + drag on the frame area (only when not renaming)
    if !is_renaming {
        let item_id = egui::Id::new(("req_item", &req.id));
        let resp = ui.interact(frame_resp.response.rect, item_id, egui::Sense::click_and_drag());

        // Drag: sets payload only on drag_started(), cursor stays default until actually dragging
        resp.dnd_set_drag_payload(payload);

        if resp.clicked() {
            action = TreeAction::SelectRequest(req.id.clone());
        }
        if resp.double_clicked() {
            start_rename(state, &req.id, &req.name, RenameTarget::Request(req.id.clone()));
        }
        resp.context_menu(|ui| {
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

    let response = frame_resp.response;

    // Show drop indicator when another request is hovering over this one
    if let (Some(pointer), Some(hovered_payload)) = (
        ui.input(|i| i.pointer.interact_pos()),
        response.dnd_hover_payload::<DragPayload>(),
    ) {
        // Only show hline indicators for request-kind payloads
        if hovered_payload.source_idx != row_idx && hovered_payload.kind == DragKind::Request {
            let rect = response.rect;
            let stroke = egui::Stroke::new(2.0, super::theme::ACCENT);

            if pointer.y < rect.center().y {
                ui.painter().hline(rect.x_range(), rect.top(), stroke);
            } else {
                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
            }
        }

        // Check for release
        if let Some(dropped) = response.dnd_release_payload::<DragPayload>() {
            if dropped.source_idx != row_idx {
                match dropped.kind {
                    DragKind::Request => {
                        let position = if pointer.y < response.rect.center().y {
                            DropPosition::Before(req.id.clone())
                        } else {
                            DropPosition::After(req.id.clone())
                        };
                        action = TreeAction::MoveRequest(
                            dropped.item_id.clone(),
                            collection_id.to_string(),
                            folder_id.map(|s| s.to_string()),
                            position,
                        );
                    }
                    DragKind::Folder => {
                        // Folder dropped onto a request — move folder into the same container
                        action = TreeAction::MoveFolder(
                            dropped.item_id.clone(),
                            collection_id.to_string(),
                            folder_id.map(|s| s.to_string()),
                        );
                    }
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

    // Reserve a fixed width so names stay aligned, but paint only text — no background box.
    let muted = color.gamma_multiply(0.8);
    ui.label(
        egui::RichText::new(short)
            .size(10.0)
            .strong()
            .color(muted)
            .family(egui::FontFamily::Monospace),
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

// ── Anti-cycle helper ────────────────────────────────────────────

/// Returns true if `folder_id` is a descendant of `potential_ancestor_id`
/// by walking up the parent chain.
fn is_descendant_of(folders: &[Folder], folder_id: &str, potential_ancestor_id: &str) -> bool {
    let mut current = folder_id.to_string();
    for _ in 0..100 {
        if let Some(f) = folders.iter().find(|f| f.id == current) {
            match &f.parent_folder_id {
                Some(pid) => {
                    if pid == potential_ancestor_id {
                        return true;
                    }
                    current = pid.clone();
                }
                None => return false,
            }
        } else {
            return false;
        }
    }
    false
}

// ── Utility ─────────────────────────────────────────────────────

fn toggle_set(set: &mut std::collections::HashSet<String>, key: &str) {
    if set.contains(key) {
        set.remove(key);
    } else {
        set.insert(key.to_string());
    }
}
