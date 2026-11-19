//! Server hosting management: Phantom Rust Native, Crossplay Paper (Java + GeyserMC), Bedrock Dedicated.

use crate::{config::{self, Config}, java, launch, theme};
use eframe::egui::{
    self, vec2, Align, Align2, Color32, CursorIcon, FontId, Frame, Layout, Margin,
    Rect, RichText, Rounding, Sense, Stroke, TextEdit,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Write},
    net::UdpSocket,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ServerKind {
    PhantomRust,
    CrossplayPaper,
    Bedrock,
}

impl ServerKind {
    pub fn label(self) -> &'static str {
        match self {
            ServerKind::PhantomRust => "Phantom Native Server (Rust 26.3)",
            ServerKind::CrossplayPaper => "Crossplay Server (Java + Bedrock GeyserMC)",
            ServerKind::Bedrock => "Bedrock Dedicated Server",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            ServerKind::PhantomRust => "RUST NATIVE",
            ServerKind::CrossplayPaper => "JAVA + BEDROCK",
            ServerKind::Bedrock => "BEDROCK",
        }
    }

    pub fn default_port(self) -> u16 {
        match self {
            ServerKind::PhantomRust => 25565,
            ServerKind::CrossplayPaper => 25565,
            ServerKind::Bedrock => 19132,
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            ServerKind::PhantomRust => {
                "Нативное Rust ядро: ~200 МБ ОЗУ, 0 фризов, параллельный взрыв 1000+ TNT"
            }
            ServerKind::CrossplayPaper => {
                "PaperMC + GeyserMC + Floodgate: совместная игра с ПК (Java/Phantom) и телефонов (Bedrock)"
            }
            ServerKind::Bedrock => {
                "Официальный Bedrock Dedicated Server: прямой порт 19132 UDP для Minecraft Bedrock"
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub id: String,
    pub name: String,
    pub kind: ServerKind,
    pub port: u16,
    pub created_at: String,
}

#[derive(Default)]
pub struct ServerProperties {
    pub props: BTreeMap<String, String>,
}

impl ServerProperties {
    pub fn load_or_create(path: &Path, kind: ServerKind, default_port: u16) -> Self {
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(path) {
                let mut map = BTreeMap::new();
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with('#') || line.is_empty() {
                        continue;
                    }
                    if let Some((k, v)) = line.split_once('=') {
                        map.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
                // Ensure whitelist is disabled and offline mode by default
                map.insert("white-list".into(), "false".into());
                map.insert("enforce-whitelist".into(), "false".into());
                map.insert("view-distance".into(), "12".into());
                if !map.contains_key("online-mode") {
                    map.insert("online-mode".into(), "false".into());
                }
                if !map.contains_key("motd") {
                    map.insert("motd".into(), "Phantom Server (RUST NATIVE)".into());
                }
                let sp = ServerProperties { props: map };
                sp.save(path);
                return sp;
            }
        }

        // Default properties
        let mut map = BTreeMap::new();
        map.insert("server-port".into(), default_port.to_string());
        map.insert("white-list".into(), "false".into());
        map.insert("enforce-whitelist".into(), "false".into());
        map.insert("online-mode".into(), "false".into()); // Default false for offline nicknames!
        map.insert("view-distance".into(), "12".into());
        map.insert("pvp".into(), "true".into());
        map.insert("difficulty".into(), "easy".into());
        map.insert("gamemode".into(), "survival".into());
        map.insert("motd".into(), if kind == ServerKind::PhantomRust { "Phantom Server (RUST NATIVE)".into() } else { format!("Phantom Server ({})", kind.badge()) });
        map.insert("max-players".into(), "20".into());
        let sp = ServerProperties { props: map };
        sp.save(path);
        sp
    }

    pub fn save(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut out = String::from("# Minecraft server properties (configured by Phantom Launcher)\n");
        for (k, v) in &self.props {
            out.push_str(&format!("{k}={v}\n"));
        }
        let _ = fs::write(path, out);
    }

    pub fn get_str(&self, key: &str, def: &str) -> String {
        self.props.get(key).cloned().unwrap_or_else(|| def.into())
    }

    pub fn set_str(&mut self, key: &str, val: &str) {
        self.props.insert(key.into(), val.into());
    }

    pub fn get_bool(&self, key: &str, def: bool) -> bool {
        self.props
            .get(key)
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(def)
    }

    pub fn set_bool(&mut self, key: &str, val: bool) {
        self.props.insert(key.into(), if val { "true".into() } else { "false".into() });
    }
}

pub struct ServerProcess {
    pub child: Child,
    pub stdin_tx: Sender<String>,
}

pub struct ServerManager {
    pub servers: Vec<ServerEntry>,
    pub selected_idx: usize,
    pub running_id: Option<String>,
    pub process: Option<ServerProcess>,
    pub console_logs: Arc<Mutex<Vec<String>>>,
    pub cmd_input: String,
    pub status_msg: Option<(bool, String)>,
    pub is_downloading: bool,
    // Sub-tab: 0 = Консоль, 1 = Настройки (server.properties), 2 = Моды и Плагины
    pub sub_tab: usize,
    // Modal for creating server
    pub creating: bool,
    pub new_name: String,
    pub new_kind: ServerKind,
    pub new_port: String,
    pub properties_cache: Option<ServerProperties>,
    // Rx for background tasks (downloading/installing)
    pub bg_rx: Receiver<Result<String, String>>,
    pub bg_tx: Sender<Result<String, String>>,
    // Internet exposure / Tunnel
    pub public_ip: Option<String>,
    pub tunnel_url: Option<String>,
    pub bedrock_tunnel_url: Option<String>,
    pub is_tunneling: bool,
    pub tunnel_proc: Arc<Mutex<Option<Child>>>,
    /// Start again as soon as the background download has finished.
    pub pending_start: bool,
    /// The last Playit address seen, kept between launches.
    pub last_tunnel: Option<String>,
    pub cached_local_ip: String,
}

impl ServerManager {
    pub fn new() -> Self {
        let (bg_tx, bg_rx) = channel();
        let servers = load_servers();
        let cached_local_ip = local_ip();
        let mut sm = ServerManager {
            servers,
            selected_idx: 0,
            running_id: None,
            process: None,
            console_logs: Arc::new(Mutex::new(Vec::new())),
            cmd_input: String::new(),
            status_msg: None,
            is_downloading: false,
            sub_tab: 0,
            creating: false,
            new_name: "Мой сервер".into(),
            new_kind: ServerKind::PhantomRust,
            new_port: "25565".into(),
            properties_cache: None,
            bg_rx,
            bg_tx,
            public_ip: None,
            tunnel_url: None,
            bedrock_tunnel_url: None,
            is_tunneling: false,
            tunnel_proc: Arc::new(Mutex::new(None)),
            pending_start: false,
            last_tunnel: fs::read_to_string(Self::servers_root().join("last_tunnel.txt"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            cached_local_ip,
        };
        sm.reload_properties();
        sm
    }

    pub fn servers_root() -> PathBuf {
        config::data_root().join("phantom").join("servers")
    }

    pub fn server_dir(&self, entry: &ServerEntry) -> PathBuf {
        Self::servers_root().join(&entry.id)
    }

    pub fn current_server(&self) -> Option<&ServerEntry> {
        self.servers.get(self.selected_idx)
    }

    pub fn reload_properties(&mut self) {
        if let Some(srv) = self.current_server().cloned() {
            let prop_path = self.server_dir(&srv).join("server.properties");
            self.properties_cache = Some(ServerProperties::load_or_create(
                &prop_path,
                srv.kind,
                srv.port,
            ));
        } else {
            self.properties_cache = None;
        }
    }

    pub fn save_current_properties(&self) {
        if let (Some(srv), Some(props)) = (self.current_server(), &self.properties_cache) {
            let prop_path = self.server_dir(srv).join("server.properties");
            props.save(&prop_path);
        }
    }

    pub fn poll(&mut self) {
        // Check background tasks
        while let Ok(res) = self.bg_rx.try_recv() {
            match res {
                Ok(msg) => {
                    self.is_tunneling = false;
                    if let Some(rest) = msg.strip_prefix("UPNP_OK:") {
                        self.public_ip = Some(rest.to_string());
                        self.status_msg = Some((true, format!("UPnP открыт: {rest}")));
                    } else if let Some(rest) = msg.strip_prefix("PLAYIT_CLAIM:") {
                        self.tunnel_url = Some(rest.to_string());
                        self.status_msg = Some((true, format!("Ссылка Playit: {rest}")));
                    } else if let Some(rest) = msg.strip_prefix("PLAYIT_OK:") {
                        let (java, bedrock) = if let Some((j, b)) = rest.split_once('|') {
                            (j.trim(), b.trim())
                        } else {
                            (rest.trim(), "")
                        };
                        self.tunnel_url = if java.is_empty() { None } else { Some(java.to_string()) };
                        self.bedrock_tunnel_url = if bedrock.is_empty() { None } else { Some(bedrock.to_string()) };

                        let display = match (self.tunnel_url.as_deref(), self.bedrock_tunnel_url.as_deref()) {
                            (Some(j), Some(b)) => format!("{j} (Java) / {b} (Bedrock)"),
                            (Some(j), None) => j.to_string(),
                            (None, Some(b)) => b.to_string(),
                            _ => "Туннель активен".into(),
                        };
                        self.last_tunnel = self.tunnel_url.clone().or_else(|| self.bedrock_tunnel_url.clone());
                        let _ = fs::write(Self::servers_root().join("last_tunnel.txt"), &display);
                        self.status_msg = Some((true, format!("Playit адрес: {display}")));
                    } else if let Some(rest) = msg.strip_prefix("BORE_OK:") {
                        self.tunnel_url = Some(rest.to_string());
                        self.status_msg = Some((true, format!("Туннель активен: {rest}")));
                    } else {
                        self.is_downloading = false;
                        self.status_msg = Some((true, msg));
                    }
                }
                Err(err) => {
                    self.is_downloading = false;
                    self.pending_start = false;
                    self.is_tunneling = false;
                    self.status_msg = Some((false, err));
                }
            }
        }

        // Check running process
        if let Some(ref mut proc) = self.process {
            if let Ok(Some(exit)) = proc.child.try_wait() {
                let msg = format!("Сервер завершил работу (код {exit})");
                if let Ok(mut logs) = self.console_logs.lock() {
                    logs.push(format!("[Phantom] {msg}"));
                }
                self.process = None;
                self.running_id = None;
                self.stop_tunnel();
            }
        }
    }

    pub fn start_server(&mut self, cfg: &Config) {
        let Some(srv) = self.current_server().cloned() else {
            return;
        };
        self.cached_local_ip = local_ip();
        let srv_dir = self.server_dir(&srv);
        let _ = fs::create_dir_all(&srv_dir);

        // Always write eula.txt = true to prevent stopping due to EULA
        let _ = fs::write(srv_dir.join("eula.txt"), "eula=true\n");

        // Make sure properties are saved with white-list=false, enforce-whitelist=false, online-mode=false, view-distance=12
        if let Some(ref mut props) = self.properties_cache {
            props.set_bool("white-list", false);
            props.set_bool("enforce-whitelist", false);
            props.set_bool("online-mode", false);
            props.set_str("view-distance", "12");
            if !props.props.contains_key("motd") {
                props.set_str("motd", "Phantom Server (RUST NATIVE)");
            }
        }
        self.save_current_properties();

        let srv_id = srv.id.clone();
        let logs_arc = Arc::clone(&self.console_logs);
        if let Ok(mut logs) = logs_arc.lock() {
            logs.clear();
            logs.push(format!("[Phantom] Запуск {}...", srv.name));
        }

        match srv.kind {
            ServerKind::PhantomRust => {
                let client = match launch::find_client(cfg) {
                    Some(c) => c,
                    None => {
                        self.status_msg = Some((
                            false,
                            "phantom-client.exe не найден. Соберите клиент: cargo build --profile fast"
                                .into(),
                        ));
                        return;
                    }
                };
                let res_path = launch::find_resources(
                    client.parent().unwrap_or(Path::new(".")),
                    cfg,
                );

                let mut cmd = Command::new(&client);
                cmd.current_dir(&srv_dir)
                    .arg("--resources")
                    .arg(&res_path)
                    .arg("--dedicated")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                match spawn_process(cmd, logs_arc) {
                    Ok(p) => {
                        self.process = Some(p);
                        self.running_id = Some(srv_id);
                        self.status_msg = Some((true, format!("{} запущен!", srv.name)));
                    }
                    Err(e) => {
                        self.status_msg = Some((false, format!("Ошибка запуска: {e}")));
                    }
                }
            }
            ServerKind::CrossplayPaper => {
                let paper_jar = srv_dir.join("paper.jar");
                let geyser_jar = srv_dir.join("plugins").join("Geyser-Spigot.jar");
                let floodgate_jar = srv_dir.join("plugins").join("Floodgate-Spigot.jar");

                // Check if files exist; if not, download in background first!
                let via_jar = srv_dir.join("plugins").join("ViaVersion.jar");
                if !paper_jar.is_file() || !geyser_jar.is_file() || !floodgate_jar.is_file() || !via_jar.is_file() {
                    self.is_downloading = true;
                    self.status_msg = Some((
                        true,
                        "Скачивание PaperMC + GeyserMC + Floodgate + ViaVersion (первый запуск)...".into(),
                    ));

                    self.pending_start = true;
                    let tx = self.bg_tx.clone();
                    let s_dir = srv_dir.clone();
                    thread::spawn(move || {
                        let res = download_crossplay_files(&s_dir);
                        let _ = tx.send(res);
                    });
                    return;
                }

                // Make sure eula.txt exists
                let _ = fs::write(srv_dir.join("eula.txt"), "eula=true\n");

                // Java must never reach for the network for the core: fetch it ourselves first (rustls).
                if !mojang_ready(&srv_dir) {
                    self.is_downloading = true;
                    self.pending_start = true;
                    self.status_msg = Some((
                        true,
                        "Скачиваю ядро Mojang напрямую, без Java и SSL...".into(),
                    ));
                    let (tx, s_dir) = (self.bg_tx.clone(), srv_dir.clone());
                    thread::spawn(move || {
                        let _ = tx.send(prefetch_mojang(&s_dir));
                    });
                    return;
                }

                // The newest installed Java wins (25 > 21 > 17); no manual path needed.
                let detected = java::detect(cfg.game_dir());
                let javaw = java::pick(&detected, 17, &cfg.java_path)
                    .or_else(|| java::pick(&detected, 9, ""))
                    .unwrap_or_else(|| PathBuf::from("java"));
                let java_bin = java::console(&javaw);

                let mut cmd = Command::new(&java_bin);
                cmd.current_dir(&srv_dir)
                    .arg(format!("-Xmx{}G", cfg.ram_gb))
                    .arg(format!("-Xms{}G", cfg.ram_gb.min(2)))
                    .arg("-jar")
                    .arg("paper.jar")
                    .arg("nogui")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                match spawn_process(cmd, logs_arc) {
                    Ok(p) => {
                        self.process = Some(p);
                        self.running_id = Some(srv_id);
                        self.status_msg = Some((true, format!("{} запущен!", srv.name)));
                    }
                    Err(e) => {
                        self.status_msg = Some((false, format!("Ошибка запуска: {e}")));
                    }
                }
            }
            ServerKind::Bedrock => {
                let exe = srv_dir.join("bedrock_server.exe");
                if !exe.is_file() {
                    self.is_downloading = true;
                    self.status_msg = Some((
                        true,
                        "Скачивание Bedrock Dedicated Server...".into(),
                    ));
                    let tx = self.bg_tx.clone();
                    let s_dir = srv_dir.clone();
                    thread::spawn(move || {
                        let res = download_bedrock_files(&s_dir);
                        let _ = tx.send(res);
                    });
                    return;
                }

                let mut cmd = Command::new(&exe);
                cmd.current_dir(&srv_dir)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                match spawn_process(cmd, logs_arc) {
                    Ok(p) => {
                        self.process = Some(p);
                        self.running_id = Some(srv_id);
                        self.status_msg = Some((true, format!("{} запущен!", srv.name)));
                    }
                    Err(e) => {
                        self.status_msg = Some((false, format!("Ошибка запуска: {e}")));
                    }
                }
            }
        }
    }

    pub fn stop_tunnel(&mut self) {
        self.is_tunneling = false;
        self.public_ip = None;
        self.tunnel_url = None;
        self.bedrock_tunnel_url = None;
        if let Ok(mut tp) = self.tunnel_proc.lock() {
            if let Some(mut p) = tp.take() {
                let _ = p.kill();
            }
        }
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("taskkill");
            crate::java::quiet(&mut cmd);
            let _ = cmd.args(["/F", "/IM", "playit.exe"]).status();
        }
    }

    pub fn stop_server(&mut self) {
        self.stop_tunnel();
        if let Some(mut proc) = self.process.take() {
            let _ = proc.stdin_tx.send("stop\n".into());
            let _ = proc.stdin_tx.send("/stop\n".into());
            if let Ok(mut logs) = self.console_logs.lock() {
                logs.push("[Phantom] Отправлена команда остановки (stop)...".into());
            }
            // Give 2.5 seconds, then kill if still running
            thread::spawn(move || {
                for _ in 0..25 {
                    if let Ok(Some(_)) = proc.child.try_wait() {
                        return;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                let _ = proc.child.kill();
            });
            self.running_id = None;
            self.status_msg = Some((true, "Сервер остановлен".into()));
        }
    }

    pub fn send_command(&mut self) {
        let mut cmd = self.cmd_input.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        if cmd.starts_with('/') {
            cmd = cmd[1..].to_string();
        }
        if let Some(ref proc) = self.process {
            if let Ok(mut logs) = self.console_logs.lock() {
                logs.push(format!("> {cmd}"));
            }
            let _ = proc.stdin_tx.send(format!("{cmd}\n"));
            self.cmd_input.clear();
        }
    }

    pub fn execute_command(&mut self, raw_cmd: &str) {
        let mut cmd = raw_cmd.trim().to_string();
        if cmd.starts_with('/') {
            cmd = cmd[1..].to_string();
        }
        if let Some(ref proc) = self.process {
            if let Ok(mut logs) = self.console_logs.lock() {
                logs.push(format!("> {cmd}"));
            }
            let _ = proc.stdin_tx.send(format!("{cmd}\n"));
        } else {
            self.cmd_input = format!("/{cmd}");
        }
    }

    /// Open port 25565 to the internet via UPnP (igd_next) or Playit.gg tunnel fallback
    pub fn open_to_internet(&mut self, cfg: &Config) {
        let Some(srv) = self.current_server().cloned() else {
            return;
        };
        let port = srv.port;
        self.is_tunneling = true;
        self.status_msg = Some((true, "Открытие доступа: UPnP + Playit.gg туннель...".into()));

        let logs_arc = Arc::clone(&self.console_logs);
        let phantom_dir = PathBuf::from(&cfg.phantom_dir);
        let launcher_dir = phantom_dir.join("launcher_tools");
        let _ = fs::create_dir_all(&launcher_dir);
        let mc_launcher_dir = cfg.game_dir().join("launcher_tools");
        let tx = self.bg_tx.clone();
        let tunnel_proc_arc = Arc::clone(&self.tunnel_proc);

        thread::spawn(move || {
            // 0. Kill any lingering playit.exe process on Windows
            #[cfg(target_os = "windows")]
            {
                let mut cmd = Command::new("taskkill");
                crate::java::quiet(&mut cmd);
                let _ = cmd.args(["/F", "/IM", "playit.exe"]).status();
                thread::sleep(Duration::from_millis(200));
            }

            // 1. Try UPnP with igd_next
            let upnp_res = match igd_next::search_gateway(Default::default()) {
                Ok(gateway) => {
                    let local_addr = local_ip();
                    if let Ok(ip) = local_addr.parse() {
                        let socket_addr = std::net::SocketAddr::new(ip, port);
                        match gateway.add_port(
                            igd_next::PortMappingProtocol::TCP,
                            port,
                            socket_addr,
                            3600,
                            "Phantom Server",
                        ) {
                            Ok(()) => {
                                let ext_ip = gateway
                                    .get_external_ip()
                                    .map(|ip| ip.to_string())
                                    .unwrap_or_else(|_| "Не удалось получить IP".into());
                                if is_private_or_cgnat(&ext_ip) {
                                    Err(format!("Двойной NAT (внешний IP роутера {ext_ip} серый/локальный)"))
                                } else {
                                    Ok(format!("UPNP_OK:{ext_ip}:{port}"))
                                }
                            }
                            Err(e) => Err(format!("UPnP ошибка: {e}")),
                        }
                    } else {
                        Err("Неверный локальный IP".into())
                    }
                }
                Err(e) => Err(format!("Шлюз UPnP не найден: {e}")),
            };

            if let Ok(ref msg) = upnp_res {
                let _ = tx.send(Ok(msg.clone()));
                if let Ok(mut logs) = logs_arc.lock() {
                    logs.push(format!(
                        "[Phantom UPnP] Порт {port} успешно открыт на роутере! Внешний адрес: {msg}"
                    ));
                }
                return;
            }

            // 2. UPnP failed or behind double NAT -> Playit.gg tunnel agent
            if let Ok(mut logs) = logs_arc.lock() {
                logs.push(
                    "[Phantom] Двойной NAT / UPnP закрыт. Запуск Playit.gg агента (поддержка Java + Bedrock UDP)..."
                        .into(),
                );
            }

            // Find playit.exe in known paths
            let mut playit_path = launcher_dir.join("playit.exe");
            if !playit_path.is_file() {
                if mc_launcher_dir.join("playit.exe").is_file() {
                    playit_path = mc_launcher_dir.join("playit.exe");
                } else if let Ok(current_exe) = std::env::current_exe() {
                    if let Some(parent) = current_exe.parent() {
                        let candidate = parent.join("playit.exe");
                        if candidate.is_file() {
                            playit_path = candidate;
                        }
                    }
                }
            }

            if !playit_path.is_file() {
                if let Ok(mut logs) = logs_arc.lock() {
                    logs.push("[Phantom] Скачивание официального playit.exe...".into());
                }
                let dl_url = "https://github.com/playit-cloud/playit-agent/releases/latest/download/playit-windows-x86_64.exe";
                let agent = ureq::AgentBuilder::new()
                    .timeout(Duration::from_secs(60))
                    .build();
                let dl_ok = agent
                    .get(dl_url)
                    .call()
                    .ok()
                    .and_then(|resp| {
                        let mut file = fs::File::create(&playit_path).ok()?;
                        std::io::copy(&mut resp.into_reader(), &mut file).ok()
                    })
                    .is_some();
                if !dl_ok {
                    let ps_cmd = format!(
                        "Invoke-WebRequest -Uri '{dl_url}' -OutFile '{}'",
                        playit_path.display()
                    );
                    let _ = Command::new("powershell")
                        .args(["-NoProfile", "-Command", &ps_cmd])
                        .status();
                }
            }

            if !playit_path.is_file() {
                let _ = tx.send(Err("Не удалось найти или скачать playit.exe".into()));
                return;
            }

            let work_dir = playit_path.parent().unwrap_or(&launcher_dir);
            let mut cmd = Command::new(&playit_path);
            crate::java::quiet(&mut cmd);
            cmd.current_dir(work_dir)
                .arg("--stdout")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(Err(format!("Ошибка запуска playit.exe: {e}")));
                    return;
                }
            };

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            if let Ok(mut tp) = tunnel_proc_arc.lock() {
                *tp = Some(child);
            }

            let (event_tx, event_rx) = channel::<PlayitEvent>();

            if let Some(stdout) = stdout {
                let event_tx_c = event_tx.clone();
                let logs_c = Arc::clone(&logs_arc);
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().flatten() {
                        let clean = strip_ansi(&line);
                        if let Ok(mut l) = logs_c.lock() {
                            if l.len() > 1000 {
                                l.remove(0);
                            }
                            l.push(format!("[Playit] {clean}"));
                        }
                        if clean.contains("playit.gg/claim/") {
                            if let Some(pos) = clean.find("https://playit.gg/claim/") {
                                let claim_url = clean[pos..]
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string();
                                let _ = event_tx_c.send(PlayitEvent::Claim(claim_url));
                            }
                        } else if clean.contains("playit connected")
                            || clean.contains("tunnels loaded")
                            || clean.contains("tunnel_count")
                        {
                            let _ = event_tx_c.send(PlayitEvent::Connected);
                        } else if clean.contains(".ply.gg")
                            || clean.contains(".playit.gg")
                            || clean.contains(".joinmc.link")
                        {
                            for word in clean.split_whitespace() {
                                if word.contains(".ply.gg")
                                    || word.contains(".playit.gg")
                                    || word.contains(".joinmc.link")
                                {
                                    let clean_word = word
                                        .trim_matches(|c: char| {
                                             !c.is_alphanumeric() && c != '.' && c != ':' && c != '-'
                                        })
                                        .to_string();
                                    let _ = event_tx_c.send(PlayitEvent::DirectAddress(clean_word));
                                }
                            }
                        }
                    }
                });
            }

            if let Some(stderr) = stderr {
                let logs_c = Arc::clone(&logs_arc);
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().flatten() {
                        let clean = strip_ansi(&line);
                        if let Ok(mut l) = logs_c.lock() {
                            if l.len() > 1000 {
                                l.remove(0);
                            }
                            l.push(format!("[Playit ERR] {clean}"));
                        }
                    }
                });
            }

            // Fast check: if secret key is configured and tunnels are already present, resolve immediately!
            if let Some(ref key) = find_playit_secret_key(work_dir) {
                if let Ok(mut logs) = logs_arc.lock() {
                    logs.push("[Playit] Проверка активных туннелей в API Playit.gg...".into());
                }
                if let Some((java_addr, bedrock_addr)) = fetch_playit_tunnels(key) {
                    let j = java_addr.unwrap_or_default();
                    let b = bedrock_addr.unwrap_or_default();
                    if !j.is_empty() || !b.is_empty() {
                        let _ = tx.send(Ok(format!("PLAYIT_OK:{j}|{b}")));
                        if let Ok(mut logs) = logs_arc.lock() {
                            logs.push(format!("[Playit] Туннели активны! Java: {j}, Bedrock: {b}"));
                        }
                        return;
                    }
                }
            }

            let start_time = std::time::Instant::now();
            let mut last_api_check = std::time::Instant::now();
            let mut resolved = false;

            while start_time.elapsed() < Duration::from_secs(20) {
                if let Ok(mut tp) = tunnel_proc_arc.lock() {
                    if let Some(ref mut c) = *tp {
                        if let Ok(Some(status)) = c.try_wait() {
                            let _ = tx.send(Err(format!("playit.exe завершился (код {status})")));
                            return;
                        }
                    }
                }

                if let Ok(event) = event_rx.recv_timeout(Duration::from_millis(300)) {
                    match event {
                        PlayitEvent::Claim(url) => {
                            let _ = tx.send(Ok(format!("PLAYIT_CLAIM:{url}")));
                            resolved = true;
                            break;
                        }
                        PlayitEvent::Connected => {
                            if let Some(ref key) = find_playit_secret_key(work_dir) {
                                if let Some((java_addr, bedrock_addr)) = fetch_playit_tunnels(key) {
                                    let j = java_addr.unwrap_or_default();
                                    let b = bedrock_addr.unwrap_or_default();
                                    if !j.is_empty() || !b.is_empty() {
                                        let _ = tx.send(Ok(format!("PLAYIT_OK:{j}|{b}")));
                                        resolved = true;
                                        break;
                                    }
                                }
                            }
                        }
                        PlayitEvent::DirectAddress(addr) => {
                            let _ = tx.send(Ok(format!("PLAYIT_OK:{addr}|")));
                            resolved = true;
                            break;
                        }
                    }
                }

                // Check API periodically no more than once per 3 seconds if not yet resolved
                if !resolved && last_api_check.elapsed() >= Duration::from_secs(3) {
                    last_api_check = std::time::Instant::now();
                    if let Some(ref key) = find_playit_secret_key(work_dir) {
                        if let Some((java_addr, bedrock_addr)) = fetch_playit_tunnels(key) {
                            let j = java_addr.unwrap_or_default();
                            let b = bedrock_addr.unwrap_or_default();
                            if !j.is_empty() || !b.is_empty() {
                                let _ = tx.send(Ok(format!("PLAYIT_OK:{j}|{b}")));
                                resolved = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !resolved {
                if let Some(ref key) = find_playit_secret_key(work_dir) {
                    if let Some((java_addr, bedrock_addr)) = fetch_playit_tunnels(key) {
                        let j = java_addr.unwrap_or_default();
                        let b = bedrock_addr.unwrap_or_default();
                        if !j.is_empty() || !b.is_empty() {
                            let _ = tx.send(Ok(format!("PLAYIT_OK:{j}|{b}")));
                            return;
                        }
                    }
                }
                let _ = tx.send(Err("Таймаут подключения к Playit.gg. Проверьте интернет.".into()));
            }
        });
    }
}

fn spawn_process(
    mut cmd: Command,
    logs: Arc<Mutex<Vec<String>>>,
) -> Result<ServerProcess, String> {
    crate::java::quiet(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("{e}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdin = child.stdin.take();

    if let Some(stdout) = stdout {
        let logs_c = Arc::clone(&logs);
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                if let Ok(mut l) = logs_c.lock() {
                    if l.len() > 1000 {
                        l.remove(0);
                    }
                    l.push(line);
                }
            }
        });
    }

    if let Some(stderr) = stderr {
        let logs_c = Arc::clone(&logs);
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                if let Ok(mut l) = logs_c.lock() {
                    if l.len() > 1000 {
                        l.remove(0);
                    }
                    l.push(line);
                }
            }
        });
    }

    let (stdin_tx, stdin_rx) = channel::<String>();
    if let Some(mut stdin) = stdin {
        thread::spawn(move || {
            while let Ok(cmd) = stdin_rx.recv() {
                let line = if cmd.ends_with('\n') {
                    cmd
                } else {
                    format!("{cmd}\n")
                };
                if stdin.write_all(line.as_bytes()).is_err() {
                    break;
                }
                let _ = stdin.flush();
            }
        });
    }

    Ok(ServerProcess { child, stdin_tx })
}

