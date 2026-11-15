//! Skin manager tab: preview character (front and side) and customize offline skin.
use crate::{app::App, theme};
use egui::{vec2, Align, Color32, CursorIcon, Frame, Layout, Margin, Response, RichText, Rounding, Stroke, TextureOptions, Ui};
use image::{imageops, GenericImageView, Rgba, RgbaImage};
use std::path::{Path, PathBuf};

const DEFAULT_SKIN_BYTES: &[u8] = include_bytes!("../../assets/default_skin.png");

/// Alpha blend src pixel over dst pixel
fn blend_pixel(dst: &mut Rgba<u8>, src: Rgba<u8>) {
    let src_a = src[3] as f32 / 255.0;
    if src_a <= 0.0 {
        return;
    }
    if src_a >= 1.0 {
        *dst = src;
        return;
    }
    let dst_a = dst[3] as f32 / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a <= 0.0 {
        return;
    }
    for i in 0..3 {
        let sc = src[i] as f32 / 255.0;
        let dc = dst[i] as f32 / 255.0;
        let out_c = (sc * src_a + dc * dst_a * (1.0 - src_a)) / out_a;
        dst[i] = (out_c * 255.0).clamp(0.0, 255.0) as u8;
    }
    dst[3] = (out_a * 255.0).clamp(0.0, 255.0) as u8;
}

fn overlay_rect(
    dst: &mut RgbaImage,
    dst_x: u32,
    dst_y: u32,
    src: &RgbaImage,
    src_x: u32,
    src_y: u32,
    w: u32,
    h: u32,
    mirror_x: bool,
) {
    let (sw, sh) = src.dimensions();
    for y in 0..h {
        let sy = src_y + y;
        if sy >= sh {
            continue;
        }
        let dy = dst_y + y;
        if dy >= dst.height() {
            continue;
        }
        for x in 0..w {
            let sx = if mirror_x {
                src_x + (w - 1 - x)
            } else {
                src_x + x
            };
            if sx >= sw {
                continue;
            }
            let dx = dst_x + x;
            if dx >= dst.width() {
                continue;
            }
            let p = src.get_pixel(sx, sy);
            blend_pixel(dst.get_pixel_mut(dx, dy), *p);
        }
    }
}

