#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
#![allow(warnings)]

pub mod app;
pub mod config;
pub mod fx;
pub mod java;
pub mod launch;
pub mod modrinth;
pub mod tabs;
pub mod theme;

use eframe::egui;

fn load_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../phantom-64.png");
    if let Ok(img) = image::load_from_memory(png_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Phantom Launcher")
        .with_inner_size([1280., 800.])
        .with_min_inner_size([900., 560.]);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Phantom Launcher",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app::App::new()))
        }),
    )
}
