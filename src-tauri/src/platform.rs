//! Platform boundaries for default-handler registration.
//! Windows cannot silently change a user's HTTP/HTTPS default since Windows 10;
//! BrowserRoute opens the system Settings page and registers its capabilities at install time.

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStatus {
    pub platform: String,
    pub can_register: bool,
    pub detail: String,
}

pub fn status() -> PlatformStatus {
    if cfg!(target_os = "windows") {
        PlatformStatus {
            platform: "Windows".into(),
            can_register: true,
            detail: "HTTP/HTTPS association requires a user confirmation in Windows Default apps."
                .into(),
        }
    } else if cfg!(target_os = "macos") {
        PlatformStatus { platform: "macOS".into(), can_register: true, detail: "BrowserRoute registers URL capabilities through its application bundle; Launch Services selection remains user-controlled.".into() }
    } else {
        PlatformStatus { platform: "Linux".into(), can_register: true, detail: "Desktop integration uses XDG MIME and x-scheme-handler associations where supported by the desktop environment.".into() }
    }
}

pub fn open_default_apps() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg("ms-settings:defaultapps")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["x-apple.systempreferences:com.apple.preference.general"])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-settings")
            .args(["get", "default-web-browser"])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