/// Generate 16x32 front and side character previews upscaled to 128x256
pub fn render_previews(skin: &RgbaImage, is_slim: bool) -> (egui::ColorImage, egui::ColorImage) {
    let (sw, sh) = skin.dimensions();
    let is_64x64 = sh >= 64;
    let arm_w = if is_slim { 3 } else { 4 };

    // --- FRONT VIEW ---
    let mut front_canvas = RgbaImage::new(16, 32);

    // 1. Head base: (8, 8, 8, 8) -> (4, 0)
    overlay_rect(&mut front_canvas, 4, 0, skin, 8, 8, 8, 8, false);
    // 2. Head hat overlay: (40, 8, 8, 8) -> (4, 0)
    overlay_rect(&mut front_canvas, 4, 0, skin, 40, 8, 8, 8, false);

    // 3. Torso base: (20, 20, 8, 12) -> (4, 8)
    overlay_rect(&mut front_canvas, 4, 8, skin, 20, 20, 8, 12, false);
    if is_64x64 {
        // Torso jacket overlay: (20, 36, 8, 12) -> (4, 8)
        overlay_rect(&mut front_canvas, 4, 8, skin, 20, 36, 8, 12, false);
    }

    // 4. Right Arm base: (44, 20, arm_w, 12) -> (4 - arm_w, 8)
    overlay_rect(&mut front_canvas, 4 - arm_w, 8, skin, 44, 20, arm_w, 12, false);
    if is_64x64 {
        // Right Arm sleeve overlay: (44, 36, arm_w, 12) -> (4 - arm_w, 8)
        overlay_rect(&mut front_canvas, 4 - arm_w, 8, skin, 44, 36, arm_w, 12, false);
    }

    // 5. Left Arm base:
    if is_64x64 {
        overlay_rect(&mut front_canvas, 12, 8, skin, 36, 52, arm_w, 12, false);
        overlay_rect(&mut front_canvas, 12, 8, skin, 52, 52, arm_w, 12, false);
    } else {
        // In 64x32, mirror right arm
        overlay_rect(&mut front_canvas, 12, 8, skin, 44, 20, arm_w, 12, true);
    }

    // 6. Right Leg base: (4, 20, 4, 12) -> (4, 20)
    overlay_rect(&mut front_canvas, 4, 20, skin, 4, 20, 4, 12, false);
    if is_64x64 {
        overlay_rect(&mut front_canvas, 4, 20, skin, 4, 36, 4, 12, false);
    }

    // 7. Left Leg base:
    if is_64x64 {
        overlay_rect(&mut front_canvas, 8, 20, skin, 20, 52, 4, 12, false);
        overlay_rect(&mut front_canvas, 8, 20, skin, 4, 52, 4, 12, false);
    } else {
        // In 64x32, mirror right leg
        overlay_rect(&mut front_canvas, 8, 20, skin, 4, 20, 4, 12, true);
    }

    // --- SIDE VIEW (Right Profile) ---
    let mut side_canvas = RgbaImage::new(16, 32);

    // 1. Head right: (0, 8, 8, 8) -> (4, 0)
    overlay_rect(&mut side_canvas, 4, 0, skin, 0, 8, 8, 8, false);
    overlay_rect(&mut side_canvas, 4, 0, skin, 32, 8, 8, 8, false);

    // 2. Torso right side: (16, 20, 4, 12) -> (6, 8)
    overlay_rect(&mut side_canvas, 6, 8, skin, 16, 20, 4, 12, false);
    if is_64x64 {
        overlay_rect(&mut side_canvas, 6, 8, skin, 16, 36, 4, 12, false);
    }

    // 3. Right Arm outer side: (40, 20, 4, 12) -> (6, 8)
    overlay_rect(&mut side_canvas, 6, 8, skin, 40, 20, 4, 12, false);
    if is_64x64 {
        overlay_rect(&mut side_canvas, 6, 8, skin, 40, 36, 4, 12, false);
    }

    // 4. Right Leg outer side: (0, 20, 4, 12) -> (6, 20)
    overlay_rect(&mut side_canvas, 6, 20, skin, 0, 20, 4, 12, false);
    if is_64x64 {
        overlay_rect(&mut side_canvas, 6, 20, skin, 0, 36, 4, 12, false);
    }

    // Nearest-neighbor upscale x8 -> 128 x 256 for razor-sharp rendering
    let scale = 8;
    let upscaled_front = imageops::resize(
        &front_canvas,
        front_canvas.width() * scale,
        front_canvas.height() * scale,
        imageops::FilterType::Nearest,
    );
    let upscaled_side = imageops::resize(
        &side_canvas,
        side_canvas.width() * scale,
        side_canvas.height() * scale,
        imageops::FilterType::Nearest,
    );

    let to_egui = |img: RgbaImage| -> egui::ColorImage {
        let size = [img.width() as usize, img.height() as usize];
        let pixels = img
            .pixels()
            .map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        egui::ColorImage { size, pixels }
    };

    (to_egui(upscaled_front), to_egui(upscaled_side))
}

pub fn load_current_skin(path: &str) -> RgbaImage {
    if !path.is_empty() && Path::new(path).is_file() {
        if let Ok(img) = image::open(path) {
            return img.to_rgba8();
        }
    }
    // Fallback to embedded default skin
    image::load_from_memory(DEFAULT_SKIN_BYTES)
        .map(|img| img.to_rgba8())
        .unwrap_or_else(|_| RgbaImage::new(64, 64))
}