fn load_servers() -> Vec<ServerEntry> {
    let path = ServerManager::servers_root().join("servers.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(list) = serde_json::from_str::<Vec<ServerEntry>>(&content) {
            if !list.is_empty() {
                return list;
            }
        }
    }

    // Default template server
    let s = ServerEntry {
        id: "phantom-rust-main".into(),
        name: "Phantom Rust Server (LAN)".into(),
        kind: ServerKind::PhantomRust,
        port: 25565,
        created_at: "2026-11-13".into(),
    };
    let list = vec![s];
    save_servers(&list);
    list
}

pub fn save_servers(list: &[ServerEntry]) {
    let root = ServerManager::servers_root();
    let _ = fs::create_dir_all(&root);
    let path = root.join("servers.json");
    if let Ok(text) = serde_json::to_string_pretty(list) {
        let _ = fs::write(path, text);
    }
}

pub fn local_ip() -> String {
    // 1. Try ipconfig on Windows to find the real physical adapter with gateway
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("ipconfig");
        crate::java::quiet(&mut cmd);
        if let Ok(output) = cmd.output() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut candidates = Vec::new();
            for line in text.lines() {
                let trimmed = line.trim();
                // Check English and Russian ipconfig patterns: "IPv4 Address", "IPv4-адрес"
                if trimmed.contains("IPv4") {
                    if let Some((_, ip_part)) = trimmed.split_once(':') {
                        let ip = ip_part.trim();
                        if let Ok(parsed) = ip.parse::<std::net::Ipv4Addr>() {
                            let octets = parsed.octets();
                            // Reject loopback (127.*), link-local (169.254.*), Meta/Clash TUN (198.18.* / 198.19.*), unspecified (0.0.0.0)
                            if octets[0] == 127
                                || (octets[0] == 169 && octets[1] == 254)
                                || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                                || octets == [0, 0, 0, 0]
                            {
                                continue;
                            }
                            // Prioritize standard RFC 1918 private subnets: 192.168.x.x, 10.x.x.x, 172.16-31.x.x
                            let is_prio = (octets[0] == 192 && octets[1] == 168)
                                || octets[0] == 10
                                || (octets[0] == 172 && (16..=31).contains(&octets[1]));
                            if is_prio {
                                return ip.to_string();
                            }
                            candidates.push(ip.to_string());
                        }
                    }
                }
            }
            if let Some(first) = candidates.first() {
                return first.clone();
            }
        }
    }

    // 2. Fallback: UDP connect to router/DNS
    for probe_target in ["192.168.1.1:80", "192.168.0.1:80", "10.0.0.1:80", "8.8.8.8:80"] {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(probe_target).is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    let ip = addr.ip().to_string();
                    if !ip.starts_with("198.18.")
                        && !ip.starts_with("198.19.")
                        && !ip.starts_with("169.254.")
                        && !ip.starts_with("127.")
                        && ip != "0.0.0.0"
                    {
                        return ip;
                    }
                }
            }
        }
    }

    "127.0.0.1".to_string()
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_esc = true;
        } else if in_esc {
            if c.is_ascii_alphabetic() {
                in_esc = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn is_private_or_cgnat(ip_str: &str) -> bool {
    if let Ok(ip) = ip_str.parse::<std::net::Ipv4Addr>() {
        let octets = ip.octets();
        if octets[0] == 10 {
            return true;
        }
        if octets[0] == 172 && (16..=31).contains(&octets[1]) {
            return true;
        }
        if octets[0] == 192 && octets[1] == 168 {
            return true;
        }
        if octets[0] == 100 && (64..=127).contains(&octets[1]) {
            return true;
        }
        if octets[0] == 198 && (18..=19).contains(&octets[1]) {
            return true;
        }
        if octets[0] == 169 && octets[1] == 254 {
            return true;
        }
        if octets[0] == 127 {
            return true;
        }
    }
    false
}

fn find_playit_secret_key(launcher_dir: &Path) -> Option<String> {
    let mut candidate_paths = Vec::new();
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        candidate_paths.push(PathBuf::from(local_app_data).join("playit_gg").join("playit.toml"));
    }
    if let Ok(app_data) = std::env::var("APPDATA") {
        candidate_paths.push(PathBuf::from(&app_data).join("playit_gg").join("playit.toml"));
        candidate_paths.push(PathBuf::from(&app_data).join("playit").join("playit.toml"));
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        candidate_paths.push(PathBuf::from(&userprofile).join(".config").join("playit").join("playit.toml"));
        candidate_paths.push(PathBuf::from(&userprofile).join(".playit").join("playit.toml"));
    }
    candidate_paths.push(launcher_dir.join("playit.toml"));

    for path in candidate_paths {
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("secret_key") {
                        if let Some((_, val)) = line.split_once('=') {
                            let clean = val.trim().trim_matches('"').trim_matches('\'').trim();
                            if !clean.is_empty() {
                                return Some(clean.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn fetch_playit_tunnels(secret_key: &str) -> Option<(Option<String>, Option<String>)> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .build();
    let resp = agent
        .post("https://api.playit.gg/v1/tunnels/list")
        .set("Authorization", &format!("Agent-Key {}", secret_key.trim()))
        .set("Content-Type", "application/json")
        .send_bytes(b"{}")
        .ok()?;
    let json: serde_json::Value = resp.into_json().ok()?;
    let tunnels = json.get("data")?.get("tunnels")?.as_array()?;

    let mut java_addr = None;
    let mut bedrock_addr = None;
    for tun in tunnels {
        let t_type = tun.get("tunnel_type").and_then(|v| v.as_str()).unwrap_or("");
        
        // 1. Prioritize public_allocations -> PortAllocation (contains exact auto_domain + external port)
        let mut full_addr = None;
        if let Some(allocs) = tun.get("public_allocations").and_then(|a| a.as_array()) {
            for alloc in allocs {
                if let Some(details) = alloc.get("details") {
                    let domain = details.get("auto_domain").and_then(|s| s.as_str());
                    let port = details.get("port").and_then(|p| p.as_u64());
                    if let (Some(d), Some(p)) = (domain, port) {
                        full_addr = Some(format!("{d}:{p}"));
                        break;
                    }
                }
            }
        }

        // 2. Fallback to connect_addresses
        if full_addr.is_none() {
            if let Some(conns) = tun.get("connect_addresses").and_then(|a| a.as_array()) {
                for conn in conns {
                    if let Some(addr) = conn.get("value").and_then(|v| v.get("address")).and_then(|s| s.as_str()) {
                        full_addr = Some(addr.to_string());
                        break;
                    }
                }
            }
        }

        if let Some(a) = full_addr {
            if t_type.contains("bedrock") {
                bedrock_addr = Some(a);
            } else if t_type.contains("java") {
                java_addr = Some(a);
            } else if java_addr.is_none() {
                java_addr = Some(a);
            }
        }
    }
    Some((java_addr, bedrock_addr))
}

enum PlayitEvent {
    Claim(String),
    Connected,
    DirectAddress(String),
}

/// The vanilla core Paperclip wants in `cache/`: its URL and file name, read from the server jar's
/// `META-INF/download-context` (a line with the hash, the Mojang URL and the file name).
fn mojang_target(dir: &Path) -> Option<(String, PathBuf)> {
    let file = fs::File::open(dir.join("paper.jar")).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    let mut text = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("META-INF/download-context").ok()?, &mut text).ok()?;
    let line = text.lines().find(|l| !l.trim().is_empty())?;
    let (mut url, mut name) = (None, None);
    for tok in line.split_whitespace() {
        if tok.starts_with("http") {
            url = Some(tok.to_string());
        } else if tok.ends_with(".jar") {
            name = Some(tok.to_string());
        }
    }
    let name = Path::new(&name?).file_name()?.to_string_lossy().to_string();
    Some((url?, dir.join("cache").join(name)))
}

/// True when there is nothing to fetch (no context in the jar, or the core is already in cache/).
pub fn mojang_ready(dir: &Path) -> bool {
    match mojang_target(dir) {
        Some((_, path)) => path.is_file(),
        None => true,
    }
}

/// Downloads the Mojang core into `cache/` with Rust's TLS, so Java never does it (no SSLHandshakeException
/// behind a VPN or zapret). Paperclip checks the file's hash itself.
pub fn prefetch_mojang(dir: &Path) -> Result<String, String> {
    let Some((url, target)) = mojang_target(dir) else {
        return Ok("Ядро Mojang не требуется".into());
    };
    let name = target.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    if target.is_file() {
        return Ok(format!("Ядро {name} уже есть"));
    }
    fs::create_dir_all(target.parent().unwrap_or(dir)).map_err(|e| format!("cache: {e}"))?;
    let agent = ureq::AgentBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) PhantomLauncher/1.0")
        .timeout(Duration::from_secs(600))
        .build();
    let mut reader = agent
        .get(&url)
        .call()
        .map_err(|e| format!("Mojang: {e}"))?
        .into_reader();
    let part = target.with_extension("jar.part");
    let mut out = fs::File::create(&part).map_err(|e| format!("{}: {e}", part.display()))?;
    std::io::copy(&mut reader, &mut out).map_err(|e| format!("Скачивание: {e}"))?;
    drop(out);
    fs::rename(&part, &target).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(format!("Ядро Mojang скачано: {name}"))
}

// ---------------------------------------------------------------- Status cards

fn status_card(ui: &mut egui::Ui, t: f32, title: &str, state: &str, col: Color32) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 64.), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, 2., theme::CARD);
    p.rect_stroke(r, 2., Stroke::new(1., col.gamma_multiply(0.7)));
    p.text(r.left_top() + vec2(16., 10.), Align2::LEFT_TOP, title, FontId::proportional(14.), theme::MUTED);
    let c = r.left_top() + vec2(24., 44.);
    let pulse = if col == theme::YELLOW { 0.6 + 0.4 * (t * 4.).sin().abs() } else { 1. };
    p.circle_filled(c, 6., col.gamma_multiply(pulse));
    p.text(c + vec2(16., 0.), Align2::LEFT_CENTER, state, FontId::proportional(17.), col);
}

/// A big card that copies `addr` when clicked; `hint` shows while there is no address yet.
fn copy_card(ui: &mut egui::Ui, title: &str, addr: Option<&str>, hint: &str) -> bool {
    let live = addr.is_some();
    let (r, resp) = ui.allocate_exact_size(
        vec2(ui.available_width(), 64.),
        if live { Sense::click() } else { Sense::hover() },
    );
    let hot = live && resp.hovered();
    let p = ui.painter();
    p.rect_filled(r, 2., if hot { theme::CARD_HI } else { theme::CARD });
    p.rect_stroke(r, 2., Stroke::new(1., if hot { theme::YELLOW } else { theme::BORDER }));
    p.text(r.left_top() + vec2(16., 10.), Align2::LEFT_TOP, title, FontId::proportional(14.), theme::MUTED);
    let (txt, col) = match addr {
        Some(a) => (a.to_string(), theme::TEXT),
        None => (hint.to_string(), theme::MUTED),
    };
    p.text(r.left_top() + vec2(16., 32.), Align2::LEFT_TOP, txt, FontId::proportional(17.), col);
    if live {
        p.text(r.right_center() - vec2(16., 0.), Align2::RIGHT_CENTER, "КОПИРОВАТЬ", FontId::proportional(14.), theme::YELLOW);
    }
    live && resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
}

// ---------------------------------------------------------------- Downloading

fn download_crossplay_files(dir: &Path) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|e| format!("{e}"))?;
    let plugins = dir.join("plugins");
    fs::create_dir_all(&plugins).map_err(|e| format!("{e}"))?;

    let agent = ureq::AgentBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) PhantomLauncher/1.0")
        .timeout(Duration::from_secs(60))
        .build();

    // 1. Geyser Spigot
    let geyser_path = plugins.join("Geyser-Spigot.jar");
    if !geyser_path.is_file() {
        let url = "https://download.geysermc.org/v2/projects/geyser/versions/latest/downloads/spigot";
        if let Ok(resp) = agent.get(url).call() {
            if let Ok(mut out) = fs::File::create(&geyser_path) {
                let _ = std::io::copy(&mut resp.into_reader(), &mut out);
            }
        }
    }

    // 2. Floodgate Spigot
    let floodgate_path = plugins.join("Floodgate-Spigot.jar");
    if !floodgate_path.is_file() {
        let url = "https://download.geysermc.org/v2/projects/floodgate/versions/latest/downloads/spigot";
        if let Ok(resp) = agent.get(url).call() {
            if let Ok(mut out) = fs::File::create(&floodgate_path) {
                let _ = std::io::copy(&mut resp.into_reader(), &mut out);
            }
        }
    }

    // 3. ViaVersion (Modrinth API: projects/viaversion/version -> files[0].url)
    let via_path = plugins.join("ViaVersion.jar");
    if !via_path.is_file() {
        let mut via_downloaded = false;
        let modrinth_api = "https://api.modrinth.com/v2/project/viaversion/version";
        if let Ok(resp) = agent.get(modrinth_api).call() {
            if let Ok(versions) = resp.into_json::<serde_json::Value>() {
                if let Some(arr) = versions.as_array() {
                    if let Some(first_ver) = arr.first() {
                        if let Some(files) = first_ver["files"].as_array() {
                            if let Some(first_file) = files.first() {
                                if let Some(dl_url) = first_file["url"].as_str() {
                                    if let Ok(dl_resp) = agent.get(dl_url).call() {
                                        if let Ok(mut out) = fs::File::create(&via_path) {
                                            let _ = std::io::copy(&mut dl_resp.into_reader(), &mut out);
                                            via_downloaded = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if !via_downloaded {
            let fallback_url = "https://hangarcdn.papermc.io/plugins/ViaVersion/ViaVersion/versions/5.2.1/PAPER/ViaVersion-5.2.1.jar";
            if let Ok(resp) = agent.get(fallback_url).call() {
                if let Ok(mut out) = fs::File::create(&via_path) {
                    let _ = std::io::copy(&mut resp.into_reader(), &mut out);
                }
            }
        }
    }

    // Configure Geyser auth-type: floodgate
    let geyser_cfg_dir = plugins.join("Geyser-Spigot");
    let _ = fs::create_dir_all(&geyser_cfg_dir);
    let geyser_cfg_file = geyser_cfg_dir.join("config.yml");
    if geyser_cfg_file.is_file() {
        if let Ok(cfg_txt) = fs::read_to_string(&geyser_cfg_file) {
            let new_txt = cfg_txt.replace("auth-type: online", "auth-type: floodgate")
                .replace("auth-type: offline", "auth-type: floodgate");
            let _ = fs::write(&geyser_cfg_file, new_txt);
        }
    } else {
        let default_geyser = "remote:\n  auth-type: floodgate\n";
        let _ = fs::write(&geyser_cfg_file, default_geyser);
    }

    // 4. Server Core (Purpur with fallback to Paper v3 / direct)
    let paper_path = dir.join("paper.jar");
    if !paper_path.is_file() {
        let mut server_downloaded = false;

        // Try Purpur API
        let purpur_api = "https://api.purpurmc.org/v2/purpur";
        if let Ok(resp) = agent.get(purpur_api).call() {
            if let Ok(v) = resp.into_json::<serde_json::Value>() {
                if let Some(versions) = v["versions"].as_array() {
                    if let Some(latest) = versions.last().and_then(|val| val.as_str()) {
                        let purpur_dl = format!("https://api.purpurmc.org/v2/purpur/{latest}/latest/download");
                        if let Ok(dl_resp) = agent.get(&purpur_dl).call() {
                            if let Ok(mut out) = fs::File::create(&paper_path) {
                                if std::io::copy(&mut dl_resp.into_reader(), &mut out).is_ok() {
                                    server_downloaded = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: Paper v3 API or direct download
        if !server_downloaded {
            let paper_v3_api = "https://fill.papermc.io/v3/projects/paper";
            if let Ok(resp) = agent.get(paper_v3_api).call() {
                if let Ok(v) = resp.into_json::<serde_json::Value>() {
                    if let Some(versions) = v["versions"].as_array() {
                        if let Some(latest) = versions.last().and_then(|val| val.as_str()) {
                            let paper_v3_dl = format!("https://fill.papermc.io/v3/projects/paper/versions/{latest}/builds/latest/download");
                            if let Ok(dl_resp) = agent.get(&paper_v3_dl).call() {
                                if let Ok(mut out) = fs::File::create(&paper_path) {
                                    if std::io::copy(&mut dl_resp.into_reader(), &mut out).is_ok() {
                                        server_downloaded = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Secondary fallback
        if !server_downloaded {
            let direct_purpur = "https://api.purpurmc.org/v2/purpur/1.21.4/latest/download";
            if let Ok(dl_resp) = agent.get(direct_purpur).call() {
                if let Ok(mut out) = fs::File::create(&paper_path) {
                    let _ = std::io::copy(&mut dl_resp.into_reader(), &mut out);
                }
            }
        }
    }

    let _ = fs::write(dir.join("eula.txt"), "eula=true\n");
    Ok("PaperMC + GeyserMC + Floodgate успешно установлены!".into())
}

fn download_bedrock_files(dir: &Path) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|e| format!("{e}"))?;
    let zip_path = dir.join("bedrock_server.zip");
    let url = "https://minecraft.azureedge.net/bin-win/bedrock-server-1.21.51.01.zip";

    let agent = ureq::AgentBuilder::new()
        .user_agent("Mozilla/5.0")
        .timeout(Duration::from_secs(120))
        .build();

    let mut r = agent.get(url).call().map_err(|e| format!("Bedrock: {e}"))?.into_reader();
    let mut out = fs::File::create(&zip_path).map_err(|e| format!("{e}"))?;
    std::io::copy(&mut r, &mut out).map_err(|e| format!("{e}"))?;
    drop(out);

    // Unpack with PowerShell Expand-Archive
    let cmd = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        zip_path.display(),
        dir.display()
    );
    let _ = Command::new("powershell").args(["-Command", &cmd]).status();
    let _ = fs::remove_file(&zip_path);

    Ok("Bedrock Dedicated Server успешно распакован!".into())
}

// ---------------------------------------------------------------- UI Rendering

pub fn render_server_tab(
    sm: &mut ServerManager,
    cfg: &Config,
    ui: &mut egui::Ui,
    t: f32,
) {
    sm.poll();
    if sm.pending_start && !sm.is_downloading {
        sm.pending_start = false;
        sm.start_server(cfg);
    }

    egui::ScrollArea::vertical()
        .id_source("server_tab_main_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(14.0);
            crate::app::heading(ui, t, "ХОСТИНГ И СЕРВЕРЫ", "Запуск локального сервера для игры с друзьями");

            let is_running = sm.process.is_some();
            let running_srv_id = sm.running_id.clone();

    // Top action bar: Server selector, Add button, Status badge, IP, Folder, Start/Stop button
    Frame::none()
        .fill(theme::CARD)
        .stroke(Stroke::new(1., theme::BORDER))
        .rounding(Rounding::same(4.))
        .inner_margin(Margin::same(12.))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("СЕРВЕР:").color(theme::MUTED).strong());

                let cur_label = sm
                    .current_server()
                    .map(|s| format!("{} [{}]", s.name, s.kind.badge()))
                    .unwrap_or_else(|| "Нет серверов".into());

                let mut chosen_idx = None;
                egui::ComboBox::from_id_source("server_select")
                    .selected_text(RichText::new(cur_label).color(theme::TEXT).strong())
                    .width(200.)
                    .show_ui(ui, |ui| {
                        for (i, s) in sm.servers.iter().enumerate() {
                            let text = format!("{} [{}]", s.name, s.kind.badge());
                            if ui
                                .selectable_label(sm.selected_idx == i, text)
                                .clicked()
                            {
                                chosen_idx = Some(i);
                            }
                        }
                    });
                if let Some(idx) = chosen_idx {
                    if sm.selected_idx != idx {
                        sm.selected_idx = idx;
                        sm.reload_properties();
                        if sm.running_id.as_deref() != sm.current_server().map(|s| s.id.as_str()) {
                            sm.public_ip = None;
                            sm.tunnel_url = None;
                            sm.bedrock_tunnel_url = None;
                            sm.is_tunneling = false;
                        }
                    }
                }

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("+ СОЗДАТЬ СЕРВЕР")
                                .size(13.)
                                .color(theme::YELLOW)
                                .strong(),
                        )
                        .fill(theme::SIDE)
                        .stroke(Stroke::new(1., theme::BORDER)),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    sm.creating = true;
                }

                if let Some(srv) = sm.current_server().cloned() {
                    // Status badge
                    let (status_text, status_col) = if sm.is_downloading {
                        ("⏳ СКАЧИВАНИЕ...", theme::YELLOW)
                    } else if is_running && running_srv_id.as_deref() == Some(&srv.id) {
                        ("● ОНЛАЙН", Color32::from_rgb(50, 220, 100))
                    } else {
                        ("● ОФЛАЙН", theme::MUTED)
                    };
                    ui.add_space(4.);
                    ui.label(RichText::new(status_text).color(status_col).strong().size(13.));

                    // IP copy button: [📋 IP: {ip}:{port}]
                    let ip = &sm.cached_local_ip;
                    let full_addr = format!("{ip}:{}", srv.port);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("📋 IP: {full_addr}"))
                                    .size(13.)
                                    .color(theme::TEXT),
                            )
                            .fill(theme::SIDE)
                            .stroke(Stroke::new(1., theme::BORDER)),
                        )
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text("Нажмите, чтобы скопировать адрес для друзей")
                        .clicked()
                    {
                        ui.ctx().output_mut(|o| o.copied_text = full_addr);
                        sm.status_msg = Some((true, "Адрес сервера скопирован в буфер обмена!".into()));
                    }

                    // Folder button: [📁 Папка]
                    let srv_dir = sm.server_dir(&srv);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("📁 Папка")
                                    .size(13.)
                                    .color(theme::TEXT),
                            )
                            .fill(theme::SIDE)
                            .stroke(Stroke::new(1., theme::BORDER)),
                        )
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        let _ = fs::create_dir_all(&srv_dir);
                        let _ = Command::new("explorer").arg(&srv_dir).spawn();
                    }

                    // Internet / Tunnel exposure button: [🌐 ОТКРЫТЬ В ИНТЕРНЕТ]
                    let internet_label = if let Some(ref url) = sm.tunnel_url {
                        if url.contains("claim") {
                            "🌐 АКТИВАЦИЯ PLAYIT".into()
                        } else {
                            format!("🌐 Playit: {url}")
                        }
                    } else if let Some(ref b_url) = sm.bedrock_tunnel_url {
                        format!("🌐 Playit: {b_url}")
                    } else if let Some(ref pub_ip) = sm.public_ip {
                        format!("🌐 UPnP: {pub_ip}")
                    } else if sm.is_tunneling {
                        "🌐 ПОДКЛЮЧЕНИЕ...".into()
                    } else {
                        "🌐 ОТКРЫТЬ В ИНТЕРНЕТ".into()
                    };

                    let has_tunnel = sm.tunnel_url.is_some() || sm.bedrock_tunnel_url.is_some() || sm.public_ip.is_some();
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(&internet_label)
                                    .size(13.)
                                    .color(if has_tunnel {
                                        Color32::from_rgb(0, 220, 255)
                                    } else {
                                        theme::YELLOW
                                    })
                                    .strong(),
                            )
                            .fill(theme::SIDE)
                            .stroke(Stroke::new(1., theme::BORDER)),
                        )
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text("Автоматически открывает порт через UPnP или запускает Playit.gg туннель (Java + Bedrock UDP) без Radmin VPN!")
                        .clicked()
                    {
                        if let Some(ref addr) = sm.tunnel_url.clone().or_else(|| sm.bedrock_tunnel_url.clone()).or_else(|| sm.public_ip.clone()) {
                            ui.ctx().output_mut(|o| o.copied_text = addr.clone());
                            sm.status_msg = Some((true, format!("Адрес {addr} скопирован в буфер для друзей!")));
                        } else if !sm.is_tunneling {
                            sm.open_to_internet(cfg);
                        }
                    }

                    // Big Action Button (Start / Stop)
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if is_running && running_srv_id.as_deref() == Some(&srv.id) {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("■ ОСТАНОВИТЬ")
                                            .size(13.)
                                            .color(Color32::WHITE)
                                            .strong(),
                                    )
                                    .fill(theme::DANGER)
                                    .rounding(Rounding::same(2.)),
                                )
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                sm.stop_server();
                            }
                        } else {
                            let btn_text = if sm.is_downloading {
                                "⏳ СКАЧИВАНИЕ ЯДРА..."
                            } else {
                                "▶ ЗАПУСТИТЬ СЕРВЕР"
                            };
                            let btn = egui::Button::new(
                                RichText::new(btn_text)
                                    .size(13.)
                                    .color(Color32::BLACK)
                                    .strong(),
                            )
                            .fill(theme::YELLOW)
                            .rounding(Rounding::same(2.));

                            if ui
                                .add_enabled(!sm.is_downloading && !is_running, btn)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                sm.start_server(cfg);
                            }
                        }
                    });
                }
            });
        });

    ui.add_space(8.);

    // Status banner if present
    if let Some((ok, ref msg)) = sm.status_msg {
        let (bg, border, col) = if ok {
            (
                theme::YELLOW.gamma_multiply(0.1),
                theme::YELLOW,
                theme::YELLOW,
            )
        } else {
            (
                theme::DANGER.gamma_multiply(0.1),
                theme::DANGER,
                theme::DANGER,
            )
        };
        Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1., border))
            .rounding(Rounding::same(4.))
            .inner_margin(Margin::same(8.))
            .show(ui, |ui| {
                ui.label(RichText::new(msg).color(col).strong());
            });
        ui.add_space(6.);
    }

    // Modal creation dialog
    if sm.creating {
        ui.add_space(6.);
        Frame::none()
            .fill(theme::CARD_HI)
            .stroke(Stroke::new(1.5, theme::YELLOW))
            .rounding(Rounding::same(6.))
            .inner_margin(Margin::same(16.))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("СОЗДАНИЕ НОВОГО СЕРВЕРА")
                        .size(15.)
                        .color(theme::YELLOW)
                        .strong(),
                );
                ui.add_space(8.);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Название:").color(theme::MUTED));
                    ui.add(TextEdit::singleline(&mut sm.new_name).desired_width(240.));
                });
                ui.add_space(4.);

                ui.label(RichText::new("Выберите игровое ядро:").color(theme::MUTED));
                for k in [
                    ServerKind::PhantomRust,
                    ServerKind::CrossplayPaper,
                    ServerKind::Bedrock,
                ] {
                    ui.horizontal(|ui| {
                        if ui
                            .radio(sm.new_kind == k, k.label())
                            .clicked()
                        {
                            sm.new_kind = k;
                            sm.new_port = k.default_port().to_string();
                        }
                    });
                    ui.label(RichText::new(format!("  ↳ {}", k.desc())).size(13.).color(theme::TEXT_SOFT));
                }
                ui.add_space(4.);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Порт сервера:").color(theme::MUTED));
                    ui.add(TextEdit::singleline(&mut sm.new_port).desired_width(80.));
                });
                ui.add_space(10.);

                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("СОЗДАТЬ")
                                    .color(Color32::BLACK)
                                    .strong(),
                            )
                            .fill(theme::YELLOW),
                        )
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        let id = format!(
                            "{}-{}",
                            sm.new_name
                                .to_lowercase()
                                .chars()
                                .filter(|c| c.is_alphanumeric())
                                .collect::<String>(),
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                % 10000
                        );
                        let port: u16 = sm.new_port.parse().unwrap_or(sm.new_kind.default_port());
                        let entry = ServerEntry {
                            id: if id.is_empty() { "server".into() } else { id },
                            name: sm.new_name.clone(),
                            kind: sm.new_kind,
                            port,
                            created_at: "2026-11-13".into(),
                        };
                        let new_srv_dir = sm.server_dir(&entry);
                        let _ = fs::create_dir_all(&new_srv_dir);
                        let _ = fs::write(new_srv_dir.join("eula.txt"), "eula=true\n");
                        let _ = ServerProperties::load_or_create(&new_srv_dir.join("server.properties"), entry.kind, entry.port);

                        sm.servers.push(entry);
                        sm.selected_idx = sm.servers.len() - 1;
                        save_servers(&sm.servers);
                        sm.reload_properties();
                        sm.creating = false;
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Отмена").color(theme::MUTED),
                            )
                            .fill(theme::SIDE),
                        )
                        .clicked()
                    {
                        sm.creating = false;
                    }
                });
            });
        ui.add_space(8.);
    }

    let Some(srv) = sm.current_server().cloned() else {
        ui.label(RichText::new("Нет созданных серверов").color(theme::MUTED));
        return;
    };

    ui.add_space(8.);

    // Two plain steps instead of a wall of console text.
    let server_up = is_running && running_srv_id.as_deref() == Some(&srv.id);
    let (s1_text, s1_col) = if sm.is_downloading {
        ("ПОДГОТОВКА ЯДРА…", theme::YELLOW)
    } else if server_up {
        ("ОНЛАЙН", theme::OK)
    } else {
        ("ВЫКЛЮЧЕН", theme::MUTED)
    };
    let (s2_text, s2_col) = match (&sm.tunnel_url, &sm.bedrock_tunnel_url, &sm.public_ip) {
        (Some(u), _, _) if u.contains("claim") => ("НУЖНА АКТИВАЦИЯ PLAYIT", theme::YELLOW),
        (Some(_), _, _) | (_, Some(_), _) | (_, _, Some(_)) => ("ПОДКЛЮЧЁН", theme::OK),
        _ if sm.is_tunneling => ("ПОДКЛЮЧЕНИЕ…", theme::YELLOW),
        _ => ("НЕ ПОДКЛЮЧЁН", theme::MUTED),
    };
    ui.columns(2, |c| {
        status_card(&mut c[0], t, "1. СЕРВЕР", s1_text, s1_col);
        status_card(&mut c[1], t, "2. ИНТЕРНЕТ (Playit)", s2_text, s2_col);
    });
    ui.add_space(6.);

    // Big copy buttons: the address for friends, ready to paste.
    let current_local_ip = &sm.cached_local_ip;
    let lan_main = format!("{}:{}", current_local_ip, srv.port);
    let claim = sm.tunnel_url.clone().filter(|u| u.contains("claim"));
    let mut copied: Option<String> = None;

    if let Some(url) = &claim {
        ui.columns(2, |c| {
            if copy_card(&mut c[0], "В ЛОКАЛЬНОЙ СЕТИ", Some(lan_main.as_str()), "") {
                copied = Some(lan_main.clone());
            }
            if copy_card(&mut c[1], "PLAYIT: НАЖМИТЕ, ЧТОБЫ АКТИВИРОВАТЬ", Some(url.as_str()), "") {
                c[1].ctx().open_url(egui::OpenUrl::new_tab(url));
            }
        });
    } else if srv.kind == ServerKind::CrossplayPaper {
        // Crossplay has separate Java and Bedrock ports
        let lan_bedrock = format!("{}:19132", current_local_ip);
        let net_java = sm.tunnel_url.as_deref().or(sm.public_ip.as_deref());
        let net_bedrock = sm.bedrock_tunnel_url.as_deref();

        ui.columns(2, |c| {
            if copy_card(&mut c[0], "В ЛОКАЛЬНОЙ СЕТИ: JAVA (ПК)", Some(lan_main.as_str()), "") {
                copied = Some(lan_main.clone());
            }
            if copy_card(
                &mut c[1],
                "ДЛЯ ДРУЗЕЙ: JAVA (ПК)",
                net_java,
                "нажмите «ОТКРЫТЬ В ИНТЕРНЕТ» выше",
            ) {
                copied = net_java.map(|s| s.to_string());
            }
        });
        ui.add_space(4.);
        ui.columns(2, |c| {
            if copy_card(&mut c[0], "В ЛОКАЛЬНОЙ СЕТИ: BEDROCK", Some(lan_bedrock.as_str()), "") {
                copied = Some(lan_bedrock.clone());
            }
            if copy_card(
                &mut c[1],
                "ДЛЯ ДРУЗЕЙ: BEDROCK (ТЕЛЕФОНЫ)",
                net_bedrock,
                "нажмите «ОТКРЫТЬ В ИНТЕРНЕТ» выше",
            ) {
                copied = net_bedrock.map(|s| s.to_string());
            }
        });
    } else if srv.kind == ServerKind::Bedrock {
        let net_bedrock = sm.bedrock_tunnel_url.as_deref().or(sm.tunnel_url.as_deref()).or(sm.public_ip.as_deref());
        ui.columns(2, |c| {
            if copy_card(&mut c[0], "В ЛОКАЛЬНОЙ СЕТИ (BEDROCK)", Some(lan_main.as_str()), "") {
                copied = Some(lan_main.clone());
            }
            if copy_card(
                &mut c[1],
                "ДЛЯ ДРУЗЕЙ ЧЕРЕЗ ИНТЕРНЕТ (BEDROCK)",
                net_bedrock,
                "нажмите «ОТКРЫТЬ В ИНТЕРНЕТ» выше",
            ) {
                copied = net_bedrock.map(|s| s.to_string());
            }
        });
    } else {
        let net = sm.tunnel_url.as_deref().or(sm.public_ip.as_deref());
        ui.columns(2, |c| {
            if copy_card(&mut c[0], "В ЛОКАЛЬНОЙ СЕТИ (LAN)", Some(lan_main.as_str()), "") {
                copied = Some(lan_main.clone());
            }
            if copy_card(
                &mut c[1],
                "ДЛЯ ДРУЗЕЙ ЧЕРЕЗ ИНТЕРНЕТ (JAVA)",
                net,
                "нажмите «ОТКРЫТЬ В ИНТЕРНЕТ» выше",
            ) {
                copied = net.map(|s| s.to_string());
            }
        });
        if let Some(bedrock_net) = sm.bedrock_tunnel_url.as_deref() {
            ui.add_space(4.);
            let lan_b = format!("{}:19132", current_local_ip);
            ui.columns(2, |c| {
                if copy_card(&mut c[0], "В ЛОКАЛЬНОЙ СЕТИ: BEDROCK", Some(lan_b.as_str()), "") {
                    copied = Some(lan_b);
                }
                if copy_card(&mut c[1], "ДЛЯ ДРУЗЕЙ: BEDROCK (ТЕЛЕФОНЫ)", Some(bedrock_net), "") {
                    copied = Some(bedrock_net.to_string());
                }
            });
        }
    }

    if let Some(addr) = copied {
        ui.ctx().output_mut(|o| o.copied_text = addr.clone());
        sm.status_msg = Some((true, format!("Скопировано: {addr}")));
    }
    ui.add_space(8.);

    let right_width = 280.0;
    let panel_height = (ui.available_height() - 16.0).max(320.0);
    ui.horizontal_top(|ui| {
        // Left: Console
        let left_width = (ui.available_width() - right_width - 12.0).max(200.0);
        ui.vertical(|ui| {
            ui.set_width(left_width);
            render_console(sm, ui, panel_height);
        });

        ui.add_space(8.0);

        // Right: Compact Settings & Quick Actions Panel (Scrollable so all buttons fit)
        ui.vertical(|ui| {
            ui.set_width(right_width);
            egui::ScrollArea::vertical()
                .id_source("quick_settings_scroll")
                .auto_shrink([false; 2])
                .max_height(panel_height)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    render_quick_settings(sm, &srv, cfg, ui);
                });
        });
    });
    ui.add_space(24.0);
    });
}

