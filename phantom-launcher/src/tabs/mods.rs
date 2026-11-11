//! Mods & Shaders management tab: 310px left column for installed files + Modrinth catalog.

use crate::{
    app::{self, App},
    fx,
    modrinth::{Hit, Kind},
    theme,
};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CursorIcon, FontId, Frame, Layout, Margin, Rect,
    RichText, Rounding, Sense, Stroke, TextEdit,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone)]
pub struct InstalledFile {
    pub name: String,
    pub clean_name: String,
    pub is_enabled: bool,
    pub size: u64,
    pub path: PathBuf,
}

pub fn scan_installed(target_dir: &Path) -> Vec<InstalledFile> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(target_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.ends_with(".part") {
                        let is_enabled = !name.ends_with(".disabled");
                        let clean_name = name.strip_suffix(".disabled").unwrap_or(&name).to_string();
                        files.push(InstalledFile {
                            name,
                            clean_name,
                            is_enabled,
                            size: meta.len(),
                            path: entry.path(),
                        });
                    }
                }
            }
        }
    }
    files.sort_by(|a, b| a.clean_name.to_lowercase().cmp(&b.clean_name.to_lowercase()));
    files
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn render(app: &mut App, ui: &mut egui::Ui, t: f32) {
    let sub = match app.kind {
        Kind::Shader => "Управление шейдерами и каталог Modrinth",
        Kind::ResourcePack => "Управление ресурспаками и каталог Modrinth",
        Kind::Mod => "Управление модами и каталог Modrinth",
    };
    app::heading(ui, t, "МОДЫ И ШЕЙДЕРЫ", sub);

    let ctx = ui.ctx().clone();
    let mut search = false;
    let target_dir = app.target_folder();
    let installed_files = scan_installed(&target_dir);

    let mut file_to_delete: Option<PathBuf> = None;
    let mut rename_action: Option<(PathBuf, PathBuf)> = None;

    let avail_w = ui.available_width();
    let left_w = 310.0f32.min(avail_w * 0.4);
    let right_w = (avail_w - left_w - 16.).max(320.);

    ui.horizontal_top(|ui| {
        // =========================================================================
        // LEFT COLUMN (310px): INSTALLED FILES
        // =========================================================================
        ui.allocate_ui_with_layout(
            vec2(left_w, ui.available_height()),
            Layout::top_down(Align::Min),
            |ui| {
                Frame::none()
                    .fill(theme::CARD)
                    .stroke(Stroke::new(1., theme::BORDER))
                    .rounding(Rounding::same(4.))
                    .inner_margin(Margin::same(12.))
                    .show(ui, |ui| {
                        let count = installed_files.len();
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("УСТАНОВЛЕНО ({count})"))
                                    .size(13.)
                                    .color(theme::TEXT)
                                    .strong(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("📁 Папка")
                                                .size(11.)
                                                .color(theme::YELLOW),
                                        )
                                        .fill(theme::SIDE)
                                        .stroke(Stroke::new(1., theme::BORDER)),
                                    )
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    let _ = fs::create_dir_all(&target_dir);
                                    let _ = Command::new("explorer").arg(&target_dir).spawn();
                                }
                            });
                        });

                        // "+ Добавить с ПК"
                        ui.add_space(4.);
                        if ui
                            .add_sized(
                                [ui.available_width(), 26.],
                                egui::Button::new(
                                    RichText::new("+ Добавить с ПК")
                                        .size(11.5)
                                        .color(theme::YELLOW),
                                )
                                .fill(theme::SIDE)
                                .stroke(Stroke::new(1., theme::BORDER)),
                            )
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        {
                            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                                let _ = fs::create_dir_all(&target_dir);
                                if let Some(file_name) = file_path.file_name() {
                                    let dest = target_dir.join(file_name);
                                    let _ = fs::copy(&file_path, &dest);
                                }
                            }
                        }

                        let total_size: u64 = installed_files.iter().map(|f| f.size).sum();
                        ui.label(app::muted(&format!("Всего: {}", format_size(total_size))));

                        ui.add_space(6.);
                        ui.separator();
                        ui.add_space(6.);

                        if installed_files.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.);
                                ui.label(
                                    RichText::new("Нет установленных файлов")
                                        .size(13.)
                                        .color(theme::MUTED),
                                );
                                ui.label(
                                    RichText::new("Выберите дополнение справа в каталоге")
                                        .size(11.)
                                        .color(theme::TEXT_SOFT),
                                );
                            });
                        } else {
                            egui::ScrollArea::vertical()
                                .id_source("installed_scroll")
                                .auto_shrink([false; 2])
                                .max_height(ui.available_height() - 20.)
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = vec2(0., 6.);

                                    for file in &installed_files {
                                        let item_w = ui.available_width();
                                        let (rect, resp) =
                                            ui.allocate_exact_size(vec2(item_w, 42.), Sense::hover());
                                        let p = ui.painter();
                                        let hover = resp.hovered();

                                        let bg = if hover { theme::CARD_HI } else { theme::SIDE };
                                        p.rect_filled(rect, 2., bg);
                                        let stroke = if file.is_enabled {
                                            if hover {
                                                Stroke::new(1., theme::YELLOW)
                                            } else {
                                                Stroke::new(1., theme::BORDER)
                                            }
                                        } else {
                                            Stroke::new(1., theme::BORDER.gamma_multiply(0.5))
                                        };
                                        p.rect_stroke(rect, 2., stroke);

                                        // Left: Toggle Checkbox
                                        let check_rect = Rect::from_center_size(
                                            pos2(rect.left() + 16., rect.center().y),
                                            vec2(18., 18.),
                                        );
                                        let mut state = file.is_enabled;
                                        let check_resp = ui.interact(
                                            check_rect,
                                            ui.id().with(format!("chk_{}", file.name)),
                                            Sense::click(),
                                        );
                                        if check_resp.clicked() {
                                            state = !state;
                                            if state {
                                                let new_p = file.path.with_file_name(&file.clean_name);
                                                rename_action = Some((file.path.clone(), new_p));
                                            } else {
                                                let new_name = format!("{}.disabled", file.name);
                                                let new_p = file.path.with_file_name(&new_name);
                                                rename_action = Some((file.path.clone(), new_p));
                                            }
                                        }
                                        if check_resp.hovered() {
                                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                        }

                                        let box_fill = if state { theme::YELLOW } else { theme::CARD };
                                        p.rect_filled(check_rect, 2., box_fill);
                                        p.rect_stroke(check_rect, 1., Stroke::new(1., theme::YELLOW));
                                        if state {
                                            p.text(
                                                check_rect.center(),
                                                Align2::CENTER_CENTER,
                                                "✓",
                                                FontId::proportional(13.),
                                                Color32::BLACK,
                                            );
                                        }

                                        // Right: Delete button [✕]
                                        let del_rect = Rect::from_center_size(
                                            pos2(rect.right() - 16., rect.center().y),
                                            vec2(20., 20.),
                                        );
                                        let del_resp = ui.interact(
                                            del_rect,
                                            ui.id().with(format!("del_{}", file.name)),
                                            Sense::click(),
                                        );
                                        let del_hover = del_resp.hovered();
                                        if del_hover {
                                            p.rect_filled(
                                                del_rect,
                                                2.,
                                                theme::DANGER.gamma_multiply(0.2),
                                            );
                                            p.rect_stroke(
                                                del_rect,
                                                1.,
                                                Stroke::new(1., theme::DANGER),
                                            );
                                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                        }
                                        p.text(
                                            del_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "✕",
                                            FontId::proportional(12.),
                                            if del_hover { theme::DANGER } else { theme::MUTED },
                                        );
                                        if del_resp.clicked() {
                                            file_to_delete = Some(file.path.clone());
                                        }

                                        // Middle: Title & size
                                        let text_left = rect.left() + 32.;
                                        let title_color = if file.is_enabled {
                                            theme::TEXT
                                        } else {
                                            theme::MUTED
                                        };

                                        let name_display: String = if file.clean_name.len() > 22 {
                                            format!("{}…", &file.clean_name[..20])
                                        } else {
                                            file.clean_name.clone()
                                        };

                                        p.text(
                                            pos2(text_left, rect.top() + 6.),
                                            Align2::LEFT_TOP,
                                            name_display,
                                            FontId::proportional(12.),
                                            title_color,
                                        );
                                        p.text(
                                            pos2(text_left, rect.top() + 22.),
                                            Align2::LEFT_TOP,
                                            format_size(file.size),
                                            FontId::proportional(10.),
                                            theme::TEXT_SOFT,
                                        );
                                    }
                                });
                        }
                    });
            },
        );

        ui.add_space(8.);

        // =========================================================================
        // RIGHT COLUMN (65%): MODRINTH CATALOG & FILTERS
        // =========================================================================
        ui.allocate_ui_with_layout(
            vec2(right_w, ui.available_height()),
            Layout::top_down(Align::Min),
            |ui| {
                // Kind Tabs: Shaders, ResourcePacks, Mods
                ui.horizontal(|ui| {
                    for (k, label) in [
                        (Kind::Shader, "Шейдеры"),
                        (Kind::ResourcePack, "Ресурспаки"),
                        (Kind::Mod, "Моды"),
                    ] {
                        if ui.selectable_label(app.kind == k, label).clicked() && app.kind != k {
                            app.kind = k;
                            app.hits.clear();
                            search = true;
                        }
                    }
                    ui.separator();
                    ui.label(app::muted("Сборка:"));
                    let current_target = app::build_title(&app.install_target);
                    ui.menu_button(RichText::new(current_target).color(theme::TEXT), |ui| {
                        ui.set_min_width(220.);
                        for (id, _) in app.entries() {
                            if ui
                                .selectable_label(id == app.install_target, app::build_title(&id))
                                .clicked()
                            {
                                app.install_target = id;
                                ui.close_menu();
                            }
                        }
                    });
                });

                // Engine notice for Phantom Native
                if app.install_target == "phantom" {
                    ui.add_space(4.);
                    Frame::none()
                        .fill(theme::CARD_HI)
                        .stroke(Stroke::new(1., theme::YELLOW))
                        .rounding(Rounding::same(2.))
                        .inner_margin(Margin::symmetric(10., 6.))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(
                                    "⚡ Сборка Phantom (Rust): нативно работают Шейдеры, Ресурспаки и .hmod плагины. Для Forge/Fabric .jar выберите Java-сборку.",
                                )
                                .size(11.)
                                .color(theme::YELLOW),
                            );
                        });
                }

                ui.add_space(6.);

                // Search Bar & Loader Filters
                ui.horizontal(|ui| {
                    if app.kind == Kind::Mod {
                        for l in ["fabric", "forge", "neoforge", "quilt"] {
                            if ui.selectable_label(app.loader == l, l).clicked() {
                                app.loader = l.into();
                                search = true;
                            }
                        }
                        ui.separator();
                    }
                    ui.label(app::muted("Версия:"));
                    let r_ver = ui.add(
                        TextEdit::singleline(&mut app.game_ver)
                            .desired_width(70.)
                            .hint_text("1.20.1"),
                    );
                    if r_ver.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        search = true;
                    }
                });

                ui.horizontal(|ui| {
                    let r = ui.add(
                        TextEdit::singleline(&mut app.query)
                            .desired_width(320.)
                            .hint_text("Поиск: sodium, complementary, bare bones, iris…"),
                    );
                    if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        search = true;
                    }
                    if ui
                        .add_enabled(!app.searching, app::primary_button("НАЙТИ"))
                        .clicked()
                    {
                        search = true;
                    }
                    if app.searching {
                        ui.spinner();
                    }
                });

                if search {
                    app.start_search(&ctx);
                }

                app::status_line(ui, &app.mod_status);

                // Modrinth hits
                let mut get: Option<String> = None;
                egui::ScrollArea::vertical()
                    .id_source("catalog_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let hits = app.hits.clone();
                        ui.columns(2, |cols| {
                            for (i, h) in hits.iter().enumerate() {
                                let col_ui = &mut cols[i % 2];
                                if let Some(id) = app.hit_card(col_ui, &ctx, t, i, h, &installed_files) {
                                    get = Some(id);
                                }
                                col_ui.add_space(8.);
                            }
                        });
                    });

                if let Some(id) = get {
                    app.start_download(&ctx, id);
                }
            },
        );
    });

    if let Some((old_p, new_p)) = rename_action {
        let _ = fs::rename(old_p, new_p);
    }

    if let Some(path) = file_to_delete {
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if let Err(e) = fs::remove_file(&path) {
            app.mod_status = Some((false, format!("Ошибка удаления: {e}")));
        } else {
            app.mod_status = Some((true, format!("Файл удален: {name}")));
        }
    }
}
