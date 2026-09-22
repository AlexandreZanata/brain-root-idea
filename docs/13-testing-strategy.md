# Testing strategy

## Goal

Tests prove the user outcome, security boundary, resource lifecycle, and performance contract—not merely function return values. Prefer deterministic tests and a small set of high-value cross-platform journeys.

## Layers

### Rust unit and property tests

State machines, path policy, permission matching, event ordering, redaction, lifecycle transitions, migrations, checkpoint manifests, and command classification. Property/fuzz tests target untrusted path and protocol parsing.

### TypeScript/UI component tests

Task-state rendering, language translation, layout presets, keyboard/focus behavior, permission prompts, error recovery, and accessibility semantics. Test from typed events rather than mocked provider text.

### Contract tests

Versioned UI/core commands and events, Agent Adapter capability negotiation, Tool Layer inputs/results, process exit/cancel behavior, and persistence schemas. Include malformed, duplicate, reordered, oversized, and unsupported messages.

### Integration tests

Open a fixture workspace, start/stop owned commands, enforce path scope, create/restore checkpoints, resume persisted tasks, and verify the Governor reaches zero unwanted resources.

### End-to-end tests

The critical MVP journey: open fixture → prompt mock agent → edit files → start preview → load Canvas → run agent QA → show result → undo → close project with no orphan resources. Use deterministic local fixtures and a scripted fake agent before live-provider tests.

### Platform tests

Linux is the first required platform and establishes the MVP-0 contract. Windows and macOS are added sequentially after Linux behavior is proven. Each platform eventually covers WebView lifecycle, focus, profile isolation, path semantics, PTY/process groups, secure storage, sandbox enforcement, packaging, and cleanup. Capability differences are allowed when documented and surfaced honestly; a Linux result never counts as Windows/macOS evidence.

### Performance and soak tests

Startup, idle CPU/RSS, frontend bundle, create/destroy latency, task cancellation, repeated project open/close, WebView cycles, process leaks, and bounded log memory follow `10-performance-budget.md`.

### Security tests

Hostile repository instructions, prompt injection from pages, symlink/path traversal, untrusted origin IPC, secret redaction, approval expiry/revocation, destructive command attempts, dependency scripts, and compromised adapter behavior.

## Visual validation

Playwright is an on-demand test process. It can load the project preview, exercise key interactions, inspect DOM/console/network, and create screenshots/traces. Image snapshots must tolerate documented platform rendering differences and never replace semantic assertions.

## Test doubles

Use a fake Agent Adapter, fake clock, fixture process, and deterministic local preview. Tests must not require paid providers, internet access, personal cookies, or real credentials. A small opt-in live-agent suite validates integration separately.

## Standard Definition of Done

- Acceptance criteria and non-goals satisfied.
- Errors are understandable in Default Mode and diagnosable in Technical details.
- Permissions are least privilege; denial and revocation work.
- Relevant unit, contract, integration, E2E, accessibility, and platform tests pass.
- Relevant performance budgets are measured and no unexplained regression remains.
- All created processes, listeners, watchers, PTYs, WebViews, and temp artifacts clean up.
- Logging is useful, bounded, and secret-safe.
- Documentation and ADRs remain synchronized.