// ---------------------------------------------------------------- Quick Settings & Actions Panel

fn render_quick_settings(
    sm: &mut ServerManager,
    srv: &ServerEntry,
    cfg: &Config,
    ui: &mut egui::Ui,
) {
    let mut changed = false;
    let mut new_port: Option<u16> = None;

    // 1. Server settings frame
    Frame::none()
        .fill(theme::CARD)
        .stroke(Stroke::new(1., theme::BORDER))
        .rounding(Rounding::same(4.))
        .inner_margin(Margin::same(10.))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.label(
                    RichText::new("НАСТРОЙКИ СЕРВЕРА")
                        .color(theme::YELLOW)
                        .strong()
                        .size(13.),
                );
                ui.add_space(8.);

                if let Some(ref mut props) = sm.properties_cache {
                    // 1. Offline mode checkbox (Пиратка)
                    let mut pirate = !props.get_bool("online-mode", false);
                    if ui
                        .checkbox(
                            &mut pirate,
                            RichText::new("Пиратка (offline-mode)")
                                .color(theme::TEXT)
                                .size(13.),
                        )
                        .changed()
                    {
                        props.set_bool("online-mode", !pirate);
                        changed = true;
                    }
                    ui.add_space(5.);

                    // 2. PvP checkbox
                    let mut pvp = props.get_bool("pvp", true);
                    if ui
                        .checkbox(
                            &mut pvp,
                            RichText::new("PvP (урон игрокам)")
                                .color(theme::TEXT)
                                .size(13.),
                        )
                        .changed()
                    {
                        props.set_bool("pvp", pvp);
                        changed = true;
                    }
                    ui.add_space(5.);

                    // 3. Spawn monsters checkbox
                    let mut monsters = props.get_bool("spawn-monsters", true);
                    if ui
                        .checkbox(
                            &mut monsters,
                            RichText::new("Спавн монстров")
                                .color(theme::TEXT)
                                .size(13.),
                        )
                        .changed()
                    {
                        props.set_bool("spawn-monsters", monsters);
                        changed = true;
                    }
                    ui.add_space(5.);

                    // 4. White list checkbox
                    let mut white_list = props.get_bool("white-list", false);
                    if ui
                        .checkbox(
                            &mut white_list,
                            RichText::new("Белый список")
                                .color(theme::TEXT)
                                .size(13.),
                        )
                        .changed()
                    {
                        props.set_bool("white-list", white_list);
                        changed = true;
                    }
                    ui.add_space(6.);

                    // 5. Difficulty ComboBox
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Сложность:").color(theme::MUTED).size(13.));
                        let mut cur_diff = props.get_str("difficulty", "easy");
                        egui::ComboBox::from_id_source("quick_diff_select")
                            .selected_text(RichText::new(&cur_diff).color(theme::TEXT).size(13.))
                            .width(110.)
                            .show_ui(ui, |ui| {
                                for d in ["peaceful", "easy", "normal", "hard"] {
                                    if ui.selectable_label(cur_diff == d, d).clicked() {
                                        cur_diff = d.into();
                                        props.set_str("difficulty", &cur_diff);
                                        changed = true;
                                    }
                                }
                            });
                    });
                    ui.add_space(5.);

                    // 6. Gamemode ComboBox
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Режим:").color(theme::MUTED).size(13.));
                        let mut cur_gm = props.get_str("gamemode", "survival");
                        egui::ComboBox::from_id_source("quick_gm_select")
                            .selected_text(RichText::new(&cur_gm).color(theme::TEXT).size(13.))
                            .width(110.)
                            .show_ui(ui, |ui| {
                                for g in ["survival", "creative", "adventure", "spectator"] {
                                    if ui.selectable_label(cur_gm == g, g).clicked() {
                                        cur_gm = g.into();
                                        props.set_str("gamemode", &cur_gm);
                                        changed = true;
                                    }
                                }
                            });
                    });
                    ui.add_space(5.);

                    // 7. Server Port TextEdit
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Порт:").color(theme::MUTED).size(13.));
                        let mut port_str = props.get_str("server-port", &srv.port.to_string());
                        if ui.add(TextEdit::singleline(&mut port_str).desired_width(70.)).changed() {
                            props.set_str("server-port", &port_str);
                            changed = true;
                            if let Ok(p) = port_str.trim().parse::<u16>() {
                                new_port = Some(p);
                            }
                        }
                    });
                } else {
                    ui.label(RichText::new("Настройки загружаются...").color(theme::MUTED).size(13.));
                }
            });
        });

    if changed {
        sm.save_current_properties();
    }
    if let Some(p) = new_port {
        if let Some(s) = sm.servers.get_mut(sm.selected_idx) {
            s.port = p;
            save_servers(&sm.servers);
        }
    }

    ui.add_space(8.);

    // 2. Quick action buttons frame
    Frame::none()
        .fill(theme::CARD)
        .stroke(Stroke::new(1., theme::BORDER))
        .rounding(Rounding::same(4.))
        .inner_margin(Margin::same(10.))
        .show(ui, |ui| {
            ui.label(
                RichText::new("БЫСТРЫЕ КОМАНДЫ")
                    .color(theme::YELLOW)
                    .strong()
                    .size(13.),
            );
            ui.add_space(6.);

            let op_btn = format!("★ Выдать OP ({})", cfg.nickname);
            let action_btn = |ui: &mut egui::Ui, label: &str, text_col: Color32| -> egui::Response {
                ui.add_sized(
                    [ui.available_width(), 26.],
                    egui::Button::new(RichText::new(label).color(text_col).size(13.).strong())
                        .fill(theme::SIDE)
                        .stroke(Stroke::new(1., theme::BORDER)),
                )
                .on_hover_cursor(CursorIcon::PointingHand)
            };

            if action_btn(ui, &op_btn, theme::YELLOW).clicked() {
                sm.execute_command(&format!("op {}", cfg.nickname));
            }
            ui.add_space(3.);

            if action_btn(ui, "☀ Сделать день", theme::TEXT).clicked() {
                sm.execute_command("time set day");
            }
            ui.add_space(3.);

            if action_btn(ui, "⚡ Очистить погоду", theme::TEXT).clicked() {
                sm.execute_command("weather clear");
            }
            ui.add_space(3.);

            if action_btn(ui, "📦 Режим Творчество", theme::TEXT).clicked() {
                sm.execute_command("gamemode creative @a");
            }
            ui.add_space(3.);

            if action_btn(ui, "⚔ Режим Выживание", theme::TEXT).clicked() {
                sm.execute_command("gamemode survival @a");
            }
            ui.add_space(6.);

            // Folder shortcut for mods/plugins
            let srv_dir = sm.server_dir(srv);
            let target_folder = match srv.kind {
                ServerKind::CrossplayPaper => srv_dir.join("plugins"),
                ServerKind::PhantomRust | ServerKind::Bedrock => srv_dir.join("mods"),
            };
            let folder_label = match srv.kind {
                ServerKind::CrossplayPaper => "📁 Папка плагинов",
                ServerKind::PhantomRust | ServerKind::Bedrock => "📁 Папка модов",
            };
            if action_btn(ui, folder_label, Color32::from_rgb(0, 220, 255)).clicked() {
                let _ = fs::create_dir_all(&target_folder);
                let _ = Command::new("explorer").arg(&target_folder).spawn();
            }
        });
}