pub fn render(app: &mut App, ui: &mut Ui, _t: f32) {
    // Check if preview textures need initialization / refresh
    if app.skin_preview_dirty || app.skin_front_tex.is_none() || app.skin_side_tex.is_none() {
        let skin_img = load_current_skin(&app.cfg.skin_path);
        let is_slim = app.cfg.skin_model == "slim";
        let (front_img, side_img) = render_previews(&skin_img, is_slim);

        app.skin_front_tex = Some(ui.ctx().load_texture(
            "skin_front_preview",
            front_img,
            TextureOptions::NEAREST,
        ));
        app.skin_side_tex = Some(ui.ctx().load_texture(
            "skin_side_preview",
            side_img,
            TextureOptions::NEAREST,
        ));
        app.skin_preview_dirty = false;
    }

    ui.add_space(14.0);

    // Header
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("МЕНЕДЖЕР СКИНОВ")
                    .size(22.0)
                    .color(theme::YELLOW)
                    .strong(),
            );
            ui.label(
                RichText::new("Настройка внешнего вида и типа модели вашего персонажа")
                    .size(12.5)
                    .color(theme::MUTED),
            );
        });
    });

    ui.add_space(16.0);

    // Main content: Left = Previews, Right = Controls
    ui.horizontal_top(|ui| {
        // --- PREVIEW PANEL (Left) ---
        Frame::none()
            .fill(theme::CARD)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(18.0))
            .show(ui, |ui| {
                ui.set_width(320.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("ПРЕДПРОСМОТР ПЕРСОНАЖА")
                            .size(13.0)
                            .color(theme::YELLOW)
                            .strong(),
                    );
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        // Front View Card
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Вид спереди").size(12.0).color(theme::MUTED));
                            ui.add_space(4.0);
                            Frame::none()
                                .fill(theme::BG)
                                .stroke(Stroke::new(1.0, Color32::from_rgb(38, 38, 42)))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::same(8.0))
                                .show(ui, |ui| {
                                    if let Some(tex) = &app.skin_front_tex {
                                        ui.image((tex.id(), vec2(120.0, 240.0)));
                                    } else {
                                        ui.allocate_space(vec2(120.0, 240.0));
                                    }
                                });
                        });

                        ui.add_space(12.0);

                        // Side View Card
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Вид сбоку").size(12.0).color(theme::MUTED));
                            ui.add_space(4.0);
                            Frame::none()
                                .fill(theme::BG)
                                .stroke(Stroke::new(1.0, Color32::from_rgb(38, 38, 42)))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::same(8.0))
                                .show(ui, |ui| {
                                    if let Some(tex) = &app.skin_side_tex {
                                        ui.image((tex.id(), vec2(120.0, 240.0)));
                                    } else {
                                        ui.allocate_space(vec2(120.0, 240.0));
                                    }
                                });
                        });
                    });
                });
            });

        ui.add_space(16.0);

        // --- CONTROLS PANEL (Right) ---
        let right_width = (ui.available_width() - 20.0).max(300.0);
        ui.vertical(|ui| {
            ui.set_width(right_width);

            Frame::none()
                .fill(theme::CARD)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(18.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("УПРАВЛЕНИЕ СКИНOM")
                            .size(13.0)
                            .color(theme::YELLOW)
                            .strong(),
                    );
                    ui.add_space(14.0);

                    // 1. Upload Skin Button (Big yellow cyber button)
                    let upload_btn = egui::Button::new(
                        RichText::new("📁  ЗАГРУЗИТЬ СВОЙ СКИН (.PNG)")
                            .color(theme::BG)
                            .strong()
                            .size(13.5),
                    )
                    .fill(theme::YELLOW)
                    .rounding(Rounding::same(4.0));

                    if ui
                        .add_sized([ui.available_width(), 38.0], upload_btn)
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        if let Some(picked) = rfd::FileDialog::new()
                            .add_filter("Minecraft Skin (*.png)", &["png"])
                            .set_title("Выберите файл скина")
                            .pick_file()
                        {
                            let picked_str = picked.to_string_lossy().to_string();
                            app.cfg.skin_path = picked_str;
                            app.skin_preview_dirty = true;
                            // Immediately copy to %APPDATA%\phantom\skin.png for the client
                            let target = PathBuf::from(&app.cfg.phantom_dir).join("skin.png");
                            let _ = std::fs::create_dir_all(&app.cfg.phantom_dir);
                            let _ = std::fs::copy(&app.cfg.skin_path, &target);
                            app.cfg.save();
                            app.status = Some((true, "Скин успешно загружен и применён!".into()));
                        }
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(14.0);

                    // 2. Model Selection (Classic vs Slim)
                    ui.label(RichText::new("Модель рук персонажа:").color(theme::TEXT).size(13.0).strong());
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let is_classic = app.cfg.skin_model != "slim";
                        let classic_btn = |ui: &mut Ui, label: &str, active: bool| -> Response {
                            let fill = if active { theme::YELLOW } else { theme::SIDE };
                            let text_color = if active { theme::BG } else { theme::TEXT };
                            ui.add_sized(
                                [(ui.available_width() - 8.0) * 0.5, 30.0],
                                egui::Button::new(RichText::new(label).color(text_color).strong().size(12.5))
                                    .fill(fill)
                                    .stroke(Stroke::new(1.0, theme::BORDER))
                                    .rounding(Rounding::same(4.0)),
                            )
                            .on_hover_cursor(CursorIcon::PointingHand)
                        };

                        if classic_btn(ui, "Классический (Steve 4px)", is_classic).clicked() && !is_classic {
                            app.cfg.skin_model = "classic".into();
                            app.skin_preview_dirty = true;
                            app.cfg.save();
                        }

                        if classic_btn(ui, "Тонкие руки (Alex 3px)", !is_classic).clicked() && is_classic {
                            app.cfg.skin_model = "slim".into();
                            app.skin_preview_dirty = true;
                            app.cfg.save();
                        }
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(14.0);

                    // 3. Reset Button
                    let reset_btn = egui::Button::new(
                        RichText::new("↺  Сбросить на стандартный")
                            .color(theme::TEXT)
                            .size(12.5),
                    )
                    .fill(theme::SIDE)
                    .stroke(Stroke::new(1.0, theme::BORDER))
                    .rounding(Rounding::same(4.0));

                    if ui
                        .add_sized([ui.available_width(), 32.0], reset_btn)
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        app.cfg.skin_path.clear();
                        app.cfg.skin_model = "classic".into();
                        app.skin_preview_dirty = true;
                        // Remove custom skin.png from %APPDATA%\phantom\ if any
                        let target = PathBuf::from(&app.cfg.phantom_dir).join("skin.png");
                        let _ = std::fs::remove_file(target);
                        app.cfg.save();
                        app.status = Some((true, "Установлен стандартный скин по умолчанию".into()));
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // 4. Current Status info
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Текущий файл:").color(theme::MUTED).size(12.0));
                        let display_name = if app.cfg.skin_path.is_empty() {
                            "Стандартный скин (Steve)".to_string()
                        } else {
                            Path::new(&app.cfg.skin_path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| app.cfg.skin_path.clone())
                        };
                        ui.label(RichText::new(display_name).color(theme::TEXT).strong().size(12.0));
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Формат скина:").color(theme::MUTED).size(12.0));
                        let format_str = if app.cfg.skin_path.is_empty() {
                            "64x64 (Стандарт)".to_string()
                        } else if let Ok(img) = image::open(&app.cfg.skin_path) {
                            let (w, h) = img.dimensions();
                            format!("{w}x{h} (Minecraft)")
                        } else {
                            "—".to_string()
                        };
                        ui.label(RichText::new(format_str).color(theme::YELLOW).size(12.0));
                    });

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("✓ Скин автоматически применяется в одиночной игре и при подключении к серверам.")
                            .color(Color32::from_rgb(120, 200, 140))
                            .size(11.5),
                    );
                });
        });
    });
}
