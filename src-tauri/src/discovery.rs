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
        vec![]
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        vec![]
    }
}