// ---------------------------------------------------------------- Console View

fn render_console(sm: &mut ServerManager, ui: &mut egui::Ui, height: f32) {
    ui.vertical(|ui| {
        let avail_h = (height - 34.0 - 6.0).max(180.0);

        Frame::none()
            .fill(Color32::from_rgb(10, 11, 14))
            .stroke(Stroke::new(1., theme::BORDER))
            .rounding(Rounding::same(4.))
            .inner_margin(Margin::same(8.))
            .show(ui, |ui| {
                ui.set_height(avail_h);
                ui.set_width(ui.available_width());
                egui::ScrollArea::vertical()
                    .id_source("server_console_scroll")
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical(|ui| {
                            if let Ok(logs) = sm.console_logs.lock() {
                                if logs.is_empty() {
                                    ui.label(
                                        RichText::new("Консоль пуста. Нажмите [▶ ЗАПУСТИТЬ СЕРВЕР] для старта.")
                                            .color(theme::MUTED)
                                            .size(13.),
                                    );
                                } else {
                                    for line in logs.iter() {
                                        let col = if line.contains("WARN") {
                                            theme::YELLOW
                                        } else if line.contains("ERROR") || line.contains("Exception") {
                                            theme::DANGER
                                        } else if line.starts_with('>') {
                                            Color32::from_rgb(0, 220, 255)
                                        } else {
                                            theme::TEXT_SOFT
                                        };
                                        ui.label(
                                            RichText::new(line)
                                                .color(col)
                                                .font(FontId::monospace(11.5)),
                                        );
                                    }
                                }
                            }
                        });
                    });
            });

        ui.add_space(6.);

        // Command input line directly below black console block
        ui.horizontal(|ui| {
            ui.label(RichText::new(">").color(theme::YELLOW).strong());
            let r = ui.add_sized(
                [ui.available_width() - 110., 28.],
                TextEdit::singleline(&mut sm.cmd_input)
                    .hint_text("Команда (/op Ник, /gamemode creative, /tp, stop)..."),
            );
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                sm.send_command();
            }

            if ui
                .add_sized(
                    [100., 28.],
                    egui::Button::new(
                        RichText::new("ОТПРАВИТЬ")
                            .size(13.)
                            .color(Color32::BLACK)
                            .strong(),
                    )
                    .fill(theme::YELLOW),
                )
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                sm.send_command();
            }
        });
    });
}

