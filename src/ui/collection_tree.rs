use eframe::egui;
use crate::models::*;

pub struct CollectionTreeState {
    pub expanded_collections: std::collections::HashSet<String>,
    pub expanded_folders: std::collections::HashSet<String>,
    pub selected_request_id: Option<String>,
    pub selected_collection_id: Option<String>,
    pub rename_id: Option<String>,
    pub rename_buf: String,
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
}

pub fn render_collection_tree(
    ui: &mut egui::Ui,
    collections: &[Collection],
    folders: &[Folder],
    requests: &[Request],
    state: &mut CollectionTreeState,
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
            if super::theme::icon_button(ui, "+", "New collection") {
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
                let is_expanded = state.expanded_collections.contains(&collection.id);
                let is_selected_collection =
                    state.selected_collection_id.as_deref() == Some(&collection.id);

                // Collection header frame
                let frame_fill = if is_selected_collection {
                    super::theme::ACCENT.gamma_multiply(0.12)
                } else {
                    egui::Color32::TRANSPARENT
                };

                egui::Frame::default()
                    .fill(frame_fill)
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let icon = if is_expanded { "▼" } else { "▶" };
                            if ui
                                .add(
                                    egui::Label::new(
                                        egui::RichText::new(icon).size(10.0).color(super::theme::TEXT_MUTED),
                                    )
                                    .sense(egui::Sense::click()),
                                )
                                .clicked()
                            {
                                if is_expanded {
                                    state.expanded_collections.remove(&collection.id);
                                } else {
                                    state.expanded_collections.insert(collection.id.clone());
                                }
                            }

                            // Collection name — clicking selects the collection
                            let name_label = egui::RichText::new(&collection.name)
                                .strong()
                                .size(14.0)
                                .color(if is_selected_collection {
                                    super::theme::ACCENT
                                } else {
                                    super::theme::TEXT_PRIMARY
                                });

                            if ui
                                .add(egui::Label::new(name_label).sense(egui::Sense::click()))
                                .clicked()
                            {
                                action = TreeAction::SelectCollection(collection.id.clone());
                                // Also expand
                                state.expanded_collections.insert(collection.id.clone());
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if super::theme::icon_button(ui, "x", "Delete collection") {
                                    action = TreeAction::DeleteCollection(collection.id.clone());
                                }
                                if super::theme::icon_button(ui, "+r", "New request") {
                                    action = TreeAction::NewRequest(collection.id.clone(), None);
                                }
                                if super::theme::icon_button(ui, "+f", "New folder") {
                                    action = TreeAction::NewFolder(collection.id.clone());
                                }
                            });
                        });
                    });

                // Children (if expanded)
                if is_expanded {
                    ui.indent(&collection.id, |ui| {
                        // Folders
                        let child_folders: Vec<&Folder> = folders
                            .iter()
                            .filter(|f| {
                                f.collection_id == collection.id && f.parent_folder_id.is_none()
                            })
                            .collect();

                        for folder in &child_folders {
                            let folder_expanded = state.expanded_folders.contains(&folder.id);

                            ui.add_space(1.0);
                            egui::Frame::default()
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let icon = if folder_expanded { "▼" } else { "▶" };
                                        if ui
                                            .add(
                                                egui::Label::new(
                                                    egui::RichText::new(format!("{icon}"))
                                                        .size(10.0)
                                                        .color(super::theme::TEXT_MUTED),
                                                )
                                                .sense(egui::Sense::click()),
                                            )
                                            .clicked()
                                        {
                                            if folder_expanded {
                                                state.expanded_folders.remove(&folder.id);
                                            } else {
                                                state.expanded_folders.insert(folder.id.clone());
                                            }
                                        }

                                        ui.label(
                                            egui::RichText::new(&folder.name)
                                                .size(13.0)
                                                .color(super::theme::TEXT_PRIMARY),
                                        );

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if super::theme::icon_button(ui, "x", "Delete folder") {
                                                    action = TreeAction::DeleteFolder(folder.id.clone());
                                                }
                                                if super::theme::icon_button(ui, "+", "New request") {
                                                    action = TreeAction::NewRequest(
                                                        collection.id.clone(),
                                                        Some(folder.id.clone()),
                                                    );
                                                }
                                            },
                                        );
                                    });
                                });

                            if folder_expanded {
                                ui.indent(&folder.id, |ui| {
                                    let folder_requests: Vec<&Request> = requests
                                        .iter()
                                        .filter(|r| r.folder_id.as_deref() == Some(&folder.id))
                                        .collect();

                                    for req in folder_requests {
                                        let a = render_request_item(ui, req, state);
                                        if !matches!(a, TreeAction::None) {
                                            action = a;
                                        }
                                    }
                                });
                            }
                        }

                        // Top-level requests
                        let orphan_requests: Vec<&Request> = requests
                            .iter()
                            .filter(|r| r.collection_id == collection.id && r.folder_id.is_none())
                            .collect();

                        for req in orphan_requests {
                            let a = render_request_item(ui, req, state);
                            if !matches!(a, TreeAction::None) {
                                action = a;
                            }
                        }
                    });
                }

                ui.add_space(4.0);
            }
        });

    action
}

fn render_request_item(
    ui: &mut egui::Ui,
    req: &Request,
    state: &mut CollectionTreeState,
) -> TreeAction {
    let mut action = TreeAction::None;
    let is_selected = state.selected_request_id.as_deref() == Some(&req.id);
    let is_renaming = state.rename_id.as_deref() == Some(&req.id);

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

    egui::Frame::default()
        .fill(fill)
        .stroke(stroke)
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Small colored dot for visual interest
                ui.label(egui::RichText::new("●").size(8.0).color(super::theme::ACCENT));

                if is_renaming {
                    // Inline rename text edit
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut state.rename_buf)
                            .desired_width(140.0)
                            .font(egui::TextStyle::Body),
                    );
                    if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let new_name = state.rename_buf.clone();
                        let id = req.id.clone();
                        state.rename_id = None;
                        if !new_name.is_empty() {
                            action = TreeAction::RenameRequest(id, new_name);
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        state.rename_id = None;
                    }
                    // Auto-focus
                    resp.request_focus();
                } else {
                    // Request name label
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

                    if resp.clicked() {
                        action = TreeAction::SelectRequest(req.id.clone());
                    }

                    // Double-click to rename
                    if resp.double_clicked() {
                        state.rename_id = Some(req.id.clone());
                        state.rename_buf = req.name.clone();
                    }
                }

                // Right-side actions
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if super::theme::icon_button(ui, "x", "Delete request") {
                        action = TreeAction::DeleteRequest(req.id.clone());
                    }
                    if !is_renaming {
                        if super::theme::icon_button(ui, "ab", "Rename") {
                            state.rename_id = Some(req.id.clone());
                            state.rename_buf = req.name.clone();
                        }
                    }
                });
            });
        });

    ui.add_space(1.0);
    action
}
