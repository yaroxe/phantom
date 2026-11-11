//! The window: left rail, middle pages, bottom dock.

use crate::{
    config::Config,
    fx, java, launch,
    modrinth::{self, Hit, Kind},
    tabs::{builds_tab, mods::{self, InstalledFile}},
    theme,
};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Button, Color32, CursorIcon, FontId, Frame, Layout, Margin,
    Painter, Pos2, Rect, RichText, Rounding, Sense, Shape, Stroke, TextEdit, TextureHandle,
};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Duration,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Home,
    Builds,
    Mods,
    Settings,
}

pub enum Msg {
    Launch(Result<String, String>),
    Search(Result<Vec<Hit>, String>),
    Download(String, Result<String, String>),
    Icon(String, Result<Vec<u8>, String>),
}

pub struct App {
    pub cfg: Config,
    pub tab: Tab,
    pub javas: Vec<PathBuf>,
    pub versions: Vec<String>,
    pub status: Option<(bool, String)>,
    pub launching: bool,
    pub tx: Sender<Msg>,
    pub rx: Receiver<Msg>,
    // Modrinth
    pub query: String,
    pub kind: Kind,
    pub loader: String,
    pub game_ver: String,
    pub hits: Vec<Hit>,
    pub searching: bool,
    pub mods_loaded: bool,
    pub downloading: HashSet<String>,
    pub mod_status: Option<(bool, String)>,
    pub install_target: String,
    // Icon cache
    pub icons: HashMap<String, TextureHandle>,
    pub loading_icons: HashSet<String>,
    pub creating_profile: bool,
    pub new_profile_name: String,
}

impl App {
    pub fn new() -> App {
        let cfg = Config::load();
        let (tx, rx) = channel();
        let selected = cfg.selected.clone();
        App {
            javas: java::detect(cfg.game_dir()),
            versions: launch::installed_versions(cfg.game_dir()),
            cfg,
            tab: Tab::Home,
            status: None,
            launching: false,
            tx,
            rx,
            query: String::new(),
            kind: Kind::Shader,
            loader: "fabric".into(),
            game_ver: String::new(),
            hits: Vec::new(),
            searching: false,
            mods_loaded: false,
            downloading: HashSet::new(),
            mod_status: None,
            install_target: selected,
            icons: HashMap::new(),
            loading_icons: HashSet::new(),
            creating_profile: false,
            new_profile_name: String::new(),
        }
    }

    pub fn rescan(&mut self) {
        self.javas = java::detect(self.cfg.game_dir());
        self.versions = launch::installed_versions(self.cfg.game_dir());
    }

