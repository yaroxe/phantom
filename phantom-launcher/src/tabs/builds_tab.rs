//! Builds & Profiles management tab (Qwen Design)

use crate::{
    app::{self, App},
    fx, theme,
};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CursorIcon, FontId, Frame, Margin, Rect, RichText,
    Rounding, Sense, Stroke, TextEdit,
};
use std::{fs, path::PathBuf};

pub fn render(app: &mut App, ui: &mut egui::Ui, t: f32) {
    app::heading(ui, t, "СБОРКИ И ПРОФИЛИ", "Управление версиями и профилями запуска");

    let mut add_jar = false;

    // Top action bar
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    RichText::new("+ СОЗДАТЬ ПРОФИЛЬ")
                        .size(13.)
                        .color(Color32::BLACK)
                        .strong(),
                )
                .fill(theme::YELLOW)
                .stroke(Stroke::new(1., theme::YELLOW)),
            )
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            app.creating_profile = true;
            app.new_profile_name.clear();
        }

        if ui
            .add(
                egui::Button::new(
                    RichText::new("+ ДОБАВИТЬ .JAR")
                        .size(13.)
                        .color(theme::TEXT),
                )
                .fill(theme::CARD)
                .stroke(Stroke::new(1., theme::BORDER)),
            )
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            add_jar = true;
        }

        if ui.add(app::outline_button("ОБНОВИТЬ СПИСОК")).clicked() {
            app.rescan();
        }
    });

    // Modal Window for profile creation
    if app.creating_profile {
        egui::Window::new("СОЗДАТЬ ПРОФИЛЬ")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
            .frame(
                Frame::none()
                    .fill(theme::SIDE)
                    .stroke(Stroke::new(1.5, theme::YELLOW))
                    .rounding(Rounding::same(6.))
                    .inner_margin(Margin::same(20.)),
            )
            .show(ui.ctx(), |ui| {
                ui.set_min_width(340.);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Новый изолированный профиль")
                            .size(16.)
                            .color(theme::TEXT)
                            .strong(),
                    );
                    ui.label(app::muted(
                        "Создаст папку в versions/ с индивидуальными модами и настройками",
                    ));
                    ui.add_space(10.);

                    ui.label(
                        RichText::new("ИМЯ ПРОФИЛЯ:")
                            .size(11.)
                            .color(theme::YELLOW)
                            .strong(),
                    );
                    let te = ui.add(
                        TextEdit::singleline(&mut app.new_profile_name)
                            .desired_width(320.)
                            .hint_text("Например: OptiFine-1.20 или Modded"),
                    );
                    ui.add_space(12.);

                    let can_create = !app.new_profile_name.trim().is_empty();
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(can_create, app::primary_button("СОЗДАТЬ"))
                            .clicked()
                            || (can_create
                                && te.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        {
                            let raw = app.new_profile_name.trim();
                            let clean: String = raw
                                .chars()
                                .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
                                .collect();

                            if !clean.is_empty() {
                                let ver_dir = app.cfg.game_dir().join("versions").join(&clean);
                                let _ = fs::create_dir_all(&ver_dir);
                                let json_content = format!(
                                    "{{\n  \"id\": \"{clean}\",\n  \"type\": \"custom\"\n}}\n"
                                );
                                let _ = fs::write(
                                    ver_dir.join(format!("{clean}.json")),
                                    json_content,
                                );
                                let _ = fs::write(ver_dir.join(format!("{clean}.jar")), &[]);

                                app.rescan();
                                app.cfg.selected = format!("java:{clean}");
                                app.cfg.save();
                                app.status = Some((
                                    true,
                                    format!("Профиль \"{clean}\" успешно создан!"),
                                ));
                                app.creating_profile = false;
                                app.new_profile_name.clear();
                            }
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("ОТМЕНА")
                                        .size(13.)
                                        .color(theme::MUTED),
                                )
                                .fill(theme::CARD)
                                .stroke(Stroke::new(1., theme::BORDER)),
                            )
                            .clicked()
                        {
                            app.creating_profile = false;
                            app.new_profile_name.clear();
                        }
                    });
                });
            });
    }

    if let Some((ok, msg)) = &app.status {
        ui.add_space(4.);
        let col = if *ok { theme::OK } else { theme::DANGER };
        ui.label(RichText::new(msg).size(12.).color(col));
    }

    ui.add_space(12.);

    let entries = app.entries();
    let mut pick: Option<String> = None;
    let mut delete_id: Option<String> = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(14., 14.);
            ui.horizontal_wrapped(|ui| {
                for (id, tag) in &entries {
                    let is_selected = *id == app.cfg.selected;
                    let is_native = id == "phantom" || *tag == "NATIVE";

                    let (r, del) = render_build_tile(ui, t, id, tag, is_selected, is_native);
                    if r.clicked() {
                        pick = Some(id.clone());
                    }
                    if del {
                        delete_id = Some(id.clone());
                    }
                }

                if app::add_tile(ui).clicked() {
                    add_jar = true;
                }
            });

            if app.versions.is_empty() {
                ui.add_space(14.);
                ui.label(app::muted(
                    "Java-версии не найдены в папке игры (versions/). Создайте профиль или добавьте существующий .jar.",
                ));
            }
        });

    if add_jar {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("Java Archive", &["jar"])
            .pick_file()
        {
            let s = p.to_string_lossy().to_string();
            if !app.cfg.custom_jars.contains(&s) {
                app.cfg.custom_jars.push(s.clone());
            }
            pick = Some(format!("jar:{s}"));
        }
    }

    if let Some(id) = pick {
        app.cfg.selected = id;
        app.cfg.save();
    }

    if let Some(del) = delete_id {
        // Enforce protection for Phantom Native
        if del != "phantom" {
            if del.starts_with("jar:") {
                let jar_path = &del["jar:".len()..];
                app.cfg.custom_jars.retain(|j| j != jar_path);
                app.status = Some((true, "Файл .jar удален из списка".into()));
            } else if del.starts_with("java:") {
                let ver_name = &del["java:".len()..];
                let ver_dir = app.cfg.game_dir().join("versions").join(ver_name);
                if ver_dir.exists() {
                    let _ = fs::remove_dir_all(&ver_dir);
                }
                app.status = Some((true, format!("Профиль \"{ver_name}\" удален")));
            }

            if app.cfg.selected == del {
                app.cfg.selected = "phantom".into();
            }
            app.cfg.save();
            app.rescan();
        }
    }
}

