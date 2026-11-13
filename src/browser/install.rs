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
    }
    report += &format!(
        "\nExtension unpacked to: {}\nLoad it automatically via `fastuse browser-open`, or once via Developer Mode -> 'Load unpacked' at chrome://extensions / brave://extensions.",
        ext_dir.display()
    );
    Ok(report)
}

struct BrowserBin {
    name: &'static str,
    path: String,
    proc_name: &'static str,
}

fn find_browser() -> Result<BrowserBin, String> {
    #[cfg(windows)]
    {
        let local_app = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let candidates = [
            ("Brave", r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe".to_string(), "brave.exe"),
            ("Brave", format!(r"{local_app}\BraveSoftware\Brave-Browser\Application\brave.exe"), "brave.exe"),
            ("Chrome", r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(), "chrome.exe"),
            ("Chrome", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe".to_string(), "chrome.exe"),
            ("Chrome", format!(r"{local_app}\Google\Chrome\Application\chrome.exe"), "chrome.exe"),
            ("Edge", r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".to_string(), "msedge.exe"),
            ("Edge", r"C:\Program Files\Microsoft\Edge\Application\msedge.exe".to_string(), "msedge.exe"),
        ];
        for (name, path, proc_name) in candidates {
            if std::path::Path::new(&path).exists() {
                return Ok(BrowserBin { name, path, proc_name });
            }
        }
        Err("No Chromium browser (Brave, Chrome, Edge) found in standard locations".into())
    }

    #[cfg(target_os = "macos")]
    {
        let apps = [
            ("Brave", "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser", "Brave Browser"),
            ("Chrome", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "Google Chrome"),
            ("Edge", "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge", "Microsoft Edge"),
        ];
        for (name, path, proc_name) in apps {
            if std::path::Path::new(path).exists() {
                return Ok(BrowserBin { name, path: path.into(), proc_name });
            }
        }
        Err("No Chromium browser (Brave, Chrome, Edge) found in /Applications".into())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for (name, bin) in [
            ("Brave", "brave-browser"),
            ("Chrome", "google-chrome"),
            ("Chromium", "chromium"),
            ("Edge", "microsoft-edge"),
        ] {
            if std::process::Command::new("which")
                .arg(bin)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return Ok(BrowserBin { name, path: bin.into(), proc_name: bin });
            }
        }
        Err("No Chromium browser command found on PATH".into())
    }
}

fn browser_running(proc_name: &str) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {proc_name}"), "/NH"])
            .output()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout).to_ascii_lowercase();
                s.contains(&proc_name.to_ascii_lowercase())
            })
            .unwrap_or(false)
    }
    #[cfg(unix)]
    {
        std::process::Command::new("pgrep")
            .args(["-x", proc_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

fn stop_browser(proc_name: &str) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill").args(["/IM", proc_name, "/T"]).output();
        for _ in 0..10 {
            if !browser_running(proc_name) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        let _ = std::process::Command::new("taskkill").args(["/F", "/IM", proc_name, "/T"]).output();
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript")
            .args(["-e", &format!("tell application \"{proc_name}\" to quit")])
            .output();
        for _ in 0..12 {
            if !browser_running(proc_name) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        let _ = std::process::Command::new("pkill").args(["-x", proc_name]).output();
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("pkill").args(["-x", proc_name]).output();
        std::thread::sleep(std::time::Duration::from_millis(400));
    }
}

pub fn open_browser(url: Option<&str>, relaunch: bool, isolated: bool) -> Result<String, String> {
    let _ = install()?;
    let ext_dir = extension_dir();
    let bin = find_browser()?;
    let already_connected = !super::live_hosts().is_empty();

    if already_connected && !isolated && !relaunch {
        if let Some(u) = url {
            let mut cmd = std::process::Command::new(&bin.path);
            cmd.arg(u);
            cmd.spawn().map_err(|e| format!("failed to open tab in {}: {e}", bin.name))?;
            return Ok(format!("{} is already connected; opened {u}.", bin.name));
        }
        return Ok(format!("{} is already running and connected to fastuse.", bin.name));
    }

    let running = browser_running(bin.proc_name);
    let mut use_isolated = isolated;
    let mut restore_session = false;

    if relaunch && running {
        stop_browser(bin.proc_name);
        restore_session = true;
    } else if running && !isolated {
        // Chromium ignores --load-extension when delegating to an existing browser process.
        use_isolated = true;
    }

    let mut cmd = std::process::Command::new(&bin.path);
    cmd.arg(format!("--load-extension={}", ext_dir.display()));
    if use_isolated {
        let profile = data_dir().join("browser-profile");
        let _ = std::fs::create_dir_all(&profile);
        #[cfg(unix)]
        {
            let nm = profile.join("NativeMessagingHosts");
            if std::fs::create_dir_all(&nm).is_ok() {
                let _ = std::fs::copy(manifest_path(), nm.join(format!("{HOST_NAME}.json")));
            }
        }
        cmd.arg(format!("--user-data-dir={}", profile.display()));
        cmd.arg("--no-first-run");
    } else if restore_session {
        cmd.arg("--restore-last-session");
    }
    if let Some(u) = url {
        cmd.arg(u);
    }
    cmd.spawn().map_err(|e| format!("failed to launch {}: {e}", bin.name))?;

    if use_isolated && running && !isolated {
        Ok(format!(
            "Launched {} in an isolated fastuse profile (your existing {} window was already open without the extension).\nTo load the extension in your main logged-in profile instead, run:\n  fastuse browser-open --relaunch",
            bin.name, bin.name
        ))
    } else if use_isolated {
        Ok(format!("Launched {} (isolated profile) with fastuse extension from {}", bin.name, ext_dir.display()))
    } else {
        Ok(format!("Launched {} with fastuse extension from {}", bin.name, ext_dir.display()))
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
