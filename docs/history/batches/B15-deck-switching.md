# B15 — Deck switching and resource return (CB-B B4)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B15-Deck-Switching). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Ready (awaiting maintainer approval to merge)
- Objective: Prove Preview ↔ Browser switching as a destroy/recreate lifecycle — exactly one view after every switch, the previous view destroyed, bounded latency and a bounded per-role resource footprint — and state the state-loss behavior honestly on both surfaces.
- Branch: `batch/b15-deck-switching`
- Draft/final PR: [#92](https://github.com/AlexandreZanata/brain-root-idea/pull/92)
- Baseline commit: `e6c017a` (B14 released)
- Target/resulting version: `0.0.11` (deck lifecycle proof; no artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned)
- Repository history mirror: this file (created at batch close)

## Non-goals

- COLD placeholders/snapshots, more than two destinations, WARM/SUSPENDED evaluation, permission controls, and import.
- Any policy, permission, capability, or dependency change.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B15-S01 | [#91](https://github.com/AlexandreZanata/brain-root-idea/issues/91) | Deck destroy/recreate is proven through the production commands: presence flags confirm exactly one view after every switch, switch latency measured 0–3 ms at the view layer, and the footprint settles at one context per role without growth across cycles. The first run exposed a per-switch `WebContext` leak (5 → 6 → 7 processes); the fix caches one context per role, after which the harness returns `go`. Both surfaces now state the state-loss behavior. | `5750aba`, `1d45688` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/91#issuecomment-5801832943) | The module gate caught the harness at the `features/` root; it moved to `features/deck/mod.rs` and the gate stayed unmodified | Closed |
| B15-S02 | [#93](https://github.com/AlexandreZanata/brain-root-idea/issues/93) | Finalization: version `0.0.11` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.11` section; this history record created; PR #92 marked Ready and the full Linux gate run on the latest head; merge/tag deferred to maintainer approval | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/93) | The authenticated repository owner cannot approve its own PR; no tag is created before the merge commit exists on `main` | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B15-S01): switching destroys the previous view and recreates the target; exactly one content view exists after each switch and the presence check fails the run otherwise.
- Decision (B15-S01): each role caches one `WebContext` (keyed by the profile root) for the application lifetime, which bounds the WebKit footprint and keeps the profile ownership; the harness proves no growth across cycles.
- Decision (B15-S01): state-loss messaging is explicit — the Browser states that switching tabs closes the page and in-memory state may be lost; the Preview states that the view closes and the app reloads on return while the dev server keeps running until stopped.
- Decision (B15-S02): a lifecycle batch still changes the version exactly once at finalization (`0.0.11`) per `docs/19-release-and-versioning.md`, with no artifact; merge, tag, and release wait for maintainer approval.

## Failures and recovery

- Per-switch `WebContext` creation leaked one WebKit process per switch; fixed by caching the context per role and verified by a second cycle with no growth.
- The module gate caught the harness at the `features/` root because direct children of the folder are scanned; it moved to `features/deck/mod.rs` and the gate stayed unmodified.

## Final gates

- Full CI: `check-full-linux` on the finalization head — run URL recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/93) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B15-Deck-Switching)
- Review: requested; awaiting maintainer approval (documented single-maintainer administrator path)
- Security/privacy: no policy or permission change; the harness only uses the owned fixture and BrainRoot-owned profiles
- Performance: switch latency 0–3 ms; footprint bounded by one context per role; RSS +4 MB across two cycles
- Cleanup: harness exit 0 with no process, listener, or view left behind
- Artifact/checksum: none for this batch
- Known limitations: COLD placeholders/snapshots, more than two destinations, permission controls, and import remain open

## Result and next batch

B15 proves the deck destroy/recreate lifecycle and fixes the per-switch context leak it found: switching shows exactly one content view, destroys the previous one, keeps latency at a few milliseconds, and bounds the WebKit footprint to one context per role. Next: the remaining browser candidates (permission controls, import) or the roadmap's deferred phases. Rollback: revert the batch commits; the commands and surfaces are unchanged.
