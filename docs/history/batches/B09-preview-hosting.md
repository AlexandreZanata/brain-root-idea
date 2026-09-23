# B09 — Linux preview hosting decision (CB-A gate 2)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B09-Preview-Hosting). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Ready (awaiting maintainer approval to merge)
- Objective: Complete the ADR 0006 review with measured evidence so the CB-A preview batch can be assigned — test an alternative Linux hosting path after the managed child-WebView geometry no-go, record the hosting decision and dependency review, and leave the shipped app unchanged.
- Branch: `batch/b09-preview-hosting`
- Draft/final PR: [#69](https://github.com/AlexandreZanata/brain-root-idea/pull/69)
- Baseline commit: `179b3fa668fa6bf87bde8116e428008e0166e65f`
- Target/resulting version: `0.0.5` (decision batch; no artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned, Node.js v26.3.1, pnpm 11.13.0)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Preview implementation, dev-server start, Canvas UI, navigation UI, or IPC.
- Any dependency shipped in the default build; `gtk`/`wry` stay optional behind the `probe` feature.
- Windows/macOS, Human Browser, import, Deck, phone, or remote browsing.
- Assigning the CB-A implementation batch ID until the hosting ADR is accepted.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B09-S01 | [#68](https://github.com/AlexandreZanata/brain-root-idea/issues/68) | A disposable hosting probe built a `wry` WebView directly into a `gtk::Fixed` overlay inside the Tauri window and measured `go`: two views at exact bounds (0,0,320,200) and (340,0,200,300), exact resize to (60,30,240,180), `file://` denial, shell allocation unchanged, zero fixed children after drop, clean exit, empty stderr; two consecutive release runs `go` | `b11c39c` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/68#issuecomment-5797296771) | Iterations inside the issue: the first run loaded nothing because the wry builder had no URL (fixed before commit); the security gate caught a literal localhost URL in the probe and the prefix is now built from the runtime fixture address; `Cargo.lock` was added to the issue allowlist before the commit | Closed |
| B09-S02 | [#70](https://github.com/AlexandreZanata/brain-root-idea/issues/70) | The hosting decision is recorded as ADR 0012: the Preview role is a `wry` view in a `GtkFixed` overlay outside the Tauri manager (no IPC by construction), one view owned by the Rust core with shell-reported geometry, allowlist and isolated profile at creation, Linux-only, with the unmeasured checks listed as preview-slice acceptance criteria; linked from `docs/08` and `docs/17` | `b034459` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/70#issuecomment-5797322356) | ADR 0006 and ADR 0011 remain unmodified historical records | Closed |
| B09-S03 | [#71](https://github.com/AlexandreZanata/brain-root-idea/issues/71) | Finalization: version `0.0.5` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.5` section; this history record created; PR #69 marked Ready and the full Linux gate run on the latest head; merge/tag deferred to maintainer approval | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/71) | The authenticated repository owner cannot approve its own PR; no tag is created before the merge commit exists on `main` | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B09-S01): host the preview as a `wry` WebView created directly into a `GtkFixed` overlay child of the Tauri window (`GtkOverlay` keeps the shell `gtk::Box` as the main child). The view is outside the Tauri webview manager, so it has no Tauri IPC by construction; the navigation handler enforces the allowlist. Evidence: `docs/specs/linux-preview-hosting-probe.md`.
- Dependency review (B09-S01): `gtk 0.18` and `wry 0.55` become optional direct dependencies for the Linux target behind the existing `probe` feature; both were already transitive to Tauri at exactly these versions, the default build and bundle are unchanged, and the security gate now enumerates target-specific dependency sections while allowing only the approved set.
- Decision (B09-S02): the ADR 0006 review is closed by ADR 0012 — Linux hosts the preview in a `GtkFixed` overlay outside the Tauri webview manager, one Rust-owned view with shell-reported geometry, allowlist and isolated profile at creation; Windows/macOS keep the probe-first rule, and the unmeasured behavior stays explicit until the preview slice measures it.
- Original assumption (ADR 0006 prototype): Tauri-managed child WebViews could be positioned on Linux. Evidence: `docs/specs/linux-webview-probe.md`. Result: rejected for the managed path; the GtkFixed overlay path replaces it.
- Remaining unknowns become CB-A acceptance checks: focus/keyboard routing between shell and preview, zoom/scale behavior beyond scale factor 1, DPR/media behavior, repeated create/destroy soak, and preview crash handling.

## Failures and recovery

- No gate failed and no fixture or gate was weakened. Two probe iterations were fixed inside B09-S01 (missing `with_url` in the disposable probe; a literal localhost URL rejected by the security gate, replaced with runtime construction from the fixture address). The security gate change closes a blind spot: target-specific dependency sections are now enumerated.

## Final gates

- Full CI: `check-full-linux` on the finalization head — run URL recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/71) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B09-Preview-Hosting)
- Review: requested; awaiting maintainer approval (documented single-maintainer administrator path)
- Security/privacy: `check-docs.sh` secret scan green (188 files); `check-security.sh` green with `rust dependencies: gtk, keyring, serde, serde_json, tauri, tauri-build, ureq, uuid, wry`; the probe only contacts its own `127.0.0.1` fixture and the preview view has no Tauri IPC by construction
- Performance: probe-scale timings recorded with `MEASURED` labels (create 51–53 ms, load 150–202 ms in the release probe); no product budget claimed
- Cleanup: probe process exited 0 with no probe process, listener, or profile left behind; zero fixed children after dropping the views
- Artifact/checksum: none for this batch
- Known limitations: focus, zoom/scale, DPR, soak, and crash behavior are unmeasured; the hosting decision is not a preview capability; the probe binaries are disposable and not part of the default build

## Result and next batch

B09 closes the ADR 0006 review: the managed child-WebView path is a measured no-go on geometry, the GtkFixed overlay + direct `wry` hosting path is a measured `go`, and ADR 0012 records the decision with its ownership, security, and unmeasured-acceptance constraints. The preview capability itself remains unimplemented. Next: the CB-A preview batch is assigned with the approved origin policy, ADR 0011, and ADR 0012 as its acceptance constraints. Rollback: revert the batch commits; the shipped app, runtime state, tag, and artifact are unaffected.
