# B08 — Companion Browser prerequisites (CB-A gate)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B08-Preview-Prerequisites). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Ready (awaiting maintainer approval to merge)
- Objective: Resolve the CB-A prerequisites for the Companion Canvas before any preview implementation is assigned — the Linux child-WebView probe with a measured go/no-go, the approved localhost preview origin policy, and the owned dev-server lifecycle contract — without shipping any preview, browser, or navigation capability.
- Branch: `batch/b08-preview-prerequisites`
- Draft/final PR: [#64](https://github.com/AlexandreZanata/brain-root-idea/pull/64)
- Baseline commit: `b3b07adf7f855b48f5064af80b15210460b1e291`
- Target/resulting version: `0.0.4` (prerequisites only; no artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned, Node.js v26.3.1, pnpm 11.13.0)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Any preview WebView, dev-server start, browser, navigation UI, or IPC in the product.
- Any new runtime dependency; the probe uses the existing `tauri` dependency behind an optional Cargo feature.
- Remote browsing, Human Browser, import, Deck, phone, Windows, or macOS work.
- Assigning CB-A implementation batch IDs; those wait for the probe's accepted decision and the ADR 0006 review.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B08-S01 | [#63](https://github.com/AlexandreZanata/brain-root-idea/issues/63) | Disposable Linux child-WebView probe built behind an optional `probe` feature: 100/100 create/load/destroy cycles, profile isolation, forced-close recovery, and `file://` navigation denial pass; child-view position/size is not honored on the Wayland stack (children pack into the window `GtkBox`); measurements and a **no-go on geometry** are recorded in `docs/specs/linux-webview-probe.md`, and the ADR 0006 review is required before preview work | `f4d1b44` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/63#issuecomment-5796469358) | The spike's decisive result is a no-go on one CB-A assumption; the shipped app and default build are unchanged | Closed |
| B08-S02 | [#65](https://github.com/AlexandreZanata/brain-root-idea/issues/65) | The Preview origin policy is approved: canonical loopback schemes/hosts on BrainRoot-owned dev-server ports, fail-closed denial rules, denial behavior, the Rust-owned A4 enforcement contract, and the negative test corpus; linked from `docs/08`, `docs/11`, and `docs/17`; no runtime code ships | `7696830` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/65#issuecomment-5796515026) | The policy is explicitly **not yet enforced**; enforcement and its tests belong to CB-A A4 | Closed |
| B08-S03 | [#66](https://github.com/AlexandreZanata/brain-root-idea/issues/66) | The owned dev-server lifecycle contract is approved as ADR 0011: single owner per project, declared command only, assigned loopback port and registry entry, process-group termination with bounded grace, readiness/crash handling without restart loops, stop-on-close idle policy, untrusted bounded output, and the A4 evidence list; linked from `docs/09` and `docs/17` | `47df101` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/66#issuecomment-5796555299) | No dependency is approved; process-group termination review belongs to the implementing issue | Closed |
| B08-S04 | [#67](https://github.com/AlexandreZanata/brain-root-idea/issues/67) | Finalization: version `0.0.4` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.4` section; this history record created; PR #64 marked Ready and the full Linux gate run on the latest head; merge/tag deferred to maintainer approval | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/67) | The authenticated repository owner cannot approve its own PR; no tag is created before the merge commit exists on `main` | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B08-S01): the probe is a disposable binary behind the optional `probe` Cargo feature (`tauri/unstable`); the shipped app, its dependency list, and its default build are unchanged.
- Decision (B08-S01): **no-go on child-view geometry** for the current Tauri/wry stack on Wayland — position and size are ignored because children attach to the window's `GtkBox`; the plan's stop rule applies, so ADR 0006 is reviewed with the probe's options before any preview implementation batch is assigned. Create/destroy, localhost navigation policy, profile isolation, and forced-close recovery are feasible and recorded as `MEASURED`.
- Decision (B08-S02): the Preview role loads only canonical loopback origins on BrainRoot-owned dev-server ports, fail-closed, with no privileged IPC and no in-preview permission prompts; the policy ships as an approved gate with its negative corpus, and enforcement plus tests are CB-A A4 acceptance criteria rather than dead code in B08. See `docs/specs/preview-origin-policy.md`.
- Decision (B08-S03): the preview dev server is owned by the Rust core — one per project, declared command only, BrainRoot-assigned loopback port registered for the preview origin, process-group termination that never kills foreign processes, no automatic restart loop, stop-on-close idle policy, and untrusted bounded output; implementation and its evidence land in CB-A A4 after the ADR 0006 review, and no dependency is approved. See ADR 0011.
- Assumption tested and rejected (Companion Browser plan): a child WebView can be positioned and resized on the Linux reference stack. Evidence: `docs/specs/linux-webview-probe.md`. Result: rejected for now; CB-A implementation is blocked on the ADR 0006 review.
- The remaining prerequisites (origin policy and dev-server lifecycle) are independent of the geometry result and are approved in B08.

## Failures and recovery

- The geometry no-go is recorded in the [Failure Log](https://github.com/AlexandreZanata/brain-root-idea/wiki/Failure-Log) (2026-09-23, B08-S01) as an unresolved blocker pending the ADR 0006 review. No gate failed and no fixture or gate was weakened.

## Final gates

- Full CI: `check-full-linux` on the finalization head — run URL recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/67) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B08-Preview-Prerequisites)
- Review: requested; awaiting maintainer approval (documented single-maintainer administrator path)
- Security/privacy: `check-docs.sh` secret scan and `check-security.sh` green; the probe only contacts its own `127.0.0.1` fixture; no runtime surface or destination added to the product
- Performance: probe measurements recorded with `MEASURED` labels; no budget claimed
- Cleanup: probe process exited 0 with no remaining probe process; one shared WebKit process outlives all closed views and is recorded as an observation
- Artifact/checksum: none for this batch
- Known limitations: child-view geometry unsupported on the current stack; memory growth over 100 cycles unclassified; focus/WARM behavior unmeasured; the origin policy and ADR 0011 are not implemented yet

## Result and next batch

B08 resolves the CB-A prerequisites: the probe measured what the current stack supports and rejected the geometry assumption with evidence, the preview origin policy is approved with its negative corpus, and the owned dev-server lifecycle is accepted as ADR 0011. The preview capability itself remains unimplemented. Next: the ADR 0006 review with the probe's recorded options (a `GtkFixed` hosting path, a native-window preview surface, upstream support, or the X11 native-child path); after that decision, the CB-A implementation batch is assigned with the origin policy and lifecycle contract as its acceptance constraints. Rollback: revert the batch commits; no runtime state, tag, or artifact is affected.
