//! Skin manager tab: Interactive 3D character preview, customize offline skin,
//! smooth vertical scrolling skin library, authentic 3D Minecraft poses, and 64x32 conversion.

use crate::{app::App, theme};
use egui::{
    pos2, vec2, Align, Color32, CursorIcon, Frame, Layout, Margin, Rect, RichText, Rounding,
    Stroke, TextureOptions, Ui,
};
use image::{GenericImageView, RgbaImage};
use std::{
    fs,
    path::{Path, PathBuf},
};

// Include generated embedded skins catalog from build.rs
include!(concat!(env!("OUT_DIR"), "/skins_catalog.rs"));

const DEFAULT_SKIN_BYTES: &[u8] = include_bytes!("../../assets/skins/zek_batya.png");

/// Upgrades a vintage 64x32 skin (e.g. Minecraft 0.12.3 / Beta / Alpha) to standard 64x64 format
/// with exact vanilla mirroring for the left arm and left leg.
pub fn convert_64x32_to_64x64(skin: &RgbaImage) -> RgbaImage {
    let (w, h) = skin.dimensions();
    let k = (w / 64).max(1);
    if h >= 64 * k {
        return skin.clone();
    }
    let mut out = RgbaImage::new(64 * k, 64 * k);
    for y in 0..h {
        for x in 0..w {
            out.put_pixel(x, y, *skin.get_pixel(x, y));
        }
    }

    // Minecraft 1.8+ legacy layout rects: (sx, sy, ox, oy, w, h)
    let rects: [(i32, i32, i32, i32, i32, i32); 12] = [
        (4, 16, 16, 32, 4, 4),    // leg top
        (8, 16, 16, 32, 4, 4),    // leg bottom
        (0, 20, 24, 32, 4, 12),   // leg outer
        (4, 20, 16, 32, 4, 12),   // leg front
        (8, 20, 8, 32, 4, 12),    // leg inner
        (12, 20, 16, 32, 4, 12),  // leg back
        (44, 16, -8, 32, 4, 4),   // arm top
        (48, 16, -8, 32, 4, 4),   // arm bottom
        (40, 20, 0, 32, 4, 12),   // arm outer
        (44, 20, -8, 32, 4, 12),  // arm front
        (48, 20, -16, 32, 4, 12), // arm inner
        (52, 20, -8, 32, 4, 12),  // arm back
    ];

    let ki = k as i32;
    for (sx, sy, ox, oy, rw, rh) in rects {
        for y in 0..(rh * ki) {
            for x in 0..(rw * ki) {
                let spx = sx * ki + x;
                let spy = sy * ki + y;
                let dpx = (sx + ox + rw) * ki - 1 - x;
                let dpy = (sy + oy) * ki + y;
                if spx >= 0 && spy >= 0 && dpx >= 0 && dpy >= 0 {
                    let (spx, spy, dpx, dpy) = (spx as u32, spy as u32, dpx as u32, dpy as u32);
                    if spx < w && spy < h && dpx < 64 * k && dpy < 64 * k {
                        let p = *out.get_pixel(spx, spy);
                        out.put_pixel(dpx, dpy, p);
                    }
                }
            }
        }
    }
    out
}

