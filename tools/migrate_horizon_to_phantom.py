#!/usr/bin/env python3
"""
Migrates the entire codebase from `horizon-*` to `phantom-*`:
- Renames crates in crates/
- Updates Cargo.toml files (root, crates, launcher, xtask)
- Updates Rust sources, JSON configs, native-hooks, mod-api-gen templates
- Preserves terrain.wgsl, Frustum::intersects, and occlusion.rs math
"""

import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# 1. Crates mapping
CRATE_DIRS = [d.name for d in (ROOT / "crates").iterdir() if d.is_dir()]
CRATE_MAP = {}
for c in CRATE_DIRS:
    if c == "cargo-phantom-mod":
        CRATE_MAP[c] = "cargo-phantom-mod"
    elif c.startswith("horizon-"):
        CRATE_MAP[c] = "phantom-" + c[len("horizon-"):]

MODULE_MAP = {k.replace("-", "_"): v.replace("-", "_") for k, v in CRATE_MAP.items()}

# Also explicitly include phantom_client / phantom_client and any common substrings
# Sort by length descending so longer matches take precedence
SORTED_CRATE_REPLACEMENTS = sorted(
    CRATE_MAP.items(), key=lambda x: len(x[0]), reverse=True
)
SORTED_MODULE_REPLACEMENTS = sorted(
    MODULE_MAP.items(), key=lambda x: len(x[0]), reverse=True
)

FORBIDDEN_FILES = {
    "terrain.wgsl",
}

def should_process_file(path: Path) -> bool:
    if path.name in FORBIDDEN_FILES:
        return False
    # Avoid .git, target, .local, target_*, node_modules
    parts = set(path.parts)
    if any(p in parts for p in [".git", "target", ".local", "fast_pack_tmp"]):
        return False
    # Process text files: .rs, .toml, .json, .md, .py, .manifest, .txt
    ext = path.suffix.lower()
    return ext in [".rs", ".toml", ".json", ".md", ".py", ".manifest", ".txt", ""]

def replace_in_content(content: str, file_path: Path) -> str:
    # Special safety check: do not touch Frustum::intersects math or terrain.wgsl
    if file_path.name == "terrain.wgsl":
        return content

    new_content = content

    # 1. Replace crate folder paths in strings / Cargo.toml
    # e.g. path = "../phantom-client" -> path = "../phantom-client"
    # "crates/phantom-client" -> "crates/phantom-client"
    for old_crate, new_crate in SORTED_CRATE_REPLACEMENTS:
        new_content = re.sub(
            r'\b' + re.escape(old_crate) + r'\b', new_crate, new_content
        )

    # 2. Replace rust module names
    # e.g. phantom_core:: -> phantom_core::
    # phantom_client:: -> phantom_client::
    for old_mod, new_mod in SORTED_MODULE_REPLACEMENTS:
        new_content = re.sub(
            r'\b' + re.escape(old_mod) + r'\b', new_mod, new_content
        )

    # 3. Common specific identifiers / strings
    # "Phantom:" log prefix -> "Phantom:"
    new_content = new_content.replace('"Phantom:', '"Phantom:')
    new_content = new_content.replace('Phantom: ', 'Phantom: ')
    new_content = new_content.replace('phantom_trait_dispatch', 'phantom_trait_dispatch')
    new_content = new_content.replace('PHANTOM_MOD_API_INJECTIONS', 'PHANTOM_MOD_API_INJECTIONS')
    new_content = new_content.replace('PHANTOM_SPIKES', 'PHANTOM_SPIKES')
    new_content = new_content.replace('phantom-ctl', 'phantom-ctl')
    new_content = new_content.replace('phantom_ctl', 'phantom_ctl')

    return new_content

def process_all_files():
    print("[*] Processing file contents...")
    count = 0
    modified = 0

    scan_dirs = [
        ROOT / "crates",
        ROOT / "xtask",
        ROOT / "launcher",
        ROOT / "phantom-launcher",
        ROOT / "tools",
        ROOT / "tests",
    ]

    files_to_check = [
        ROOT / "Cargo.toml",
        ROOT / "Cargo.lock",
        ROOT / "native-hooks.json",
    ]

    for d in scan_dirs:
        if d.is_dir():
            for root, _, files in os.walk(d):
                for f in files:
                    files_to_check.append(Path(root) / f)

    for p in files_to_check:
        if not p.is_file():
            continue
        if not should_process_file(p):
            continue

        count += 1
        try:
            with open(p, "r", encoding="utf-8") as f:
                old_text = f.read()
        except Exception:
            try:
                with open(p, "r", encoding="latin-1") as f:
                    old_text = f.read()
            except Exception:
                continue

        new_text = replace_in_content(old_text, p)
        if new_text != old_text:
            with open(p, "w", encoding="utf-8") as f:
                f.write(new_text)
            modified += 1

    print(f"[+] Checked {count} files, updated {modified} files.")

def rename_internal_bins():
    print("[*] Checking internal bin directories...")
    # e.g. crates/phantom-client/src/bin/phantom-ctl -> phantom-ctl
    # could be either under phantom-client or phantom-client
    for c_dir in ["phantom-client", "phantom-client"]:
        ctl_dir = ROOT / "crates" / c_dir / "src" / "bin" / "phantom-ctl"
        if ctl_dir.is_dir():
            target = ROOT / "crates" / c_dir / "src" / "bin" / "phantom-ctl"
            print(f"[+] Renaming {ctl_dir} -> {target}")
            ctl_dir.rename(target)

def rename_crates_dirs():
    print("[*] Renaming crates directories...")
    crates_dir = ROOT / "crates"
    for old_name, new_name in CRATE_MAP.items():
        old_path = crates_dir / old_name
        new_path = crates_dir / new_name
        if old_path.is_dir() and old_path != new_path:
            print(f"[+] Renaming {old_name} -> {new_name}")
            old_path.rename(new_path)

if __name__ == "__main__":
    rename_internal_bins()
    process_all_files()
    rename_crates_dirs()
    print("[✓] Migration script completed successfully.")
