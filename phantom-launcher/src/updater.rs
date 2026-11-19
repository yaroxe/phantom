//! Auto-updater module for Phantom Launcher via GitHub Releases.
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

pub const CURRENT_VERSION: &str = "v1.2.0";
const GITHUB_API_URL: &str = "https://api.github.com/repos/yaroxe/phantom/releases/latest";
const USER_AGENT: &str = "PhantomLauncher/1.1";

#[derive(Clone, Debug)]
pub struct UpdateInfo {
    pub tag: String,
    pub download_url: String,
    pub asset_name: String,
    pub is_zip: bool,
    pub size_bytes: u64,
}

#[derive(Clone, Debug)]
pub enum UpdateState {
    Idle,
    Available(UpdateInfo),
    Downloading {
        tag: String,
        downloaded: u64,
        total: u64,
    },
    Extracting {
        tag: String,
    },
    ReadyToRestart {
        tag: String,
    },
    Error(String),
}

#[derive(serde::Deserialize, Debug)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(serde::Deserialize, Debug)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Returns current version from version.txt next to exe, or default CURRENT_VERSION.
pub fn get_current_version() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let ver_file = dir.join("version.txt");
            if ver_file.is_file() {
                if let Ok(content) = fs::read_to_string(&ver_file) {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
        }
    }
    let local_file = Path::new("version.txt");
    if local_file.is_file() {
        if let Ok(content) = fs::read_to_string(local_file) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    CURRENT_VERSION.to_string()
}

/// Removes leftover phantom-launcher.old.exe from previous self-update.
pub fn cleanup_old_launcher() {
    if let Ok(exe) = std::env::current_exe() {
        let old1 = exe.with_file_name("phantom-launcher.old.exe");
        if old1.is_file() {
            let _ = fs::remove_file(old1);
        }
        let old2 = exe.with_extension("old.exe");
        if old2.is_file() {
            let _ = fs::remove_file(old2);
        }
    }
    let old3 = Path::new("phantom-launcher.old.exe");
    if old3.is_file() {
        let _ = fs::remove_file(old3);
    }
}

/// Parses version string like "v1.2.0" into (major, minor, patch).
pub fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let s = v.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = if parts.len() > 2 {
            let p_num: String = parts[2].chars().take_while(|c| c.is_ascii_digit()).collect();
            p_num.parse().unwrap_or(0)
        } else {
            0
        };
        Some((major, minor, patch))
    } else {
        None
    }
}

/// Checks whether remote version is strictly newer than current version.
pub fn is_newer_version(remote: &str, current: &str) -> bool {
    match (parse_version(remote), parse_version(current)) {
        (Some(r), Some(c)) => r > c,
        _ => remote.trim() != current.trim(),
    }
}

/// Checks GitHub releases API for updates. Returns None on failure or if up-to-date.
pub fn check_for_updates(current_version: &str) -> Option<UpdateInfo> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .build();

    let resp = agent
        .get(GITHUB_API_URL)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github.v3+json")
        .call()
        .ok()?;

    if resp.status() != 200 {
        return None;
    }

    let rel: GhRelease = resp.into_json().ok()?;
    let tag = rel.tag_name.trim();

    if !is_newer_version(tag, current_version) {
        return None;
    }

    // Prefer .zip asset (e.g. Phantom-v*.zip)
    let zip_asset = rel.assets.iter().find(|a| a.name.ends_with(".zip"));
    if let Some(asset) = zip_asset {
        return Some(UpdateInfo {
            tag: tag.to_string(),
            download_url: asset.browser_download_url.clone(),
            asset_name: asset.name.clone(),
            is_zip: true,
            size_bytes: asset.size,
        });
    }

    // Otherwise check for direct executable assets
    let exe_asset = rel.assets.iter().find(|a| {
        a.name == "phantom-client.exe" || a.name == "phantom-launcher.exe"
    });
    if let Some(asset) = exe_asset {
        return Some(UpdateInfo {
            tag: tag.to_string(),
            download_url: asset.browser_download_url.clone(),
            asset_name: asset.name.clone(),
            is_zip: false,
            size_bytes: asset.size,
        });
    }

    None
}

