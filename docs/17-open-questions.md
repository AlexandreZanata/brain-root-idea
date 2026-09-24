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

- **Freebuff Desktop reference and BrainRoot polish:** the published v0.0.142 Desktop binary has no corresponding Desktop source in the nearest public snapshot, so UI-specific internals and exact build provenance remain `UNKNOWN`. An approved disposable visual audit and a BrainRoot before/after streaming/layout benchmark are needed before asserting parity or a performance gain. See [the release-reference plan](specs/freebuff-inspired-experience-plan.md); the frozen reference, current friction inventory, and the B17-S07 before/after procedure are recorded in [the B17 interaction baseline](specs/b17-interaction-baseline.md).
- **WebView lifecycle:** Measured create/destroy latency, memory release, crash recovery, focus, storage isolation, and whether any platform-specific WARM state is worth supporting. Partial measurement 2026-09-23 (B08-S01): create/load/destroy, storage isolation, and forced-close recovery are feasible; memory growth over 100 cycles is unclassified and focus/WARM remain unmeasured; see [the Linux child-WebView probe](specs/linux-webview-probe.md).
- **Checkpoint backend:** Safe Git object strategy without touching user history, non-Git fallback, untracked/ignored/large files, and restore conflicts.
- **PTY/process library:** Whether `portable-pty`, direct ConPTY/POSIX integration, or another maintained open-source crate best supports process trees and cleanup. Interactive PTY is not automatically required for MVP. Scoped B10-S01 decision: `libc 0.2` is the approved direct Unix dependency for `killpg` (std spawns with a process group but cannot signal it); the dev-server supervisor tests prove group cleanup, and the broader PTY choice remains open.
- **Async/runtime choice:** Smallest Rust concurrency stack consistent with Tauri, cancellation, and process I/O. Scoped B04-S03 decision: one `std::thread` worker exists only while the blocking OpenCode Go request is active, with no new runtime dependency. This does not settle later agent/process concurrency; see issue #35.
- **SQLite integration:** crate, migration tool, journal mode, backup, corruption recovery, and encryption needs.
- **Credential abstraction:** exact macOS Keychain, Windows Credential Locker, and Linux Secret Service integration; fallback when a Linux secret service is unavailable.
- **Human browser policy:** the role, navigation, profile ownership/isolation, denied permissions, lifecycle, and retention boundaries are decided in [ADR 0013](adr/0013-human-browser-policy.md) (arbitrary `http`/`https`, no downloads/popups/permission prompts in the first slice). Measured 2026-09-23 (B13-S02): two role profiles with separate `WebContext` data directories are isolated in both directions, persist across destroy/recreate, live inside the application data directory, and clean up ([isolation probe](specs/human-profile-isolation-probe.md)). Implemented engine slice 2026-09-23 (B14-S01): the typed navigation policy, the isolated remote view in the shared Canvas overlay, and the show/navigate/back/forward/reload/hide/status commands pass the corpus tests and the debug harness with clean cleanup ([navigation record](specs/human-browser-navigation.md)). Permission requests (geolocation, media, notifications, pointer lock, device info, encrypted media, website-data access) are denied and recorded, and `human_browser_clear_data` removes the BrainRoot-owned profile data; both measured 2026-09-23 (B16-S01) in [the permissions record](specs/human-browser-permissions.md). Still open: whether a curated-destination mode is ever worth adding, the exact future user-mediated share flow and its threat model, the concrete retention/delete UX, Deck switching, and cookie/cache/deletion/crash behavior.
- **Browser data portability:** exact import categories and retained history policy; HTML bookmarks can be proposed first, but direct profile reads, credentials, cookies, and sessions require separate security decisions. See [the Companion Browser plan](specs/companion-browser-plan.md).
- **Deck switching and resource return:** measured 2026-09-23 (B15-S01): Preview ↔ Browser switching destroys the previous view (presence flags fail otherwise), every switch measured 0–3 ms at the view layer, and the WebKit footprint is bounded by one cached context per role instead of growing per switch; the first run exposed a per-switch context leak that the harness fixed before release. COLD placeholders/snapshots and more than two destinations remain for the Deck phase. See [the deck record](specs/deck-switching.md).
- **Side browser feasibility:** Linux WebKitGTK child-view positioning/resizing under the supported X11/Wayland stacks, profile isolation, resource return, and whether one-HOT switching is fast enough. Measured 2026-09-23 (B08-S01): the Tauri-managed child-WebView path does not honor position/size on the Wayland reference stack (`GtkBox` hosting), while profile isolation, navigation policy, and recreate pass. Measured 2026-09-23 (B09-S01): a `wry` view created directly into a `GtkFixed` overlay inside the Tauri window holds exact bounds, resizes exactly, denies `file://`, leaves the shell allocation unchanged, and cleans up. Measured 2026-09-23 (B12-S01): scale/DPR (1.0 and 2.0), page zoom (480 → 240 → 480 CSS px), widget focus, and a 100-update bounds soak (p50 0 µs, max 31 µs, RSS +20 KB, stable child count) pass; keyboard event routing into preview content and screen-reader behavior remain `UNKNOWN` without synthetic input. Decision recorded as [ADR 0012](adr/0012-linux-preview-hosting.md); see [the child-WebView probe](specs/linux-webview-probe.md), [the hosting probe](specs/linux-preview-hosting-probe.md), [the quality measurements](specs/preview-quality-measurements.md), the [Companion Browser plan](specs/companion-browser-plan.md), and ADR 0006.
- **Preview networking:** the Preview origin allowlist is decided — canonical loopback hosts on BrainRoot-owned dev-server ports, fail-closed, no privileged IPC; see [the preview origin policy](specs/preview-origin-policy.md). Port assignment is decided by [ADR 0011](adr/0011-dev-server-lifecycle.md): BrainRoot assigns a free loopback port; discovery for frameworks that choose their own port remains open as a future adapter. HTTPS/dev certificates, remote device testing, and SSR/dev-server variants remain open and are handled with the ADR 0006 review.
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
