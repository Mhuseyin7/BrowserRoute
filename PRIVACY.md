# Privacy

BrowserRoute is designed to route links locally.

- It has no account or analytics.
- It performs no network fetches for routing.
- It does not inspect browser history, cookies, or credentials.
- History is disabled from storing full URLs by default and records only a host, selected browser, and timestamp.
- Sensitive query parameter values (including token, code, secret, password, session, key, auth, and state) are redacted from safe URL displays.

Config and optional history remain in the OS configuration directory. Exporting rules excludes executable paths and history unless an explicit full-backup workflow is implemented.
