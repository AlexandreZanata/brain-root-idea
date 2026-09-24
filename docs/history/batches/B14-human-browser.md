# B14 — Human Browser: minimal human navigation (CB-B)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B14-Human-Browser). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: Add the first user-directed Human Browser slice per ADR 0013 — typed navigation policy, one isolated remote view with a persistent BrainRoot-owned profile and no BrainRoot IPC, and a Canvas surface with address entry, back/forward/reload, title/URL, external-open, and clear error states — explicitly reordering the roadmap to prioritize the browser.
- Branch: `batch/b14-human-browser` (deleted after merge)
- Draft/final PR: [#88](https://github.com/AlexandreZanata/brain-root-idea/pull/88)
- Merge commit: `c88574394b4a607719c4117adc816db9bd6ab6fd`
- Baseline commit: `6bf6526` (B13 released)
- Target/resulting version: `0.0.10` (annotated tag on the merge commit; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.10) without artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Downloads, popups, permission prompts, history/bookmark/cookie import, and any agent sharing or credential bridge.
- Deck switching, phone presentation, Deck snapshots, and remote/social destinations.
- Reading installed browser profiles.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B14-S01 | [#87](https://github.com/AlexandreZanata/brain-root-idea/issues/87) | The Human Browser engine: typed policy (`http`/`https` allow, `mailto`/`tel`/`sms`/`magnet` external-open, everything else denied with codes, userinfo denied), one isolated `wry` view in the shared Canvas overlay with the persistent `human-profile`, denied popups/downloads, title/URL/history status, and show/navigate/back/forward/reload/set_bounds/hide/status commands; the policy corpus test and the debug harness pass (`go`: /a → /b → back → forward, `file://` denial, view destroyed), and the preview harness plus its tests still pass after the shared-host refactor | `6ee3a80` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/87#issuecomment-5801209547) | `webkit2gtk 2.0` added direct for history state that wry does not surface; the Canvas surface is B14-S02 | Closed |
| B14-S02 | [#89](https://github.com/AlexandreZanata/brain-root-idea/issues/89) | The Canvas Browser surface: address entry, Back/Forward/Reload with truthful `aria-disabled` states, title/address line, blocked-link alert, slot geometry sync, one visible content slot with tab switching, and typed `human-browser-status` events instead of polling; 16 frontend tests pass, accessibility/security/docs gates are green, and both harnesses stay `go` | `21c5abf` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/89#issuecomment-5801265977) | The frontend test reads URLs from the shared corpus fixture because the security gate correctly rejects destination literals in source | Closed |
| B14-S03 | [#90](https://github.com/AlexandreZanata/brain-root-idea/issues/90) | Finalization: version `0.0.10` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.10` section; this history record created; PR #88 marked Ready and the full Linux gate run on the latest head; merge/tag deferred to maintainer approval | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/90) | The authenticated repository owner cannot approve its own PR; no tag is created before the merge commit exists on `main` | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B14-S01): the human view shares the Canvas `GtkFixed` host with the preview (extracted to `features/canvas_host.rs`), stays outside the Tauri webview manager, and keeps at most one instance.
- Decision (B14-S01): external protocol handlers are surfaced as an explicit user action (`OpenExternal`) and never loaded; popups and downloads are denied and recorded; the status carries the last denial code for the shell.
- Dependency review (B14-S01): `webkit2gtk 2.0` is a direct Linux dependency for `can_go_back`/`can_go_forward`/`go_back`/`go_forward`; it was already transitive at exactly this version, the bundle is unchanged, and the security gate now allows it.
- Decision (B14-S02): the Canvas has two live destinations (Preview and Browser) and one content slot: switching tabs unmounts the other surface, whose cleanup hides its view; link-driven changes reach the shell through the typed `human-browser-status` event, not polling.
- Decision (B14-S02): the address field normalizes a bare host to `https://` and passes explicit schemes through so the engine policy stays the only authority.
- Roadmap deviation (B14, maintainer order): the Human Browser slice was implemented ahead of the roadmap's Phases 3/5/6/7; the reorder is explicit and recorded, and the remaining browser work (Deck, permissions, import) stays open.
- Decision (B14-S03): a capability batch still changes the version exactly once at finalization (`0.0.10`) per `docs/19-release-and-versioning.md`, with no artifact. Resolution at merge: the authenticated owner cannot self-approve, so the merge used the documented single-maintainer administrator path; the annotated tag was created only after the merge commit existed on `main`.

## Failures and recovery

- The shared-host refactor initially duplicated the module path and left unused imports; both were corrected before the commit and no gate was weakened.
- The frontend test first embedded destination literals and the security gate correctly rejected them; the test now reads the shared corpus fixture, and the gate stayed strict.

## Final gates

- Full CI: [`check-full-linux` success on the finalization head](https://github.com/AlexandreZanata/brain-root-idea/actions/runs/35908488331) (`d711b95`); post-merge `push` runs on `main` for `c885743` and the finalization commit are recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/90) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B14-Human-Browser)
- Review: single-maintainer administrator bypass documented; merged at B14-S03
- Release: annotated `v0.0.10` on merge commit `c885743`; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.10) without artifact
- Security/privacy: the policy denies risky schemes, userinfo URLs, popups, and downloads; the view has no BrainRoot IPC and uses a BrainRoot-owned persistent profile; `check-security.sh` green with `webkit2gtk` on the approved list
- Performance: one remote view at most, created on demand and destroyed on hide or close; no polling
- Cleanup: both harnesses exited 0 with no process, listener, or probe profile left behind
- Artifact/checksum: none for this batch
- Known limitations: downloads, permission prompts, import, agent sharing, Deck switching, and phone presentation are unimplemented; site compatibility findings, cookie/cache isolation, deletion/reset, crash recovery, and accessibility inside remote content remain open

## Result and next batch

Batch B14 is released as [`v0.0.10`](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.10) on merge commit `c885743`: a typed policy that allows `http`/`https` and denies the risky surface, one isolated remote view with a BrainRoot-owned profile and no IPC, and a Canvas Browser tab with address entry, navigation controls, truthful states, and plain-language denials. The maintainer explicitly reordered the roadmap for this work. This final record was completed in one documented post-merge commit on `main` through the administrator bypass because the merge commit, CI run, tag, and release URL cannot exist before the merge. Next: the remaining browser candidates (Deck switching, permission controls, import) or the roadmap's deferred phases, whichever the maintainer schedules. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier.
