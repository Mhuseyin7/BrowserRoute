pub mod config;
pub mod core;
mod discovery;
mod platform;

use config::{AppConfig, HistoryEntry};
use core::{
    launch_arguments, overlaps, parse_routable_url, simulate, validate_rule, Browser, Condition,
    ConditionMode, RouteAction, Rule,
};
use std::{path::PathBuf, process::Command, sync::Mutex};
use tauri::{Emitter, Manager, State};

struct AppState {
    config: Mutex<AppConfig>,
    path: PathBuf,
}
fn persist(state: &AppState) -> Result<(), String> {
    config::save(
        &state.path,
        &*state
            .config
            .lock()
            .map_err(|_| "configuration lock poisoned")?,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    Ok(state
        .config
        .lock()
        .map_err(|_| "configuration lock poisoned")?
        .clone())
}
#[tauri::command]
fn get_platform_status() -> platform::PlatformStatus {
    platform::status()
}
#[tauri::command]
fn open_default_apps() -> Result<(), String> {
    platform::open_default_apps()
}
#[tauri::command]
fn register_handlers() -> Result<(), String> {
    platform::register_handlers()
}
#[tauri::command]
fn detect_browsers(state: State<AppState>) -> Result<Vec<Browser>, String> {
    let found = discovery::detect();
    {
        let mut c = state
            .config
            .lock()
            .map_err(|_| "configuration lock poisoned")?;
        c.browsers = found.clone();
    }
    persist(&state)?;
    Ok(found)
}
#[tauri::command]
fn save_rule(rule: Rule, state: State<AppState>) -> Result<(), String> {
    validate_rule(&rule).map_err(|e| e.to_string())?;
    {
        let mut c = state
            .config
            .lock()
            .map_err(|_| "configuration lock poisoned")?;
        if let Some(index) = c.rules.iter().position(|r| r.id == rule.id) {
            c.rules[index] = rule;
        } else {
            c.rules.push(rule);
        }
    }
    persist(&state)
}
#[tauri::command]
fn delete_rule(id: String, state: State<AppState>) -> Result<(), String> {
    {
        let mut c = state
            .config
            .lock()
            .map_err(|_| "configuration lock poisoned")?;
        c.rules.retain(|r| r.id.to_string() != id);
    }
    persist(&state)
}
#[tauri::command]
fn simulate_url(raw: String, state: State<AppState>) -> Result<core::Simulation, String> {
    let c = state
        .config
        .lock()
        .map_err(|_| "configuration lock poisoned")?;
    simulate(&raw, &c.rules).map_err(|e| e.to_string())
}
#[tauri::command]
fn find_conflicts(rule: Rule, state: State<AppState>) -> Result<Vec<Rule>, String> {
    let c = state
        .config
        .lock()
        .map_err(|_| "configuration lock poisoned")?;
    Ok(c.rules
        .iter()
        .filter(|existing| existing.id != rule.id && overlaps(existing, &rule))
        .cloned()
        .collect())
}
#[tauri::command]
fn route_url(raw: String, state: State<AppState>) -> Result<Option<core::RouteAction>, String> {
    let url = parse_routable_url(&raw).map_err(|e| e.to_string())?;
    let (result, browser) = {
        let c = state
            .config
            .lock()
            .map_err(|_| "configuration lock poisoned")?;
        let paused = c
            .settings
            .paused_until
            .is_some_and(|until| until > chrono::Utc::now());
        let sim = if paused {
            core::Simulation {
                normalized_url: core::redact_url(&url),
                matches: vec![],
                result: None,
            }
        } else {
            simulate(&raw, &c.rules).map_err(|e| e.to_string())?
        };
        let b = sim
            .result
            .as_ref()
            .and_then(|a| c.browsers.iter().find(|b| b.id == a.browser_id))
            .cloned();
        (sim.result, b)
    };
    if let (Some(action), Some(browser)) = (&result, browser) {
        Command::new(&browser.executable)
            .args(launch_arguments(action, &browser, &url))
            .spawn()
            .map_err(|e| format!("could not launch {}: {e}", browser.name))?;
        let mut c = state
            .config
            .lock()
            .map_err(|_| "configuration lock poisoned")?;
        if c.settings.history_enabled {
            c.history.insert(
                0,
                HistoryEntry {
                    timestamp: chrono::Utc::now(),
                    host: url.host_str().unwrap_or("mailto").to_string(),
                    rule_id: None,
                    browser_id: browser.id,
                },
            );
            c.history.truncate(200);
        }
    }
    persist(&state)?;
    Ok(result)
}

#[tauri::command]
fn launch_choice(
    raw: String,
    browser_id: String,
    profile: Option<String>,
    private: bool,
    remember: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let url = parse_routable_url(&raw).map_err(|e| e.to_string())?;
    let browser = {
        state
            .config
            .lock()
            .map_err(|_| "configuration lock poisoned")?
            .browsers
            .iter()
            .find(|b| b.id == browser_id)
            .cloned()
    };
    let Some(browser) = browser else {
        return Err(format!("Browser '{browser_id}' is not detected"));
    };
    let action = RouteAction {
        browser_id: browser_id.clone(),
        profile: profile.clone(),
        private,
    };
    Command::new(&browser.executable)
        .args(launch_arguments(&action, &browser, &url))
        .spawn()
        .map_err(|e| e.to_string())?;
    if remember {
        let host = url
            .host_str()
            .ok_or_else(|| "URL has no host".to_string())?;
        let now = chrono::Utc::now();
        let rule = Rule {
            id: uuid::Uuid::new_v4(),
            name: format!("{host} routing"),
            enabled: true,
            priority: 50,
            condition_mode: ConditionMode::All,
            conditions: vec![Condition::HostEquals {
                value: host.to_string(),
            }],
            action,
            created_at: now,
            updated_at: now,
        };
        state
            .config
            .lock()
            .map_err(|_| "configuration lock poisoned")?
            .rules
            .push(rule);
    }
    persist(&state)
}
#[tauri::command]
fn export_rules(full_backup: bool, state: State<AppState>) -> Result<String, String> {
    let c = state
        .config
        .lock()
        .map_err(|_| "configuration lock poisoned")?;
    if full_backup {
        serde_json::to_string_pretty(&*c).map_err(|e| e.to_string())
    } else {
        serde_json::to_string_pretty(&serde_json::json!({"version": 1, "rules": c.rules}))
            .map_err(|e| e.to_string())
    }
}

/// Lightweight command-line surface. It deliberately shares the parser and rule engine
/// with the desktop app so `test` and `open` can never disagree with the UI.
pub fn cli(command: String, args: Vec<String>) -> bool {
    match command.as_str() {
        "test" => {
            let Some(raw) = args.first() else {
                eprintln!("Usage: browserroute test <url>");
                return true;
            };
            let path = config::config_path();
            match config::load(&path)
                .and_then(|c| simulate(raw, &c.rules).map_err(std::io::Error::other))
            {
                Ok(result) => println!(
                    "{}",
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                ),
                Err(error) => {
                    eprintln!("BrowserRoute: {error}");
                    std::process::exit(2);
                }
            }
            true
        }
        "rules" => {
            let path = config::config_path();
            let loaded = config::load(&path).unwrap_or_default();
            match args.first().map(String::as_str) {
                Some("list") => {
                    for rule in loaded.rules {
                        println!(
                            "{}\t{}\t{}",
                            rule.priority,
                            if rule.enabled { "enabled" } else { "paused" },
                            rule.name
                        );
                    }
                }
                Some("export") => println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &serde_json::json!({"version": 1, "rules": loaded.rules})
                    )
                    .unwrap_or_default()
                ),
                _ => eprintln!("Usage: browserroute rules <list|export>"),
            }
            true
        }
        "open" => {
            let Some(raw) = args.first() else {
                eprintln!("Usage: browserroute open <url>");
                return true;
            };
            let path = config::config_path();
            let mut loaded = config::load(&path).unwrap_or_default();
            if loaded.browsers.is_empty() {
                loaded.browsers = discovery::detect();
            }
            match simulate(raw, &loaded.rules) {
                Ok(result) => {
                    if let Some(action) = result.result {
                        let Some(browser) = loaded
                            .browsers
                            .iter()
                            .find(|browser| browser.id == action.browser_id)
                        else {
                            eprintln!(
                                "Browser '{}' is not detected. Open BrowserRoute settings first.",
                                action.browser_id
                            );
                            return true;
                        };
                        match parse_routable_url(raw) {
                            Ok(url) => {
                                if let Err(error) = std::process::Command::new(&browser.executable)
                                    .args(launch_arguments(&action, browser, &url))
                                    .spawn()
                                {
                                    eprintln!("Could not launch {}: {error}", browser.name);
                                }
                            }
                            Err(error) => eprintln!("BrowserRoute: {error}"),
                        }
                    } else {
                        eprintln!("No rule matched; open BrowserRoute to use the chooser.");
                    }
                }
                Err(error) => eprintln!("BrowserRoute: {error}"),
            }
            true
        }
        _ => false,
    }
}

