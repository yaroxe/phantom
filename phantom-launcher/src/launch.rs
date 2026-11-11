//! Starting a game: Phantom Engine (own binary), a Java version from .minecraft, or a plain .jar.
use crate::{config::Config, java};
use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const OS_NAME: &str = if cfg!(windows) {
    "windows"
} else if cfg!(target_os = "macos") {
    "osx"
} else {
    "linux"
};

pub fn valid_nick(n: &str) -> bool {
    (3..=16).contains(&n.chars().count())
        && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Minecraft's offline UUID: MD5("OfflinePlayer:<name>") as a version-3 UUID.
pub fn offline_uuid(name: &str) -> String {
    let mut d = md5::compute(format!("OfflinePlayer:{name}")).0;
    d[6] = (d[6] & 0x0f) | 0x30;
    d[8] = (d[8] & 0x3f) | 0x80;
    let h: String = d.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}

/// Installed Java versions: `versions/<id>/<id>.json` and `<id>.jar` both present.
pub fn installed_versions(game_dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(game_dir.join("versions"))
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let id = e.file_name().to_string_lossy().to_string();
                    let dir = e.path();
                    (dir.join(format!("{id}.json")).is_file()
                        && dir.join(format!("{id}.jar")).is_file())
                    .then_some(id)
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out.reverse();
    out
}

pub fn play(cfg: &Config) -> Result<String, String> {
    if !valid_nick(&cfg.nickname) {
        return Err("Ник: 3–16 символов, латиница, цифры и _".into());
    }
    let id = cfg.selected.as_str();
    if id.is_empty() || id == "phantom" {
        phantom(cfg)
    } else if let Some(v) = id.strip_prefix("java:") {
        vanilla(cfg, v)
    } else if let Some(j) = id.strip_prefix("jar:") {
        jar(cfg, j)
    } else {
        Err(format!("Неизвестная сборка: {id}"))
    }
}

// ---------------------------------------------------------------- Phantom Engine

pub fn find_resources(home: &Path, cfg: &Config) -> PathBuf {
    let data_root = crate::config::data_root();
    let mut candidates = vec![
        PathBuf::from(&cfg.phantom_dir).join("resources"),
        home.join("resources"),
        home.join(".local").join("phantom-client").join("26.3"),
        home.join(".local").join("horizon-client").join("26.3"),
    ];

    for base in [
        data_root.join("phantom-launcher").join("versions"),
        data_root.join("horizon-launcher").join("versions"),
        PathBuf::from(&cfg.phantom_dir).join("versions"),
    ] {
        if let Ok(entries) = fs::read_dir(&base) {
            let mut vers: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
            vers.sort();
            vers.reverse();
            for v in vers {
                candidates.push(v.join(".local").join("phantom-client").join("26.3"));
                candidates.push(v.join(".local").join("horizon-client").join("26.3"));
                candidates.push(v.join("resources"));
            }
        }
    }
    candidates.push(PathBuf::from(".local").join("phantom-client").join("26.3"));
    candidates.push(PathBuf::from(".local").join("horizon-client").join("26.3"));

    for c in &candidates {
        if c.join("horizon-resources.json").is_file() {
            return c.clone();
        }
    }
    candidates
        .into_iter()
        .find(|p| p.is_dir())
        .unwrap_or_else(|| PathBuf::from(&cfg.phantom_dir).join("resources"))
}

pub fn find_client(cfg: &Config) -> Option<PathBuf> {
    let exe = if cfg!(windows) {
        "phantom-client.exe"
    } else {
        "phantom-client"
    };
    let explicit = PathBuf::from(cfg.phantom_client.trim());
    if !cfg.phantom_client.trim().is_empty() && explicit.is_file() {
        return Some(explicit);
    }
    let data_root = crate::config::data_root();
    let dirs = [
        PathBuf::from(&cfg.phantom_dir),
        data_root.join("phantom-launcher"),
        data_root.join("horizon-launcher"),
    ];
    let mut c = Vec::new();
    for dir in &dirs {
        c.push(dir.join(exe));
        c.push(dir.join("bin").join(exe));
        let mut versions: Vec<PathBuf> = fs::read_dir(dir.join("versions"))
            .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
            .unwrap_or_default();
        versions.sort();
        versions.reverse(); // newest name first
        c.extend(versions.into_iter().map(|v| v.join(exe)));
    }
    if let Some(side) = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join(exe)))
    {
        c.push(side);
    }
    c.push(PathBuf::from("target/release").join(exe));
    c.into_iter().find(|p| p.is_file())
}

fn phantom(cfg: &Config) -> Result<String, String> {
    let client = find_client(cfg).ok_or(
        "phantom-client не найден. Укажите путь в Настройках или соберите: cargo build --release",
    )?;
    let home = client
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let resources = find_resources(&home, cfg);

    let mut cmd = Command::new(&client);
    cmd.current_dir(&home)
        .arg("--resources")
        .arg(&resources)
        .arg("--name")
        .arg(&cfg.nickname)
        .arg("--game-dir")
        .arg(cfg.game_dir());
    cmd.spawn()
        .map_err(|e| format!("Не удалось запустить {}: {e}", client.display()))?;
    Ok("Phantom Engine запущен".to_string())
}

// ---------------------------------------------------------------- Java

fn memory(cfg: &Config) -> Vec<String> {
    vec![
        format!("-Xmx{}G", cfg.ram_gb),
        format!("-Xms{}G", cfg.ram_gb.min(2)),
    ]
}

