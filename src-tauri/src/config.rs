use crate::core::{sample_rules, Browser, Rule};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub const CURRENT_CONFIG_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_history_enabled")]
    pub history_enabled: bool,
    #[serde(default)]
    pub store_full_urls: bool,
    #[serde(default)]
    pub paused_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub launch_at_login: bool,
}
fn default_history_enabled() -> bool {
    true
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            history_enabled: true,
            store_full_urls: false,
            paused_until: None,
            launch_at_login: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub host: String,
    #[serde(default)]
    pub full_url: Option<String>,
    pub rule_id: Option<String>,
    pub browser_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "current_config_version")]
    pub version: u32,
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub browsers: Vec<Browser>,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}
fn current_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            rules: sample_rules(),
            browsers: vec![],
            settings: Settings::default(),
            history: vec![],
        }
    }
}
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("BrowserRoute")
        .join("config.json")
}
pub fn load(path: &Path) -> io::Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    match serde_json::from_slice::<AppConfig>(&fs::read(path)?) {
        Ok(mut config) => {
            if config.version > CURRENT_CONFIG_VERSION {
                return Err(io::Error::other(
                    "configuration was created by a newer BrowserRoute version",
                ));
            }
            config.version = CURRENT_CONFIG_VERSION;
            Ok(config)
        }
        Err(e) => {
            let bad = path.with_extension(format!("corrupt-{}.json", Utc::now().timestamp()));
            fs::rename(path, bad)?;
            let backup = path.with_extension("bak");
            if backup.exists() {
                serde_json::from_slice(&fs::read(backup)?).map_err(io::Error::other)
            } else {
                Err(io::Error::other(e))
            }
        }
    }
}
pub fn save(path: &Path, config: &AppConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() {
        fs::copy(path, path.with_extension("bak"))?;
    }
    let temp = path.with_extension("tmp");
    fs::write(
        &temp,
        serde_json::to_vec_pretty(config).map_err(io::Error::other)?,
    )?;
    fs::rename(temp, path)
}
