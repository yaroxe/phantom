//! Finds javaw.exe: JAVA_HOME, Program Files vendors, the .minecraft runtime, PATH.
use std::{
    collections::HashMap,
    env, fs,
    sync::{Mutex, OnceLock},
    path::{Path, PathBuf},
    process::Command,
};

pub fn exe() -> &'static str {
    if cfg!(windows) {
        "javaw.exe"
    } else {
        "java"
    }
}

fn children(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default()
}

fn push(found: &mut Vec<PathBuf>, p: PathBuf) {
    if p.is_file() && !found.contains(&p) {
        found.push(p);
    }
}

pub fn detect(game_dir: &Path) -> Vec<PathBuf> {
    let mut f = Vec::new();

    if let Some(home) = env::var_os("JAVA_HOME") {
        push(&mut f, PathBuf::from(home).join("bin").join(exe()));
    }

    const VENDORS: [&str; 8] = [
        "Java",
        "Eclipse Adoptium",
        "Microsoft",
        "Zulu",
        "BellSoft",
        "Amazon Corretto",
        "Semeru",
        "AdoptOpenJDK",
    ];
    for var in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(pf) = env::var_os(var) {
            for vendor in VENDORS {
                for d in children(&PathBuf::from(&pf).join(vendor)) {
                    push(&mut f, d.join("bin").join(exe()));
                }
            }
        }
    }

    // The official launcher's bundled runtimes: runtime/<name>/<platform>/<name>/bin/javaw.exe
    let mut runtimes = vec![game_dir.join("runtime")];
    if let Some(local) = env::var_os("LOCALAPPDATA") {
        runtimes.push(
            PathBuf::from(local)
                .join("Packages")
                .join("Microsoft.4297127D64EC6_8wekyb3d8bbwe")
                .join("LocalCache")
                .join("Local")
                .join("runtime"),
        );
    }
    for root in runtimes {
        for a in children(&root) {
            for b in children(&a) {
                for c in children(&b) {
                    push(&mut f, c.join("bin").join(exe()));
                }
            }
        }
    }

    if let Some(path) = env::var_os("PATH") {
        for d in env::split_paths(&path) {
            push(&mut f, d.join(exe()));
        }
    }
    f
}

#[cfg(windows)]
pub fn quiet(c: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    c.creation_flags(0x0800_0000) // CREATE_NO_WINDOW
}

#[cfg(not(windows))]
pub fn quiet(c: &mut Command) -> &mut Command {
    c
}

/// The Java's major version ("1.8.0_301" -> 8, "21.0.2" -> 21). Asks java.exe: javaw prints nothing.
pub fn major(javaw: &Path) -> Option<u32> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Option<u32>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(v) = cache.lock().ok().and_then(|c| c.get(javaw).copied()) {
        return v;
    }
    let v = query_major(javaw);
    if let Ok(mut c) = cache.lock() {
        c.insert(javaw.to_path_buf(), v);
    }
    v
}

/// The console twin of javaw (java.exe): the only one whose output a launcher can capture.
pub fn console(javaw: &Path) -> PathBuf {
    let c = javaw.with_file_name(if cfg!(windows) { "java.exe" } else { "java" });
    if c.is_file() {
        c
    } else {
        javaw.to_path_buf()
    }
}

fn query_major(javaw: &Path) -> Option<u32> {
    let console = javaw.with_file_name(if cfg!(windows) { "java.exe" } else { "java" });
    let bin = if console.is_file() {
        console
    } else {
        javaw.to_path_buf()
    };
    let out = quiet(Command::new(bin).arg("-version")).output().ok()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let version = text.split('"').nth(1)?;
    let mut parts = version.split(|c: char| matches!(c, '.' | '_' | '-' | '+'));
    let first: u32 = parts.next()?.parse().ok()?;
    if first == 1 {
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}

/// An explicit choice wins. Otherwise the newest installed Java that is at least `need` (25 > 21 > 17),
/// so nobody has to paste a path; only legacy games (`need` <= 8) keep the oldest suitable Java.
pub fn pick(detected: &[PathBuf], need: u32, explicit: &str) -> Option<PathBuf> {
    if !explicit.trim().is_empty() {
        return Some(PathBuf::from(explicit.trim()));
    }
    let mut ok: Vec<(u32, &PathBuf)> = detected
        .iter()
        .filter_map(|p| major(p).map(|m| (m, p)))
        .filter(|(m, _)| *m >= need)
        .collect();
    ok.sort_by_key(|(m, _)| *m);
    let best = if need <= 8 { ok.first() } else { ok.last() };
    best.map(|(_, p)| (*p).clone())
}