// ---------------------------------------------------------------- Properties Editor

fn render_properties_editor(
    sm: &mut ServerManager,
    srv: &ServerEntry,
    ui: &mut egui::Ui,
) {
    let mut changed = false;
    let ip = &sm.cached_local_ip;
    let port = srv.port;

    // IP address banner
    Frame::none()
        .fill(theme::CARD)
        .stroke(Stroke::new(1., theme::BORDER))
        .rounding(Rounding::same(4.))
        .inner_margin(Margin::same(12.))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("ЛОКАЛЬНЫЙ IP ДЛЯ ДРУЗЕЙ:").color(theme::MUTED).strong());
                let full_addr = format!("{ip}:{port}");
                ui.label(RichText::new(&full_addr).color(theme::YELLOW).strong().size(14.));

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("📋 Скопировать IP")
                                .size(13.)
                                .color(theme::TEXT),
                        )
                        .fill(theme::SIDE)
                        .stroke(Stroke::new(1., theme::BORDER)),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    ui.ctx().output_mut(|o| o.copied_text = full_addr);
                    sm.status_msg = Some((true, "Адрес сервера скопирован в буфер!".into()));
                }
            });
        });

    ui.add_space(8.);

    let Some(ref mut props) = sm.properties_cache else {
        return;
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        Frame::none()
            .fill(theme::CARD)
            .stroke(Stroke::new(1., theme::BORDER))
            .rounding(Rounding::same(4.))
            .inner_margin(Margin::same(12.))
            .show(ui, |ui| {
                // Online Mode (Лицензия / Оффлайн)
                ui.horizontal(|ui| {
                    let mut online = props.get_bool("online-mode", false);
                    if ui.checkbox(&mut online, "").changed() {
                        props.set_bool("online-mode", online);
                        changed = true;
                    }
                    ui.label(RichText::new("Вход только с лицензией (online-mode)").color(theme::TEXT));
                    ui.label(
                        RichText::new("(отключите для бесплатного входа с любым ником)")
                            .color(theme::MUTED)
                            .size(13.),
                    );
                });
                ui.add_space(8.);

                // PvP
                ui.horizontal(|ui| {
                    let mut pvp = props.get_bool("pvp", true);
                    if ui.checkbox(&mut pvp, "").changed() {
                        props.set_bool("pvp", pvp);
                        changed = true;
                    }
                    ui.label(RichText::new("Урон между игроками (PvP)").color(theme::TEXT));
                });
                ui.add_space(8.);

                // Difficulty
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Сложность (difficulty):").color(theme::MUTED));
                    let mut cur_diff = props.get_str("difficulty", "easy");
                    egui::ComboBox::from_id_source("diff_select")
                        .selected_text(RichText::new(&cur_diff).color(theme::TEXT))
                        .show_ui(ui, |ui| {
                            for d in ["peaceful", "easy", "normal", "hard"] {
                                if ui.selectable_label(cur_diff == d, d).clicked() {
                                    cur_diff = d.into();
                                    props.set_str("difficulty", &cur_diff);
                                    changed = true;
                                }
                            }
                        });
                });
                ui.add_space(8.);

                // Gamemode
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Режим игры (gamemode):").color(theme::MUTED));
                    let mut cur_gm = props.get_str("gamemode", "survival");
                    egui::ComboBox::from_id_source("gm_select")
                        .selected_text(RichText::new(&cur_gm).color(theme::TEXT))
                        .show_ui(ui, |ui| {
                            for g in ["survival", "creative", "adventure", "spectator"] {
                                if ui.selectable_label(cur_gm == g, g).clicked() {
                                    cur_gm = g.into();
                                    props.set_str("gamemode", &cur_gm);
                                    changed = true;
                                }
                            }
                        });
                });
                ui.add_space(8.);

                // Server Port
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Порт сервера (server-port):").color(theme::MUTED));
                    let mut port_str = props.get_str("server-port", &srv.port.to_string());
                    if ui.add(TextEdit::singleline(&mut port_str).desired_width(80.)).changed() {
                        props.set_str("server-port", &port_str);
                        changed = true;
                    }
                });
                ui.add_space(8.);

                // MOTD
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Описание сервера (motd):").color(theme::MUTED));
                    let mut motd = props.get_str("motd", "Phantom Server");
                    if ui.add(TextEdit::singleline(&mut motd).desired_width(260.)).changed() {
                        props.set_str("motd", &motd);
                        changed = true;
                    }
                });
            });
    });

    if changed {
        sm.save_current_properties();
    }
}

