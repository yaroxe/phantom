//! Virtual asset archive reader: reads `assets.tar` directly without unpacking.
//!
//! Windows Defender and NTFS handle 15,000 loose files very slowly. Packing
//! `assets/` into an uncompressed `assets.tar` allows startup in ~20 ms with
//! random parallel seeks via `FileExt::seek_read`.

use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
};

pub const ARCHIVE_NAME: &str = "assets.tar";

fn parse_octal(bytes: &[u8]) -> Option<u64> {
    let s = std::str::from_utf8(bytes).ok()?;
    let trimmed = s.trim_matches(|c: char| c == ' ' || c == '\0');
    if trimmed.is_empty() {
        return Some(0);
    }
    u64::from_str_radix(trimmed, 8).ok()
}

#[cfg(unix)]
fn read_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<()> {
    std::os::unix::fs::FileExt::read_exact_at(file, buf, offset)
}

#[cfg(windows)]
fn read_at(file: &File, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
    while !buf.is_empty() {
        match std::os::windows::fs::FileExt::seek_read(file, buf, offset) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(n) => {
                buf = &mut buf[n..];
                offset += n as u64;
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn read_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<()> {
    static LOCK: Mutex<()> = Mutex::new(());
    let _held = LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let mut file = file;
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(buf)
}

/// An open virtual asset archive (`assets.tar`).
pub struct AssetArchive {
    root: PathBuf,
    archive_path: PathBuf,
    file: File,
    /// Sorted list of `(normalized_path, offset, size)`.
    /// `normalized_path` always starts with `assets/` and uses `/`.
    entries: Box<[(String, u64, u64)]>,
    namespaces: Box<[String]>,
}

impl AssetArchive {
    /// Opens `assets.tar` under `root` if it exists.
    pub fn open(root: &Path) -> Option<Self> {
        let archive_path = if root.join(ARCHIVE_NAME).is_file() {
            root.join(ARCHIVE_NAME)
        } else if root.join("assets").join(ARCHIVE_NAME).is_file() {
            root.join("assets").join(ARCHIVE_NAME)
        } else {
            return None;
        };

        #[cfg(windows)]
        let mut file = {
            use std::os::windows::fs::OpenOptionsExt;
            let mut opts = fs::OpenOptions::new();
            opts.read(true);
            // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE (7) allows deleting/renaming
            // cache directories on Windows without access violation errors.
            opts.share_mode(7);
            opts.open(&archive_path).ok()?
        };
        #[cfg(not(windows))]
        let mut file = File::open(&archive_path).ok()?;
        let mut entries = Vec::new();
        let mut namespaces_set = BTreeSet::new();

        let mut header = [0u8; 512];
        let mut current_offset: u64 = 0;
        let mut pending_long_name: Option<String> = None;
        let mut zero_blocks = 0;

        loop {
            if file.read_exact(&mut header).is_err() {
                break;
            }
            current_offset += 512;

            if header.iter().all(|&b| b == 0) {
                zero_blocks += 1;
                if zero_blocks >= 2 {
                    break;
                }
                continue;
            }
            zero_blocks = 0;

            let typeflag = header[156];
            let size = parse_octal(&header[124..136]).unwrap_or(0);
            let padded_size = (size + 511) / 512 * 512;

            if typeflag == b'L' {
                // GNU LongLink
                let mut name_buf = vec![0u8; size as usize];
                if file.read_exact(&mut name_buf).is_ok() {
                    let name_str = String::from_utf8_lossy(&name_buf)
                        .trim_matches('\0')
                        .to_string();
                    pending_long_name = Some(name_str);
                }
                let skip = padded_size.saturating_sub(size);
                if skip > 0 && file.seek(SeekFrom::Current(skip as i64)).is_err() {
                    break;
                }
                current_offset += padded_size;
                continue;
            }

            if typeflag == b'x' {
                // PAX extended header
                let mut pax_buf = vec![0u8; size as usize];
                if file.read_exact(&mut pax_buf).is_ok() {
                    let text = String::from_utf8_lossy(&pax_buf);
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if let Some(space_idx) = trimmed.find(' ') {
                            let kv = &trimmed[space_idx + 1..];
                            if let Some((k, v)) = kv.split_once('=') {
                                if k == "path" {
                                    pending_long_name = Some(v.to_string());
                                }
                            }
                        }
                    }
                }
                let skip = padded_size.saturating_sub(size);
                if skip > 0 && file.seek(SeekFrom::Current(skip as i64)).is_err() {
                    break;
                }
                current_offset += padded_size;
                continue;
            }

            // Normal file entry
            if typeflag == b'0' || typeflag == 0 {
                let raw_name = if let Some(long) = pending_long_name.take() {
                    long
                } else {
                    let name_end = header[0..100].iter().position(|&b| b == 0).unwrap_or(100);
                    let name = String::from_utf8_lossy(&header[0..name_end]).to_string();

                    let prefix_end = header[345..500].iter().position(|&b| b == 0).unwrap_or(155);
                    let prefix = String::from_utf8_lossy(&header[345..345 + prefix_end]).to_string();

                    if prefix.is_empty() {
                        name
                    } else {
                        format!("{prefix}/{name}")
                    }
                };

                let norm = raw_name
                    .replace('\\', "/")
                    .trim_start_matches("./")
                    .trim_start_matches('/')
                    .to_string();

                let with_assets = if norm.starts_with("assets/") {
                    norm
                } else {
                    format!("assets/{norm}")
                };

                // Extract namespace (e.g. assets/<namespace>/...)
                if let Some(rest) = with_assets.strip_prefix("assets/") {
                    if let Some((ns, _)) = rest.split_once('/') {
                        if !ns.is_empty() {
                            namespaces_set.insert(ns.to_string());
                        }
                    }
                }

                entries.push((with_assets, current_offset, size));

                if padded_size > 0 && file.seek(SeekFrom::Current(padded_size as i64)).is_err() {
                    break;
                }
                current_offset += padded_size;
            } else {
                pending_long_name = None;
                if padded_size > 0 && file.seek(SeekFrom::Current(padded_size as i64)).is_err() {
                    break;
                }
                current_offset += padded_size;
            }
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let namespaces: Vec<String> = namespaces_set.into_iter().collect();

        Some(Self {
            root: root.to_owned(),
            archive_path,
            file,
            entries: entries.into_boxed_slice(),
            namespaces: namespaces.into_boxed_slice(),
        })
    }

    /// Namespaces discovered in `assets.tar` (e.g. `["minecraft"]`).
    pub fn namespaces(&self) -> Vec<String> {
        self.namespaces.to_vec()
    }

    /// Path to the archive file on disk.
    pub fn archive_path(&self) -> &Path {
        &self.archive_path
    }

    /// Resolves candidate normalized paths inside the archive for `path`.
    fn candidate_keys(&self, path: &str) -> Vec<String> {
        let clean = path
            .replace('\\', "/")
            .trim_start_matches("./")
            .trim_start_matches('/')
            .to_string();

        let mut candidates = Vec::with_capacity(4);

        if clean.starts_with("assets/") {
            candidates.push(clean.clone());
        } else if let Some((ns, sub)) = clean.split_once(':') {
            candidates.push(format!("assets/{ns}/{sub}"));
        } else {
            candidates.push(format!("assets/{clean}"));
            candidates.push(format!("assets/minecraft/{clean}"));
        }

        candidates
    }

    /// Looks up `(offset, size)` for `path`.
    pub fn find_entry(&self, path: &str) -> Option<(u64, u64)> {
        for candidate in self.candidate_keys(path) {
            if let Ok(idx) = self.entries.binary_search_by(|e| e.0.as_str().cmp(&candidate)) {
                return Some((self.entries[idx].1, self.entries[idx].2));
            }
        }
        None
    }

    /// Reads an asset: checks loose disk file first, then reads from `assets.tar`.
    pub fn read(&self, rel_path: &str) -> Option<Vec<u8>> {
        // 1. Loose file on disk has highest priority
        let clean = rel_path.replace('\\', "/").trim_start_matches('/').to_string();
        let loose1 = self.root.join(&clean);
        if loose1.is_file() {
            if let Ok(bytes) = fs::read(&loose1) {
                return Some(bytes);
            }
        }
        let loose2 = self.root.join("assets").join(&clean);
        if loose2.is_file() {
            if let Ok(bytes) = fs::read(&loose2) {
                return Some(bytes);
            }
        }
        let loose3 = self.root.join("assets/minecraft").join(&clean);
        if loose3.is_file() {
            if let Ok(bytes) = fs::read(&loose3) {
                return Some(bytes);
            }
        }

        // 2. Read directly from assets.tar
        self.read_from_archive(rel_path)
    }

    /// Reads directly from `assets.tar` without checking loose disk files.
    pub fn read_from_archive(&self, rel_path: &str) -> Option<Vec<u8>> {
        let (offset, size) = self.find_entry(rel_path)?;
        let mut buf = vec![0u8; size as usize];
        read_at(&self.file, &mut buf, offset).ok()?;
        Some(buf)
    }

    /// Reads asset as UTF-8 string.
    pub fn read_text(&self, rel_path: &str) -> Option<String> {
        let bytes = self.read(rel_path)?;
        String::from_utf8(bytes).ok()
    }

    /// Checks whether the asset exists on disk or in `assets.tar`.
    pub fn exists(&self, rel_path: &str) -> bool {
        let clean = rel_path.replace('\\', "/").trim_start_matches('/').to_string();
        if self.root.join(&clean).is_file()
            || self.root.join("assets").join(&clean).is_file()
            || self.root.join("assets/minecraft").join(&clean).is_file()
        {
            return true;
        }
        self.find_entry(rel_path).is_some()
    }

    /// Lists all files in the archive matching `prefix` (e.g. `"assets/minecraft/textures/"`).
    pub fn list(&self, prefix: &str) -> Vec<String> {
        let norm_prefix = prefix
            .replace('\\', "/")
            .trim_start_matches("./")
            .trim_start_matches('/')
            .to_string();

        let search_prefix = if norm_prefix.starts_with("assets/") {
            norm_prefix
        } else {
            format!("assets/{norm_prefix}")
        };

        let start = self
            .entries
            .partition_point(|e| e.0.as_str() < search_prefix.as_str());

        let mut results_set: BTreeSet<String> = self.entries[start..]
            .iter()
            .take_while(|e| e.0.starts_with(&search_prefix))
            .map(|e| e.0.clone())
            .collect();

        // Also merge loose files if present on disk
        let disk_dir = self.root.join(&search_prefix);
        if disk_dir.is_dir() {
            let mut stack = vec![disk_dir];
            while let Some(dir) = stack.pop() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            stack.push(path);
                        } else if path.is_file() {
                            if let Ok(rel) = path.strip_prefix(&self.root) {
                                let s = rel.to_string_lossy().replace('\\', "/");
                                results_set.insert(s);
                            }
                        }
                    }
                }
            }
        }

        results_set.into_iter().collect()
    }
}

