//! Visual effects: the iridescent neon sweep, gradients, glow, HUD brackets, the reactor.
//! Everything is drawn by hand with the painter, so there are no image assets.
use eframe::egui::{
    epaint::Mesh, pos2, vec2, Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke,
};
use std::f32::consts::TAU;

pub fn hsv(h: f32, s: f32, v: f32) -> Color32 {
    let h = h.rem_euclid(1.) * 6.;
    let i = h.floor();
    let f = h - i;
    let (p, q, t) = (v * (1. - s), v * (1. - s * f), v * (1. - s * (1. - f)));
    let (r, g, b) = match i as i32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Color32::from_rgb((r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8)
}

/// The shimmer: cyber yellow -> orange -> red -> hot pink and back, looping with `x` (period 1).
/// Warm hues only: change the two numbers (start hue, span in degrees) to retune it.
pub fn shimmer(x: f32) -> Color32 {
    let w = 0.5 - 0.5 * (x * TAU).cos();
    hsv((58. - 100. * w) / 360., 0.92, 1.)
}

/// A flowing gradient bar; `vertical` runs it top to bottom. `alpha` 0..=1.
pub fn gradient(p: &Painter, rect: Rect, t: f32, vertical: bool, alpha: f32) {
    const N: usize = 32;
    let mut m = Mesh::default();
    for i in 0..=N {
        let u = i as f32 / N as f32;
        let c = shimmer(t * 0.12 + u * 0.55).gamma_multiply(alpha);
        let (a, b) = if vertical {
            let y = rect.top() + u * rect.height();
            (pos2(rect.left(), y), pos2(rect.right(), y))
        } else {
            let x = rect.left() + u * rect.width();
            (pos2(x, rect.top()), pos2(x, rect.bottom()))
        };
        m.colored_vertex(a, c);
        m.colored_vertex(b, c);
        if i > 0 {
            let k = (i * 2) as u32;
            m.add_triangle(k - 2, k - 1, k);
            m.add_triangle(k - 1, k + 1, k);
        }
    }
    p.add(Shape::mesh(m));
}

/// A soft neon halo around `rect`: stacked translucent rounded rects.
pub fn glow(p: &Painter, rect: Rect, color: Color32, strength: f32, radius: f32) {
    for i in 1..=8 {
        let e = i as f32 * radius / 8.;
        let a = strength * (1. - i as f32 / 9.) * 0.45;
        p.rect_filled(
            rect.expand(e),
            Rounding::same(2. + e),
            color.gamma_multiply(a),
        );
    }
}

/// HUD corner brackets.
pub fn brackets(p: &Painter, r: Rect, len: f32, col: Color32) {
    let s = Stroke::new(2., col);
    for (c, dx, dy) in [
        (r.left_top(), 1., 1.),
        (r.right_top(), -1., 1.),
        (r.left_bottom(), 1., -1.),
        (r.right_bottom(), -1., -1.),
    ] {
        p.line_segment([c, c + vec2(dx * len, 0.)], s);
        p.line_segment([c, c + vec2(0., dy * len)], s);
    }
}

pub fn dotgrid(p: &Painter, rect: Rect, step: f32, col: Color32) {
    let mut y = rect.top() + step / 2.;
    while y < rect.bottom() {
        let mut x = rect.left() + step / 2.;
        while x < rect.right() {
            p.rect_filled(Rect::from_min_size(pos2(x, y), vec2(1.5, 1.5)), 0., col);
            x += step;
        }
        y += step;
    }
}

pub fn hexagon(c: Pos2, r: f32, rot: f32) -> Vec<Pos2> {
    (0..6)
        .map(|i| {
            let a = rot + i as f32 * TAU / 6.;
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect()
}

/// The hero's "reactor": nested hexagons turning against each other, orbiting sparks, a hot core.
pub fn reactor(p: &Painter, c: Pos2, radius: f32, t: f32) {
    for k in 0..4 {
        let kf = k as f32;
        let r = radius * (1. - kf * 0.2);
        let dir = if k % 2 == 0 { 1. } else { -1. };
        let col = shimmer(t * 0.15 + kf * 0.12);
        let w = (2.6 - kf * 0.45).max(1.);
        p.add(Shape::closed_line(
            hexagon(c, r, t * (0.25 + 0.12 * kf) * dir),
            Stroke::new(w, col),
        ));
    }
    for j in 0..3 {
        let a = t * 0.8 + j as f32 * TAU / 3.;
        p.circle_filled(
            c + vec2(a.cos(), a.sin()) * radius * 1.1,
            3.,
            shimmer(t * 0.2 + j as f32 * 0.2),
        );
    }
    let core = shimmer(t * 0.3);
    for i in (1..=5).rev() {
        p.circle_filled(c, 6. + i as f32 * 5., core.gamma_multiply(0.07));
    }
    p.circle_filled(c, 7., core);
}
