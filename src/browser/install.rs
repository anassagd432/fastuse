//! `fastuse browser-install`: puts a stable copy of this program where the browser can start it as
//! a native host and tells Chrome, Brave and Edge about it. `browser-uninstall` undoes that.
//! Only the extensions listed in `allowed_origins` can start the host (the browser enforces it).

use std::path::PathBuf;

use serde_json::json;

use super::data_dir;

pub const HOST_NAME: &str = "com.fastuse.browser";
/// The unpacked development extension (its ID is fixed by the `key` in extension/manifest.json).
/// Set FASTUSE_EXTENSION_ID to add another one, e.g. the Chrome Web Store ID later.
pub const EXTENSION_ID: &str = "andnpcpahlilpjlipaogpekbnljedfcb";
const EXTENSION_IDS: &[&str] = &[EXTENSION_ID];

const EXTENSION_FILES: &[(&str, &str)] = &[
    ("manifest.json", include_str!("../../extension/manifest.json")),
    ("background.js", include_str!("../../extension/background.js")),
    ("actions.js", include_str!("../../extension/actions.js")),
    ("reader.js", include_str!("../../extension/reader.js")),
    ("popup.html", include_str!("../../extension/popup.html")),
    ("popup.js", include_str!("../../extension/popup.js")),
];

pub fn extension_dir() -> PathBuf {
    data_dir().join("extension")
}

pub fn unpack_extension() -> Result<PathBuf, String> {
    let dir = extension_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    for (name, content) in EXTENSION_FILES {
        let dest = dir.join(name);
        std::fs::write(&dest, content).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
    }
    Ok(dir)
}

fn manifest_path() -> PathBuf {
    data_dir().join(format!("{HOST_NAME}.json"))
}

fn origins() -> Vec<String> {
    let mut v: Vec<String> = EXTENSION_IDS.iter().map(|i| format!("chrome-extension://{i}/")).collect();
    if let Ok(extra) = std::env::var("FASTUSE_EXTENSION_ID") {
        let id = extra.trim();
        if id.len() == 32 && id.chars().all(|c| c.is_ascii_lowercase()) {
            v.push(format!("chrome-extension://{id}/"));
        }
    }
    v
}

/// FNV-1a over the file, so each build gets its own copy and a running host never blocks an update.
fn fingerprint(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", (h >> 32) as u32 ^ h as u32)
}

#[cfg(windows)]
const BROWSER_KEYS: &[(&str, &str)] = &[
    ("Chrome", r"Software\Google\Chrome\NativeMessagingHosts"),
    ("Brave", r"Software\BraveSoftware\Brave-Browser\NativeMessagingHosts"),
    ("Edge", r"Software\Microsoft\Edge\NativeMessagingHosts"),
];

#[cfg(windows)]
const BROWSER_EXT_KEYS: &[(&str, &str)] = &[
    ("Chrome", r"Software\Google\Chrome\Extensions"),
    ("Brave", r"Software\BraveSoftware\Brave-Browser\Extensions"),
    ("Edge", r"Software\Microsoft\Edge\Extensions"),
];

#[cfg(target_os = "macos")]
const BROWSER_DIRS: &[(&str, &str)] = &[
    ("Chrome", "Library/Application Support/Google/Chrome/NativeMessagingHosts"),
    ("Brave", "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts"),
    ("Edge", "Library/Application Support/Microsoft Edge/NativeMessagingHosts"),
    ("Chromium", "Library/Application Support/Chromium/NativeMessagingHosts"),
];

#[cfg(target_os = "macos")]
const BROWSER_EXT_DIRS: &[(&str, &str)] = &[
    ("Chrome", "Library/Application Support/Google/Chrome/External Extensions"),
    ("Brave", "Library/Application Support/BraveSoftware/Brave-Browser/External Extensions"),
    ("Edge", "Library/Application Support/Microsoft Edge/External Extensions"),
    ("Chromium", "Library/Application Support/Chromium/External Extensions"),
];

#[cfg(all(unix, not(target_os = "macos")))]
const BROWSER_DIRS: &[(&str, &str)] = &[
    ("Chrome", ".config/google-chrome/NativeMessagingHosts"),
    ("Brave", ".config/BraveSoftware/Brave-Browser/NativeMessagingHosts"),
    ("Edge", ".config/microsoft-edge/NativeMessagingHosts"),
    ("Chromium", ".config/chromium/NativeMessagingHosts"),
];

