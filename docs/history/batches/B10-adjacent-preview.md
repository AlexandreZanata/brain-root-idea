# B10 — Adjacent localhost preview (CB-A)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B10-Adjacent-Preview). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: The Companion Canvas shows the user's local app beside the chat — BrainRoot owns the dev server, hosts the preview view per ADR 0012, loads only the approved loopback origin, reports truthful states, and cleans up on close — without remote browsing or project mutation.
- Branch: `batch/b10-adjacent-preview` (deleted after merge)
- Draft/final PR: [#73](https://github.com/AlexandreZanata/brain-root-idea/pull/73)
- Merge commit: `db3a025cfae149ac8441697e5b4b643f860f1ee5`
- Baseline commit: `c24542db205dd6ba64a7285863324ffa57a4d384`
- Target/resulting version: `0.0.6` (annotated tag on the merge commit; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.6) without artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned, Node.js v26.3.1, pnpm 11.13.0)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Remote browsing, Human Browser, data import, Deck, phone, Windows, and macOS.
- Project opening and file mutation; the preview command is declared explicitly.
- Persistence of the preview command or the running server across restarts.
- Any privileged Tauri IPC from preview content.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B10-S01 | [#72](https://github.com/AlexandreZanata/brain-root-idea/issues/72) | The Rust core owns the dev-server lifecycle per ADR 0011: assigned loopback port, declared command with `PORT`, process-group spawn, bounded readiness with early-exit detection, SIGTERM/grace/SIGKILL group stop, bounded never-logged output, and the typed `preview_start`/`preview_stop`/`preview_status` commands with stop-on-close; seven unit tests pass with a real `sh`/`python3` fixture and no leftover process or listener | `22e3ad1` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/72#issuecomment-5797895377) | `libc 0.2` approved for `killpg` only (std spawns the group but cannot signal it); no persistence and no UI yet, so the command surface is trusted-shell-only until the Canvas permission moment lands | Closed |
| B10-S02 | [#74](https://github.com/AlexandreZanata/brain-root-idea/issues/74) | The production preview view is implemented per ADR 0012: a `wry` WebView in a `GtkFixed` overlay outside the Tauri manager with no Tauri IPC, `preview_show`/`preview_set_bounds`/`preview_view_status`/`preview_hide`, the origin policy with its corpus, an isolated profile under the app data directory, and destroy on hide/close; eight tests pass and the debug harness proves the real view end to end with a clean shutdown and no leftovers | `8f69fd0` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/74#issuecomment-5798332494) | The origin corpus moved to `src-tauri/tests/preview-origin-corpus.json` (test data outside the gate's scanned destination surface); the host window resolves through `get_webview_window` because `Manager::get_window` is behind `tauri/unstable` | Closed |
| B10-S03 | [#75](https://github.com/AlexandreZanata/brain-root-idea/issues/75) | The Canvas Preview tab is integrated: permission surface (command + folder), states from typed `preview-status` events without polling, slot geometry sync, failed state with retry, stop/hide and cleanup, plus a crash-after-ready monitor in the supervisor; 11 frontend tests and 9 Rust tests pass, the accessibility/security/docs gates are green and the harness still proves the view end to end | `efefc4e` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/75#issuecomment-5798543063) | One fixture process from the pre-fix failing crash test was cleaned before the commit; the fixed test leaves nothing behind | Closed |
| B10-S04 | [#76](https://github.com/AlexandreZanata/brain-root-idea/issues/76) | Finalization: version `0.0.6` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.6` section; this history record created; PR #73 marked Ready, merged with a merge commit through the documented single-maintainer administrator bypass, branch deleted, annotated `v0.0.6` tagged with a pre-release, and this record completed on `main` | `9de9512`, `db3a025` (merge) | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/76) | Merge and release used the documented single-maintainer administrator bypass; no artifact for this batch | Closed |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B10-S01): one supervisor per process, in-memory only; the declared command receives the assigned port through the `PORT` environment variable; readiness is port-connectable and process-alive within 1–60 s (default 30 s); stop is SIGTERM, 0.2–10 s grace (default 5 s), then SIGKILL on the owned process group only; output is bounded at 8 KiB per stream and never logged.
- Dependency review (B10-S01): `libc 0.2` becomes a direct Unix dependency for `killpg`; it was already transitive to the stack, the bundle is unchanged, and the security gate allows exactly the approved set (`gtk`, `keyring`, `libc`, `serde`, `serde_json`, `tauri`, `tauri-build`, `ureq`, `uuid`, `wry`).
- Decision (B10-S02): the preview view is hosted per ADR 0012 — `wry` WebView in a `GtkFixed` overlay outside the Tauri webview manager (no Tauri IPC by construction), one view at most, shell-supplied geometry, the origin allowlist from the policy, and an isolated `preview-profile` under the app data directory; `gtk`/`wry` are promoted from optional probe dependencies to plain Linux dependencies.
- Decision (B10-S03): the Canvas Preview tab is the visible permission moment (one command, one folder, plain-language explanation); states come from typed `preview-status` events plus one initial read, so no polling exists; geometry is reported from the slot element with a `ResizeObserver` and `requestAnimationFrame` debounce; the supervisor watches the owned server after ready and surfaces `preview_exited_early` as a visible failed state.
- Known limitation (B10-S02): cleanup is proven on the graceful shutdown path; an abrupt `SIGKILL` of the application leaves the dev server to the OS and is not covered.
- Known limitation (B10-S03): visual alignment of the view over the Canvas slot still needs the maintainer's rendered confirmation; focus/zoom/DPR/soak remain unmeasured; the command and folder are in-memory only and project opening does not exist yet.

## Failures and recovery

- B10-S02: the origin corpus initially lived in Rust string literals and the security gate correctly flagged destination-like URLs; it moved to `src-tauri/tests/preview-origin-corpus.json` (test data outside the scanned destination surface) with the issue allowlist updated before the commit.
- B10-S03: the first crash-after-ready fixture used `kill %1`, which is unreliable in non-interactive `sh`; the fixture now captures `server=$!` and kills the pid. The pre-fix failing run left one fixture process, cleaned before the commit, and the fixed test leaves nothing behind. No gate failed and no fixture or gate was weakened.

## Final gates

- Full CI: [`check-full-linux` success on the finalization head](https://github.com/AlexandreZanata/brain-root-idea/actions/runs/35889105296) (10 m 53 s, `9de9512`); post-merge `push` runs on `main` for `db3a025` and the finalization commit are recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/76) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B10-Adjacent-Preview)
- Review: single-maintainer administrator bypass documented; merged at B10-S04
- Release: annotated `v0.0.6` on merge commit `db3a025`; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.6) without artifact
- Security/privacy: `check-docs.sh` secret scan green (199 files); `check-security.sh` green with the approved dependency list; the preview view has no Tauri IPC by construction; the origin policy rejects the negative corpus; the address is rendered from runtime values only; child output is bounded and never logged or shown
- Performance: no new budget; supervisor readiness is bounded by the declared timeout; no polling loop exists (events plus one initial read); the harness and unit tests run in seconds
- Cleanup: supervisor tests prove group stop and port rebinding; the debug harness proves the real view is destroyed and the owned server stopped with no leftover process or listener; the fixed crash test leaves nothing behind
- Artifact/checksum: none for this batch
- Known limitations: visual alignment still needs the rendered confirmation; focus/zoom/DPR/soak remain unmeasured; the command and folder are in-memory only and project opening does not exist yet

## Result and next batch

Batch B10 is released as [`v0.0.6`](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.6) on merge commit `db3a025`: the user states a command and folder, BrainRoot owns one dev server on an assigned loopback port, the preview view renders beside the chat with no Tauri IPC, states and failures are truthful and event-driven, and everything stops on request or close. This final record was completed in one documented post-merge commit on `main` through the administrator bypass because the merge commit, CI run, tag, and release URL cannot exist before the merge. Next: the roadmap's next capability step in its own batch, with the unmeasured acceptance checks (focus/keyboard routing, zoom/scale, DPR, soak) kept explicit. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier.
