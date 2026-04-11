use eframe::egui;

// Accent color
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);
pub const SURFACE_0: egui::Color32 = egui::Color32::from_rgb(24, 24, 27);
pub const SURFACE_1: egui::Color32 = egui::Color32::from_rgb(32, 32, 36);
pub const SURFACE_2: egui::Color32 = egui::Color32::from_rgb(42, 42, 48);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(55, 55, 62);
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(228, 228, 231);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(140, 140, 150);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(90, 90, 100);

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    let mut visuals = egui::Visuals::dark();
    visuals.dark_mode = true;
    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = SURFACE_0;
    visuals.window_fill = SURFACE_1;
    visuals.window_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.bg_fill = SURFACE_1;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
    visuals.widgets.inactive.bg_fill = SURFACE_2;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(52, 52, 60);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);
    visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 60, 70);
    visuals.widgets.active.rounding = egui::Rounding::same(6.0);
    visuals.selection.bg_fill = egui::Color32::from_rgb(59, 130, 246).gamma_multiply(0.25);
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.indent = 16.0;

    // Bigger text
    use egui::FontFamily::Proportional;
    use egui::TextStyle;
    style.text_styles.insert(TextStyle::Body, egui::FontId::new(14.0, Proportional));
    style.text_styles.insert(TextStyle::Button, egui::FontId::new(14.0, Proportional));
    style.text_styles.insert(TextStyle::Heading, egui::FontId::new(18.0, Proportional));
    style.text_styles.insert(TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Monospace));
    style.text_styles.insert(TextStyle::Small, egui::FontId::new(11.0, Proportional));

    ctx.set_style(style);
}

pub fn status_color(status: u16) -> egui::Color32 {
    match status {
        200..=299 => egui::Color32::from_rgb(73, 204, 144),
        300..=399 => egui::Color32::from_rgb(252, 161, 48),
        400..=499 => egui::Color32::from_rgb(249, 62, 62),
        500..=599 => egui::Color32::from_rgb(255, 87, 87),
        _ => egui::Color32::GRAY,
    }
}

/// Draw a section header with a subtle background
pub fn section_header(ui: &mut egui::Ui, text: &str) {
    let rect = ui.available_rect_before_wrap();
    let header_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(rect.width(), 28.0),
    );
    ui.painter().rect_filled(header_rect, 0.0, SURFACE_2);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(header_rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new(text)
                    .strong()
                    .size(12.0)
                    .color(TEXT_SECONDARY),
            );
        });
    });
    ui.advance_cursor_after_rect(header_rect);
    ui.add_space(4.0);
}

/// Styled pill button (colored background)
pub fn pill_button(ui: &mut egui::Ui, text: &str, color: egui::Color32) -> bool {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .strong()
                .size(13.0)
                .color(egui::Color32::WHITE),
        )
        .fill(color)
        .rounding(egui::Rounding::same(6.0))
        .min_size(egui::vec2(0.0, 30.0)),
    )
    .clicked()
}

/// Subtle icon button
pub fn icon_button(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> bool {
    ui.add(
        egui::Button::new(egui::RichText::new(icon).size(15.0))
            .frame(false)
            .min_size(egui::vec2(24.0, 24.0)),
    )
    .on_hover_text(tooltip)
    .clicked()
}

/// Framed section with border and padding
pub fn framed_section(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::default()
        .fill(SURFACE_1)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            add_contents(ui);
        });
}