// ---------------------------------------------------------------- Server Plugins / Mods

fn render_server_plugins(
    sm: &mut ServerManager,
    srv: &ServerEntry,
    ui: &mut egui::Ui,
) {
    let srv_dir = sm.server_dir(srv);
    let target_folder = match srv.kind {
        ServerKind::CrossplayPaper => srv_dir.join("plugins"),
        ServerKind::PhantomRust | ServerKind::Bedrock => srv_dir.join("mods"),
    };
    let _ = fs::create_dir_all(&target_folder);

    let folder_name = match srv.kind {
        ServerKind::CrossplayPaper => "plugins",
        ServerKind::PhantomRust | ServerKind::Bedrock => "mods",
    };

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("ФАЙЛЫ В ПАПКЕ {folder_name}/:"))
                .color(theme::MUTED)
                .strong(),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(format!("📁 Открыть {folder_name}"))
                            .size(13.)
                            .color(theme::YELLOW),
                    )
                    .fill(theme::SIDE)
                    .stroke(Stroke::new(1., theme::BORDER)),
                )
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                let _ = Command::new("explorer").arg(&target_folder).spawn();
            }
        });
    });

    ui.add_space(6.);

    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&target_folder) {
        for e in entries.flatten() {
            if let Ok(meta) = e.metadata() {
                if meta.is_file() {
                    let name = e.file_name().to_string_lossy().to_string();
                    let enabled = !name.ends_with(".disabled");
                    let clean = name.strip_suffix(".disabled").unwrap_or(&name).to_string();
                    files.push((name, clean, enabled, meta.len(), e.path()));
                }
            }
        }
    }
    files.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

    if files.is_empty() {
        Frame::none()
            .fill(theme::CARD)
            .stroke(Stroke::new(1., theme::BORDER))
            .rounding(Rounding::same(4.))
            .inner_margin(Margin::same(16.))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("В папке {folder_name}/ пока нет файлов. Вы можете скинуть .jar файлы плагинов или модов прямо туда."))
                        .color(theme::MUTED),
                );
            });
    } else {
        let mut toggle_file = None;
        let mut delete_file = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (name, clean, enabled, size, path) in &files {
                Frame::none()
                    .fill(if *enabled { theme::CARD } else { theme::SIDE })
                    .stroke(Stroke::new(
                        1.,
                        if *enabled { theme::BORDER } else { theme::BORDER.gamma_multiply(0.5) },
                    ))
                    .rounding(Rounding::same(3.))
                    .inner_margin(Margin::same(8.))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut chk = *enabled;
                            if ui.checkbox(&mut chk, "").changed() {
                                toggle_file = Some((path.clone(), name.clone(), clean.clone(), chk));
                            }

                            let col = if *enabled { theme::TEXT } else { theme::MUTED };
                            ui.label(RichText::new(clean).color(col).strong());

                            let sz_kb = format!("{:.1} KB", *size as f64 / 1024.0);
                            ui.label(RichText::new(sz_kb).color(theme::TEXT_SOFT).size(13.));

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("✕")
                                                .color(theme::DANGER)
                                                .size(13.),
                                        )
                                        .fill(theme::SIDE),
                                    )
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    delete_file = Some(path.clone());
                                }
                            });
                        });
                    });
                ui.add_space(4.);
            }
        });

        if let Some((path, _name, clean, make_enabled)) = toggle_file {
            let target_p = if make_enabled {
                path.with_file_name(&clean)
            } else {
                path.with_file_name(format!("{clean}.disabled"))
            };
            let _ = fs::rename(&path, &target_p);
        }

        if let Some(p) = delete_file {
            let _ = fs::remove_file(p);
        }
    }
}
