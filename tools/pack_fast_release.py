import os
import tarfile
import zipfile
import shutil

def main():
    appdata = os.environ.get("APPDATA", "")
    src_local = os.path.join(appdata, "phantom-launcher", "versions", "nightly-2026-11-05-66f3981", ".local")
    
    desktop = os.path.join(os.environ.get("USERPROFILE", ""), "Desktop")
    out_zip = os.path.join(desktop, "Phantom-v1.0-Instant.zip")
    
    client_res = os.path.join(src_local, "phantom-client", "26.3")
    assets_dir = os.path.join(client_res, "assets")
    assets_tar = os.path.join(client_res, "assets.tar")
    
    # Exclude music & records from assets.tar
    exclude_dirs = {"music", "records", "sounds/music", "sounds/records"}
    
    def asset_filter(tarinfo):
        name = tarinfo.name.replace("\\", "/")
        if name.endswith(".log") or name.endswith(".tmp") or name.endswith(".lock"):
            return None
        for ed in exclude_dirs:
            if f"/{ed}/" in f"/{name}/" or name.endswith(f"/{ed}"):
                return None
        return tarinfo

    print(f"[*] Packaging assets -> {assets_tar}...")
    if os.path.isdir(assets_dir):
        with tarfile.open(assets_tar, "w") as tar:
            # Store files as assets/...
            tar.add(assets_dir, arcname="assets", filter=asset_filter)
        print(f"[+] assets.tar created: {os.path.getsize(assets_tar) / (1024*1024):.1f} MB")
    elif os.path.isfile(assets_tar):
        print(f"[+] Using existing assets.tar: {os.path.getsize(assets_tar) / (1024*1024):.1f} MB")
    else:
        print("[!] No assets directory or assets.tar found!")
        
    tmp_dir = os.path.join(os.getcwd(), "target", "fast_pack_tmp")
    os.makedirs(tmp_dir, exist_ok=True)
    tar_path = os.path.join(tmp_dir, "resources.tar")
    print(f"[*] Packaging core resources -> {tar_path}...")
    
    def resource_filter(tarinfo):
        name = tarinfo.name.replace("\\", "/")
        # DO NOT include loose assets/ files in resources.tar! Only assets.tar is needed!
        if "/phantom-client/26.3/assets/" in name:
            return None
        if name.endswith("/phantom-client/26.3/assets"):
            return None
        if name.endswith(".log") or name.endswith(".tmp") or name.endswith(".lock"):
            return None
        return tarinfo

    with tarfile.open(tar_path, "w") as tar:
        tar.add(src_local, arcname=".local", filter=resource_filter)
        
    print(f"[+] resources.tar created: {os.path.getsize(tar_path) / (1024*1024):.1f} MB")
    
    launcher_exe = os.path.join(os.getcwd(), "target", "release", "phantom-launcher.exe")
    client_exe = os.path.join(os.getcwd(), "target", "release", "phantom-client.exe")
    
    if not os.path.isfile(launcher_exe):
        launcher_exe = os.path.join(os.getcwd(), "phantom-launcher", "target", "release", "phantom-launcher.exe")
    if not os.path.isfile(client_exe):
        client_exe = os.path.join(os.getcwd(), "target", "fast", "phantom-client.exe")
        
    print(f"[*] Building 3-file instant zip: {out_zip}...")
    with zipfile.ZipFile(out_zip, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        print(f"  + Adding {os.path.basename(launcher_exe)}...")
        z.write(launcher_exe, arcname="phantom-launcher.exe")
        print(f"  + Adding {os.path.basename(client_exe)}...")
        z.write(client_exe, arcname="phantom-client.exe")
        print(f"  + Adding {os.path.basename(tar_path)}...")
        z.write(tar_path, arcname="resources.tar")
        
    # Clean up temp tar
    try:
        os.remove(tar_path)
        os.rmdir(tmp_dir)
    except Exception:
        pass
        
    print(f"[✓] Успешно собран мгновенный релизный 3-файловый архив: {out_zip} ({os.path.getsize(out_zip) / (1024*1024):.1f} MB)")

if __name__ == "__main__":
    main()
