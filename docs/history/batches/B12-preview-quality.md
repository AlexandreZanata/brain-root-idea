# B12 — Preview quality measurements (CB-A A6 closure)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B12-Preview-Quality). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Ready (awaiting maintainer approval to merge)
- Objective: Close the remaining CB-A acceptance checks from ADR 0012 by measuring the production preview module — scale/DPR, page zoom, widget focus, and a bounds soak — and record the results honestly, keeping the unmeasurable keyboard routing explicit.
- Branch: `batch/b12-preview-quality`
- Draft/final PR: [#81](https://github.com/AlexandreZanata/brain-root-idea/pull/81)
- Baseline commit: `5c15d5a` (B11 released)
- Target/resulting version: `0.0.8` (measurement batch; no artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned) plus the XWayland `GDK_SCALE=2` run for the factor-2 measurement
- Repository history mirror: this file (created at batch close)

## Non-goals

- Release-build code paths, product behavior, UI, dependency, capability, or network changes.
- Claims about keyboard routing or screen-reader behavior that were not measured.
- Rewriting ADR 0012.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B12-S01 | [#80](https://github.com/AlexandreZanata/brain-root-idea/issues/80) | Debug-only helpers and harness phases measured the production preview: scale factor and exact logical allocation at scale 1 and 2, DPR 1.0/2.0, page zoom 480 → 240 → 480 CSS px, widget focus true with the window active, and a 100-update bounds soak (p50 0 µs, max 31 µs, RSS +20 KB, child count stable) with clean exits and no leftovers; recorded in `docs/specs/preview-quality-measurements.md` and `docs/17` | `208198b` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/80#issuecomment-5800010324) | Keyboard event routing and screen-reader behavior stay `UNKNOWN` (no synthetic input on this stack); the factor-2 measurement runs under XWayland `GDK_SCALE=2` because Wayland ignores `GDK_SCALE`, and is labelled as such | Closed |
| B12-S02 | [#82](https://github.com/AlexandreZanata/brain-root-idea/issues/82) | Finalization: version `0.0.8` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.8` section; this history record created; PR #81 marked Ready and the full Linux gate run on the latest head; merge/tag deferred to maintainer approval | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/82) | The authenticated repository owner cannot approve its own PR; no tag is created before the merge commit exists on `main` | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B12-S01): the remaining ADR 0012 acceptance checks are closed for the measured environments — scale/DPR, page zoom, widget-level focus, and a bounded resize soak — with every number `MEASURED` and labelled by environment; no claim is made about keyboard routing or screen-reader behavior inside preview content.
- Decision (B12-S01): the measurements run through the production preview module via `#[cfg(debug_assertions)]` helpers, so release builds never include or execute the harness.
- Decision (B12-S02): a measurement-only batch still changes the version exactly once at finalization (`0.0.8`) per `docs/19-release-and-versioning.md`, with no artifact; merge, tag, and release wait for maintainer approval.

## Failures and recovery

- The first harness run read `innerWidth` before the page had loaded and the widget allocation before layout; the harness now retries both with bounded deadlines, and no gate was weakened.

## Final gates

- Full CI: `check-full-linux` on the finalization head — run URL recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/82) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B12-Preview-Quality)
- Review: requested; awaiting maintainer approval (documented single-maintainer administrator path)
- Security/privacy: debug-only code and documentation; `check-security.sh` green with the dependency list unchanged
- Performance: measured soak p50 0 µs, max 31 µs per bounds update; RSS +20 KB over 100 updates; child processes stable
- Cleanup: both harness runs exited 0 with no app process, listener, or fixture server left behind
- Artifact/checksum: none for this batch
- Known limitations: keyboard routing and screen-reader behavior remain `UNKNOWN`; the soak is 100 updates, not a long-duration soak; visual confirmation still needs the rendered check

## Result and next batch

B12 closes the CB-A A6 quality slice: the preview's scale/DPR behavior, page zoom, widget focus, and resize soak are measured and recorded, and the unmeasurable keyboard routing stays explicit instead of being claimed. Next: the roadmap's next capability step in its own batch. Rollback: revert the batch commits; no runtime state, tag, or artifact is affected.
