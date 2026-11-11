//! Modrinth open REST API v2: search and download shaders, resourcepacks and mods.
use serde::Deserialize;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

const API: &str = "https://api.modrinth.com/v2";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Shader,
    ResourcePack,
    Mod,
}

impl Kind {
    pub fn facet(self) -> &'static str {
        match self {
            Kind::Shader => "shader",
            Kind::ResourcePack => "resourcepack",
            Kind::Mod => "mod",
        }
    }
    pub fn folder(self) -> &'static str {
        match self {
            Kind::Shader => "shaderpacks",
            Kind::ResourcePack => "resourcepacks",
            Kind::Mod => "mods",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Kind::Shader => "Шейдеры",
            Kind::ResourcePack => "Ресурспаки",
            Kind::Mod => "Моды",
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Hit {
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub icon_url: Option<String>,
}

#[derive(Deserialize)]
struct Search {
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct Version {
    files: Vec<VFile>,
}

#[derive(Deserialize)]
struct VFile {
    url: String,
    filename: String,
    #[serde(default)]
    primary: bool,
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .user_agent("phantom/phantom-launcher/1.0")
        .timeout(Duration::from_secs(30))
        .build()
}

fn clean(s: &str) -> String {
    s.trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect()
}

pub fn search(
    query: &str,
    kind: Kind,
    loader: &str,
    game_version: &str,
) -> Result<Vec<Hit>, String> {
    let (loader, ver) = (clean(loader), clean(game_version));
    let mut facets = vec![format!("[\"project_type:{}\"]", kind.facet())];
    if kind == Kind::Mod && !loader.is_empty() {
        facets.push(format!("[\"categories:{loader}\"]"));
    }
    if !ver.is_empty() {
        facets.push(format!("[\"versions:{ver}\"]"));
    }
    let facets = format!("[{}]", facets.join(","));
    let mut req = agent()
        .get(&format!("{API}/search"))
        .query("facets", &facets)
        .query("limit", "24");

    let q = query.trim();
    if !q.is_empty() {
        req = req.query("query", q);
    } else {
        req = req.query("index", "downloads");
    }

    let res: Search = req
        .call()
        .map_err(|e| format!("Modrinth: {e}"))?
        .into_json()
        .map_err(|e| format!("Modrinth: {e}"))?;
    Ok(res.hits)
}

pub fn download(
    project_id: &str,
    kind: Kind,
    loader: &str,
    game_version: &str,
    dest: &Path,
) -> Result<String, String> {
    let (loader, ver) = (clean(loader), clean(game_version));
    let url = format!("{API}/project/{}/version", clean(project_id));
    
    // First try with filters
    let mut req = agent().get(&url);
    if kind == Kind::Mod && !loader.is_empty() {
        req = req.query("loaders", &format!("[\"{loader}\"]"));
    }
    if !ver.is_empty() {
        req = req.query("game_versions", &format!("[\"{ver}\"]"));
    }
    
    let mut versions: Vec<Version> = req
        .call()
        .ok()
        .and_then(|r| r.into_json().ok())
        .unwrap_or_default();

    // Fallback: if no versions matched strict filter, fetch the latest available version
    if versions.is_empty() {
        let mut fallback_req = agent().get(&url);
        if kind == Kind::Mod && !loader.is_empty() {
            fallback_req = fallback_req.query("loaders", &format!("[\"{loader}\"]"));
        }
        versions = fallback_req
            .call()
            .map_err(|e| format!("Modrinth: {e}"))?
            .into_json()
            .map_err(|e| format!("Modrinth: {e}"))?;
    }

    let file = versions
        .into_iter()
        .next()
        .and_then(|v| {
            let primary = v.files.iter().position(|f| f.primary).unwrap_or(0);
            v.files.into_iter().nth(primary)
        })
        .ok_or("Нет подходящей версии на Modrinth")?;

    let name = Path::new(&file.filename)
        .file_name()
        .ok_or("Некорректное имя файла")?
        .to_string_lossy()
        .to_string();

    fs::create_dir_all(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    let target: PathBuf = dest.join(&name);
    let part = dest.join(format!("{name}.part"));

    let mut reader = agent()
        .get(&file.url)
        .call()
        .map_err(|e| format!("Скачивание: {e}"))?
        .into_reader();
    let mut out = fs::File::create(&part).map_err(|e| format!("{}: {e}", part.display()))?;
    std::io::copy(&mut reader, &mut out).map_err(|e| format!("Скачивание: {e}"))?;
    drop(out);
    fs::rename(&part, &target).map_err(|e| format!("{}: {e}", target.display()))?;

    Ok(name)
}

pub fn fetch_icon(url: &str) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    agent()
        .get(url)
        .call()
        .map_err(|e| format!("{e}"))?
        .into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| format!("{e}"))?;
    Ok(buf)
}