    pub fn entries(&self) -> Vec<(String, &'static str)> {
        let mut e: Vec<(String, &'static str)> = vec![("phantom".into(), "NATIVE")];
        e.extend(self.versions.iter().map(|v| (format!("java:{v}"), "JAVA")));
        e.extend(
            self.cfg
                .custom_jars
                .iter()
                .map(|j| (format!("jar:{j}"), "JAR")),
        );
        e
    }

    pub fn target_folder(&self) -> PathBuf {
        if self.install_target == "phantom" {
            crate::config::data_root()
                .join("phantom")
                .join(self.kind.folder())
        } else {
            PathBuf::from(&self.cfg.game_dir).join(self.kind.folder())
        }
    }

    pub fn start_play(&mut self, ctx: &egui::Context) {
        self.launching = true;
        self.status = None;
        let (cfg, tx, ctx) = (self.cfg.clone(), self.tx.clone(), ctx.clone());
        thread::spawn(move || {
            let _ = tx.send(Msg::Launch(launch::play(&cfg)));
            ctx.request_repaint();
        });
    }

    pub fn start_search(&mut self, ctx: &egui::Context) {
        if self.searching {
            return;
        }
        self.searching = true;
        self.mod_status = None;
        let (q, kind, loader, ver) = (
            self.query.clone(),
            self.kind,
            self.loader.clone(),
            self.game_ver.clone(),
        );
        let (tx, ctx) = (self.tx.clone(), ctx.clone());
        thread::spawn(move || {
            let _ = tx.send(Msg::Search(modrinth::search(&q, kind, &loader, &ver)));
            ctx.request_repaint();
        });
    }

    pub fn start_download(&mut self, ctx: &egui::Context, id: String) {
        if !self.downloading.insert(id.clone()) {
            return;
        }
        let dest = self.target_folder();
        let is_phantom = self.install_target == "phantom";
        let is_shader = self.kind == Kind::Shader;
        let (kind, loader, ver) = (self.kind, self.loader.clone(), self.game_ver.clone());
        let (tx, ctx) = (self.tx.clone(), ctx.clone());

        thread::spawn(move || {
            let res = modrinth::download(&id, kind, &loader, &ver, &dest);
            // If downloading shader for Phantom Engine, duplicate into version folder too
            if let Ok(ref filename) = res {
                if is_phantom && is_shader {
                    let versions_dir = crate::config::data_root()
                        .join("phantom-launcher")
                        .join("versions");
                    if let Ok(entries) = fs::read_dir(&versions_dir) {
                        for e in entries.flatten() {
                            let p = e.path();
                            if p.is_dir() {
                                let v_shader_dir = p.join("shaderpacks");
                                let _ = fs::create_dir_all(&v_shader_dir);
                                let _ = fs::copy(dest.join(filename), v_shader_dir.join(filename));
                            }
                        }
                    }
                }
            }
            let _ = tx.send(Msg::Download(id, res));
            ctx.request_repaint();
        });
    }

    pub fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Launch(r) => {
                    self.launching = false;
                    self.status = Some(match r {
                        Ok(m) => (true, m),
                        Err(e) => (false, e),
                    });
                }
                Msg::Search(r) => {
                    self.searching = false;
                    match r {
                        Ok(h) => {
                            if h.is_empty() {
                                self.mod_status = Some((true, "Ничего не найдено".into()));
                            }
                            self.hits = h;
                        }
                        Err(e) => self.mod_status = Some((false, e)),
                    }
                }
                Msg::Download(id, r) => {
                    self.downloading.remove(&id);
                    self.mod_status = Some(match r {
                        Ok(f) => (true, format!("Установлено: {f}")),
                        Err(e) => (false, e),
                    });
                }
                Msg::Icon(id, r) => {
                    self.loading_icons.remove(&id);
                    if let Ok(bytes) = r {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let rgba = img.to_rgba8();
                            let (w, h) = rgba.dimensions();
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                [w as usize, h as usize],
                                &rgba.into_raw(),
                            );
                            let texture = ctx.load_texture(
                                format!("icon_{id}"),
                                color_image,
                                egui::TextureOptions::LINEAR,
                            );
                            self.icons.insert(id, texture);
                        }
                    }
                }
            }
        }
    }

    pub fn hit_card(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        t: f32,
        i: usize,
        h: &Hit,
        installed: &[InstalledFile],
    ) -> Option<String> {
        let mut download_click = None;

        // Check if installed
        let is_installed = installed.iter().any(|f| {
            let c = f.clean_name.to_lowercase();
            let t = h.title.to_lowercase();
            c.contains(&t) || t.contains(&c)
        });

        // Request icon if needed
        if let Some(ref url) = h.icon_url {
            if !self.icons.contains_key(&h.project_id)
                && !self.loading_icons.contains(&h.project_id)
            {
                self.loading_icons.insert(h.project_id.clone());
                let (tx, id, url, ctx) = (
                    self.tx.clone(),
                    h.project_id.clone(),
                    url.clone(),
                    ctx.clone(),
                );
                thread::spawn(move || {
                    let res = modrinth::fetch_icon(&url);
                    let _ = tx.send(Msg::Icon(id, res));
                    ctx.request_repaint();
                });
            }
        }

        let r = Frame::none()
            .fill(theme::CARD)
            .stroke(Stroke::new(1., theme::BORDER))
            .rounding(Rounding::same(2.))
            .inner_margin(Margin::same(12.))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let icon_size = vec2(44., 44.);
                    if let Some(tex) = self.icons.get(&h.project_id) {
                        ui.image((tex.id(), icon_size));
                    } else {
                        let (a, _) = ui.allocate_exact_size(icon_size, Sense::hover());
                        ui.painter().rect_filled(a, 2., theme::SIDE);
                        ui.painter()
                            .rect_stroke(a, 2., Stroke::new(1., theme::BORDER));
                        let letter = h
                            .title
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();
                        ui.painter().text(
                            a.center(),
                            Align2::CENTER_CENTER,
                            letter,
                            FontId::proportional(20.),
                            fx::shimmer(t * 0.2 + i as f32 * 0.08),
                        );
                    }

                    let mid_w = (ui.available_width() - 130.0).max(100.0);
                    ui.allocate_ui_with_layout(
                        vec2(mid_w, 44.),
                        Layout::top_down(Align::Min),
                        |ui| {
                            let title: String = h.title.chars().take(24).collect();
                            ui.label(RichText::new(title).size(14.).color(theme::TEXT).strong());
                            ui.label(muted(&format!(
                                "{} · {}",
                                h.author,
                                short_num(h.downloads)
                            )));
                        },
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.downloading.contains(&h.project_id) {
                            ui.spinner();
                        } else if is_installed {
                            let _ = ui.add_sized(
                                [110., 32.],
                                Button::new(RichText::new("УСТАНОВЛЕНО ✓").size(11.).color(Color32::BLACK).strong())
                                    .fill(theme::OK),
                            );
                        } else if ui
                            .add_sized([110., 32.], primary_button("СКАЧАТЬ"))
                            .clicked()
                        {
                            download_click = Some(h.project_id.clone());
                        }
                    });
                });
                ui.add_space(4.);
                let desc_raw: String = h.description.chars().take(65).collect();
                let desc = if h.description.chars().count() > 65 {
                    format!("{}...", desc_raw.trim_end())
                } else {
                    desc_raw
                };
                ui.label(RichText::new(desc).size(12.));
            });

        if r.response.hovered() {
            ui.painter().rect_stroke(
                r.response.rect,
                2.,
                Stroke::new(1.5, fx::shimmer(t * 0.3 + i as f32 * 0.1)),
            );
        }

        download_click
    }

    fn rail(&mut self, ctx: &egui::Context, t: f32) {
        let frame = Frame::none()
            .fill(theme::SIDE)
            .stroke(Stroke::new(1., theme::BORDER));
        egui::SidePanel::left("rail")
            .exact_width(92.)
            .resizable(false)
            .frame(frame)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = vec2(0., 0.);
                let (r, _) = ui.allocate_exact_size(vec2(92., 88.), Sense::hover());
                let c = r.center();
                ui.painter().add(Shape::closed_line(
                    fx::hexagon(c, 21., t * 0.3),
                    Stroke::new(2.5, fx::shimmer(t * 0.2)),
                ));
                ui.painter().text(
                    c,
                    Align2::CENTER_CENTER,
                    "P",
                    FontId::proportional(20.),
                    theme::TEXT,
                );

                for (tab, label) in [
                    (Tab::Home, "ГЛАВНАЯ"),
                    (Tab::Builds, "СБОРКИ"),
                    (Tab::Mods, "МОДЫ"),
                    (Tab::Settings, "НАСТРОЙКИ"),
                ] {
                    let (r, resp) = ui.allocate_exact_size(vec2(92., 80.), Sense::click());
                    let active = self.tab == tab;
                    let hot = resp.hovered();
                    let p = ui.painter();
                    if active || hot {
                        let a = if active { 1. } else { 0.6 };
                        p.rect_filled(r, 0., theme::CARD_HI.gamma_multiply(a));
                    }
                    if active {
                        fx::gradient(
                            p,
                            Rect::from_min_size(r.left_top(), vec2(3., r.height())),
                            t,
                            true,
                            1.,
                        );
                    }
                    let col = if active {
                        fx::shimmer(t * 0.2)
                    } else if hot {
                        theme::TEXT
                    } else {
                        theme::MUTED
                    };
                    tab_icon(p, tab, pos2(r.center().x, r.top() + 31.), col);
                    p.text(
                        pos2(r.center().x, r.bottom() - 17.),
                        Align2::CENTER_CENTER,
                        label,
                        FontId::proportional(10.5),
                        col,
                    );
                    if resp.clicked() {
                        self.tab = tab;
                    }
                    resp.on_hover_cursor(CursorIcon::PointingHand);
                }
            });
    }

    fn dock(&mut self, ctx: &egui::Context, t: f32) {
        let frame = Frame::none()
            .fill(theme::SIDE)
            .inner_margin(Margin::symmetric(24., 0.));
        egui::TopBottomPanel::bottom("dock")
            .exact_height(100.)
            .frame(frame)
            .show(ctx, |ui| {
                let r = ui.max_rect();
                ui.painter().line_segment(
                    [r.left_top(), r.right_top()],
                    Stroke::new(1., theme::BORDER),
                );
                ui.horizontal_centered(|ui| {
                    // Player input
                    let (avatar_r, _) = ui.allocate_exact_size(vec2(44., 44.), Sense::hover());
                    draw_avatar(ui.painter(), avatar_r, &self.cfg.nickname, t);

                    ui.vertical(|ui| {
                        ui.label(muted("ИГРОК · OFFLINE"));
                        let r = ui.add(
                            TextEdit::singleline(&mut self.cfg.nickname)
                                .desired_width(140.)
                                .font(FontId::proportional(14.)),
                        );
                        if r.lost_focus() {
                            self.cfg.save();
                        }
                    });

                    ui.add_space(20.);

                    // Build selector
                    ui.vertical(|ui| {
                        ui.label(muted("СБОРКА"));
                        let title = build_title(&self.cfg.selected);
                        ui.menu_button(
                            RichText::new(title)
                                .size(14.)
                                .color(theme::TEXT)
                                .strong(),
                            |ui| {
                                ui.set_min_width(240.);
                                for (id, tag) in self.entries() {
                                    let active = id == self.cfg.selected;
                                    let label = format!("[{tag}] {}", build_title(&id));
                                    if ui.selectable_label(active, label).clicked() {
                                        self.cfg.selected = id.clone();
                                        self.install_target = id;
                                        self.cfg.save();
                                        ui.close_menu();
                                    }
                                }
                            },
                        );
                    });

                    ui.add_space(16.);

                    // Short status message without wrapping under button
                    ui.allocate_ui_with_layout(
                        vec2(180., 40.),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            if let Some((ok, msg)) = &self.status {
                                let col = if *ok { theme::TEXT_SOFT } else { theme::DANGER };
                                let short_msg = if msg.len() > 24 {
                                    format!("{}…", &msg[..22])
                                } else {
                                    msg.clone()
                                };
                                ui.label(RichText::new(short_msg).size(11.5).color(col));
                            }
                        },
                    );

                    // PLAY button on right
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let btn_size = vec2(200., 52.);
                        let (r, resp) = ui.allocate_exact_size(btn_size, Sense::click());
                        let hot = resp.hovered();
                        let p = ui.painter();

                        let fill = if self.launching {
                            theme::CARD_HI
                        } else if hot {
                            theme::YELLOW
                        } else {
                            theme::YELLOW.gamma_multiply(0.9)
                        };

                        p.rect_filled(r, 2., fill);
                        p.rect_stroke(r, 2., Stroke::new(1., theme::YELLOW));
                        if hot && !self.launching {
                            fx::gradient(p, r, t, false, 0.4);
                        }

                        let text = if self.launching {
                            "ЗАПУСК..."
                        } else {
                            "▶  ИГРАТЬ"
                        };
                        p.text(
                            r.center(),
                            Align2::CENTER_CENTER,
                            text,
                            FontId::proportional(16.),
                            Color32::BLACK,
                        );

                        if resp.clicked() && !self.launching {
                            self.start_play(ctx);
                        }
                        resp.on_hover_cursor(CursorIcon::PointingHand);
                    });
                });
            });
    }

    fn home(&mut self, ui: &mut egui::Ui, t: f32) {
        heading(ui, t, "PHANTOM ENGINE", "Быстрый лаунчер для Minecraft 26.3 и Java");

        let hero_h = 200.;
        let (hero, _) = ui.allocate_exact_size(vec2(ui.available_width(), hero_h), Sense::hover());
        let p = ui.painter();
        p.rect_filled(hero, 4., theme::CARD);
        p.rect_stroke(hero, 4., Stroke::new(1., theme::BORDER));

        let c = pos2(hero.left() + 110., hero.center().y);
        fx::reactor(p, c, 52., t);

        let text_left = hero.left() + 200.;
        p.text(
            pos2(text_left, hero.top() + 45.),
            Align2::LEFT_TOP,
            "Phantom Engine 26.3",
            FontId::proportional(24.),
            theme::TEXT,
        );
        p.text(
            pos2(text_left, hero.top() + 82.),
            Align2::LEFT_TOP,
            "Движок на Rust + wgpu (Vulkan) с поддержкой SIMD Frustum Culling и Fat LTO.",
            FontId::proportional(13.),
            theme::TEXT_SOFT,
        );
        p.text(
            pos2(text_left, hero.top() + 104.),
            Align2::LEFT_TOP,
            "Нативная загрузка шейдеров, физика взрывов через Rayon и стабильный тик.",
            FontId::proportional(13.),
            theme::MUTED,
        );

        ui.add_space(20.);

        ui.columns(3, |cols| {
            cols[0].vertical(|ui| {
                card(ui, |ui| {
                    ui.label(muted("СБОРКА ДВИЖКА"));
                    ui.label(RichText::new("Phantom Multi-Core").size(15.).color(theme::YELLOW).strong());
                    ui.label(muted("26.3 · Vulkan/Rayon"));
                });
            });
            cols[1].vertical(|ui| {
                card(ui, |ui| {
                    ui.label(muted("АКТИВНЫЙ ИГРОК"));
                    ui.label(RichText::new(&self.cfg.nickname).size(15.).color(theme::TEXT).strong());
                    ui.label(muted("Offline UUID"));
                });
            });
            cols[2].vertical(|ui| {
                card(ui, |ui| {
                    ui.label(muted("ВЫДЕЛЕНИЕ RAM"));
                    ui.label(RichText::new(format!("{} GB", self.cfg.ram_gb)).size(15.).color(theme::TEXT).strong());
                    ui.label(muted("Для Java версий"));
                });
            });
        });
    }

    fn builds(&mut self, ui: &mut egui::Ui, t: f32) {
        builds_tab::render(self, ui, t);
    }

    fn mods(&mut self, ui: &mut egui::Ui, t: f32) {
        mods::render(self, ui, t);
    }

    fn settings(&mut self, ui: &mut egui::Ui, t: f32) {
        heading(ui, t, "НАСТРОЙКИ", "Память, папки и Java");
        let mut changed = false;
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                card(ui, |ui| {
                    ui.label(muted("ОПЕРАТИВНАЯ ПАМЯТЬ (RAM) ДЛЯ JAVA"));
                    changed |= ui
                        .add(egui::Slider::new(&mut self.cfg.ram_gb, 2..=16).suffix(" GB"))
                        .changed();
                });
                card(ui, |ui| {
                    changed |= path_row(
                        ui,
                        "ПАПКА ИГРЫ JAVA (.minecraft)",
                        &mut self.cfg.game_dir,
                        true,
                    );
                });
                card(ui, |ui| {
                    changed |= path_row(
                        ui,
                        "ПАПКА PHANTOM (%APPDATA%\\phantom)",
                        &mut self.cfg.phantom_dir,
                        true,
                    );
                });
                card(ui, |ui| {
                    changed |= path_row(
                        ui,
                        "ПУТЬ К КЛИЕНТУ (phantom-client.exe)",
                        &mut self.cfg.phantom_client,
                        false,
                    );
                });
                card(ui, |ui| {
                    changed |= path_row(
                        ui,
                        "ПУТЬ К JAVAW (javaw.exe)",
                        &mut self.cfg.java_path,
                        false,
                    );
                    if ui.add(outline_button("ПОИСК JAVA ЗАНОВО")).clicked() {
                        self.rescan();
                    }
                });
            });
        if changed {
            self.cfg.save();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll(ctx);
        if self.tab == Tab::Mods && !self.mods_loaded {
            self.mods_loaded = true;
            self.start_search(ctx);
        }
        ctx.request_repaint_after(Duration::from_millis(33));
        let t = ctx.input(|i| i.time) as f32;

        self.rail(ctx, t);
        self.dock(ctx, t);
        let area = ctx.available_rect();
        let frame = Frame::none()
            .fill(theme::BG)
            .inner_margin(Margin::same(28.));
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            fx::dotgrid(ui.painter(), area, 28., theme::BORDER.gamma_multiply(0.7));
            match self.tab {
                Tab::Home => self.home(ui, t),
                Tab::Builds => self.builds(ui, t),
                Tab::Mods => self.mods(ui, t),
                Tab::Settings => self.settings(ui, t),
            }
        });
    }
}