pub fn run(start_url: Option<String>) {
    let path = config::config_path();
    let loaded = config::load(&path).unwrap_or_else(|_| AppConfig::default());
    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(loaded),
            path,
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_platform_status,
            open_default_apps,
            register_handlers,
            detect_browsers,
            save_rule,
            delete_rule,
            simulate_url,
            find_conflicts,
            route_url,
            launch_choice,
            export_rules
        ])
        .setup(move |app| {
            let open_item = tauri::menu::MenuItem::with_id(
                app,
                "open",
                "Open BrowserRoute",
                true,
                None::<&str>,
            )?;
            let pause_item =
                tauri::menu::MenuItem::with_id(app, "pause", "Pause routing", true, None::<&str>)?;
            let quit_item =
                tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&open_item, &pause_item, &quit_item])?;
            tauri::tray::TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("BrowserRoute — routing active")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "pause" => {
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Ok(mut config) = state.config.lock() {
                                config.settings.paused_until =
                                    Some(chrono::Utc::now() + chrono::Duration::minutes(10));
                                let _ = config::save(&state.path, &config);
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;
            if let Some(window) = app.get_webview_window("main") {
                window.show()?;
                if let Some(url) = start_url.as_ref() {
                    app.emit("incoming-url", url.clone())?;
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running BrowserRoute");
}