fn jar(cfg: &Config, path: &str) -> Result<String, String> {
    let javaw = java::pick(&java::detect(cfg.game_dir()), 0, &cfg.java_path)
        .ok_or("Java не найдена. Установите Temurin или укажите javaw в Настройках.")?;
    let _ = fs::create_dir_all(cfg.game_dir());
    Command::new(&javaw)
        .current_dir(cfg.game_dir())
        .args(memory(cfg))
        .arg("-jar")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Не удалось запустить {}: {e}", javaw.display()))?;
    Ok(format!("Запущен {path}"))
}

/// One rule list the way version JSON writes it: the last matching rule decides; no rules = allowed.
/// Entries gated on launcher features (demo mode, custom resolution) never match.
fn rules_ok(rules: Option<&Value>) -> bool {
    let Some(rules) = rules.and_then(|r| r.as_array()) else {
        return true;
    };
    let mut ok = false;
    for r in rules {
        let mut hit = r.get("features").is_none();
        if let Some(os) = r.get("os") {
            if os["name"].as_str().is_some_and(|n| n != OS_NAME) {
                hit = false;
            }
            if os["arch"].as_str() == Some("x86") {
                hit = false; // 64-bit only
            }
        }
        if hit {
            ok = r["action"] == "allow";
        }
    }
    ok
}

fn collect(args: Option<&Value>, out: &mut Vec<String>) {
    for a in args.and_then(|a| a.as_array()).into_iter().flatten() {
        match a {
            Value::String(s) => out.push(s.clone()),
            Value::Object(_) if rules_ok(a.get("rules")) => match &a["value"] {
                Value::String(s) => out.push(s.clone()),
                Value::Array(list) => {
                    out.extend(list.iter().filter_map(|s| s.as_str().map(String::from)))
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn subst(s: &str, vars: &[(&str, String)]) -> String {
    let mut s = s.to_string();
    for (k, v) in vars {
        s = s.replace(&format!("${{{k}}}"), v);
    }
    s
}

/// A vanilla 1.13+ version (the JSON has `arguments`). Libraries and assets must already be in
/// the game directory (run the version once from the official launcher). Forge/Fabric profiles
/// (`inheritsFrom`) and pre-1.13 versions (natives extraction) are not handled in v1.0.
fn vanilla(cfg: &Config, id: &str) -> Result<String, String> {
    let game = cfg.game_dir();
    let vdir = game.join("versions").join(id);
    let text = fs::read_to_string(vdir.join(format!("{id}.json")))
        .map_err(|e| format!("{id}.json: {e}"))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{id}.json: {e}"))?;
    if v.get("arguments").is_none() {
        return Err(format!("{id}: версии до 1.13 пока не поддерживаются"));
    }

    let sep = if cfg!(windows) { ";" } else { ":" };
    let libs = game.join("libraries");
    let mut classpath = Vec::new();
    let mut missing = 0;
    for lib in v["libraries"].as_array().into_iter().flatten() {
        if !rules_ok(lib.get("rules")) {
            continue;
        }
        if let Some(path) = lib["downloads"]["artifact"]["path"].as_str() {
            let file = libs.join(path);
            if file.is_file() {
                classpath.push(file.to_string_lossy().to_string());
            } else {
                missing += 1;
            }
        }
    }
    if missing > 0 {
        return Err(format!(
            "Не хватает {missing} библиотек: один раз запустите {id} из официального лаунчера."
        ));
    }
    classpath.push(vdir.join(format!("{id}.jar")).to_string_lossy().to_string());

    let natives = vdir.join("natives");
    let _ = fs::create_dir_all(&natives);
    let need = v["javaVersion"]["majorVersion"].as_u64().unwrap_or(8) as u32;
    let javaw = java::pick(&java::detect(game), need, &cfg.java_path).ok_or(format!(
        "Нужна Java {need}+: установите её или укажите javaw в Настройках."
    ))?;

    let vars = [
        ("auth_player_name", cfg.nickname.clone()),
        ("version_name", id.to_string()),
        ("game_directory", game.to_string_lossy().to_string()),
        (
            "assets_root",
            game.join("assets").to_string_lossy().to_string(),
        ),
        (
            "assets_index_name",
            v["assetIndex"]["id"].as_str().unwrap_or(id).to_string(),
        ),
        ("auth_uuid", offline_uuid(&cfg.nickname)),
        ("auth_access_token", "0".into()),
        ("clientid", "0".into()),
        ("auth_xuid", "0".into()),
        ("user_type", "legacy".into()),
        ("version_type", "Phantom".into()),
        ("classpath", classpath.join(sep)),
        ("natives_directory", natives.to_string_lossy().to_string()),
        ("library_directory", libs.to_string_lossy().to_string()),
        ("classpath_separator", sep.into()),
        ("launcher_name", "PhantomLauncher".into()),
        ("launcher_version", "1.0".into()),
    ];

    let mut jvm = Vec::new();
    let mut game_args = Vec::new();
    collect(v["arguments"].get("jvm"), &mut jvm);
    collect(v["arguments"].get("game"), &mut game_args);
    let main = v["mainClass"].as_str().ok_or("в JSON нет mainClass")?;

    Command::new(&javaw)
        .current_dir(game)
        .args(memory(cfg))
        .args(jvm.iter().map(|a| subst(a, &vars)))
        .arg(main)
        .args(game_args.iter().map(|a| subst(a, &vars)))
        .spawn()
        .map_err(|e| format!("Не удалось запустить {}: {e}", javaw.display()))?;
    Ok(format!(
        "Minecraft {id} запущен (Java {need}+, {} GB)",
        cfg.ram_gb
    ))
}