// ---------------------------------------------------------------- helpers & widgets

pub fn muted(s: &str) -> RichText {
    RichText::new(s).size(12.).color(theme::MUTED)
}

pub fn heading(ui: &mut egui::Ui, t: f32, title: &str, sub: &str) {
    ui.label(RichText::new(title).size(26.).color(theme::TEXT).strong());
    ui.label(muted(sub));
    let (r, _) = ui.allocate_exact_size(vec2(120., 3.), Sense::hover());
    fx::gradient(ui.painter(), r, t, false, 1.);
    ui.add_space(8.);
}

pub fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    Frame::none()
        .fill(theme::CARD)
        .stroke(Stroke::new(1., theme::BORDER))
        .rounding(Rounding::same(2.))
        .inner_margin(Margin::same(16.))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

pub fn primary_button(text: &str) -> Button<'static> {
    Button::new(RichText::new(text).size(13.).color(Color32::BLACK).strong())
        .fill(theme::YELLOW)
        .stroke(Stroke::new(1., theme::YELLOW))
}

pub fn outline_button(text: &str) -> Button<'static> {
    Button::new(RichText::new(text).size(13.).color(theme::YELLOW))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1., theme::YELLOW))
}

pub fn status_line(ui: &mut egui::Ui, status: &Option<(bool, String)>) {
    if let Some((ok, msg)) = status {
        let color = if *ok { theme::OK } else { theme::DANGER };
        ui.label(RichText::new(msg).color(color));
    }
}

