# B16 — Human Browser permission controls (CB-B B5)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B16-Permission-Controls). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: Deny every web permission fail-closed with recorded codes and let the user clear the BrainRoot-owned browser data, with the deny/clear paths measured before any permission UX ships.
- Branch: `batch/b16-permission-controls` (deleted after merge)
- Draft/final PR: [#95](https://github.com/AlexandreZanata/brain-root-idea/pull/95)
- Merge commit: `2bf1c262e64126f75ccfadc7ccf7d357b2dda663`
- Baseline commit: `7de12cf` (B15 released)
- Target/resulting version: `0.0.12` (annotated tag on the merge commit; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.12) without artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Downloads, import, agent sharing, and granting any permission; the first slice stays deny-only.
- Any policy, capability, or dependency change.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B16-S01 | [#94](https://github.com/AlexandreZanata/brain-root-idea/issues/94) | Permission denial and browser-data clear: the WebKit permission handler denies every request with `human_permission_denied:<kind>`, and `human_browser_clear_data` destroys the view, drops the cached context, and removes the profile data; the harness proves `permission_denied: true` and `data_cleared: true` with navigation intact, and a debug-only profile override lets the command path run against a probe directory | `331e6e1` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/94#issuecomment-5802285620) | The first clear-data attempt used the real profile root while the harness used a probe root; the debug override fixed the mismatch and the gate stayed unmodified | Closed |
| B16-S02 | [#96](https://github.com/AlexandreZanata/brain-root-idea/issues/96) | The Canvas Browser surface shows the permission block as one plain-language line (no grant path) and offers a confirmed **Clear browser data** action (Clear everything / Cancel) that calls the existing command; frontend tests cover the permission mapping and all gates pass | `36f710a` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/96#issuecomment-5802311033) | No engine change; the control reuses the B16-S01 command | Closed |
| B16-S03 | [#97](https://github.com/AlexandreZanata/brain-root-idea/issues/97) | Finalization: version `0.0.12` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.12` section; this history record created; PR #95 marked Ready and the full Linux gate run on the latest head; merge/tag deferred to maintainer approval | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/97) | The authenticated repository owner cannot approve its own PR; no tag is created before the merge commit exists on `main` | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B16-S01): permission requests are denied fail-closed with an honest kind code; a new WebKit permission type is denied without a code change.
- Decision (B16-S01): in the current single-profile design, clear site data and profile reset are the same operation — BrainRoot owns the whole profile and there is no separate settings store; the command is labeled "clear browser data".
- Decision (B16-S02): the surface never offers a permission grant; the denial is explained in product language and the clear action requires an explicit confirmation.
- Decision (B16-S03): a capability batch still changes the version exactly once at finalization (`0.0.12`) per `docs/19-release-and-versioning.md`, with no artifact. Resolution at merge: the authenticated owner cannot self-approve, so the merge used the documented single-maintainer administrator path; the annotated tag was created only after the merge commit existed on `main`.

## Failures and recovery

- The first clear-data run failed because the command cleared the real profile while the harness used a probe root; a debug-only profile override points the command at the probe root, and the harness now returns `go`. No gate was weakened.

## Final gates

- Full CI: [`check-full-linux` success on the finalization head](https://github.com/AlexandreZanata/brain-root-idea/actions/runs/35915859872) (`f564a68`); post-merge `push` runs on `main` for `2bf1c26` and the finalization commit are recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/97) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B16-Permission-Controls)
- Review: single-maintainer administrator bypass documented; merged at B16-S03
- Release: annotated `v0.0.12` on merge commit `2bf1c26`; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.12) without artifact
- Security/privacy: every permission denied fail-closed; clear-data removes only the BrainRoot-owned profile; the harness runs against a probe root
- Performance: no new resource; denial is in-process
- Cleanup: all three harnesses exit 0 with no process, listener, or probe profile behind
- Artifact/checksum: none for this batch
- Known limitations: permissions are deny-only; downloads and import remain unimplemented; the failure/compatibility UX stays minimal

## Result and next batch

Batch B16 is released as [`v0.0.12`](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.12) on merge commit `2bf1c26`: the browser denies every web permission fail-closed with plain-language reporting and clears its BrainRoot-owned data on explicit confirmation. This final record was completed in one documented post-merge commit on `main` through the administrator bypass because the merge commit, CI run, tag, and release URL cannot exist before the merge. Next: the remaining browser candidates (portable import, phone presentation) or the roadmap's deferred phases. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier.