// ---------------------------------------------------------------- Global helpers & Cache

type OpenedArchives = Vec<(PathBuf, Option<Arc<AssetArchive>>)>;
static OPENED: Mutex<OpenedArchives> = Mutex::new(Vec::new());

/// Invalidates cached archive handles for `root` so its folder can be removed or rebuilt.
pub fn invalidate_cache(root: &Path) {
    let mut opened = OPENED.lock().unwrap_or_else(PoisonError::into_inner);
    opened.retain(|(r, _)| r != root && !r.starts_with(root));
}

/// Opens or retrieves cached `AssetArchive` for `root`.
pub fn open_cached(root: &Path) -> Option<Arc<AssetArchive>> {
    let mut opened = OPENED.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some((_, archive)) = opened.iter().find(|(r, _)| r == root) {
        return archive.clone();
    }
    let archive = AssetArchive::open(root).map(Arc::new);
    opened.push((root.to_owned(), archive.clone()));
    archive
}

/// Reads an asset relative to `root`. Checks loose files, then `assets.tar`.
pub fn read_asset(root: &Path, rel_path: &str) -> Option<Vec<u8>> {
    if let Some(archive) = open_cached(root) {
        archive.read(rel_path)
    } else {
        let clean = rel_path.replace('\\', "/").trim_start_matches('/').to_string();
        fs::read(root.join(&clean))
            .or_else(|_| fs::read(root.join("assets").join(&clean)))
            .or_else(|_| fs::read(root.join("assets/minecraft").join(&clean)))
            .ok()
    }
}

