# ADR 0011: Owned dev-server lifecycle for the Companion Canvas preview

**Status:** Accepted, implementation deferred to CB-A A4  
**Date:** 2026-09-23

## Context

The Companion Canvas preview shows the user's local app, which means a dev server must run. The adopted [Companion Browser plan](../specs/companion-browser-plan.md) requires "owned dev-server/process lifecycle" as a CB-A prerequisite because the product promises one HOT content view, zero idle resources, and deterministic cleanup: a server that BrainRoot did not start cannot guarantee any of those.

The approved [preview origin policy](../specs/preview-origin-policy.md) only allows loopback origins whose port is registered as owned by a BrainRoot-managed dev server, so port assignment and the registry are part of this contract. `docs/09-resource-governor.md` already fixes the ownership principle — "a resource without an owner is a bug" — and the leaf-first cleanup order; `docs/11-security-and-permissions.md` fixes that project code is untrusted and Safe Mode is the default; `docs/15-roadmap.md` Phase 3 covers the broader process work. The child-WebView probe (B08-S01) showed the hosting path still needs an ADR 0006 review, but the lifecycle contract is independent of how the preview is hosted.

## Decision

1. **Single owner.** The Rust core owns at most one dev-server process per active project. Nothing starts at app launch or project open; a server starts only for an explicit preview action and is registered in the Resource Governor with an owner, state, and cleanup handle.
2. **Declared command only.** The start command comes from the project's declared configuration (a documented command field or a recognized manifest script). BrainRoot never guesses or synthesizes commands. Without a declared command, preview is unavailable with a plain-language explanation.
3. **Assigned port and registry.** BrainRoot selects a free loopback port before start and passes it to the command through the project's documented convention (`--port`/`PORT`). That port is the only entry in the preview origin registry. Frameworks that cannot accept an assigned port are unsupported in the first preview slice; adapters may add discovery later.
4. **Process-group isolation.** The server runs in its own process group so the whole tree can be stopped. BrainRoot sends SIGTERM, waits a `TARGET` 5 s grace, then SIGKILLs the group. It never kills a process it does not own: a foreign listener on the assigned port is a start failure with an actionable message, not something BrainRoot terminates.
5. **Readiness.** The server is ready when the assigned loopback port accepts a connection and the process is alive, bounded by `TARGET` 30 s. A readiness failure stops the owned group and surfaces a failure state with a retry action.
6. **No hidden restarts.** There is no automatic restart loop. A crash after ready moves the preview to a failed state; retries are explicit user actions after cleanup.
7. **Idle policy.** The dev server stops when the preview closes, the project closes, or the app exits. No server survives without an owner; a stopped server means the preview returns to a cold state and may require reload, which the UI states honestly.
8. **Untrusted output.** stdout/stderr are bounded, treated as untrusted input, redacted for likely secrets, never treated as instructions, and never pasted into agent context verbatim.
9. **Permission boundary.** Starting a dev server is a distinct permission moment consistent with Safe Mode: the declared command, its working directory, and its resource scope are visible before start. This contract grants no package installation, broad shell, or network permission beyond what the project command needs.
10. **Implementation timing.** This ADR is the approved gate, not implementation. The lifecycle lands in CB-A A4 after the ADR 0006 hosting review, with the evidence below. Process-group termination may require a small direct dependency (for example `libc` for `killpg` on Unix); that dependency gets its own review in the implementing issue and is **not approved here**.
11. **Required evidence at implementation.** Start/ready/stop cycle on the reference environment; no orphan process, listener, watcher, or child after close; foreign-port failure never kills the foreign process; readiness-timeout path; crash-after-ready path; output bounding and redaction; and `MEASURED` start/stop timings labeled against the `TARGET` values above.

## Alternatives considered

- **Attach to a user-started server.** Rejected for the first slice: ownership, port trust, and cleanup cannot be guaranteed. A future explicit attach mode may be considered with its own ADR.
- **Guess a start command from project files.** Rejected: unsafe guessing and a hidden action the user did not approve.
- **Stop by killing whatever listens on the port.** Rejected: it can kill another user process; the port assignment exists to avoid this.
- **Auto-restart supervisor.** Rejected: it hides failures and creates a hidden resource that violates the zero-idle invariant.
- **Background service or daemon.** Rejected: a persistent service contradicts the ownership and idle rules.

## Consequences

Preview becomes a lifecycle feature, not a webview wrapper: no server without an explicit action, one server at a time, and cleanup that can be tested. Frameworks without a configurable port are unsupported until an adapter exists, and the preview can be cold while the server restarts, which the UI must say plainly. The ADR 0006 hosting review still gates the preview UI itself.

## Performance implications

All timing values are `TARGET` until the implementing issue measures them: SIGTERM grace 5 s, readiness bound 30 s, and no idle dev server. Implementation must measure startup, readiness, stop, and post-close resource return on the frozen reference environment and record them with `MEASURED` labels; a dev server must not remain active after the owner closes.

## Security implications

Untrusted project code runs under the approved command scope with no additional privilege, its output is untrusted input, and the preview origin registry receives exactly one port that BrainRoot owns. Refusing to kill foreign processes avoids collateral damage from port collisions. The preview itself remains without privileged Tauri IPC, per ADR 0006 and the preview origin policy.

## Reversibility

Medium-high. The contract is internal to the Rust core; a future attach mode or framework adapters extend it without changing the product surface, and a superseding ADR can replace it. No published interface or artifact depends on it yet.

## References

[Companion Browser plan](../specs/companion-browser-plan.md), [preview origin policy](../specs/preview-origin-policy.md), [Resource Governor](../09-resource-governor.md), [security and permissions](../11-security-and-permissions.md), [roadmap Phase 3](../15-roadmap.md), [Linux child-WebView probe](../specs/linux-webview-probe.md), ADR 0006, and ADR 0008.