pub fn build_title(id: &str) -> String {
    if id == "phantom" {
        "Phantom Engine (Rust)".into()
    } else if let Some(v) = id.strip_prefix("java:") {
        format!("Minecraft {v}")
    } else if let Some(j) = id.strip_prefix("jar:") {
        Path::new(j)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| j.to_string())
    } else {
        id.to_string()
    }
}

fn short_num(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.)
    } else if n >= 1_000 {
        format!("{:.0}K", n as f64 / 1_000.)
    } else {
        n.to_string()
    }
}

fn tab_icon(p: &Painter, tab: Tab, c: Pos2, col: Color32) {
    let s = Stroke::new(1.8, col);
    match tab {
        Tab::Home => {
            p.line_segment([pos2(c.x - 9., c.y + 2.), pos2(c.x, c.y - 7.)], s);
            p.line_segment([pos2(c.x, c.y - 7.), pos2(c.x + 9., c.y + 2.)], s);
            p.rect_stroke(
                Rect::from_center_size(pos2(c.x, c.y + 3.), vec2(12., 10.)),
                0.,
                s,
            );
        }
        Tab::Builds => {
            for (dx, dy) in [(-6., -6.), (6., -6.), (-6., 6.), (6., 6.)] {
                p.rect_stroke(
                    Rect::from_center_size(pos2(c.x + dx, c.y + dy), vec2(8., 8.)),
                    0.,
                    s,
                );
            }
        }
        Tab::Mods => {
            p.circle_stroke(c, 7., s);
            for a in [0.0f32, 1.57, 3.14, 4.71] {
                p.line_segment(
                    [
                        pos2(c.x + a.cos() * 7., c.y + a.sin() * 7.),
                        pos2(c.x + a.cos() * 10., c.y + a.sin() * 10.),
                    ],
                    s,
                );
            }
        }
        Tab::Settings => {
            p.circle_stroke(c, 8., s);
            p.circle_filled(c, 3., col);
        }
    }
}