#[cfg(all(unix, not(target_os = "macos")))]
const BROWSER_EXT_DIRS: &[(&str, &str)] = &[
    ("Chrome", ".config/google-chrome/External Extensions"),
    ("Brave", ".config/BraveSoftware/Brave-Browser/External Extensions"),
    ("Edge", ".config/microsoft-edge/External Extensions"),
    ("Chromium", ".config/chromium/External Extensions"),
];

pub fn install() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let bytes = std::fs::read(&exe).map_err(|e| format!("cannot read {}: {e}", exe.display()))?;
    let bin_dir = data_dir().join("bin");
    std::fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
    let ext = if cfg!(windows) { ".exe" } else { "" };
    let dest = bin_dir.join(format!("fastuse-{}{ext}", fingerprint(&bytes)));
    if !dest.exists() {
        std::fs::write(&dest, &bytes).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
        }
    }
    // older copies can go; one that a browser still runs just stays until it exits
    for e in std::fs::read_dir(&bin_dir).into_iter().flatten().flatten() {
        if e.path() != dest {
            let _ = std::fs::remove_file(e.path());
        }
    }

    let manifest = json!({
        "name": HOST_NAME,
        "description": "fastuse: lets your AI agent use this browser on the sites you allow",
        "path": dest.display().to_string(),
        "type": "stdio",
        "allowed_origins": origins(),
    });
    let mp = manifest_path();
    std::fs::write(&mp, serde_json::to_string_pretty(&manifest).unwrap()).map_err(|e| e.to_string())?;

    let ext_dir = unpack_extension()?;
    let mut report = format!("host program: {}\nmanifest:     {}\nextension:    {}\n", dest.display(), mp.display(), ext_dir.display());
    #[cfg(windows)]
    for (name, key) in BROWSER_KEYS {
        let full = format!(r"HKCU\{key}\{HOST_NAME}");
        let ok = std::process::Command::new("reg")
            .args(["add", &full, "/ve", "/t", "REG_SZ", "/d", &mp.display().to_string(), "/f"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        report += &format!("{name} host: {}\n", if ok { "registered" } else { "FAILED to register" });
    }
    #[cfg(windows)]
    for (name, key) in BROWSER_EXT_KEYS {
        let full = format!(r"HKCU\{key}\{EXTENSION_ID}");
        let ok = std::process::Command::new("reg")
            .args(["add", &full, "/v", "path", "/t", "REG_SZ", "/d", &ext_dir.display().to_string(), "/f"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let _ = std::process::Command::new("reg")
            .args(["add", &full, "/v", "version", "/t", "REG_SZ", "/d", "0.0.1", "/f"])
            .output();
        report += &format!("{name} ext:  {}\n", if ok { "registered" } else { "skipped" });
    }
    #[cfg(unix)]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME is not set")?;
        for (name, rel) in BROWSER_DIRS {
            let dir = PathBuf::from(&home).join(rel);
            // only for browsers that exist, so we do not litter folders of browsers that are not installed
            let browser_root = dir.parent().map(|p| p.to_path_buf()).unwrap_or_default();
            if !browser_root.exists() {
                report += &format!("{name}: not installed, skipped\n");
                continue;
            }
            let ok = std::fs::create_dir_all(&dir).is_ok() && std::fs::copy(&mp, dir.join(format!("{HOST_NAME}.json"))).is_ok();
            report += &format!("{name} host: {}\n", if ok { "registered" } else { "FAILED to register" });
        }
        let ext_json = json!({
            "external_path": ext_dir.display().to_string(),
        });
        for (name, rel) in BROWSER_EXT_DIRS {
            let dir = PathBuf::from(&home).join(rel);
            let browser_root = dir.parent().map(|p| p.to_path_buf()).unwrap_or_default();
            if browser_root.exists() {
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::fs::write(dir.join(format!("{EXTENSION_ID}.json")), serde_json::to_string_pretty(&ext_json).unwrap());
                report += &format!("{name} ext:  registered\n");
            }
        }
    }
    report += &format!("\nExtension installed to {}.\nTo launch your browser with the extension immediately loaded, run:\n  fastuse browser-open", ext_dir.display());
    Ok(report)
}

pub fn open_browser(url: Option<&str>) -> Result<String, String> {
    let _ = install();
    let ext_dir = extension_dir();
    let arg_ext = format!("--load-extension={}", ext_dir.display());

    #[cfg(windows)]
    {
        let local_app = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let candidates = [
            ("Brave", r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe".to_string()),
            ("Brave (Local)", format!(r"{local_app}\BraveSoftware\Brave-Browser\Application\brave.exe")),
            ("Chrome", r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string()),
            ("Chrome (x86)", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe".to_string()),
            ("Chrome (Local)", format!(r"{local_app}\Google\Chrome\Application\chrome.exe")),
            ("Edge", r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".to_string()),
            ("Edge (64)", r"C:\Program Files\Microsoft\Edge\Application\msedge.exe".to_string()),
        ];
        for (name, path) in candidates {
            let p = std::path::Path::new(&path);
            if p.exists() {
                let mut cmd = std::process::Command::new(p);
                cmd.arg(&arg_ext);
                if let Some(u) = url {
                    cmd.arg(u);
                }
                cmd.spawn().map_err(|e| format!("failed to launch {name}: {e}"))?;
                return Ok(format!("Launched {name} with fastuse extension from {}", ext_dir.display()));
            }
        }
        Err("No Chromium browser (Brave, Chrome, Edge) found in standard locations".into())
    }

    #[cfg(target_os = "macos")]
    {
        let apps = [
            ("Brave", "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
            ("Chrome", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            ("Edge", "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
        ];
        for (name, path) in apps {
            if std::path::Path::new(path).exists() {
                let mut cmd = std::process::Command::new(path);
                cmd.arg(&arg_ext);
                if let Some(u) = url {
                    cmd.arg(u);
                }
                cmd.spawn().map_err(|e| format!("failed to launch {name}: {e}"))?;
                return Ok(format!("Launched {name} with fastuse extension from {}", ext_dir.display()));
            }
        }
        Err("No Chromium browser (Brave, Chrome, Edge) found in /Applications".into())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for bin in ["brave-browser", "google-chrome", "chromium", "microsoft-edge"] {
            let mut cmd = std::process::Command::new(bin);
            cmd.arg(&arg_ext);
            if let Some(u) = url {
                cmd.arg(u);
            }
            if cmd.spawn().is_ok() {
                return Ok(format!("Launched {bin} with fastuse extension from {}", ext_dir.display()));
            }
        }
        Err("No Chromium browser command found on PATH".into())
    }
}

pub fn uninstall() -> Result<String, String> {
    let mut report = String::new();
    #[cfg(windows)]
    for (name, key) in BROWSER_KEYS {
        let full = format!(r"HKCU\{key}\{HOST_NAME}");
        let ok = std::process::Command::new("reg").args(["delete", &full, "/f"]).output().map(|o| o.status.success()).unwrap_or(false);
        report += &format!("{name}: {}\n", if ok { "unregistered" } else { "was not registered" });
    }
    #[cfg(unix)]
    if let Ok(home) = std::env::var("HOME") {
        for (name, rel) in BROWSER_DIRS {
            let f = PathBuf::from(&home).join(rel).join(format!("{HOST_NAME}.json"));
            report += &format!("{name}: {}\n", if std::fs::remove_file(f).is_ok() { "unregistered" } else { "was not registered" });
        }
    }
    let _ = std::fs::remove_file(manifest_path());
    let _ = std::fs::remove_dir_all(data_dir().join("bin"));
    Ok(report)
}

/// One line for `fastuse doctor`.
pub fn status() -> String {
    let mp = manifest_path();
    match std::fs::read_to_string(&mp) {
        Ok(s) => {
            let path = serde_json::from_str::<serde_json::Value>(&s)
                .ok()
                .and_then(|v| v["path"].as_str().map(String::from))
                .unwrap_or_default();
            if std::path::Path::new(&path).exists() {
                format!("installed ({})", mp.display())
            } else {
                format!("manifest points to a missing program ({path}); run `fastuse browser-install`")
            }
        }
        Err(_) => "not installed; run `fastuse browser-install`".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_differs_per_content() {
        assert_ne!(fingerprint(b"a"), fingerprint(b"b"));
        assert_eq!(fingerprint(b"abc"), fingerprint(b"abc"));
    }

    #[test]
    fn origins_have_the_trailing_slash_chrome_requires() {
        assert!(origins().iter().all(|o| o.starts_with("chrome-extension://") && o.ends_with('/')));
    }
}
