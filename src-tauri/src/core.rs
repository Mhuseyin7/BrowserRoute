use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub condition_mode: ConditionMode,
    pub conditions: Vec<Condition>,
    pub action: RouteAction,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConditionMode {
    All,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Condition {
    HostEquals { value: String },
    DomainSuffix { value: String },
    HostWildcard { value: String },
    PathPrefix { value: String },
    PathRegex { value: String },
    QueryParameter { name: String, value: Option<String> },
    Scheme { value: String },
    Port { value: u16 },
    Localhost,
    PrivateNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RouteAction {
    pub browser_id: String,
    pub profile: Option<String>,
    #[serde(default)]
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Browser {
    pub id: String,
    pub name: String,
    pub executable: String,
    pub kind: BrowserKind,
    pub profiles: Vec<BrowserProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BrowserKind {
    Chrome,
    Edge,
    Firefox,
    Brave,
    Chromium,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProfile {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    pub normalized_url: String,
    pub matches: Vec<Rule>,
    pub result: Option<RouteAction>,
}

#[derive(thiserror::Error, Debug)]
pub enum RouteError {
    #[error("Only http, https, and mailto links are accepted")]
    DisallowedScheme,
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Invalid rule condition: {0}")]
    InvalidCondition(String),
}

pub fn parse_routable_url(raw: &str) -> Result<Url, RouteError> {
    if raw.contains('\0') {
        return Err(RouteError::InvalidUrl("contains a null byte".into()));
    }
    let candidate = if raw.contains("://") || raw.starts_with("mailto:") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };
    let url = Url::parse(&candidate).map_err(|e| RouteError::InvalidUrl(e.to_string()))?;
    match url.scheme() {
        "http" | "https" | "mailto" => Ok(url),
        _ => Err(RouteError::DisallowedScheme),
    }
}

pub fn validate_rule(rule: &Rule) -> Result<(), RouteError> {
    if rule.name.trim().is_empty() {
        return Err(RouteError::InvalidCondition(
            "rule name cannot be empty".into(),
        ));
    }
    if rule.conditions.is_empty() {
        return Err(RouteError::InvalidCondition(
            "at least one condition is required".into(),
        ));
    }
    for c in &rule.conditions {
        match c {
            Condition::HostEquals { value } | Condition::DomainSuffix { value } => {
                validate_host(value)?
            }
            Condition::HostWildcard { value } => {
                let suffix = value.strip_prefix("*.").ok_or_else(|| {
                    RouteError::InvalidCondition("wildcards must begin with *.".into())
                })?;
                validate_host(suffix)?;
            }
            Condition::PathPrefix { value } => {
                if !value.starts_with('/') {
                    return Err(RouteError::InvalidCondition(
                        "path prefix must start with /".into(),
                    ));
                }
            }
            Condition::PathRegex { value } => {
                Regex::new(value).map_err(|e| {
                    RouteError::InvalidCondition(format!("invalid safe regex: {e}"))
                })?;
            }
            Condition::QueryParameter { name, .. } => {
                if name.is_empty() {
                    return Err(RouteError::InvalidCondition(
                        "query parameter name cannot be empty".into(),
                    ));
                }
            }
            Condition::Scheme { value } => {
                if !matches!(value.as_str(), "http" | "https" | "mailto") {
                    return Err(RouteError::InvalidCondition("unsupported scheme".into()));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_host(host: &str) -> Result<(), RouteError> {
    if host.is_empty() || host.contains('/') || host.contains('*') || host.contains(' ') {
        return Err(RouteError::InvalidCondition(format!(
            "invalid host: {host}"
        )));
    }
    Ok(())
}

pub fn simulate(raw: &str, rules: &[Rule]) -> Result<Simulation, RouteError> {
    let url = parse_routable_url(raw)?;
    let mut matches: Vec<Rule> = rules
        .iter()
        .filter(|r| r.enabled && rule_matches(r, &url))
        .cloned()
        .collect();
    matches.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });
    Ok(Simulation {
        normalized_url: redact_url(&url),
        result: matches.first().map(|r| r.action.clone()),
        matches,
    })
}

pub fn rule_matches(rule: &Rule, url: &Url) -> bool {
    let mut result = rule.conditions.iter().map(|c| condition_matches(c, url));
    match rule.condition_mode {
        ConditionMode::All => result.all(|x| x),
        ConditionMode::Any => result.any(|x| x),
    }
}

fn condition_matches(condition: &Condition, url: &Url) -> bool {
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    match condition {
        Condition::HostEquals { value } => host == value.to_ascii_lowercase(),
        Condition::DomainSuffix { value } => {
            let v = value.to_ascii_lowercase();
            host == v || host.ends_with(&format!(".{v}"))
        }
        Condition::HostWildcard { value } => value
            .strip_prefix("*.")
            .is_some_and(|suffix| host.ends_with(&format!(".{}", suffix.to_ascii_lowercase()))),
        Condition::PathPrefix { value } => url.path().starts_with(value),
        Condition::PathRegex { value } => Regex::new(value).is_ok_and(|r| r.is_match(url.path())),
        Condition::QueryParameter { name, value } => url.query_pairs().any(|(k, v)| {
            k == name.as_str() && value.as_ref().is_none_or(|wanted| v == wanted.as_str())
        }),
        Condition::Scheme { value } => url.scheme() == value,
        Condition::Port { value } => url.port_or_known_default() == Some(*value),
        Condition::Localhost => host == "localhost" || host.ends_with(".localhost"),
        Condition::PrivateNetwork => host.parse::<IpAddr>().is_ok_and(is_private_ip),
    }
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => v.is_private() || v.is_loopback() || v.is_link_local(),
        IpAddr::V6(v) => v.is_loopback() || (v.segments()[0] & 0xfe00) == 0xfc00,
    }
}

pub fn redact_url(url: &Url) -> String {
    if url.scheme() == "mailto" {
        return "mailto:…".into();
    }
    let mut safe = url.clone();
    let sensitive = [
        "token", "code", "secret", "password", "session", "key", "auth", "state",
    ];
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| {
            let key = k.into_owned();
            (
                key.clone(),
                if sensitive
                    .iter()
                    .any(|s| key.to_ascii_lowercase().contains(s))
                {
                    "[redacted]".into()
                } else {
                    v.into_owned()
                },
            )
        })
        .collect();
    safe.set_query(None);
    if !pairs.is_empty() {
        safe.query_pairs_mut().extend_pairs(pairs);
    }
    safe.to_string()
}

pub fn launch_arguments(action: &RouteAction, browser: &Browser, url: &Url) -> Vec<String> {
    let mut args = Vec::new();
    match browser.kind {
        BrowserKind::Chrome | BrowserKind::Edge | BrowserKind::Brave | BrowserKind::Chromium => {
            if let Some(profile) = &action.profile {
                args.push(format!("--profile-directory={profile}"));
            }
            if action.private {
                args.push("--incognito".into());
            }
        }
        BrowserKind::Firefox => {
            if let Some(profile) = &action.profile {
                args.extend(["-P".into(), profile.clone()]);
            }
            if action.private {
                args.push("--private-window".into());
            }
        }
        BrowserKind::Custom => {}
    }
    args.push(url.as_str().to_string());
    args
}

pub fn overlaps(a: &Rule, b: &Rule) -> bool {
    let host_values = |r: &Rule| {
        r.conditions
            .iter()
            .filter_map(|c| match c {
                Condition::HostEquals { value }
                | Condition::DomainSuffix { value }
                | Condition::HostWildcard { value } => Some(value.trim_start_matches("*.")),
                _ => None,
            })
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    host_values(a).iter().any(|x| {
        host_values(b)
            .iter()
            .any(|y| x == y || x.ends_with(&format!(".{y}")) || y.ends_with(&format!(".{x}")))
    })
}

pub fn sample_rules() -> Vec<Rule> {
    let now = Utc::now();
    vec![
        Rule {
            id: Uuid::new_v4(),
            name: "GitHub Work".into(),
            enabled: true,
            priority: 90,
            condition_mode: ConditionMode::All,
            conditions: vec![
                Condition::HostEquals {
                    value: "github.com".into(),
                },
                Condition::PathPrefix {
                    value: "/company".into(),
                },
            ],
            action: RouteAction {
                browser_id: "chrome".into(),
                profile: Some("Work".into()),
                private: false,
            },
            created_at: now,
            updated_at: now,
        },
        Rule {
            id: Uuid::new_v4(),
            name: "Local development".into(),
            enabled: true,
            priority: 100,
            condition_mode: ConditionMode::All,
            conditions: vec![Condition::Localhost, Condition::Port { value: 3000 }],
            action: RouteAction {
                browser_id: "chrome".into(),
                profile: Some("Development".into()),
                private: false,
            },
            created_at: now,
            updated_at: now,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wildcard_does_not_match_root() {
        let u = parse_routable_url("https://example.com").unwrap();
        let c = Condition::HostWildcard {
            value: "*.example.com".into(),
        };
        assert!(!condition_matches(&c, &u));
    }
    #[test]
    fn priority_wins() {
        let rules = sample_rules();
        let result = simulate("https://localhost:3000", &rules).unwrap();
        assert_eq!(result.matches[0].name, "Local development");
    }
    #[test]
    fn sensitive_queries_are_redacted() {
        let u = parse_routable_url("https://a.test/path?code=abc&view=1").unwrap();
        assert_eq!(
            redact_url(&u),
            "https://a.test/path?code=%5Bredacted%5D&view=1"
        );
    }
    #[test]
    fn blocks_file_urls() {
        assert!(matches!(
            parse_routable_url("file:///etc/passwd"),
            Err(RouteError::DisallowedScheme)
        ));
    }
}
