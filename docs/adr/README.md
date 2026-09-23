# Architecture Decision Records

ADRs preserve why a durable decision was made. They are not implementation tutorials. A decision starts `Proposed`, becomes `Accepted` after review, and may later be `Superseded by ADR NNNN` or `Rejected`. Do not rewrite an accepted ADR to hide changed reasoning; add a superseding record.

## Index

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-desktop-runtime.md) | Tauri 2 and system WebViews | Accepted for prototype |
| [0002](0002-rust-core.md) | Rust owns privileged core state | Accepted |
| [0003](0003-frontend-stack.md) | Svelte + TypeScript frontend | Accepted for prototype |
| [0004](0004-code-editor-component.md) | CodeMirror 6 for optional Code View | Accepted, deferred |
| [0005](0005-agent-protocol.md) | Agent Adapter boundary with ACP preference | Accepted |
| [0006](0006-browser-webview-strategy.md) | Isolated WebView roles and one HOT heavy view | Accepted for prototype |
| [0007](0007-persistence.md) | SQLite global state plus minimal project metadata | Accepted |
| [0008](0008-zero-idle-architecture.md) | Event-driven zero-idle resource ownership | Accepted |
| [0009](0009-linux-first-opencode-go-mvp0.md) | Linux-first MVP-0 with OpenCode Go model gateway | Accepted for experiment |
| [0010](0010-project-license.md) | Apache-2.0 with BrainRoot NOTICE attribution | Accepted |
| [0011](0011-dev-server-lifecycle.md) | Owned one-per-project dev-server lifecycle for preview | Accepted, implementation deferred to CB-A A4 |
| [0012](0012-linux-preview-hosting.md) | Linux preview hosted in a GtkFixed overlay outside the Tauri manager | Accepted |

## Required sections

Every ADR contains Status, Context, Decision, Alternatives considered, Consequences, Performance implications, Security implications, Reversibility, and References. Use exact measurements only with a linked report.
