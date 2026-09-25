# Architecture

BrowserRoute has a deliberately narrow trust boundary.

```text
OS URL handler → Rust parser → rule engine → selected browser argv → browser process
                         ↓
                   local atomic config
                         ↓
                React/Tauri settings UI
```

The Rust core owns parsing, rule validation, matching, persistence, browser discovery, and process launch. The React interface only renders configuration and invokes typed Tauri commands. BrowserRoute never dereferences a link over the network to decide where it should go.

Rules are ordered by descending priority, then by creation time for deterministic ties. The first matching enabled rule is selected. Configuration is versioned and writes to a temporary file before replacing the active copy; the prior valid configuration is retained as `.bak`.

## Platform boundaries

Handler registration is intentionally abstracted from routing. Windows uses registered application/protocol capabilities but requires user action to select the default; macOS uses Launch Services roles in an app bundle; Linux uses `x-scheme-handler/http` and `x-scheme-handler/https` desktop associations. These packaged integrations should be separately tested on each supported platform.
