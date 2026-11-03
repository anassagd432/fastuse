//! `fastuse browser-install`: puts a stable copy of this program where the browser can start it as
//! a native host and tells Chrome, Brave and Edge about it. `browser-uninstall` undoes that.
//! Only the extensions listed in `allowed_origins` can start the host (the browser enforces it).

use std::path::PathBuf;

use serde_json::json;

use super::data_dir;

pub const HOST_NAME: &str = "com.fastuse.browser";
/// The unpacked development extension (its ID is fixed by the `key` in extension/manifest.json).
/// Set FASTUSE_EXTENSION_ID to add another one, e.g. the Chrome Web Store ID later.
const EXTENSION_IDS: &[&str] = &["andnpcpahlilpjlipaogpekbnljedfcb"];

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

#[cfg(target_os = "macos")]
const BROWSER_DIRS: &[(&str, &str)] = &[
    ("Chrome", "Library/Application Support/Google/Chrome/NativeMessagingHosts"),
    ("Brave", "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts"),
    ("Edge", "Library/Application Support/Microsoft Edge/NativeMessagingHosts"),
    ("Chromium", "Library/Application Support/Chromium/NativeMessagingHosts"),
];

#[cfg(all(unix, not(target_os = "macos")))]
const BROWSER_DIRS: &[(&str, &str)] = &[
    ("Chrome", ".config/google-chrome/NativeMessagingHosts"),
    ("Brave", ".config/BraveSoftware/Brave-Browser/NativeMessagingHosts"),
    ("Edge", ".config/microsoft-edge/NativeMessagingHosts"),
    ("Chromium", ".config/chromium/NativeMessagingHosts"),
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

    let mut report = format!("host program: {}\nmanifest:     {}\n", dest.display(), mp.display());
    #[cfg(windows)]
    for (name, key) in BROWSER_KEYS {
        let full = format!(r"HKCU\{key}\{HOST_NAME}");
        let ok = std::process::Command::new("reg")
            .args(["add", &full, "/ve", "/t", "REG_SZ", "/d", &mp.display().to_string(), "/f"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        report += &format!("{name}: {}\n", if ok { "registered" } else { "FAILED to register" });
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
            report += &format!("{name}: {}\n", if ok { "registered" } else { "FAILED to register" });
        }
    }
    report += "\nNext: in your browser open chrome://extensions, turn on Developer mode, choose Load unpacked and pick the extension folder of this repository. Then click the fastuse icon on a page to allow that site.";
    Ok(report)
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
