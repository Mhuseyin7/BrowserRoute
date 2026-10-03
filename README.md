# BrowserRoute

> **Akıllı URL routing utility** — doğru linki, doğru browser ve profile ile açın.

[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-63c99f?style=flat-square)](#platform-support)
[![Stack](https://img.shields.io/badge/stack-Rust%20%2B%20Tauri%20%2B%20React-17212a?style=flat-square)](#tech-stack)
[![Privacy](https://img.shields.io/badge/privacy-local--first-247052?style=flat-square)](#privacy--security)

**BrowserRoute**, işletim sistemi tarafından açılan HTTP/HTTPS linklerini local rules ile değerlendirip uygun browser, browser profile veya desktop application’a yönlendiren açık kaynak bir desktop utility’dir.

Bu project, **[muhammedkoca.com.tr](https://muhammedkoca.com.tr)** tarafından geliştirilmiştir. BrowserRoute’un source code’u açık kaynak yaklaşımıyla yayınlanır; account, telemetry veya cloud dependency gerektirmez.

## Neden BrowserRoute?

Gün içinde farklı context’ler için farklı browser profile’ları kullanmak yaygındır. BrowserRoute bunu her seferinde manuel seçim yapmadan yönetir:

```text
github.com/company/*     → Chrome / Work
github.com/personal/*    → Firefox / Personal
localhost:3000           → Chrome / Development
*.company.com            → Microsoft Edge / Work
```

Rule eşleşmezse app, hızlı bir browser chooser açar; seçiminizi yalnızca bir kez veya domain için kalıcı olarak kaydedebilirsiniz. Rule simulator ile bir URL’nin neden ve hangi priority ile eşleştiğini browser açmadan inceleyebilirsiniz.

## Features

- **Deterministic rule engine** — explicit priority ve oluşturulma zamanına göre kararlı sonuçlar.
- **Flexible matching** — exact host, domain suffix, wildcard, path prefix, safe regex, query parameter, scheme, port, localhost ve private-network conditions.
- **Profile-aware launch** — Chrome-family ve Firefox için bilinen profile/private-window arguments.
- **Safe by design** — URL’ler shell command içine eklenmez; direct process spawn ve argv arrays kullanılır.
- **Browser discovery** — Windows üzerinde Chrome, Edge, Brave ve Firefox detection; Chromium profile metadata discovery.
- **Rule simulator** — eşleşen rules, priority ve final destination’ı anında gösterir.
- **Conflict awareness** — aynı domain alanını kapsayan rules için overlap uyarısı.
- **Local-only history** — varsayılan olarak yalnızca host, selected browser ve timestamp saklanır.
- **Atomic config** — versioned JSON config, backup ve corrupt-config recovery.
- **Modern desktop UI** — Overview, Rules, Browsers, History, Simulator ve Settings screens.
- **Fallback chooser** — eşleşmeyen URL’lerde browser/profile/private-window seçimi ve “remember for domain”.
- **CLI** — `browserroute open`, `test`, `rules list` ve `rules export` commands.
- **Tray workflow** — routing pause, settings window ve quit actions.

## Screens and workflow

1. **Browsers** ekranından local browser’ları detect edin.
2. **Rules** ekranından domain veya path tabanlı bir route oluşturun.
3. **Simulator** ile URL’yi test edin.
4. Sisteminizin Default Apps ekranından BrowserRoute’u HTTP/HTTPS handler olarak seçin.

Örnek rule:

```text
WHEN host equals github.com
AND  path starts with /company
THEN open Chrome → Work profile
```

## Installation & development

### Requirements

- Node.js 20+
- Rust stable toolchain
- Platform için Tauri prerequisites

### Local development

```bash
git clone https://github.com/Mhuseyin7/BrowserRoute.git
cd BrowserRoute
npm install
npm run tauri dev
```

### CLI

```bash
browserroute open https://github.com/company/project
browserroute test https://github.com/company/project
browserroute rules list
browserroute rules export > browserroute-rules.json
```

### Validation commands

```bash
# TypeScript typecheck + production frontend bundle
npm run build

# Rust formatter
cargo fmt --manifest-path src-tauri/Cargo.toml

# Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# Native development executable
cargo build --manifest-path src-tauri/Cargo.toml
```

## Rule model

Bir rule aşağıdaki temel alanları içerir:

```json
{
  "id": "uuid",
  "name": "GitHub Work",
  "enabled": true,
  "priority": 90,
  "conditionMode": "all",
  "conditions": [
    { "kind": "hostEquals", "value": "github.com" },
    { "kind": "pathPrefix", "value": "/company" }
  ],
  "action": {
    "browserId": "chrome",
    "profile": "Work",
    "private": false
  }
}
```

Higher priority her zaman önce değerlendirilir. Eşit priority durumunda older rule kazanır; böylece sonuç rastgele değişmez.

## Privacy & security

BrowserRoute local-first bir software’dir.

- Account, analytics ve cloud sync yoktur.
- Routing kararı için network request yapılmaz.
- Browser history, cookies veya credentials okunmaz.
- Full URL history default olarak kapalıdır.
- `token`, `code`, `secret`, `password`, `session`, `key`, `auth` ve `state` gibi query parameter’lar güvenli URL display’lerinde redacted olur.
- `file:` URL routing default olarak engellenir.
- Regex matching, catastrophic backtracking riski taşımayan Rust regex engine ile yapılır.

Detaylar için [PRIVACY.md](PRIVACY.md) ve [SECURITY.md](SECURITY.md) dosyalarına bakın.

## Platform support

| Platform | Current support | Notlar |
| --- | --- | --- |
| Windows 10/11 | Browser detection, profile detection, native build | Default HTTP/HTTPS association Windows tarafından user confirmation ile seçilir. |
| macOS | Browser discovery, native Tauri build target | Launch Services association installer bundle tarafından user confirmation ile tamamlanır. |
| Linux desktop | PATH-based browser discovery, native Tauri build target | XDG MIME / `x-scheme-handler` association desktop package tarafından user confirmation ile tamamlanır. |

BrowserRoute, OS güvenlik modelini aşarak default browser’ı sessizce değiştirmeye çalışmaz. Uygulama ilgili Default Apps / system settings ekranına yönlendirir.

## Tech stack

- **Core:** Rust
- **Desktop runtime:** Tauri 2
- **UI:** React + TypeScript (strict) + CSS
- **URL parsing:** `url`
- **Rule regex:** Rust `regex`
- **Config:** versioned local JSON with atomic write pattern

Architecture detayları için [ARCHITECTURE.md](ARCHITECTURE.md) dosyasını inceleyin.

## Project status

Bu repository günlük kullanım için çalışan bir desktop release foundation içerir. Rule engine, safe launch boundary, local persistence, cross-platform browser discovery, CLI, tray pause workflow, fallback chooser ve settings interface çalışır durumdadır. Installer signing ve mağaza dağıtımı ayrı release engineering adımlarıdır.

## Contributing

Katkılar memnuniyetle karşılanır. PR göndermeden önce şu kontrolleri çalıştırın:

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Lütfen yeni rule behavior’ları için focused unit test ekleyin. Automated test’ler gerçek browser’ı açmamalı; fake executable veya argument-level test kullanılmalıdır.

## License

BrowserRoute is released under the [MIT License](LICENSE). Herkes source code’u inceleyebilir, değiştirebilir, fork’layabilir, dağıtabilir ve kendi project’lerinde kullanabilir. Değişiklikleri upstream project’e göndermek için pull request açabilirsiniz.

## Documentation

- [Architecture](ARCHITECTURE.md)
- [Privacy](PRIVACY.md)
- [Security](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

---

Built with care by **[muhammedkoca.com.tr](https://muhammedkoca.com.tr)**.  
BrowserRoute is an open-source project for a more intentional browser workflow.
