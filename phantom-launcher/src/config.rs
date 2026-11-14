//! Settings, stored in %APPDATA%\phantom\launcher.json (no Horizon paths, no accounts).
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    /// Offline nickname.
    pub nickname: String,
    /// RAM for the JVM, GB (2..=16).
    pub ram_gb: u32,
    /// The Java game directory (`.minecraft`): versions, libraries, mods, shaderpacks.
    pub game_dir: String,
    /// Phantom's data directory (`%APPDATA%\phantom`): resources, versions.
    pub phantom_dir: String,
    /// Explicit path to phantom-client(.exe); empty = search.
    pub phantom_client: String,
    /// Explicit path to javaw(.exe); empty = auto-detect.
    pub java_path: String,
    /// "phantom", "java:<version id>" or "jar:<path>".
    pub selected: String,
    pub custom_jars: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let max_ram = total_ram_gb();
        Config {
            nickname: "Player".into(),
            ram_gb: 4.min(max_ram).max(2),
            game_dir: minecraft_dir().to_string_lossy().into(),
            phantom_dir: data_root().join("phantom").to_string_lossy().into(),
            phantom_client: String::new(),
            java_path: String::new(),
            selected: "phantom".into(),
            custom_jars: Vec::new(),
        }
    }
}

/// Real physical RAM of this computer in GB, rounded to the nearest integer (min 2).
pub fn total_ram_gb() -> u32 {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct MemoryStatusEx {
            length: u32,
            memory_load: u32,
            total_phys: u64,
            avail_phys: u64,
            total_page_file: u64,
            avail_page_file: u64,
            total_virtual: u64,
            avail_virtual: u64,
            avail_extended_virtual: u64,
        }
        extern "system" {
            fn GlobalMemoryStatusEx(stat: *mut MemoryStatusEx) -> i32;
        }
        unsafe {
            let mut status = MemoryStatusEx {
                length: std::mem::size_of::<MemoryStatusEx>() as u32,
                memory_load: 0,
                total_phys: 0,
                avail_phys: 0,
                total_page_file: 0,
                avail_page_file: 0,
                total_virtual: 0,
                avail_virtual: 0,
                avail_extended_virtual: 0,
            };
            if GlobalMemoryStatusEx(&mut status) != 0 {
                let gb = (status.total_phys as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u32;
                gb.max(2)
            } else {
                8
            }
        }
    }
    #[cfg(not(windows))]
    {
        8
    }
}

pub fn data_root() -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn minecraft_dir() -> PathBuf {
    if cfg!(windows) {
        data_root().join(".minecraft")
    } else {
        env::var_os("HOME")
            .map(|h| PathBuf::from(h).join(".minecraft"))
            .unwrap_or_else(|| PathBuf::from(".minecraft"))
    }
}

fn file() -> PathBuf {
    data_root().join("phantom").join("launcher.json")
}

impl Config {
    pub fn load() -> Config {
        let mut cfg: Config = fs::read_to_string(file())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let max_ram = total_ram_gb();
        cfg.ram_gb = cfg.ram_gb.clamp(2, max_ram);
        cfg
    }

    pub fn save(&self) {
        let path = file();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, text);
        }
    }

    pub fn game_dir(&self) -> &Path {
        Path::new(&self.game_dir)
    }
}
