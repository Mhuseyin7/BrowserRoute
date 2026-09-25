# Contributing

Use strict TypeScript and format Rust with `cargo fmt`. Add focused tests for any rule-engine or configuration behavior change. Browser launch tests must use fake executables; automated tests must never open a contributor’s browser.

Before a pull request, run:

```sh
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```
