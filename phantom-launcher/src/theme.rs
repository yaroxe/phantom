//! Matte Graphite + Cyber Yellow. One flat plane, no gradients.
use eframe::egui::{self, Color32, Rounding, Shadow, Stroke, Visuals};

pub const BG: Color32 = Color32::from_rgb(0x0d, 0x0f, 0x12);
pub const SIDE: Color32 = Color32::from_rgb(0x08, 0x09, 0x0b);
pub const CARD: Color32 = Color32::from_rgb(0x14, 0x17, 0x1c);
pub const CARD_HI: Color32 = Color32::from_rgb(0x1a, 0x1e, 0x24);
pub const BORDER: Color32 = Color32::from_rgb(0x24, 0x29, 0x33);
pub const YELLOW: Color32 = Color32::from_rgb(0xfc, 0xee, 0x0a);
pub const OK: Color32 = Color32::from_rgb(0x34, 0xd3, 0x99);
pub const TEXT: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const TEXT_SOFT: Color32 = Color32::from_rgb(0xd1, 0xd5, 0xdb);
pub const MUTED: Color32 = Color32::from_rgb(0x9c, 0xa3, 0xaf);
pub const DANGER: Color32 = Color32::from_rgb(0xff, 0x5c, 0x5c);

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    let sharp = Rounding::same(2.);

    v.panel_fill = BG;
    v.window_fill = CARD;
    v.window_stroke = Stroke::new(1., BORDER);
    v.window_rounding = sharp;
    v.extreme_bg_color = SIDE; // text fields
    v.faint_bg_color = CARD;
    v.code_bg_color = SIDE;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.hyperlink_color = YELLOW;
    v.selection.bg_fill = YELLOW;
    v.selection.stroke = Stroke::new(1., Color32::BLACK);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = CARD;
    w.noninteractive.weak_bg_fill = CARD;
    w.noninteractive.bg_stroke = Stroke::new(1., BORDER);
    w.noninteractive.fg_stroke = Stroke::new(1., TEXT_SOFT);

    w.inactive.bg_fill = CARD_HI;
    w.inactive.weak_bg_fill = CARD;
    w.inactive.bg_stroke = Stroke::new(1., BORDER);
    w.inactive.fg_stroke = Stroke::new(1., TEXT_SOFT);

    w.hovered.bg_fill = CARD_HI;
    w.hovered.weak_bg_fill = CARD_HI;
    w.hovered.bg_stroke = Stroke::new(1., YELLOW);
    w.hovered.fg_stroke = Stroke::new(1., TEXT);

    w.active.bg_fill = YELLOW;
    w.active.weak_bg_fill = YELLOW;
    w.active.bg_stroke = Stroke::new(1., YELLOW);
    w.active.fg_stroke = Stroke::new(1., Color32::BLACK);

    w.open = w.hovered;

    for s in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        s.rounding = sharp;
    }

    ctx.set_visuals(v);
}
