use crate::core::{Browser, BrowserKind, BrowserProfile};
use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn exists(path: PathBuf) -> Option<String> {
    path.exists().then(|| path.to_string_lossy().to_string())
}
fn local_app_data() -> PathBuf {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_default()
}
fn program_files() -> PathBuf {
    env::var_os("PROGRAMFILES")
        .map(PathBuf::from)
        .unwrap_or_default()
}
#[cfg(not(target_os = "windows"))]
fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default()
}
#[cfg(not(target_os = "windows"))]
fn find_in_path(name: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    let command = "where";
    #[cfg(not(target_os = "windows"))]
    let command = "which";
    std::process::Command::new(command)
        .arg(name)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|value| {
            value
                .lines()
                .next()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
        })
}
fn chrome_profiles(dir: &Path) -> Vec<BrowserProfile> {
    let local_state = dir.join("Local State");
    let parsed = fs::read_to_string(local_state)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok());
    parsed
        .and_then(|v| {
            v.pointer("/profile/info_cache")
                .and_then(Value::as_object)
                .cloned()
        })
        .map(|cache| {
            cache
                .into_iter()
                .map(|(id, data)| BrowserProfile {
                    name: data
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or(&id)
                        .to_string(),
                    path: Some(dir.join(&id).to_string_lossy().to_string()),
                    id,
                })
                .collect()
        })
        .unwrap_or_else(|| {
            vec![BrowserProfile {
                id: "Default".into(),
                name: "Default".into(),
                path: Some(dir.join("Default").to_string_lossy().to_string()),
            }]
        })
}
pub fn detect() -> Vec<Browser> {
    #[cfg(target_os = "windows")]
    {
        let local = local_app_data();
        let pf = program_files();
        let candidates = vec![
            (
                "chrome",
                "Google Chrome",
                BrowserKind::Chrome,
                vec![
                    local.join("Google/Chrome/Application/chrome.exe"),
                    pf.join("Google/Chrome/Application/chrome.exe"),
                ],
                local.join("Google/Chrome/User Data"),
            ),
            (
                "edge",
                "Microsoft Edge",
                BrowserKind::Edge,
                vec![pf.join("Microsoft/Edge/Application/msedge.exe")],
                local.join("Microsoft/Edge/User Data"),
            ),
            (
                "brave",
                "Brave",
                BrowserKind::Brave,
                vec![pf.join("BraveSoftware/Brave-Browser/Application/brave.exe")],
                local.join("BraveSoftware/Brave-Browser/User Data"),
            ),
            (
                "firefox",
                "Firefox",
                BrowserKind::Firefox,
                vec![pf.join("Mozilla Firefox/firefox.exe")],
                PathBuf::new(),
            ),
        ];
        candidates
            .into_iter()
            .filter_map(|(id, name, kind, paths, data)| {
                paths
                    .into_iter()
                    .find_map(exists)
                    .map(|executable| Browser {
                        id: id.into(),
                        name: name.into(),
                        executable,
                        profiles: if kind == BrowserKind::Firefox {
                            vec![BrowserProfile {
                                id: "default".into(),
                                name: "Default".into(),
                                path: None,
                            }]
                        } else {
                            chrome_profiles(&data)
                        },
                        kind,
                    })
            })
            .collect()
    }
    #[cfg(target_os = "macos")]
    {
        let applications = PathBuf::from("/Applications");
        let home = home_dir();
        let candidates = vec![
            (
                "chrome",
                "Google Chrome",
                BrowserKind::Chrome,
                applications.join("Google Chrome.app/Contents/MacOS/Google Chrome"),
                home.join("Library/Application Support/Google/Chrome/User Data"),
            ),
            (
                "edge",
                "Microsoft Edge",
                BrowserKind::Edge,
                applications.join("Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
                home.join("Library/Application Support/Microsoft Edge/User Data"),
            ),
            (
                "brave",
                "Brave",
                BrowserKind::Brave,
                applications.join("Brave Browser.app/Contents/MacOS/Brave Browser"),
                home.join("Library/Application Support/BraveSoftware/Brave-Browser/User Data"),
            ),
            (
                "firefox",
                "Firefox",
                BrowserKind::Firefox,
                applications.join("Firefox.app/Contents/MacOS/firefox"),
                PathBuf::new(),
            ),
            (
                "arc",
                "Arc",
                BrowserKind::Custom,
                applications.join("Arc.app/Contents/MacOS/Arc"),
                PathBuf::new(),
            ),
        ];
        candidates
            .into_iter()
            .filter_map(|(id, name, kind, path, data)| {
                exists(path).map(|executable| Browser {
                    id: id.into(),
                    name: name.into(),
                    executable,
                    profiles: if kind == BrowserKind::Firefox {
                        vec![BrowserProfile {
                            id: "default".into(),
                            name: "Default".into(),
                            path: None,
                        }]
                    } else if data.as_os_str().is_empty() {
                        vec![]
                    } else {
                        chrome_profiles(&data)
                    },
                    kind,
                })
            })
            .collect()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let home = home_dir();
        let candidates = vec![
            (
                "chrome",
                "Google Chrome",
                BrowserKind::Chrome,
                ["google-chrome", "google-chrome-stable"]
                    .iter()
                    .find_map(|name| find_in_path(name)),
                home.join(".config/google-chrome"),
            ),
            (
                "chromium",
                "Chromium",
                BrowserKind::Chromium,
                ["chromium", "chromium-browser"]
                    .iter()
                    .find_map(|name| find_in_path(name)),
                home.join(".config/chromium"),
            ),
            (
                "edge",
                "Microsoft Edge",
                BrowserKind::Edge,
                find_in_path("microsoft-edge"),
                home.join(".config/microsoft-edge"),
            ),
            (
                "brave",
                "Brave",
                BrowserKind::Brave,
                find_in_path("brave-browser"),
                home.join(".config/BraveSoftware/Brave-Browser"),
            ),
            (
                "firefox",
                "Firefox",
                BrowserKind::Firefox,
                find_in_path("firefox"),
                PathBuf::new(),
            ),
        ];
        candidates
            .into_iter()
            .filter_map(|(id, name, kind, executable, data)| {
                executable.map(|executable| Browser {
                    id: id.into(),
                    name: name.into(),
                    executable,
                    profiles: if kind == BrowserKind::Firefox {
                        vec![BrowserProfile {
                            id: "default".into(),
                            name: "Default".into(),
                            path: None,
                        }]
                    } else {
                        chrome_profiles(&data)
                    },
                    kind,
                })
            })
            .collect()
    }
}
