mod db;
mod http;
mod models;
mod ui;
mod utils;

const ICON_PNG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app_icon.png"));

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let icon = eframe::icon_data::from_png_bytes(ICON_PNG)
        .expect("Failed to load app icon");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("LiteRequest")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "LiteRequest",
        options,
        Box::new(|cc| Ok(Box::new(ui::app::LiteRequestApp::new(cc)))),
    )
}

use eframe::egui;