fn render_build_tile(
    ui: &mut egui::Ui,
    t: f32,
    id: &str,
    tag: &str,
    selected: bool,
    is_native: bool,
) -> (egui::Response, bool) {
    let (rect, resp) = ui.allocate_exact_size(vec2(250., 130.), Sense::click());
    let p = ui.painter().clone();
    let mut delete_clicked = false;

    if selected {
        fx::glow(&p, rect, fx::shimmer(t * 0.25), 0.5, 8.);
    }
    p.rect_filled(
        rect,
        2.,
        if resp.hovered() {
            theme::CARD_HI
        } else {
            theme::CARD
        },
    );
    let border = if selected {
        Stroke::new(2., fx::shimmer(t * 0.25))
    } else if resp.hovered() {
        Stroke::new(1., theme::YELLOW)
    } else {
        Stroke::new(1., theme::BORDER)
    };
    p.rect_stroke(rect, 2., border);

    p.text(
        rect.left_top() + vec2(16., 16.),
        Align2::LEFT_TOP,
        tag,
        FontId::proportional(12.),
        theme::YELLOW,
    );

    let title = app::build_title(id);
    let title_font = if title.chars().count() > 18 {
        FontId::proportional(16.)
    } else {
        FontId::proportional(19.)
    };
    p.text(
        rect.left_top() + vec2(16., 38.),
        Align2::LEFT_TOP,
        title,
        title_font,
        theme::TEXT,
    );

    let sub = if is_native {
        "Движок на Rust (чистый C/Rust, wgpu)"
    } else if id.starts_with("java:") {
        "Изолированная Java-сборка"
    } else {
        "Пользовательский .jar файл"
    };
    p.text(
        rect.left_top() + vec2(16., 68.),
        Align2::LEFT_TOP,
        sub,
        FontId::proportional(11.),
        theme::TEXT_SOFT,
    );

    let status = if selected {
        "✓ ВЫБРАНО"
    } else {
        "НАЖМИТЕ ДЛЯ ВЫБОРА"
    };
    p.text(
        rect.left_bottom() + vec2(16., -14.),
        Align2::LEFT_BOTTOM,
        status,
        FontId::proportional(11.),
        if selected { theme::YELLOW } else { theme::MUTED },
    );

    // Delete button for non-native profiles
    if !is_native {
        let del_rect = Rect::from_center_size(
            pos2(rect.right() - 20., rect.top() + 20.),
            vec2(24., 24.),
        );
        let del_resp = ui.interact(
            del_rect,
            ui.id().with(format!("del_{id}")),
            Sense::click(),
        );
        let del_hover = del_resp.hovered();

        let del_fill = if del_hover {
            theme::DANGER.gamma_multiply(0.25)
        } else {
            theme::SIDE
        };
        let del_stroke = if del_hover {
            Stroke::new(1.5, theme::DANGER)
        } else {
            Stroke::new(1., theme::BORDER)
        };

        p.rect_filled(del_rect, 2., del_fill);
        p.rect_stroke(del_rect, 2., del_stroke);
        p.text(
            del_rect.center(),
            Align2::CENTER_CENTER,
            "🗑",
            FontId::proportional(13.),
            if del_hover { theme::DANGER } else { theme::MUTED },
        );

        if del_resp.clicked() {
            delete_clicked = true;
        }
        if del_hover {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
    }

    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    (resp, delete_clicked)
}
