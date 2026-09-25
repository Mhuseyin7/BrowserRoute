# Security policy

Please report vulnerabilities privately to the project maintainers before public disclosure.

Security invariants:

- never interpolate links into shell command strings;
- only allow explicitly supported URL schemes;
- disable `file:` URL routing by default;
- validate rule patterns before storing them;
- use Rust regex rather than a backtracking regex engine;
- preserve configuration backups during writes and recovery.

Do not include live authentication links, tokens, or private browser data in bug reports.
