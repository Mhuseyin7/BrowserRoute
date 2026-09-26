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

/// Registers BrowserRoute as a user-level HTTP/HTTPS candidate where the OS permits it.
/// The final default selection remains controlled by the platform's consent UI.
pub fn register_handlers() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    {
        let exe = executable.to_string_lossy().replace('"', "");
        for scheme in ["http", "https"] {
            let command = format!("\"{exe}\" \"%1\"");
            let key = format!("HKCU\\Software\\Classes\\{scheme}\\shell\\open\\command");
            std::process::Command::new("reg.exe")
                .args(["ADD", &key, "/ve", "/d", &command, "/f"])
                .status()
                .map_err(|e| e.to_string())
                .and_then(|status| {
                    if status.success() {
                        Ok(())
                    } else {
                        Err(format!("reg.exe failed for {scheme}"))
                    }
                })?;
        }
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        let _ = executable;
        return Err("macOS URL roles are declared by the signed application bundle; select BrowserRoute in Default Web Browser settings.".into());
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let applications = dirs::data_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".local/share"))
            .join("applications");
        std::fs::create_dir_all(&applications).map_err(|e| e.to_string())?;
        let desktop = applications.join("browserroute.desktop");
        let content = format!("[Desktop Entry]\nType=Application\nName=BrowserRoute\nExec=\"{}\" %u\nTerminal=false\nCategories=Network;\nMimeType=x-scheme-handler/http;x-scheme-handler/https;text/html;\n", executable.to_string_lossy().replace('"', "\\\""));
        std::fs::write(&desktop, content).map_err(|e| e.to_string())?;
        for scheme in ["http", "https"] {
            let status = std::process::Command::new("xdg-mime")
                .args([
                    "default",
                    "browserroute.desktop",
                    &format!("x-scheme-handler/{scheme}"),
                ])
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err(format!("xdg-mime failed for {scheme}"));
            }
        }
        Ok(())
    }
}