/// Renders a genuine 3D Minecraft player model with depth buffering, authentic Minecraft directional lighting,
/// overlays, limb stance posing, and 2x supersampling anti-aliasing (SSAA).
pub fn render_3d_player(
    skin_raw: &RgbaImage,
    is_slim: bool,
    yaw_deg: f32,
    pitch_deg: f32,
    img_w: u32,
    img_h: u32,
    posed: bool,
) -> egui::ColorImage {
    let skin = convert_64x32_to_64x64(skin_raw);
    let (sw, sh) = skin.dimensions();
    let k = (sw / 64).max(1);

    // 2x Supersampling dimensions for smooth anti-aliased edges and crisp textures
    let ssaa = 2usize;
    let rw = (img_w as usize) * ssaa;
    let rh = (img_h as usize) * ssaa;

    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians();
    let cy = yaw.cos();
    let sy = yaw.sin();
    let cp = pitch.cos();
    let sp = pitch.sin();

    // Scale so character (~32.5 units tall) fills frame with a neat margin
    let scale = (rh as f32) / 33.5;
    let cx = rw as f32 / 2.0;
    // Center vertically taking pitch into account
    let cy_scr = (rh as f32 / 2.0) - (1.5 * scale * cp);

    // Camera rotation and projection:
    let proj = |x: f32, y: f32, z: f32| -> (f32, f32, f32) {
        let x1 = x * cy + z * sy;
        let z1 = -x * sy + z * cy;
        let y2 = y * cp + z1 * sp;
        let z2 = -y * sp + z1 * cp;
        (cx + x1 * scale, cy_scr + y2 * scale, z2)
    };

    let mut pixels_ss = vec![Color32::TRANSPARENT; rw * rh];
    let mut zbuf_ss = vec![f32::NEG_INFINITY; rw * rh];

    struct Quad {
        pts: [(f32, f32, f32); 4],
        uv: (u32, u32, u32, u32),
        norm: (f32, f32, f32),
    }

    let transform_pt = |pt: (f32, f32, f32), pivot: (f32, f32, f32), rx: f32, ry: f32, rz: f32| -> (f32, f32, f32) {
        let x = pt.0 - pivot.0;
        let y = pt.1 - pivot.1;
        let z = pt.2 - pivot.2;

        // Rot X
        let (cx, sx) = (rx.cos(), rx.sin());
        let y1 = y * cx - z * sx;
        let z1 = y * sx + z * cx;
        let x1 = x;

        // Rot Y
        let (cy, sy) = (ry.cos(), ry.sin());
        let x2 = x1 * cy + z1 * sy;
        let z2 = -x1 * sy + z1 * cy;
        let y2 = y1;

        // Rot Z
        let (cz, sz) = (rz.cos(), rz.sin());
        let x3 = x2 * cz - y2 * sz;
        let y3 = x2 * sz + y2 * cz;
        let z3 = z2;

        (x3 + pivot.0, y3 + pivot.1, z3 + pivot.2)
    };

    let transform_norm = |norm: (f32, f32, f32), rx: f32, ry: f32, rz: f32| -> (f32, f32, f32) {
        let (cx, sx) = (rx.cos(), rx.sin());
        let y1 = norm.1 * cx - norm.2 * sx;
        let z1 = norm.1 * sx + norm.2 * cx;
        let x1 = norm.0;

        let (cy, sy) = (ry.cos(), ry.sin());
        let x2 = x1 * cy + z1 * sy;
        let z2 = -x1 * sy + z1 * cy;
        let y2 = y1;

        let (cz, sz) = (rz.cos(), rz.sin());
        let x3 = x2 * cz - y2 * sz;
        let y3 = x2 * sz + y2 * cz;
        let z3 = z2;

        (x3, y3, z3)
    };

    let mut make_box = |min_x: f32, min_y: f32, min_z: f32, max_x: f32, max_y: f32, max_z: f32,
                        u: u32, v: u32, bw: u32, bh: u32, bd: u32, expand: f32,
                        pivot: (f32, f32, f32), rx: f32, ry: f32, rz: f32,
                        quads: &mut Vec<Quad>| {
        let min_x = min_x - expand;
        let max_x = max_x + expand;
        let min_y = min_y - expand;
        let max_y = max_y + expand;
        let min_z = min_z - expand;
        let max_z = max_z + expand;

        let faces = [
            // Top (Y = min_y): normal (0, -1, 0)
            ([(min_x, min_y, min_z), (max_x, min_y, min_z), (max_x, min_y, max_z), (min_x, min_y, max_z)], (u + bd, v, bw, bd), (0.0, -1.0, 0.0)),
            // Bottom (Y = max_y): normal (0, 1, 0)
            ([(min_x, max_y, max_z), (max_x, max_y, max_z), (max_x, max_y, min_z), (min_x, max_y, min_z)], (u + bd + bw, v, bw, bd), (0.0, 1.0, 0.0)),
            // Front (Z = max_z): normal (0, 0, 1)
            ([(min_x, min_y, max_z), (max_x, min_y, max_z), (max_x, max_y, max_z), (min_x, max_y, max_z)], (u + bd, v + bd, bw, bh), (0.0, 0.0, 1.0)),
            // Back (Z = min_z): normal (0, 0, -1)
            ([(max_x, min_y, min_z), (min_x, min_y, min_z), (min_x, max_y, min_z), (max_x, max_y, min_z)], (u + 2 * bd + bw, v + bd, bw, bh), (0.0, 0.0, -1.0)),
            // Right (X = min_x): normal (-1, 0, 0)
            ([(min_x, min_y, min_z), (min_x, min_y, max_z), (min_x, max_y, max_z), (min_x, max_y, min_z)], (u, v + bd, bd, bh), (-1.0, 0.0, 0.0)),
            // Left (X = max_x): normal (1, 0, 0)
            ([(max_x, min_y, max_z), (max_x, min_y, min_z), (max_x, max_y, min_z), (max_x, max_y, max_z)], (u + bd + bw, v + bd, bd, bh), (1.0, 0.0, 0.0)),
        ];

        for (pts, uv, norm) in faces {
            let p0 = transform_pt(pts[0], pivot, rx, ry, rz);
            let p1 = transform_pt(pts[1], pivot, rx, ry, rz);
            let p2 = transform_pt(pts[2], pivot, rx, ry, rz);
            let p3 = transform_pt(pts[3], pivot, rx, ry, rz);
            let n = transform_norm(norm, rx, ry, rz);
            quads.push(Quad { pts: [p0, p1, p2, p3], uv, norm: n });
        }
    };

    let arm_w_val: f32 = if is_slim { 3.0 } else { 4.0 };
    let arm_w_u = if is_slim { 3 } else { 4 };

    // Dynamic angles for authentic 3D Minecraft pose
    let (r_arm_rx, r_arm_rz, l_arm_rx, l_arm_rz, r_leg_rx, l_leg_rx, head_ry) = if posed {
        (
            (-16.0_f32).to_radians(),
            (-5.0_f32).to_radians(),
            (16.0_f32).to_radians(),
            (5.0_f32).to_radians(),
            (18.0_f32).to_radians(),
            (-18.0_f32).to_radians(),
            (-8.0_f32).to_radians(),
        )
    } else {
        (
            0.0,
            (-2.5_f32).to_radians(),
            0.0,
            (2.5_f32).to_radians(),
            0.0,
            0.0,
            0.0,
        )
    };

    let mut quads = Vec::with_capacity(72);
    // Head & Hat (pivot at neck 0, -6, 0)
    make_box(-4.0, -14.0, -4.0, 4.0, -6.0, 4.0, 0, 0, 8, 8, 8, 0.0, (0.0, -6.0, 0.0), 0.0, head_ry, 0.0, &mut quads);
    make_box(-4.0, -14.0, -4.0, 4.0, -6.0, 4.0, 32, 0, 8, 8, 8, 0.5, (0.0, -6.0, 0.0), 0.0, head_ry, 0.0, &mut quads);

    // Torso & Jacket (pivot at 0, 0, 0)
    make_box(-4.0, -6.0, -2.0, 4.0, 6.0, 2.0, 16, 16, 8, 12, 4, 0.0, (0.0, 0.0, 0.0), 0.0, 0.0, 0.0, &mut quads);
    make_box(-4.0, -6.0, -2.0, 4.0, 6.0, 2.0, 16, 32, 8, 12, 4, 0.25, (0.0, 0.0, 0.0), 0.0, 0.0, 0.0, &mut quads);

    // Right Arm & Sleeve (shoulder pivot at -4.0, -5.0, 0.0)
    let r_arm_x = -4.0 - arm_w_val;
    let r_pivot = (-4.0, -5.0, 0.0);
    make_box(r_arm_x, -6.0, -2.0, -4.0, 6.0, 2.0, 40, 16, arm_w_u, 12, 4, 0.0, r_pivot, r_arm_rx, 0.0, r_arm_rz, &mut quads);
    make_box(r_arm_x, -6.0, -2.0, -4.0, 6.0, 2.0, 40, 32, arm_w_u, 12, 4, 0.25, r_pivot, r_arm_rx, 0.0, r_arm_rz, &mut quads);

    // Left Arm & Sleeve (shoulder pivot at 4.0, -5.0, 0.0)
    let l_arm_x = 4.0 + arm_w_val;
    let l_pivot = (4.0, -5.0, 0.0);
    make_box(4.0, -6.0, -2.0, l_arm_x, 6.0, 2.0, 32, 48, arm_w_u, 12, 4, 0.0, l_pivot, l_arm_rx, 0.0, l_arm_rz, &mut quads);
    make_box(4.0, -6.0, -2.0, l_arm_x, 6.0, 2.0, 48, 48, arm_w_u, 12, 4, 0.25, l_pivot, l_arm_rx, 0.0, l_arm_rz, &mut quads);

    // Right Leg & Pants (hip pivot at -2.0, 6.0, 0.0)
    let r_leg_pivot = (-2.0, 6.0, 0.0);
    make_box(-4.0, 6.0, -2.0, 0.0, 18.0, 2.0, 0, 16, 4, 12, 4, 0.0, r_leg_pivot, r_leg_rx, 0.0, 0.0, &mut quads);
    make_box(-4.0, 6.0, -2.0, 0.0, 18.0, 2.0, 0, 32, 4, 12, 4, 0.25, r_leg_pivot, r_leg_rx, 0.0, 0.0, &mut quads);

    // Left Leg & Pants (hip pivot at 2.0, 6.0, 0.0)
    let l_leg_pivot = (2.0, 6.0, 0.0);
    make_box(0.0, 6.0, -2.0, 4.0, 18.0, 2.0, 16, 48, 4, 12, 4, 0.0, l_leg_pivot, l_leg_rx, 0.0, 0.0, &mut quads);
    make_box(0.0, 6.0, -2.0, 4.0, 18.0, 2.0, 0, 48, 4, 12, 4, 0.25, l_leg_pivot, l_leg_rx, 0.0, 0.0, &mut quads);

    for quad in quads {
        let (nx, ny, nz) = quad.norm;
        let nx1 = nx * cy + nz * sy;
        let nz1 = -nx * sy + nz * cy;
        let ny2 = ny * cp + nz1 * sp;
        let nz2 = -ny * sp + nz1 * cp;

        // Backface culling: face normal must point towards camera (positive Z in view space)
        if nz2 <= 0.0 {
            continue;
        }

        // Authentic Minecraft directional lighting:
        // Top: 1.0 (sunlight), Front: 0.88, Right/Left: 0.78, Back: 0.65, Bottom: 0.55
        let light = if ny < -0.5 {
            1.00 // Top
        } else if ny > 0.5 {
            0.55 // Bottom
        } else if nz > 0.5 {
            0.88 // Front
        } else if nz < -0.5 {
            0.65 // Back
        } else if nx < -0.5 {
            0.80 // Right
        } else {
            0.75 // Left
        };

        let p0 = proj(quad.pts[0].0, quad.pts[0].1, quad.pts[0].2);
        let p1 = proj(quad.pts[1].0, quad.pts[1].1, quad.pts[1].2);
        let p2 = proj(quad.pts[2].0, quad.pts[2].1, quad.pts[2].2);
        let p3 = proj(quad.pts[3].0, quad.pts[3].1, quad.pts[3].2);

        let (ux, uy, uw, uh) = quad.uv;
        let u_0 = (ux * k) as f32;
        let u_1 = ((ux + uw) * k).saturating_sub(1) as f32;
        let v_0 = (uy * k) as f32;
        let v_1 = ((uy + uh) * k).saturating_sub(1) as f32;

        let mut raster_tri = |p0: (f32, f32, f32), p1: (f32, f32, f32), p2: (f32, f32, f32),
                              uv0: (f32, f32), uv1: (f32, f32), uv2: (f32, f32)| {
            let (x0, y0, z0) = p0;
            let (x1, y1, z1) = p1;
            let (x2, y2, z2) = p2;

            let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
            if denom.abs() < 1e-5 {
                return;
            }

            let min_x = (x0.min(x1).min(x2).floor() as i32).max(0) as usize;
            let max_x = (x0.max(x1).max(x2).ceil() as i32).min(rw as i32 - 1).max(0) as usize;
            let min_y = (y0.min(y1).min(y2).floor() as i32).max(0) as usize;
            let max_y = (y0.max(y1).max(y2).ceil() as i32).min(rh as i32 - 1).max(0) as usize;

            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let pxf = px as f32;
                    let pyf = py as f32;
                    let w0 = ((y1 - y2) * (pxf - x2) + (x2 - x1) * (pyf - y2)) / denom;
                    let w1 = ((y2 - y0) * (pxf - x2) + (x0 - x2) * (pyf - y2)) / denom;
                    let w2 = 1.0 - w0 - w1;

                    if w0 >= -1e-4 && w1 >= -1e-4 && w2 >= -1e-4 {
                        let pz = w0 * z0 + w1 * z1 + w2 * z2;
                        let idx = py * rw + px;
                        if pz > zbuf_ss[idx] {
                            let tu = (w0 * uv0.0 + w1 * uv1.0 + w2 * uv2.0).round() as u32;
                            let tv = (w0 * uv0.1 + w1 * uv1.1 + w2 * uv2.1).round() as u32;
                            if tu < sw && tv < sh {
                                let p = skin.get_pixel(tu, tv);
                                if p[3] > 10 {
                                    let r = ((p[0] as f32 * light).min(255.0)) as u8;
                                    let g = ((p[1] as f32 * light).min(255.0)) as u8;
                                    let b = ((p[2] as f32 * light).min(255.0)) as u8;
                                    if p[3] >= 240 {
                                        zbuf_ss[idx] = pz;
                                        pixels_ss[idx] = Color32::from_rgba_unmultiplied(r, g, b, 255);
                                    } else {
                                        let a = p[3] as f32 / 255.0;
                                        let prev = pixels_ss[idx];
                                        let nr = ((r as f32 * a) + (prev.r() as f32 * (1.0 - a))).round() as u8;
                                        let ng = ((g as f32 * a) + (prev.g() as f32 * (1.0 - a))).round() as u8;
                                        let nb = ((b as f32 * a) + (prev.b() as f32 * (1.0 - a))).round() as u8;
                                        let na = ((255.0 * a) + (prev.a() as f32 * (1.0 - a))).round() as u8;
                                        pixels_ss[idx] = Color32::from_rgba_unmultiplied(nr, ng, nb, na);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        raster_tri(p0, p1, p2, (u_0, v_0), (u_1, v_0), (u_1, v_1));
        raster_tri(p0, p2, p3, (u_0, v_0), (u_1, v_1), (u_0, v_1));
    }

    // Downsample 2x2 SSAA to target dimensions (img_w, img_h)
    let ow = img_w as usize;
    let oh = img_h as usize;
    let mut pixels_out = vec![Color32::TRANSPARENT; ow * oh];

    for oy in 0..oh {
        for ox in 0..ow {
            let mut r_acc = 0u32;
            let mut g_acc = 0u32;
            let mut b_acc = 0u32;
            let mut a_acc = 0u32;

            for dy in 0..2 {
                for dx in 0..2 {
                    let p = pixels_ss[(oy * 2 + dy) * rw + (ox * 2 + dx)];
                    r_acc += p.r() as u32;
                    g_acc += p.g() as u32;
                    b_acc += p.b() as u32;
                    a_acc += p.a() as u32;
                }
            }

            let avg_a = (a_acc / 4) as u8;
            if avg_a > 0 {
                let avg_r = (r_acc / 4) as u8;
                let avg_g = (g_acc / 4) as u8;
                let avg_b = (b_acc / 4) as u8;
                pixels_out[oy * ow + ox] = Color32::from_rgba_unmultiplied(avg_r, avg_g, avg_b, avg_a);
            }
        }
    }

    egui::ColorImage {
        size: [ow, oh],
        pixels: pixels_out,
    }
}

pub fn load_current_skin(path: &str) -> RgbaImage {
    if !path.is_empty() && Path::new(path).is_file() {
        if let Ok(img) = image::open(path) {
            return convert_64x32_to_64x64(&img.to_rgba8());
        }
    }
    image::load_from_memory(DEFAULT_SKIN_BYTES)
        .map(|img| convert_64x32_to_64x64(&img.to_rgba8()))
        .unwrap_or_else(|_| RgbaImage::new(64, 64))
}

pub fn ensure_skins_library(library_dir: &Path) {
    let _ = fs::create_dir_all(library_dir);
    // Delete obsolete or explicitly removed skins
    let obsolete = [
        "festive_steve.png", "settler.png", "halloween_zombie.png", "townsperson.png",
        "castaway.png", "balloon_boy.png", "mojang.png", "boba_fett.png", "batman.png",
        "bowser.png", "magma.png", "cyber_phantom.png", "specops.png", "steel.png", "neon.png",
        "Ветровка Стрит.png", "Спецназ.png", "Новогодний Стив.png", "Селянин-дикарь.png",
        "Магма.png", "Кибер-фантом.png", "FNAF (Balloon Boy).png", "Боба Фетт.png",
        "Бэтмен.png", "Боузер.png", "Mojang.png", "Огненный.png", "Кибер-Фантом.png", "Поселенец.png",
        "Праздничный Стив.png", "Зомби на Хэллоуин.png", "Селянин.png",
        // User requested removals:
        "Люди в чёрном.png", "Человек в чёрном.png",
        "agent_mib.png", "classic_steve.png",
        "Классический Стив 0.12.png",
    ];
    for f in obsolete {
        let _ = fs::remove_file(library_dir.join(f));
    }

    // Sync all embedded skins to skins_library
    for skin in EMBEDDED_SKINS {
        let dest = library_dir.join(format!("{}.png", skin.name));
        if !dest.is_file() {
            let _ = fs::write(dest, skin.bytes);
        }
    }
}

#[derive(Clone)]
struct SkinItem {
    name: String,
    is_slim: bool,
    is_builtin: bool,
    source: SkinSource,
}

#[derive(Clone)]
enum SkinSource {
    Embedded(&'static [u8]),
    File(PathBuf),
}

impl SkinItem {
    fn load_rgba(&self) -> RgbaImage {
        match &self.source {
            SkinSource::Embedded(bytes) => image::load_from_memory(bytes)
                .map(|img| convert_64x32_to_64x64(&img.to_rgba8()))
                .unwrap_or_else(|_| RgbaImage::new(64, 64)),
            SkinSource::File(path) => {
                if let Ok(img) = image::open(path) {
                    convert_64x32_to_64x64(&img.to_rgba8())
                } else {
                    RgbaImage::new(64, 64)
                }
            }
        }
    }

    fn write_to_file(&self, dest: &Path) -> std::io::Result<()> {
        match &self.source {
            SkinSource::Embedded(bytes) => {
                if let Ok(img) = image::load_from_memory(bytes) {
                    let conv = convert_64x32_to_64x64(&img.to_rgba8());
                    let _ = conv.save(dest);
                    Ok(())
                } else {
                    fs::write(dest, bytes)
                }
            }
            SkinSource::File(path) => {
                if let Ok(img) = image::open(path) {
                    let conv = convert_64x32_to_64x64(&img.to_rgba8());
                    let _ = conv.save(dest);
                    Ok(())
                } else {
                    fs::copy(path, dest).map(|_| ())
                }
            }
        }
    }
}

pub fn render(app: &mut App, ui: &mut Ui, _t: f32) {
    let library_dir = PathBuf::from(&app.cfg.phantom_dir).join("skins_library");
    ensure_skins_library(&library_dir);

    // Auto-clean forbidden skins from active config if selected previously
    let cur_low = app.cfg.skin_path.to_lowercase();
    if cur_low.contains("mib") || cur_low.contains("чёрн") || cur_low.contains("черн") {
        app.cfg.skin_path.clear();
        app.skin_preview_dirty = true;
        app.cfg.save();
    }

    let left_width = 280.0;
    let preview_w = 252.0f32;
    let preview_h = 260.0f32;

    // Refresh 3D preview texture if dirty or not loaded
    if app.skin_preview_dirty || app.skin_3d_tex.is_none() {
        let skin_img = load_current_skin(&app.cfg.skin_path);
        let is_slim = app.cfg.skin_model == "slim";

        let preview_3d = render_3d_player(
            &skin_img,
            is_slim,
            app.skin_preview_yaw,
            app.skin_preview_pitch,
            preview_w as u32,
            preview_h as u32,
            false, // left preview is interactive (user controls angle)
        );

        app.skin_3d_tex = Some(ui.ctx().load_texture(
            "skin_3d_preview",
            preview_3d,
            TextureOptions::NEAREST,
        ));
        app.skin_preview_dirty = false;
    }

    // =========================================================================
    // TWO-COLUMN SIDE-BY-SIDE LAYOUT
    // =========================================================================
    ui.horizontal_top(|ui| {
        // ---------------------------------------------------------------------
        // LEFT COLUMN: Fixed width 280px, pinned, always visible
        // ---------------------------------------------------------------------
        ui.vertical(|ui| {
            ui.set_width(left_width);

            Frame::none()
                .fill(theme::CARD)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(14.0))
                .show(ui, |ui| {
                    ui.set_width(left_width - 28.0);

                    // 1. Title & Active Skin Name
                    ui.label(
                        RichText::new("ВЫБРАННЫЙ СКИН")
                            .size(14.0)
                            .color(theme::YELLOW)
                            .strong(),
                    );

                    let skin_display = if app.cfg.skin_path.is_empty() {
                        "Батя [По умолчанию]".to_string()
                    } else {
                        Path::new(&app.cfg.skin_path)
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Пользовательский".into())
                    };
                    ui.label(
                        RichText::new(&skin_display)
                            .size(13.0)
                            .color(theme::TEXT)
                            .strong(),
                    );

                    ui.add_space(8.0);

                    // 2. Interactive 3D Model Display
                    Frame::none()
                        .fill(theme::BG)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 38, 44)))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::same(2.0))
                        .show(ui, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(vec2(preview_w, preview_h), egui::Sense::click_and_drag());
                            if resp.dragged() {
                                let delta = resp.drag_delta();
                                app.skin_preview_yaw = (app.skin_preview_yaw + delta.x * 0.7).rem_euclid(360.0);
                                app.skin_preview_pitch = (app.skin_preview_pitch + delta.y * 0.7).clamp(-45.0, 45.0);
                                app.skin_preview_dirty = true;
                            }

                            if let Some(tex) = &app.skin_3d_tex {
                                ui.painter().image(
                                    tex.id(),
                                    rect,
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                    Color32::WHITE,
                                );
                            }
                        });

                    // 3D Rotation Controls
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("⟲ Зажмите мышь для 3D вращения")
                                .size(10.5)
                                .color(theme::MUTED),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .small_button(RichText::new("↺ Сброс").size(10.5).color(theme::TEXT))
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                app.skin_preview_yaw = 25.0;
                                app.skin_preview_pitch = 10.0;
                                app.skin_preview_dirty = true;
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // 3. Hand model toggle: [Обычный 4px] / [Тонкий 3px]
                    ui.label(RichText::new("Модель рук персонажа:").size(11.5).color(theme::MUTED));
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        let is_classic = app.cfg.skin_model != "slim";
                        let is_slim = app.cfg.skin_model == "slim";
                        let btn_w = ((left_width - 28.0 - 8.0) / 2.0).floor();

                        let classic_btn = egui::Button::new(
                            RichText::new("Обычный 4px")
                                .size(11.5)
                                .color(if is_classic { theme::BG } else { theme::TEXT })
                                .strong(),
                        )
                        .fill(if is_classic { theme::YELLOW } else { theme::SIDE })
                        .rounding(Rounding::same(4.0));

                        if ui.add_sized([btn_w, 26.0], classic_btn).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                            app.cfg.skin_model = "classic".into();
                            app.skin_preview_dirty = true;
                            app.cfg.save();

                            let game_dir = PathBuf::from(app.cfg.game_dir());
                            let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);
                            let _ = fs::remove_file(game_dir.join("custom_skin_slim.txt"));
                            let _ = fs::remove_file(phantom_dir.join("custom_skin_slim.txt"));
                        }

                        let slim_btn = egui::Button::new(
                            RichText::new("Тонкий 3px")
                                .size(11.5)
                                .color(if is_slim { theme::BG } else { theme::TEXT })
                                .strong(),
                        )
                        .fill(if is_slim { theme::YELLOW } else { theme::SIDE })
                        .rounding(Rounding::same(4.0));

                        if ui.add_sized([btn_w, 26.0], slim_btn).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                            app.cfg.skin_model = "slim".into();
                            app.skin_preview_dirty = true;
                            app.cfg.save();

                            let game_dir = PathBuf::from(app.cfg.game_dir());
                            let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);
                            let _ = fs::write(game_dir.join("custom_skin_slim.txt"), "true");
                            let _ = fs::write(phantom_dir.join("custom_skin_slim.txt"), "true");
                        }
                    });

                    ui.add_space(10.0);

                    // 4. Action buttons: Upload and Reset
                    let upload_btn = egui::Button::new(
                        RichText::new("📁  ЗАГРУЗИТЬ СВОЙ (.PNG)")
                            .color(theme::BG)
                            .strong()
                            .size(12.0),
                    )
                    .fill(theme::YELLOW)
                    .rounding(Rounding::same(4.0));

                    if ui
                        .add_sized([ui.available_width(), 32.0], upload_btn)
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        if let Some(picked) = rfd::FileDialog::new()
                            .add_filter("Minecraft Skin (*.png)", &["png"])
                            .set_title("Выберите файл скина")
                            .pick_file()
                        {
                            let picked_str = picked.to_string_lossy().to_string();
                            let game_dir = PathBuf::from(app.cfg.game_dir());
                            let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);
                            let _ = fs::create_dir_all(&game_dir);
                            let _ = fs::create_dir_all(&phantom_dir);

                            // Load and convert 64x32 -> 64x64 if necessary
                            let skin_img = load_current_skin(&picked_str);
                            let _ = skin_img.save(game_dir.join("custom_skin.png"));
                            let _ = skin_img.save(phantom_dir.join("custom_skin.png"));
                            let _ = skin_img.save(phantom_dir.join("skin.png"));

                            // Copy converted file to library too
                            if let Some(filename) = picked.file_name() {
                                let _ = skin_img.save(library_dir.join(filename));
                            }

                            app.cfg.skin_path = picked_str;
                            app.skin_preview_dirty = true;
                            app.skin_card_cache.clear();
                            app.cfg.save();
                            app.status = Some((true, "Пользовательский скин успешно загружен!".into()));
                        }
                    }

                    ui.add_space(6.0);

                    let reset_btn = egui::Button::new(
                        RichText::new("↺  СБРОСИТЬ НА СТАНДАРТНЫЙ")
                            .size(11.5)
                            .color(theme::MUTED),
                    )
                    .fill(theme::SIDE)
                    .stroke(Stroke::new(1.0, theme::BORDER))
                    .rounding(Rounding::same(4.0));

                    if ui
                        .add_sized([ui.available_width(), 28.0], reset_btn)
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
                        app.status = Some((true, "Установлен стандартный скин Батя".into()));
                    }

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("✓ Скин автоматически применяется в игре и на серверах.")
                            .color(Color32::from_rgb(120, 200, 140))
                            .size(10.5),
                    );
                });
        });

        ui.add_space(14.0);

        // ---------------------------------------------------------------------
        // RIGHT COLUMN: Smooth vertically scrollable collection
        // ---------------------------------------------------------------------
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());

            ui.label(
                RichText::new("КОЛЛЕКЦИЯ СКИНОВ")
                    .size(16.0)
                    .color(theme::YELLOW)
                    .strong(),
            );
            ui.label(
                RichText::new("Выберите персонажа в один клик:")
                    .size(12.0)
                    .color(theme::MUTED),
            );
            ui.add_space(6.0);

            // Build total skin items list: embedded presets + any extra custom user skins
            let mut all_items: Vec<SkinItem> = Vec::new();
            for s in EMBEDDED_SKINS {
                all_items.push(SkinItem {
                    name: s.name.to_string(),
                    is_slim: s.is_slim,
                    is_builtin: true,
                    source: SkinSource::Embedded(s.bytes),
                });
            }

            let forbidden = [
                "люди в чёрном", "человек в чёрном",
                "человек в черном", "люди в черном",
                "agent_mib", "mib", "classic_steve",
                "классический стив", "стив 0.12",
            ];

            // Check library for user-added custom skins
            if let Ok(entries) = fs::read_dir(&library_dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_file() && p.extension().is_some_and(|ext| ext == "png") {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            let stem_low = stem.to_lowercase();
                            if forbidden.iter().any(|f| stem_low == *f) {
                                continue;
                            }
                            if !all_items.iter().any(|item| item.name == stem) {
                                let is_slim = stem_low.contains("slim")
                                    || stem_low.contains("алекс")
                                    || stem_low.contains("alex");
                                all_items.push(SkinItem {
                                    name: stem.to_string(),
                                    is_slim,
                                    is_builtin: false,
                                    source: SkinSource::File(p),
                                });
                            }
                        }
                    }
                }
            }

            let total_items = all_items.len();
            let per_page = 15;
            let total_pages = ((total_items + per_page - 1) / per_page).max(1);
            if app.skin_tab_page >= total_pages {
                app.skin_tab_page = total_pages - 1;
            }

            let start_idx = app.skin_tab_page * per_page;
            let end_idx = (start_idx + per_page).min(total_items);
            let page_items = &all_items[start_idx..end_idx];

            let mut skin_to_apply: Option<SkinItem> = None;
            let mut skin_to_delete: Option<PathBuf> = None;

            // Smooth scroll area
            egui::ScrollArea::vertical()
                .id_source("skin_collection_scroll_area")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let avail_w = (ui.available_width() - 14.0).max(360.0);
                    let spacing = 8.0f32;
                    let cols = 5;
                    let card_w = ((avail_w - (cols as f32 - 1.0) * spacing) / cols as f32).floor();
                    let card_h = 188.0f32;

                    let thumb_w = (card_w - 12.0).clamp(84.0, 102.0);
                    let thumb_h = 136.0f32;
                    let ctx = ui.ctx().clone();

                    for chunk in page_items.chunks(cols) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = spacing;

                            for item in chunk {
                                let is_active = if app.cfg.skin_path.is_empty() {
                                    item.name == "Батя"
                                } else {
                                    Path::new(&app.cfg.skin_path)
                                        .file_stem()
                                        .is_some_and(|s| s == item.name.as_str())
                                };

                                let (card_rect, card_resp) = ui.allocate_exact_size(
                                    vec2(card_w, card_h),
                                    egui::Sense::click(),
                                );

                                let is_hovered = card_resp.hovered();
                                if is_hovered {
                                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                }

                                let card_border = if is_active {
                                    theme::YELLOW
                                } else if is_hovered {
                                    Color32::from_rgb(70, 70, 80)
                                } else {
                                    theme::BORDER
                                };

                                let card_fill = if is_active {
                                    Color32::from_rgb(34, 34, 26)
                                } else if is_hovered {
                                    Color32::from_rgb(28, 28, 34)
                                } else {
                                    theme::CARD
                                };

                                let painter = ui.painter_at(card_rect);

                                // 1. Card background with border
                                painter.rect(
                                    card_rect,
                                    Rounding::same(8.0),
                                    card_fill,
                                    Stroke::new(if is_active { 1.5 } else { 1.0 }, card_border),
                                );

                                // 2. Title centered at top
                                let display_title = if item.name.chars().count() > 13 {
                                    format!("{}…", item.name.chars().take(12).collect::<String>())
                                } else {
                                    item.name.clone()
                                };
                                let title_pos = pos2(card_rect.center().x, card_rect.min.y + 12.0);
                                painter.text(
                                    title_pos,
                                    egui::Align2::CENTER_CENTER,
                                    display_title,
                                    egui::FontId::proportional(11.0),
                                    if is_active { theme::YELLOW } else { theme::TEXT },
                                );

                                // Optional delete cross for custom user skins
                                if !item.is_builtin {
                                    let del_rect = Rect::from_min_size(
                                        pos2(card_rect.max.x - 20.0, card_rect.min.y + 2.0),
                                        vec2(16.0, 16.0),
                                    );
                                    let del_resp = ui.allocate_rect(del_rect, egui::Sense::click());
                                    if del_resp.hovered() {
                                        painter.text(
                                            del_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "✕",
                                            egui::FontId::proportional(10.5),
                                            Color32::from_rgb(255, 100, 100),
                                        );
                                    } else {
                                        painter.text(
                                            del_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "✕",
                                            egui::FontId::proportional(10.5),
                                            Color32::from_rgb(160, 70, 70),
                                        );
                                    }
                                    if del_resp.clicked() {
                                        if let SkinSource::File(p) = &item.source {
                                            skin_to_delete = Some(p.clone());
                                        }
                                    }
                                }

                                // 3. 3D Character Thumbnail (SSAA 2x, posed, depth-buffered)
                                let cache_key = format!("skin3d_v3_{}_{}_{}", item.name, item.is_slim, thumb_w as u32);
                                let tex = app.skin_card_cache.entry(cache_key.clone()).or_insert_with(|| {
                                    let img = item.load_rgba();
                                    let thumb_3d = render_3d_player(
                                        &img,
                                        item.is_slim,
                                        18.0, // front-facing 3D angle (broad shoulders!)
                                        8.0,  // slight top tilt
                                        thumb_w as u32,
                                        thumb_h as u32,
                                        true, // classic Minecraft 3D walking/idle pose!
                                    );
                                    ctx.load_texture(&cache_key, thumb_3d, TextureOptions::NEAREST)
                                });

                                let thumb_rect = Rect::from_min_size(
                                    pos2(card_rect.center().x - thumb_w / 2.0, card_rect.min.y + 20.0),
                                    vec2(thumb_w, thumb_h),
                                );
                                painter.image(
                                    tex.id(),
                                    thumb_rect,
                                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                    Color32::WHITE,
                                );

                                // 4. Action Button at bottom
                                let btn_w = card_w - 14.0;
                                let btn_rect = Rect::from_min_size(
                                    pos2(card_rect.center().x - btn_w / 2.0, card_rect.min.y + 158.0),
                                    vec2(btn_w, 22.0),
                                );

                                if is_active {
                                    painter.rect(
                                        btn_rect,
                                        Rounding::same(4.0),
                                        theme::YELLOW,
                                        Stroke::NONE,
                                    );
                                    painter.text(
                                        btn_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "✓ ВЫБРАН",
                                        egui::FontId::proportional(11.0),
                                        Color32::BLACK,
                                    );
                                } else {
                                    let btn_hovered = is_hovered || ui.rect_contains_pointer(btn_rect);
                                    let btn_fill = if btn_hovered {
                                        Color32::from_rgb(45, 45, 54)
                                    } else {
                                        theme::SIDE
                                    };
                                    painter.rect(
                                        btn_rect,
                                        Rounding::same(4.0),
                                        btn_fill,
                                        Stroke::new(1.0, if btn_hovered { Color32::from_rgb(80, 80, 95) } else { theme::BORDER }),
                                    );
                                    painter.text(
                                        btn_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "Выбрать",
                                        egui::FontId::proportional(11.0),
                                        if btn_hovered { Color32::WHITE } else { theme::TEXT },
                                    );
                                }

                                if card_resp.clicked() && !is_active {
                                    skin_to_apply = Some(item.clone());
                                }
                            }
                        });
                        ui.add_space(spacing);
                    }

                    // Bottom pagination controls
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;

                        let prev_enabled = app.skin_tab_page > 0;
                        if ui.add_enabled(prev_enabled, egui::Button::new(RichText::new("◀").size(12.0)).min_size(vec2(28.0, 24.0))).clicked() {
                            app.skin_tab_page = app.skin_tab_page.saturating_sub(1);
                        }

                        for p in 0..total_pages {
                            let is_current = p == app.skin_tab_page;
                            let mut btn = egui::Button::new(
                                RichText::new((p + 1).to_string())
                                    .strong()
                                    .size(12.0)
                                    .color(if is_current { theme::YELLOW } else { theme::TEXT }),
                            )
                            .min_size(vec2(28.0, 24.0));

                            if is_current {
                                btn = btn.stroke(Stroke::new(1.5, theme::YELLOW));
                            }

                            if ui.add(btn).clicked() {
                                app.skin_tab_page = p;
                            }
                        }

                        let next_enabled = app.skin_tab_page + 1 < total_pages;
                        if ui.add_enabled(next_enabled, egui::Button::new(RichText::new("▶").size(12.0)).min_size(vec2(28.0, 24.0))).clicked() {
                            app.skin_tab_page += 1;
                        }

                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(format!(
                                "Стр. {} / {} • Всего: {}",
                                app.skin_tab_page + 1,
                                total_pages,
                                total_items
                            ))
                            .size(12.0)
                            .color(theme::MUTED),
                        );
                    });
                    ui.add_space(20.0);
                });

            // Handle skin apply
            if let Some(item) = skin_to_apply {
                let game_dir = PathBuf::from(app.cfg.game_dir());
                let phantom_dir = PathBuf::from(&app.cfg.phantom_dir);
                let _ = fs::create_dir_all(&game_dir);
                let _ = fs::create_dir_all(&phantom_dir);

                let target_game_skin = game_dir.join("custom_skin.png");
                let target_phantom_skin = phantom_dir.join("custom_skin.png");
                let target_lib_skin = library_dir.join(format!("{}.png", item.name));

                let _ = item.write_to_file(&target_game_skin);
                let _ = item.write_to_file(&target_phantom_skin);
                let _ = item.write_to_file(&phantom_dir.join("skin.png"));
                let _ = item.write_to_file(&target_lib_skin);

                let slim_str = if item.is_slim { "true" } else { "false" };
                let _ = fs::write(game_dir.join("custom_skin_slim.txt"), slim_str);
                let _ = fs::write(phantom_dir.join("custom_skin_slim.txt"), slim_str);

                app.cfg.skin_path = target_lib_skin.to_string_lossy().to_string();
                app.cfg.skin_model = if item.is_slim { "slim".into() } else { "classic".into() };
                app.skin_preview_dirty = true;
                app.cfg.save();
                app.status = Some((true, format!("Скин «{}» успешно применён!", item.name)));
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
    });
}