/// Reads an asset as text relative to `root`.
pub fn read_asset_text(root: &Path, rel_path: &str) -> Option<String> {
    let bytes = read_asset(root, rel_path)?;
    String::from_utf8(bytes).ok()
}

/// Checks whether an asset exists.
pub fn asset_exists(root: &Path, rel_path: &str) -> bool {
    if let Some(archive) = open_cached(root) {
        archive.exists(rel_path)
    } else {
        let clean = rel_path.replace('\\', "/").trim_start_matches('/').to_string();
        root.join(&clean).is_file()
            || root.join("assets").join(&clean).is_file()
            || root.join("assets/minecraft").join(&clean).is_file()
    }
}

/// Lists assets under `prefix`.
pub fn list_assets(root: &Path, prefix: &str) -> Vec<String> {
    if let Some(archive) = open_cached(root) {
        archive.list(prefix)
    } else {
        Vec::new()
    }
}

/// Helper that checks all ancestors of `path` to find a parent with `assets.tar`.
pub fn read_by_path(path: &Path) -> Option<Vec<u8>> {
    for ancestor in path.ancestors().skip(1) {
        if let Some(archive) = open_cached(ancestor) {
            if let Ok(rel) = path.strip_prefix(ancestor) {
                let rel_str = rel.to_str()?.replace(std::path::MAIN_SEPARATOR, "/");
                if let Some(data) = archive.read_from_archive(&rel_str) {
                    return Some(data);
                }
            }
        }
    }
    None
}

/// Helper to list files under `dir` when `dir` is inside `assets.tar`.
pub fn files_under_path(dir: &Path) -> Option<Vec<String>> {
    for ancestor in dir.ancestors().skip(1) {
        if let Some(archive) = open_cached(ancestor) {
            if let Ok(rel) = dir.strip_prefix(ancestor) {
                let rel_str = rel.to_str()?.replace(std::path::MAIN_SEPARATOR, "/");
                let list = archive.list(&rel_str);
                if !list.is_empty() {
                    let prefix = if rel_str.ends_with('/') {
                        rel_str
                    } else {
                        format!("{rel_str}/")
                    };
                    let sub: Vec<String> = list
                        .into_iter()
                        .filter_map(|p| p.strip_prefix(&prefix).map(str::to_owned))
                        .collect();
                    return Some(sub);
                }
            }
        }
    }
    None
}

/// Discovers namespaces under `assets_dir`.
pub fn namespaces_under_path(assets_dir: &Path) -> Option<Vec<String>> {
    for ancestor in assets_dir.ancestors() {
        if let Some(archive) = open_cached(ancestor) {
            let ns = archive.namespaces();
            if !ns.is_empty() {
                return Some(ns);
            }
        }
    }
    None
}