/// Downloads update with progress callbacks and extracts files.
pub fn perform_update(
    info: &UpdateInfo,
    on_progress: impl Fn(u64, u64),
    on_extracting: impl Fn(),
) -> Result<(), String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(300))
        .build();

    let resp = agent
        .get(&info.download_url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("Ошибка подключения: {e}"))?;

    if resp.status() != 200 {
        return Err(format!("Сервер вернул код {}", resp.status()));
    }

    let total_len = resp
        .header("content-length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(info.size_bytes);

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("phantom_upd_{}.tmp", std::process::id()));

    let res = (|| -> Result<(), String> {
        let mut file = fs::File::create(&temp_path)
            .map_err(|e| format!("Не удалось создать временный файл: {e}"))?;

        let mut reader = resp.into_reader();
        let mut buffer = [0u8; 65536];
        let mut downloaded = 0u64;
        let mut last_progress = 0u64;

        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|e| format!("Ошибка чтения потока: {e}"))?;
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n])
                .map_err(|e| format!("Ошибка записи: {e}"))?;
            downloaded += n as u64;

            if downloaded - last_progress >= 131072 || downloaded == total_len {
                last_progress = downloaded;
                on_progress(downloaded, total_len);
            }
        }
        file.flush().map_err(|e| format!("Ошибка сброса буфера: {e}"))?;
        drop(file);

        on_extracting();

        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        if info.is_zip {
            let zip_file = fs::File::open(&temp_path)
                .map_err(|e| format!("Не удалось открыть zip: {e}"))?;
            let mut archive = zip::ZipArchive::new(zip_file)
                .map_err(|e| format!("Повреждённый zip-архив: {e}"))?;

            for i in 0..archive.len() {
                let mut item = archive
                    .by_index(i)
                    .map_err(|e| format!("Ошибка чтения архива: {e}"))?;
                if item.is_dir() {
                    continue;
                }
                let raw_name = item.name().replace('\\', "/");
                let file_name = raw_name.rsplit('/').next().unwrap_or(&raw_name);

                if file_name == "phantom-client.exe" {
                    let dest = exe_dir.join("phantom-client.exe");
                    let mut out = fs::File::create(&dest)
                        .map_err(|e| format!("Не удалось создать {}: {e}. Закройте игру перед обновлением.", dest.display()))?;
                    std::io::copy(&mut item, &mut out)
                        .map_err(|e| format!("Не удалось записать {}: {e}. Закройте игру перед обновлением.", dest.display()))?;

                    // Mirror to AppData version directory if present
                    let versions_dir = crate::config::data_root()
                        .join("phantom-launcher")
                        .join("versions");
                    if let Ok(entries) = fs::read_dir(&versions_dir) {
                        for e in entries.flatten() {
                            let p = e.path();
                            if p.is_dir() {
                                let v_client = p.join("phantom-client.exe");
                                let _ = fs::copy(&dest, &v_client);
                            }
                        }
                    }
                } else if file_name == "phantom-launcher.exe" {
                    // Windows self-update: rename current exe to .old.exe, then write new
                    if let Ok(curr_exe) = std::env::current_exe() {
                        let old_exe = curr_exe.with_file_name("phantom-launcher.old.exe");
                        let _ = fs::remove_file(&old_exe);
                        let _ = fs::rename(&curr_exe, &old_exe);
                        let mut out = fs::File::create(&curr_exe)
                            .map_err(|e| format!("Не удалось создать {}: {e}", curr_exe.display()))?;
                        std::io::copy(&mut item, &mut out)
                            .map_err(|e| format!("Не удалось записать {}: {e}", curr_exe.display()))?;
                    }
                } else if file_name == "resources.tar" {
                    let dest = exe_dir.join("resources.tar");
                    let mut out = fs::File::create(&dest)
                        .map_err(|e| format!("Не удалось создать {}: {e}", dest.display()))?;
                    std::io::copy(&mut item, &mut out)
                        .map_err(|e| format!("Не удалось записать {}: {e}", dest.display()))?;

                    // Reset unpack marker so resources will be extracted on next play
                    let marker = exe_dir
                        .join(".local")
                        .join("phantom-client")
                        .join("26.3")
                        .join("gameplay-facts.json");
                    let _ = fs::remove_file(marker);
                }
            }
        } else {
            // Direct executable asset update
            if info.asset_name == "phantom-client.exe" {
                let dest = exe_dir.join("phantom-client.exe");
                fs::copy(&temp_path, &dest)
                    .map_err(|e| format!("Не удалось обновить клиент: {e}. Закройте игру перед обновлением."))?;

                let versions_dir = crate::config::data_root()
                    .join("phantom-launcher")
                    .join("versions");
                if let Ok(entries) = fs::read_dir(&versions_dir) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if p.is_dir() {
                            let _ = fs::copy(&dest, p.join("phantom-client.exe"));
                        }
                    }
                }
            } else if info.asset_name == "phantom-launcher.exe" {
                if let Ok(curr_exe) = std::env::current_exe() {
                    let old_exe = curr_exe.with_file_name("phantom-launcher.old.exe");
                    let _ = fs::remove_file(&old_exe);
                    let _ = fs::rename(&curr_exe, &old_exe);
                    fs::copy(&temp_path, &curr_exe)
                        .map_err(|e| format!("Не удалось обновить лаунчер: {e}"))?;
                }
            }
        }

        // Save new version tag
        let _ = fs::write(exe_dir.join("version.txt"), &info.tag);
        let appdata_dir = crate::config::data_root().join("phantom-launcher");
        let _ = fs::create_dir_all(&appdata_dir);
        let _ = fs::write(appdata_dir.join("version.txt"), &info.tag);

        Ok(())
    })();

    let _ = fs::remove_file(&temp_path);
    res
}

/// Spawns new launcher process and exits current one.
pub fn restart_launcher() {
    if let Ok(curr_exe) = std::env::current_exe() {
        let _ = std::process::Command::new(curr_exe).spawn();
    }
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing_and_comparison() {
        assert_eq!(parse_version("v1.1.0"), Some((1, 1, 0)));
        assert_eq!(parse_version("V2.0.1"), Some((2, 0, 1)));
        assert_eq!(parse_version("1.3"), Some((1, 3, 0)));
        assert_eq!(parse_version("v1.2.0-rc1"), Some((1, 2, 0)));

        assert!(is_newer_version("v1.2.0", "v1.1.0"));
        assert!(is_newer_version("v2.0.0", "v1.9.9"));
        assert!(is_newer_version("v1.1.1", "v1.1.0"));
        assert!(!is_newer_version("v1.1.0", "v1.1.0"));
        assert!(!is_newer_version("v1.0.9", "v1.1.0"));
    }
}
