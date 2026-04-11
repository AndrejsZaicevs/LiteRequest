use eframe::egui;

/// Renders a JSON value as an interactive tree
pub fn json_tree_ui(ui: &mut egui::Ui, value: &serde_json::Value, path: &str, expanded: &mut std::collections::HashSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                render_node(ui, key, val, &child_path, expanded);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let key = format!("[{i}]");
                let child_path = format!("{path}{key}");
                render_node(ui, &key, val, &child_path, expanded);
            }
        }
        _ => {
            render_leaf(ui, value);
        }
    }
}

fn render_node(
    ui: &mut egui::Ui,
    key: &str,
    value: &serde_json::Value,
    path: &str,
    expanded: &mut std::collections::HashSet<String>,
) {
    let is_expandable = matches!(value, serde_json::Value::Object(_) | serde_json::Value::Array(_));

    if is_expandable {
        let is_open = expanded.contains(path);
        let icon = if is_open { "▼" } else { "▶" };
        let count = match value {
            serde_json::Value::Object(m) => format!("{{{}}}", m.len()),
            serde_json::Value::Array(a) => format!("[{}]", a.len()),
            _ => String::new(),
        };

        ui.horizontal(|ui| {
            if ui
                .add(egui::Label::new(
                    egui::RichText::new(icon).monospace().size(10.0),
                ).sense(egui::Sense::click()))
                .clicked()
            {
                if is_open {
                    expanded.remove(path);
                } else {
                    expanded.insert(path.to_string());
                }
            }
            ui.label(
                egui::RichText::new(format!("{key}:"))
                    .color(egui::Color32::from_rgb(156, 220, 254))
                    .monospace(),
            );
            ui.label(
                egui::RichText::new(count)
                    .color(egui::Color32::GRAY)
                    .monospace()
                    .size(11.0),
            );
        });

        if is_open {
            ui.indent(path, |ui| {
                json_tree_ui(ui, value, path, expanded);
            });
        }
    } else {
        ui.horizontal(|ui| {
            ui.add_space(14.0); // align with expand icons
            ui.label(
                egui::RichText::new(format!("{key}:"))
                    .color(egui::Color32::from_rgb(156, 220, 254))
                    .monospace(),
            );
            render_leaf(ui, value);
        });
    }
}

fn render_leaf(ui: &mut egui::Ui, value: &serde_json::Value) {
    let (text, color) = match value {
        serde_json::Value::String(s) => (format!("\"{s}\""), egui::Color32::from_rgb(206, 145, 120)),
        serde_json::Value::Number(n) => (n.to_string(), egui::Color32::from_rgb(181, 206, 168)),
        serde_json::Value::Bool(b) => (b.to_string(), egui::Color32::from_rgb(86, 156, 214)),
        serde_json::Value::Null => ("null".to_string(), egui::Color32::GRAY),
        _ => (value.to_string(), egui::Color32::WHITE),
    };

    ui.label(egui::RichText::new(text).color(color).monospace());
}