fn draw_avatar(p: &Painter, r: Rect, nick: &str, t: f32) {
    p.rect_filled(r, 2., theme::CARD_HI);
    p.rect_stroke(r, 2., Stroke::new(1., theme::BORDER));
    let letter = nick.chars().next().unwrap_or('?').to_uppercase().to_string();
    p.text(
        r.center(),
        Align2::CENTER_CENTER,
        letter,
        FontId::proportional(22.),
        fx::shimmer(t * 0.15),
    );
}

fn path_row(ui: &mut egui::Ui, label: &str, value: &mut String, dir: bool) -> bool {
    let mut changed = false;
    ui.label(muted(label));
    ui.horizontal(|ui| {
        let r = ui.add(TextEdit::singleline(value).desired_width(ui.available_width() - 80.));
        changed |= r.changed();
        if ui.button("ОБЗОР").clicked() {
            let picked = if dir {
                rfd::FileDialog::new().pick_folder()
            } else {
                rfd::FileDialog::new().pick_file()
            };
            if let Some(p) = picked {
                *value = p.to_string_lossy().to_string();
                changed = true;
            }
        }
    });
    changed
}

pub fn add_tile(ui: &mut egui::Ui) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(160., 70.), Sense::click());
    let p = ui.painter();
    let hot = resp.hovered();
    let stroke = if hot { Stroke::new(1., theme::YELLOW) } else { Stroke::new(1., theme::BORDER) };
    p.rect_stroke(r, 2., stroke);
    p.text(r.center(), Align2::CENTER_CENTER, "+ Добавить .jar", FontId::proportional(12.), if hot { theme::YELLOW } else { theme::MUTED });
    resp.on_hover_cursor(CursorIcon::PointingHand)
}
