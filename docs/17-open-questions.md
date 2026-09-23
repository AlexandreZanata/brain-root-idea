# Open questions

An item leaves this file only when linked evidence and an ADR or scoped decision resolve it. Priority means impact, not implementation order.

## BLOCKER
- **First full coding agent:** MVP-0 now uses OpenCode Go as a model gateway, not a complete coding agent. Which later agent provides the most reliable task/tool/session experience under an open adapter and acceptable provider terms? Owner: Agent Host spike.
- **ACP path:** Direct ACP client first, provider adapter first, or an ACP bridge? Validate capability negotiation, permissions, cancellation, resume, edits, and Codex packaging against the current spec. Owner: Agent Host spike.
- **Safe Mode enforcement floor:** What containment can BrainRoot truthfully guarantee on supported Windows, Linux, and macOS while still running ordinary project toolchains? Owner: security spike.
- **Platform support floor:** Minimum Windows/WebView2, macOS/WKWebView, Linux distributions/WebKitGTK, CPU architectures, and accessibility requirements. Owner: shell spike.
- **Distribution and updates:** Signing, notarization, package formats, update channel, rollback, and supply-chain provenance. Owner: release design.
- **OpenCode Go credential UX:** Linux Secret Service availability, recovery when unavailable, subscription onboarding, and whether BrainRoot may delegate authentication to an installed OpenCode client without reading its plaintext auth file. Owner: MVP-0 security spike. Scoped B04-S03 decision: the initial live loop reads `dev.brainroot.experiment/default` from Linux Secret Service; no plaintext environment key or credential form is added in that microstep. The broader UX remains open; see issue #35.

## IMPORTANT

- **WebView lifecycle:** Measured create/destroy latency, memory release, crash recovery, focus, storage isolation, and whether any platform-specific WARM state is worth supporting.
- **Checkpoint backend:** Safe Git object strategy without touching user history, non-Git fallback, untracked/ignored/large files, and restore conflicts.
- **PTY/process library:** Whether `portable-pty`, direct ConPTY/POSIX integration, or another maintained open-source crate best supports process trees and cleanup. Interactive PTY is not automatically required for MVP.
- **Async/runtime choice:** Smallest Rust concurrency stack consistent with Tauri, cancellation, and process I/O. Scoped B04-S03 decision: one `std::thread` worker exists only while the blocking OpenCode Go request is active, with no new runtime dependency. This does not settle later agent/process concurrency; see issue #35.
- **SQLite integration:** crate, migration tool, journal mode, backup, corruption recovery, and encryption needs.
- **Credential abstraction:** exact macOS Keychain, Windows Credential Locker, and Linux Secret Service integration; fallback when a Linux secret service is unavailable.
- **Human browser policy:** profile creation, cookies, downloads, external navigation, explicit sharing, retention, and delete controls.
- **Preview networking:** localhost origin discovery, port ownership, HTTPS/dev certificates, remote device testing, and SSR/dev-server variants.
- **External project compatibility:** which ecosystems are supported first and how BrainRoot detects start/test commands without unsafe guessing.
- **Code signing versus sandboxing:** packaging entitlements may constrain child tools differently per platform.
- **Accessibility inside child WebViews:** focus escape, announcements, zoom, keyboard routing, and screen-reader behavior.

## LATER

- Plugin architecture or no general plugin API.
- Remote workspaces, containers, WSL, and SSH.
- Local models and their Resource Governor budgets.
- Cloud sync and collaboration.
- Deployment/publishing providers.
- Companion Deck remote/social destinations and terms-of-service constraints.
- Replay schema, editing, export, and privacy model.
- Opt-in telemetry, public schema, retention, and governance.
- Internationalization and terminology localization strategy.
- Funding/governance model for a sustainable open-source project.

## Research finding that changed the initial hypothesis

Cross-platform WebView hibernation cannot be treated as a uniform Tauri/WRY feature. Background-throttling configuration is documented only on recent Apple platforms; Windows/Linux differ, though Windows has a WRY-specific low-memory control. The portable design is destroy/recreate from minimal state, with snapshots/placeholders. Native suspension remains an optional measured enhancement.

## Resolved decisions

- **Project license (2026-09-22):** Apache License 2.0 with a `NOTICE` attribution naming BrainRoot and linking `https://github.com/AlexandreZanata/brain-root-idea`. See ADR 0010.
- **MVP-0 conversation runtime (2026-09-22):** one on-demand `std::thread` per active blocking request with typed Tauri events and no new runtime dependency; the broader async/runtime choice for later agent and process work remains open. See B04-S03.
- **OpenCode Go credential read path for MVP-0 (2026-09-22):** the live loop reads the `dev.brainroot.experiment/default` entry from Linux Secret Service; there is no plaintext environment key or in-app credential form; the broader credential UX stays open. See B04-S03.
- **Authenticated models payload shape (2026-09-22):** the live endpoint states no per-model endpoint; discovery accepts the shape and resolves the configured default model without a fallback. Live `402`/`403`/`404` semantics remain unverified until a deliberately exercised run. See B04-R01 and `docs/specs/opencode-go-contract.md`.
