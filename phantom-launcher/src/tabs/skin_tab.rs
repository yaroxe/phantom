//! Skin manager tab: 3D character preview (front and back), customize skin, and skin library.
use crate::{app::App, theme};
use egui::{
    vec2, Align, Color32, CursorIcon, Frame, Layout, Margin, Response, RichText, Rounding, Stroke,
    TextureOptions, Ui,
};
use image::{GenericImageView, Rgba, RgbaImage};
use std::{
    fs,
    path::{Path, PathBuf},
};

const DEFAULT_SKIN_BYTES: &[u8] = include_bytes!("../../assets/skins/steve.png");

pub const PRESET_SKINS: &[(&str, &[u8], bool)] = &[
    ("Зек Батя", include_bytes!("../../assets/skins/zek_batya.png"), false),
    ("Стив", include_bytes!("../../assets/skins/steve.png"), false),
    ("Алекс", include_bytes!("../../assets/skins/alex.png"), true),
    ("Кибер-фантом", include_bytes!("../../assets/skins/cyber_phantom.png"), false),
    ("Ветровка Стрит", include_bytes!("../../assets/skins/neon.png"), false),
    ("Спецназ", include_bytes!("../../assets/skins/specops.png"), false),
];

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

fn draw_textured_triangle(
    canvas: &mut RgbaImage,
    z_buf: &mut [f32],
    p0: (f32, f32, f32),
    p1: (f32, f32, f32),
    p2: (f32, f32, f32),
    uv0: (f32, f32),
    uv1: (f32, f32),
    uv2: (f32, f32),
    skin: &RgbaImage,
    uv_bounds: (u32, u32, u32, u32),
    shade: f32,
) {
    let w = canvas.width() as usize;
    let h = canvas.height() as usize;
    if w == 0 || h == 0 {
        return;
    }

    let edge = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| -> f32 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    };

    let area = edge((p0.0, p0.1), (p1.0, p1.1), (p2.0, p2.1));
    if area <= 1e-4 {
        return;
    }
    let inv_area = 1.0 / area;

    let min_x = (p0.0.min(p1.0).min(p2.0).floor() as i32).clamp(0, (w - 1) as i32) as usize;
    let max_x = (p0.0.max(p1.0).max(p2.0).ceil() as i32).clamp(0, (w - 1) as i32) as usize;
    let min_y = (p0.1.min(p1.1).min(p2.1).floor() as i32).clamp(0, (h - 1) as i32) as usize;
    let max_y = (p0.1.max(p1.1).max(p2.1).ceil() as i32).clamp(0, (h - 1) as i32) as usize;

    let (ux, uy, uw, uh) = uv_bounds;
    let (sw, sh) = skin.dimensions();

    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let p = (px, py);
            let w0 = edge((p1.0, p1.1), (p2.0, p2.1), p) * inv_area;
            let w1 = edge((p2.0, p2.1), (p0.0, p0.1), p) * inv_area;
            let w2 = edge((p0.0, p0.1), (p1.0, p1.1), p) * inv_area;

            if w0 >= -0.001 && w1 >= -0.001 && w2 >= -0.001 {
                let z = w0 * p0.2 + w1 * p1.2 + w2 * p2.2;
                let idx = y * w + x;
                if z > z_buf[idx] - 0.001 {
                    let u = w0 * uv0.0 + w1 * uv1.0 + w2 * uv2.0;
                    let v = w0 * uv0.1 + w1 * uv1.1 + w2 * uv2.1;
                    let tx = (u.floor() as u32).clamp(ux, ux + uw.saturating_sub(1));
                    let ty = (v.floor() as u32).clamp(uy, uy + uh.saturating_sub(1));

                    if tx < sw && ty < sh {
                        let tex_px = skin.get_pixel(tx, ty);
                        if tex_px[3] >= 12 {
                            let r = ((tex_px[0] as f32 * shade).clamp(0.0, 255.0)) as u8;
                            let g = ((tex_px[1] as f32 * shade).clamp(0.0, 255.0)) as u8;
                            let b = ((tex_px[2] as f32 * shade).clamp(0.0, 255.0)) as u8;
                            let col = Rgba([r, g, b, tex_px[3]]);

                            if tex_px[3] >= 240 {
                                *canvas.get_pixel_mut(x as u32, y as u32) = col;
                                z_buf[idx] = z;
                            } else {
                                blend_pixel(canvas.get_pixel_mut(x as u32, y as u32), col);
                                if tex_px[3] >= 180 {
                                    z_buf[idx] = z;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Rotates point around a limb's local pivot point (for natural running/standing pose).
fn rotate_limb(
    p: (f32, f32, f32),
    pivot: (f32, f32, f32),
    pitch_deg: f32,
    roll_deg: f32,
    yaw_deg: f32,
) -> (f32, f32, f32) {
    let vx = p.0 - pivot.0;
    let vy = p.1 - pivot.1;
    let vz = p.2 - pivot.2;

    // Pitch around X
    let rad_p = pitch_deg.to_radians();
    let y1 = vy * rad_p.cos() - vz * rad_p.sin();
    let z1 = vy * rad_p.sin() + vz * rad_p.cos();
    let x1 = vx;

    // Roll around Z
    let rad_r = roll_deg.to_radians();
    let x2 = x1 * rad_r.cos() - y1 * rad_r.sin();
    let y2 = x1 * rad_r.sin() + y1 * rad_r.cos();
    let z2 = z1;

    // Yaw around Y
    let rad_y = yaw_deg.to_radians();
    let x3 = x2 * rad_y.cos() + z2 * rad_y.sin();
    let z3 = -x2 * rad_y.sin() + z2 * rad_y.cos();
    let y3 = y2;

    (pivot.0 + x3, pivot.1 + y3, pivot.2 + z3)
}

/// Renders a full 3D isometric Minecraft character with lifelike pose, distinct limbs, and directional lighting.
pub fn render_3d_skin(
    skin: &RgbaImage,
    is_slim: bool,
    yaw_deg: f32,
    pitch_deg: f32,
    width: u32,
    height: u32,
) -> egui::ColorImage {
    let (sw, sh) = skin.dimensions();
    let is_64x64 = sh >= 64;
    let arm_w = if is_slim { 3 } else { 4 };

    let cam_yaw = yaw_deg.to_radians();
    let cam_pitch = pitch_deg.to_radians();
    // Character total height is 32 units (0 to 32). Center is at y=16.
    let scale = (height as f32 * 0.78) / 32.0;

    let rot_cam = |p: (f32, f32, f32)| -> (f32, f32, f32) {
        let (x, y, z) = p;
        // Center around (0, 16, 0)
        let cy = y - 16.0;
        // Yaw around Y
        let x1 = x * cam_yaw.cos() + z * cam_yaw.sin();
        let y1 = cy;
        let z1 = -x * cam_yaw.sin() + z * cam_yaw.cos();
        // Pitch around X
        let x2 = x1;
        let y2 = y1 * cam_pitch.cos() - z1 * cam_pitch.sin();
        let z2 = y1 * cam_pitch.sin() + z1 * cam_pitch.cos();
        (x2, y2 + 16.0, z2)
    };

    let project = |p_rot: (f32, f32, f32)| -> (f32, f32, f32) {
        let (x2, y2, z2) = p_rot;
        let sx = (width as f32) * 0.5 + x2 * scale;
        // y2=16 is middle of screen
        let sy = (height as f32) * 0.5 - (y2 - 16.0) * scale;
        (sx, sy, z2)
    };

    let interp = |a: (f32, f32, f32), b: (f32, f32, f32), t: f32| -> (f32, f32, f32) {
        (
            a.0 + (b.0 - a.0) * t,
            a.1 + (b.1 - a.1) * t,
            a.2 + (b.2 - a.2) * t,
        )
    };

    let mut canvas = RgbaImage::new(width, height);
    let mut z_buf = vec![-999999.0f32; (width * height) as usize];

    let mut add_box = |bx: f32,
                       by: f32,
                       bz: f32,
                       w: f32,
                       h: f32,
                       d: f32,
                       uv_map: &[(&str, (u32, u32, u32, u32))],
                       pivot: (f32, f32, f32),
                       limb_pitch: f32,
                       limb_roll: f32,
                       limb_yaw: f32,
                       inflate: f32| {
        let x0 = bx - inflate;
        let x1 = bx + w + inflate;
        let y0 = by - inflate;
        let y1 = by + h + inflate;
        let z0 = bz - inflate;
        let z1 = bz + d + inflate;

        let faces = [
            ("top",    [(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)], 1.15f32),
            ("bottom", [(x0, y0, z1), (x1, y0, z1), (x1, y0, z0), (x0, y0, z0)], 0.55f32),
            ("right",  [(x0, y1, z0), (x0, y1, z1), (x0, y0, z1), (x0, y0, z0)], 0.82f32),
            ("left",   [(x1, y1, z1), (x1, y1, z0), (x1, y0, z0), (x1, y0, z1)], 0.75f32),
            ("front",  [(x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1)], 1.00f32),
            ("back",   [(x1, y1, z0), (x0, y1, z0), (x0, y0, z0), (x1, y0, z0)], 0.88f32),
        ];

        for (fname, corners, shade) in faces {
            let Some(&(_, (ux, uy, uw, uh))) = uv_map.iter().find(|(name, _)| *name == fname) else {
                continue;
            };

            let c0 = project(rot_cam(rotate_limb(corners[0], pivot, limb_pitch, limb_roll, limb_yaw)));
            let c1 = project(rot_cam(rotate_limb(corners[1], pivot, limb_pitch, limb_roll, limb_yaw)));
            let c2 = project(rot_cam(rotate_limb(corners[2], pivot, limb_pitch, limb_roll, limb_yaw)));
            let c3 = project(rot_cam(rotate_limb(corners[3], pivot, limb_pitch, limb_roll, limb_yaw)));

            let uv0 = (ux as f32, uy as f32);
            let uv1 = ((ux + uw) as f32, uy as f32);
            let uv2 = ((ux + uw) as f32, (uy + uh) as f32);
            let uv3 = (ux as f32, (uy + uh) as f32);

            draw_textured_triangle(&mut canvas, &mut z_buf, c0, c1, c2, uv0, uv1, uv2, skin, (ux, uy, uw, uh), shade);
            draw_textured_triangle(&mut canvas, &mut z_buf, c0, c2, c3, uv0, uv2, uv3, skin, (ux, uy, uw, uh), shade);
        }
    };

    // 1. Head (Pivot at neck: 0, 24, 0; slight tilt)
    let head_uv = [
        ("top", (8, 0, 8, 8)),
        ("bottom", (16, 0, 8, 8)),
        ("right", (0, 8, 8, 8)),
        ("front", (8, 8, 8, 8)),
        ("left", (16, 8, 8, 8)),
        ("back", (24, 8, 8, 8)),
    ];
    let hat_uv = [
        ("top", (40, 0, 8, 8)),
        ("bottom", (48, 0, 8, 8)),
        ("right", (32, 8, 8, 8)),
        ("front", (40, 8, 8, 8)),
        ("left", (48, 8, 8, 8)),
        ("back", (56, 8, 8, 8)),
    ];
    add_box(-4.0, 24.0, -4.0, 8.0, 8.0, 8.0, &head_uv, (0.0, 24.0, 0.0), -2.0, 0.0, 6.0, 0.0);
    add_box(-4.0, 24.0, -4.0, 8.0, 8.0, 8.0, &hat_uv, (0.0, 24.0, 0.0), -2.0, 0.0, 6.0, 0.45);

    // 2. Torso (Pivot at center: 0, 18, 0)
    let torso_uv = [
        ("top", (20, 16, 8, 4)),
        ("bottom", (28, 16, 8, 4)),
        ("right", (16, 20, 4, 12)),
        ("front", (20, 20, 8, 12)),
        ("left", (28, 20, 4, 12)),
        ("back", (32, 20, 8, 12)),
    ];
    let jacket_uv = [
        ("top", (20, 32, 8, 4)),
        ("bottom", (28, 32, 8, 4)),
        ("right", (16, 36, 4, 12)),
        ("front", (20, 36, 8, 12)),
        ("left", (28, 36, 4, 12)),
        ("back", (32, 36, 8, 12)),
    ];
    add_box(-4.0, 12.0, -2.0, 8.0, 12.0, 4.0, &torso_uv, (0.0, 18.0, 0.0), 0.0, 0.0, 0.0, 0.0);
    if is_64x64 {
        add_box(-4.0, 12.0, -2.0, 8.0, 12.0, 4.0, &jacket_uv, (0.0, 18.0, 0.0), 0.0, 0.0, 0.0, 0.35);
    }

    // 3. Right Arm (Pivot at shoulder: -4, 22, 0; swings forward towards camera and slightly outward)
    let aw = arm_w as f32;
    let rarm_uv = [
        ("top", (44, 16, arm_w, 4)),
        ("bottom", (44 + arm_w, 16, arm_w, 4)),
        ("right", (40, 20, 4, 12)),
        ("front", (44, 20, arm_w, 12)),
        ("left", (44 + arm_w, 20, 4, 12)),
        ("back", (48 + arm_w, 20, arm_w, 12)),
    ];
    let rsleeve_uv = [
        ("top", (44, 32, arm_w, 4)),
        ("bottom", (44 + arm_w, 32, arm_w, 4)),
        ("right", (40, 36, 4, 12)),
        ("front", (44, 36, arm_w, 12)),
        ("left", (44 + arm_w, 36, 4, 12)),
        ("back", (48 + arm_w, 36, arm_w, 12)),
    ];
    add_box(-4.0 - aw, 12.0, -2.0, aw, 12.0, 4.0, &rarm_uv, (-4.0, 22.0, 0.0), -14.0, -5.0, 0.0, 0.0);
    if is_64x64 {
        add_box(-4.0 - aw, 12.0, -2.0, aw, 12.0, 4.0, &rsleeve_uv, (-4.0, 22.0, 0.0), -14.0, -5.0, 0.0, 0.30);
    }

    // 4. Left Arm (Pivot at shoulder: 4, 22, 0; swings backward and outward)
    if is_64x64 {
        let larm_uv = [
            ("top", (36, 48, arm_w, 4)),
            ("bottom", (36 + arm_w, 48, arm_w, 4)),
            ("right", (32, 52, 4, 12)),
            ("front", (36, 52, arm_w, 12)),
            ("left", (36 + arm_w, 52, 4, 12)),
            ("back", (40 + arm_w, 52, arm_w, 12)),
        ];
        let lsleeve_uv = [
            ("top", (52, 48, arm_w, 4)),
            ("bottom", (52 + arm_w, 48, arm_w, 4)),
            ("right", (48, 52, 4, 12)),
            ("front", (52, 52, arm_w, 12)),
            ("left", (52 + arm_w, 52, 4, 12)),
            ("back", (56 + arm_w, 52, arm_w, 12)),
        ];
        add_box(4.0, 12.0, -2.0, aw, 12.0, 4.0, &larm_uv, (4.0, 22.0, 0.0), 14.0, 5.0, 0.0, 0.0);
        add_box(4.0, 12.0, -2.0, aw, 12.0, 4.0, &lsleeve_uv, (4.0, 22.0, 0.0), 14.0, 5.0, 0.0, 0.30);
    } else {
        add_box(4.0, 12.0, -2.0, aw, 12.0, 4.0, &rarm_uv, (4.0, 22.0, 0.0), 14.0, 5.0, 0.0, 0.0);
    }

    // 5. Right Leg (Pivot at hip: -2, 12, 0; swings forward)
    let rleg_uv = [
        ("top", (4, 16, 4, 4)),
        ("bottom", (8, 16, 4, 4)),
        ("right", (0, 20, 4, 12)),
        ("front", (4, 20, 4, 12)),
        ("left", (8, 20, 4, 12)),
        ("back", (12, 20, 4, 12)),
    ];
    let rpants_uv = [
        ("top", (4, 32, 4, 4)),
        ("bottom", (8, 32, 4, 4)),
        ("right", (0, 36, 4, 12)),
        ("front", (4, 36, 4, 12)),
        ("left", (8, 36, 4, 12)),
        ("back", (12, 36, 4, 12)),
    ];
    add_box(-4.0, 0.0, -2.0, 4.0, 12.0, 4.0, &rleg_uv, (-2.0, 12.0, 0.0), 10.0, -1.0, 0.0, 0.0);
    if is_64x64 {
        add_box(-4.0, 0.0, -2.0, 4.0, 12.0, 4.0, &rpants_uv, (-2.0, 12.0, 0.0), 10.0, -1.0, 0.0, 0.30);
    }

    // 6. Left Leg (Pivot at hip: 2, 12, 0; swings backward)
    if is_64x64 {
        let lleg_uv = [
            ("top", (20, 48, 4, 4)),
            ("bottom", (24, 48, 4, 4)),
            ("right", (16, 52, 4, 12)),
            ("front", (20, 52, 4, 12)),
            ("left", (24, 52, 4, 12)),
            ("back", (28, 52, 4, 12)),
        ];
        let lpants_uv = [
            ("top", (4, 48, 4, 4)),
            ("bottom", (8, 48, 4, 4)),
            ("right", (0, 52, 4, 12)),
            ("front", (4, 52, 4, 12)),
            ("left", (8, 52, 4, 12)),
            ("back", (12, 52, 4, 12)),
        ];
        add_box(0.0, 0.0, -2.0, 4.0, 12.0, 4.0, &lleg_uv, (2.0, 12.0, 0.0), -10.0, 1.0, 0.0, 0.0);
        add_box(0.0, 0.0, -2.0, 4.0, 12.0, 4.0, &lpants_uv, (2.0, 12.0, 0.0), -10.0, 1.0, 0.0, 0.30);
    } else {
        add_box(0.0, 0.0, -2.0, 4.0, 12.0, 4.0, &rleg_uv, (2.0, 12.0, 0.0), -10.0, 1.0, 0.0, 0.0);
    }

    let pixels = canvas
        .pixels()
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    egui::ColorImage {
        size: [width as usize, height as usize],
        pixels,
    }
}

pub fn load_current_skin(path: &str) -> RgbaImage {
    if !path.is_empty() && Path::new(path).is_file() {
        if let Ok(img) = image::open(path) {
            return img.to_rgba8();
        }
    }
    image::load_from_memory(DEFAULT_SKIN_BYTES)
        .map(|img| img.to_rgba8())
        .unwrap_or_else(|_| RgbaImage::new(64, 64))
}

pub fn ensure_skins_library(library_dir: &Path) {
    let _ = fs::create_dir_all(library_dir);
    for (name, bytes, _) in PRESET_SKINS {
        let dest = library_dir.join(format!("{name}.png"));
        let _ = fs::write(dest, bytes);
    }
}

pub fn render(app: &mut App, ui: &mut Ui, _t: f32) {
    let library_dir = PathBuf::from(&app.cfg.phantom_dir).join("skins_library");
    ensure_skins_library(&library_dir);

    // Refresh 3D preview textures if dirty
    if app.skin_preview_dirty || app.skin_front_tex.is_none() || app.skin_side_tex.is_none() {
        let skin_img = load_current_skin(&app.cfg.skin_path);
        let is_slim = app.cfg.skin_model == "slim";

        // Front 3D posed view (facing front-right with separated limbs) - 2x supersampled
        let front_3d = render_3d_skin(&skin_img, is_slim, 22.0, 10.0, 240, 440);
        // Back 3D posed view (facing back-left) - 2x supersampled
        let back_3d = render_3d_skin(&skin_img, is_slim, -158.0, 10.0, 240, 440);

        app.skin_front_tex = Some(ui.ctx().load_texture(
            "skin_front_3d",
            front_3d,
            TextureOptions::NEAREST,
        ));
        app.skin_side_tex = Some(ui.ctx().load_texture(
            "skin_back_3d",
            back_3d,
            TextureOptions::NEAREST,
        ));
        app.skin_preview_dirty = false;
    }

    egui::ScrollArea::vertical()
        .id_source("skin_tab_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(8.0);

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
                        RichText::new("3D предпросмотр персонажа и встроенная коллекция скинов Phantom")
                            .size(12.5)
                            .color(theme::MUTED),
                    );
                });
            });

            ui.add_space(14.0);

            // Top Section: Left = Previews, Right = Controls
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
                                RichText::new("3D ПРЕДПРОСМОТР ПЕРСОНАЖА")
                                    .size(13.0)
                                    .color(theme::YELLOW)
                                    .strong(),
                            );
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                // Front 3D Card
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("3D Спереди").size(12.0).color(theme::MUTED));
                                    ui.add_space(4.0);
                                    Frame::none()
                                        .fill(theme::BG)
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 38, 42)))
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(Margin::same(6.0))
                                        .show(ui, |ui| {
                                            if let Some(tex) = &app.skin_front_tex {
                                                ui.image((tex.id(), vec2(120.0, 220.0)));
                                            } else {
                                                ui.allocate_space(vec2(120.0, 220.0));
                                            }
                                        });
                                });

                                ui.add_space(10.0);

                                // Back 3D Card
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("3D Сзади").size(12.0).color(theme::MUTED));
                                    ui.add_space(4.0);
                                    Frame::none()
                                        .fill(theme::BG)
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 38, 42)))
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(Margin::same(6.0))
                                        .show(ui, |ui| {
                                            if let Some(tex) = &app.skin_side_tex {
                                                ui.image((tex.id(), vec2(120.0, 220.0)));
                                            } else {
                                                ui.allocate_space(vec2(120.0, 220.0));
                                            }
                                        });
                                });
                            });
                        });
                    });

                ui.add_space(16.0);

                // --- CONTROLS PANEL (Right) ---
                let right_width = (ui.available_width() - 10.0).max(280.0);
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

                            // 1. Upload Skin Button
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
                                    app.cfg.skin_path = picked_str.clone();
                                    app.skin_preview_dirty = true;

                                    let game_dir = PathBuf::from(app.cfg.game_dir());
                                    let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);
                                    let _ = fs::create_dir_all(&game_dir);
                                    let _ = fs::create_dir_all(&phantom_dir);

                                    let _ = fs::copy(&picked, game_dir.join("custom_skin.png"));
                                    let _ = fs::copy(&picked, phantom_dir.join("custom_skin.png"));
                                    let _ = fs::copy(&picked, phantom_dir.join("skin.png"));

                                    if let Some(filename) = picked.file_name() {
                                        let _ = fs::copy(&picked, library_dir.join(filename));
                                    }

                                    let is_slim = app.cfg.skin_model == "slim";
                                    let slim_str = if is_slim { "true" } else { "false" };
                                    let _ = fs::write(game_dir.join("custom_skin_slim.txt"), slim_str);
                                    let _ = fs::write(phantom_dir.join("custom_skin_slim.txt"), slim_str);

                                    app.cfg.save();
                                    app.status = Some((true, "Скин успешно загружен и применён!".into()));
                                }
                            }

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(12.0);

                            // 2. Model Selection (Classic vs Slim)
                            ui.label(RichText::new("Модель рук персонажа:").color(theme::TEXT).size(13.0).strong());
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                let is_classic = app.cfg.skin_model != "slim";
                                let btn_w = ((ui.available_width() - 8.0) * 0.5).max(120.0);
                                let classic_btn = |ui: &mut Ui, label: &str, active: bool| -> Response {
                                    let fill = if active { theme::YELLOW } else { theme::SIDE };
                                    let text_color = if active { theme::BG } else { theme::TEXT };
                                    ui.add_sized(
                                        [btn_w, 30.0],
                                        egui::Button::new(RichText::new(label).color(text_color).strong().size(12.0))
                                            .fill(fill)
                                            .stroke(Stroke::new(1.0, theme::BORDER))
                                            .rounding(Rounding::same(4.0)),
                                    )
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                };

                                if classic_btn(ui, "Стив (4px)", is_classic).clicked() && !is_classic {
                                    app.cfg.skin_model = "classic".into();
                                    app.skin_preview_dirty = true;
                                    let game_dir = PathBuf::from(app.cfg.game_dir());
                                    let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);
                                    let _ = fs::write(game_dir.join("custom_skin_slim.txt"), "false");
                                    let _ = fs::write(phantom_dir.join("custom_skin_slim.txt"), "false");
                                    app.cfg.save();
                                }

                                if classic_btn(ui, "Алекс (3px)", !is_classic).clicked() && is_classic {
                                    app.cfg.skin_model = "slim".into();
                                    app.skin_preview_dirty = true;
                                    let game_dir = PathBuf::from(app.cfg.game_dir());
                                    let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);
                                    let _ = fs::write(game_dir.join("custom_skin_slim.txt"), "true");
                                    let _ = fs::write(phantom_dir.join("custom_skin_slim.txt"), "true");
                                    app.cfg.save();
                                }
                            });

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(12.0);

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
                                let game_dir = PathBuf::from(app.cfg.game_dir());
                                let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);

                                let _ = fs::remove_file(game_dir.join("custom_skin.png"));
                                let _ = fs::remove_file(game_dir.join("custom_skin_slim.txt"));
                                let _ = fs::remove_file(phantom_dir.join("custom_skin.png"));
                                let _ = fs::remove_file(phantom_dir.join("custom_skin_slim.txt"));
                                let _ = fs::remove_file(phantom_dir.join("skin.png"));

                                app.cfg.skin_path.clear();
                                app.cfg.skin_model = "classic".into();
                                app.skin_preview_dirty = true;
                                app.cfg.save();
                                app.status = Some((true, "Установлен стандартный классический Стив".into()));
                            }

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // 4. Current Status info
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Текущий файл:").color(theme::MUTED).size(12.0));
                                let display_name = if app.cfg.skin_path.is_empty() {
                                    "Стандартный Стив".to_string()
                                } else {
                                    Path::new(&app.cfg.skin_path)
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| app.cfg.skin_path.clone())
                                };
                                let truncated_display: String = display_name.chars().take(22).collect();
                                ui.label(RichText::new(truncated_display).color(theme::TEXT).strong().size(12.0));
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

                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("✓ Скин автоматически применяется в игре и на серверах.")
                                    .color(Color32::from_rgb(120, 200, 140))
                                    .size(11.5),
                            );
                        });
                });
            });

            ui.add_space(16.0);

            // =========================================================================
            // BOTTOM SECTION: BUILT-IN SKIN LIBRARY WITH 3D PREVIEWS
            // =========================================================================
            Frame::none()
                .fill(theme::CARD)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(18.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("КОЛЛЕКЦИЯ СКИНОВ")
                                .size(14.0)
                                .color(theme::YELLOW)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("Выберите персонажа в один клик:")
                                .size(12.0)
                                .color(theme::MUTED),
                        );
                    });

                    ui.add_space(12.0);

                    // Read skins in library directory
                    let mut library_files: Vec<PathBuf> = Vec::new();
                    if let Ok(entries) = fs::read_dir(&library_dir) {
                        for e in entries.flatten() {
                            let p = e.path();
                            if p.is_file() && p.extension().is_some_and(|ext| ext == "png") {
                                library_files.push(p);
                            }
                        }
                    }
                    library_files.sort();

                    let mut skin_to_apply: Option<(PathBuf, String, bool)> = None;
                    let mut skin_to_delete: Option<PathBuf> = None;

                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for path in &library_files {
                                let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Скин");
                                let is_active = !app.cfg.skin_path.is_empty()
                                    && Path::new(&app.cfg.skin_path).file_stem() == path.file_stem();

                                let is_slim = filename.to_lowercase().contains("алекс")
                                    || filename.to_lowercase().contains("alex")
                                    || filename.to_lowercase().contains("slim");

                                // Card frame
                                let card_border = if is_active { theme::YELLOW } else { theme::BORDER };
                                let card_fill = if is_active {
                                    Color32::from_rgb(32, 32, 26)
                                } else {
                                    theme::BG
                                };

                                Frame::none()
                                    .fill(card_fill)
                                    .stroke(Stroke::new(if is_active { 1.5 } else { 1.0 }, card_border))
                                    .rounding(Rounding::same(6.0))
                                    .inner_margin(Margin::same(10.0))
                                    .show(ui, |ui| {
                                        ui.set_width(140.0);
                                        ui.vertical_centered(|ui| {
                                            // Top row: Name + Delete button
                                            ui.horizontal(|ui| {
                                                let short_title: String = filename.chars().take(18).collect();
                                                ui.label(
                                                    RichText::new(short_title)
                                                        .size(12.0)
                                                        .color(if is_active { theme::YELLOW } else { theme::TEXT })
                                                        .strong(),
                                                );
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    let del_btn = egui::Button::new(
                                                        RichText::new("✕")
                                                            .size(11.0)
                                                            .color(Color32::from_rgb(180, 80, 80)),
                                                    )
                                                    .fill(Color32::TRANSPARENT);
                                                    if ui.add(del_btn).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                                                        skin_to_delete = Some(path.clone());
                                                    }
                                                });
                                            });

                                            ui.add_space(4.0);

                                            // 3D Full-body skin preview (100x150)
                                            let card_tex_id = format!("skin_3d_card_{}", path.to_string_lossy());
                                            if let Ok(img) = image::open(path) {
                                                let skin_rgba = img.to_rgba8();
                                                let card_color_img = render_3d_skin(
                                                    &skin_rgba,
                                                    is_slim,
                                                    22.0,
                                                    10.0,
                                                    200,
                                                    300,
                                                );
                                                let tex = ui.ctx().load_texture(
                                                    card_tex_id,
                                                    card_color_img,
                                                    TextureOptions::NEAREST,
                                                );
                                                ui.image((tex.id(), vec2(100.0, 150.0)));
                                            } else {
                                                ui.allocate_space(vec2(100.0, 150.0));
                                            }

                                            ui.add_space(6.0);

                                            // Apply button
                                            let btn_text = if is_active { "✓ АКТИВЕН" } else { "Выбрать" };
                                            let btn_fill = if is_active {
                                                theme::YELLOW
                                            } else {
                                                theme::SIDE
                                            };
                                            let btn_color = if is_active { theme::BG } else { theme::TEXT };

                                            let apply_btn = egui::Button::new(
                                                RichText::new(btn_text).size(11.5).color(btn_color).strong(),
                                            )
                                            .fill(btn_fill)
                                            .rounding(Rounding::same(4.0));

                                            if ui
                                                .add_sized([120.0, 26.0], apply_btn)
                                                .on_hover_cursor(CursorIcon::PointingHand)
                                                .clicked()
                                            {
                                                skin_to_apply = Some((path.clone(), filename.to_string(), is_slim));
                                            }
                                        });
                                    });

                                ui.add_space(10.0);
                            }
                        });
                    });

                    // Handle skin apply
                    if let Some((path, name, is_slim)) = skin_to_apply {
                        let game_dir = PathBuf::from(app.cfg.game_dir());
                        let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);
                        let _ = fs::create_dir_all(&game_dir);
                        let _ = fs::create_dir_all(&phantom_dir);

                        let _ = fs::copy(&path, game_dir.join("custom_skin.png"));
                        let _ = fs::copy(&path, phantom_dir.join("custom_skin.png"));
                        let _ = fs::copy(&path, phantom_dir.join("skin.png"));

                        let slim_str = if is_slim { "true" } else { "false" };
                        let _ = fs::write(game_dir.join("custom_skin_slim.txt"), slim_str);
                        let _ = fs::write(phantom_dir.join("custom_skin_slim.txt"), slim_str);

                        app.cfg.skin_path = path.to_string_lossy().to_string();
                        app.cfg.skin_model = if is_slim { "slim".into() } else { "classic".into() };
                        app.skin_preview_dirty = true;
                        app.cfg.save();
                        app.status = Some((true, format!("Скин «{}» применён!", name)));
                    }

                    // Handle skin delete
                    if let Some(del_path) = skin_to_delete {
                        let _ = fs::remove_file(&del_path);
                        if app.cfg.skin_path == del_path.to_string_lossy() {
                            app.cfg.skin_path.clear();
                            app.skin_preview_dirty = true;
                            app.cfg.save();
                        }
                        app.status = Some((true, "Скин удалён из библиотеки".into()));
                    }
                });

            ui.add_space(80.0);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_render_output() {
        let steve_img = image::load_from_memory(include_bytes!("../../assets/skins/steve.png")).unwrap().to_rgba8();
        let front = render_3d_skin(&steve_img, false, 22.0, 10.0, 240, 440);
        let mut out = RgbaImage::new(240, 440);
        for y in 0..440 {
            for x in 0..240 {
                let p = front.pixels[y * 240 + x];
                out.put_pixel(x as u32, y as u32, Rgba([p.r(), p.g(), p.b(), p.a()]));
            }
        }
        out.save("C:/Users/Ярослав/Phantom/tools/steve_render_front.png").unwrap();

        let batya_img = image::load_from_memory(include_bytes!("../../assets/skins/zek_batya.png")).unwrap().to_rgba8();
        let front_batya = render_3d_skin(&batya_img, false, 22.0, 10.0, 240, 440);
        let mut out_batya = RgbaImage::new(240, 440);
        for y in 0..440 {
            for x in 0..240 {
                let p = front_batya.pixels[y * 240 + x];
                out_batya.put_pixel(x as u32, y as u32, Rgba([p.r(), p.g(), p.b(), p.a()]));
            }
        }
        out_batya.save("C:/Users/Ярослав/Phantom/tools/batya_render_front.png").unwrap();
        println!("Saved steve and batya renders!");
    }
}

