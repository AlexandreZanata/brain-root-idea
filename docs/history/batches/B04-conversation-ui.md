# B04 — Minimal conversation experience

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B04-Conversation-UI). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: A Linux user can type one prompt into the agent surface, watch a streamed answer appear, cancel an active request, and understand every setup or failure state — while the larger Canvas region stays a clear "Preview comes in MVP-1" placeholder and the provider stays behind the frozen provider-neutral contract.
- Branch: `batch/b04-conversation-ui` (deleted after merge)
- Draft/final PR: [#33](https://github.com/AlexandreZanata/brain-root-idea/pull/33)
- Merge commit: pending (filled by the documented post-merge finalization commit)
- Baseline commit: `9f115ff27c42ba15bac6a30af425bc09a9f4e63a`
- Target/resulting version: `0.0.1-alpha.5` (annotated tag on the merge commit; pre-release URL filled post-merge)
- Started/completed: 2026-09-22 / —
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned)

## Non-goals

- Code edits, tools, checkpoints, browser preview, project opening; a full editor, file tree, terminal, technical dashboard, provider marketing, or markdown ecosystem.
- Conversation branching, multiple conversations, model marketplace, Windows, macOS.
- A credential-entry form or a model picker; the initial live loop reads the Secret Service entry and the configured default model.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B04-S01 | [#32](https://github.com/AlexandreZanata/brain-root-idea/issues/32) | Conversation state machine (`EMPTY`/`READY`/`SENDING`/`STREAMING`/`CANCELLING`/`SUCCEEDED`/`FAILED`) over `Submit`/`Cancel`/`Stream(StreamEvent)` with a typed `TransitionError`; 14 tests | `ba439d0` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/32#issuecomment-5781138557) | A cancelled request returns to `READY` and completion after cancel is rejected; the caller surfaces the rejection in S04 | Closed |
| B04-S02 | [#34](https://github.com/AlexandreZanata/brain-root-idea/issues/34) | Agent-first layout in `src/App.svelte`: Focus preset 30/70, core status preserved, labeled prompt field with an inert send control, the Canvas placeholder, focus-visible, reduced motion, and measured contrast | `1c2b690` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/34#issuecomment-5781229691) | Static gates only; the rendered window needed the later smoke (no UI automation) | Closed |
| B04-S03 | [#35](https://github.com/AlexandreZanata/brain-root-idea/issues/35) | Versioned `conversation_send`/`conversation_event` IPC; live OpenCode Go worker with Secret Service credential, compatible default-model discovery, debug/test fake injection; bounded Svelte history and duplicate-submit prevention | `fca7b64` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/35#issuecomment-5781606373), [live](https://github.com/AlexandreZanata/brain-root-idea/issues/35#issuecomment-5781738750) | Live re-verification showed the authenticated models payload omits `endpoint`, invalidating the B03-S02 assumption; remediation opened as R01 | Closed |
| B04-R01 | [#36](https://github.com/AlexandreZanata/brain-root-idea/issues/36) | Discovery accepts entries without a stated endpoint; a stated unsupported endpoint stays filtered; `select_default_model` accepts the configured default with `None` or `ChatCompletions` | `e923f41` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/36#issuecomment-5781806947) | Live smoke then passed end to end (`models=33`; chat `chunks=1`, normal terminal) | Closed |
| B04-S04 | [#37](https://github.com/AlexandreZanata/brain-root-idea/issues/37) | Cancellation end to end: immediate `CANCELLING`, shared B02 token, exactly one `cancelled` terminal, late-terminal safety net, window-close cancel, `cancelTurn` in the UI | `6e07da4` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/37#issuecomment-5782029595) | A cancel during a blocked socket read is bounded by the 120 s transport timeout; window automation is not approved | Closed |
| B04-A01 | [#39](https://github.com/AlexandreZanata/brain-root-idea/issues/39) | Core reorganized into `features::conversation` (`state`, `wire`, `catalog`, `runner`, `runtime`, interface) and `features::health`; private submodules and the new `scripts/check-modules.sh` enforce the registry; staged `dead_code` allowances removed | `410f8b4` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/39#issuecomment-5782273331) | Tauri's command macro requires the definition path, so the commands live in the feature `mod.rs` (no `ipc.rs`); clippy flagged a private type in the cancel signature, fixed by returning unit | Closed |
| B04-S05 | [#40](https://github.com/AlexandreZanata/brain-root-idea/issues/40) | Action-oriented copy for timeout/network/provider/malformed plus credential and no-model messages; `provider_status` read once; "Needs setup" with Send disabled; sanitized code in a "Technical details" element | `f2797a7` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/40#issuecomment-5782371503) | The naive `log::` grep false-positives on `catalog::`; the bounded `\blog::` pattern has no matches | Closed |
| B04-S06 | [#41](https://github.com/AlexandreZanata/brain-root-idea/issues/41) | Accessibility and secret-safety verification: history `aria-live` removed, `Send`/`Cancel` use `aria-disabled` so focus is never dropped, disabled contrast fixed, and `scripts/check-accessibility.sh` runs in both gates | `b8a79c2` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/41#issuecomment-5782442849) | Disabled label contrast was 4.15:1 and was raised to 7.11:1; keyboard/screen-reader behavior is verified statically, not by UI automation | Closed |
| B04-S07 | [#42](https://github.com/AlexandreZanata/brain-root-idea/issues/42) | Version `0.0.1-alpha.5` synchronized across `VERSION`, `Cargo.toml`, `package.json`, and the lockfile; changelog section; final history record; PR made Ready; full CI; merge commit; annotated tag and pre-release | pending (merge) | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/42) | Ecosystem copies plus the history record were in the allowlist; this record is finalized by the documented post-merge commit | Closed |

## Decisions and changed assumptions

- Decision (B04-S01): the reducer is pure and stores no prompt or response text; settled states accept a new submit, a cancelled request returns to `READY`, and late or duplicate events are rejected.
- Decision (B04-S02): the shell uses the `docs/03` Focus preset (30/70) with a single-column fallback below 840px and keeps the B01-S03 core readiness line as the only status.
- Decision (B04-S03): live OpenCode Go is the normal path; the deterministic fake is injectable for tests and selectable only in debug builds. The live loop reads `dev.brainroot.experiment/default` from Linux Secret Service, uses one on-demand `std::thread` for the blocking request, and emits only versioned neutral events; the scoped runtime decision is recorded in `docs/17-open-questions.md`.
- Decision (B04-R01): discovery accepts entries that do not state an endpoint and `select_default_model` accepts the configured default with `None` or `ChatCompletions`; a stated unsupported endpoint is still filtered and the request still targets the documented `chat/completions` endpoint with no fallback.
- Decision (B04-S04): cancel reuses the B02 `CancellationToken`, moves the state to `CANCELLING` immediately, and releases ownership on exactly one `cancelled` terminal; a cancelled turn is not a failure, and the close handler calls the same path guarded by `is_active()`.
- Decision (B04-A01): the core exposes exactly `features::conversation::{ConversationSession, conversation_send, conversation_cancel}` and `features::health::health`; everything else is a private submodule, and `scripts/check-modules.sh` fails on undeclared module files or unregistered feature paths.
- Decision (B04-S05): default failure copy states what stopped and the next action, with the neutral error code available separately; the credential status is read once at mount and gates Send.
- Decision (B04-S06): live regions are exactly the polite status line, the setup `role="status"`, and the failure `role="alert"`; Send/Cancel keep focus via `aria-disabled`; the accessibility invariants are enforced by `scripts/check-accessibility.sh` in both gates.
- Changed assumption (B04-S03/R01): the authenticated `GET /zen/go/v1/models` payload states only `id`/`object`/`created`/`owned_by`, invalidating the B03-S02 endpoint-required parsing; the live path was fixed and re-verified.

## Failures and recovery

- 2026-09-22, B04-S03: the live smoke stopped at discovery with `observed entry field names: ["created", "id", "object", "owned_by"]`; the endpoint-required assumption was invalidated and fixed under R01 (`e923f41`), after which the full smoke passed.
- 2026-09-22, B04-A01: clippy rejected `cancel` returning a state type from a private module and a write-only `active` flag; resolved by returning unit from `cancel` and guarding the close handler with `is_active()`.
- 2026-09-22, B04-A01: Tauri's command macro must resolve the command at its definition path, so the commands live in `features/conversation/mod.rs` instead of a separate `ipc.rs`; no interface change.
- 2026-09-22, B04-S05: the naive `log::` log scan matched `catalog::select_default_model`; the bounded `\blog::` pattern confirmed there is no logging.
- 2026-09-22, B04-S06: the disabled Send label measured 4.15:1; raised to 7.11:1 as part of the accessibility verification.

## Final gates

- Full CI: pending (filled by the documented post-merge finalization commit)
- Review: single-maintainer exception; merge performed with the documented administrator bypass (`enforce_admins: false`)
- Security/privacy: the S06 scan covered 118 files with no key material, and the built bundle contains no `Authorization`, `Bearer`, or `oc_sk_` material; the credential stays core-only and the live smoke recorded counts and timing only
- Performance: not applicable to this batch; no measurement was taken, and the live smoke timings are single-run observations, not a budget report
- Cleanup: cancellation and terminal events release request ownership (`active`/`cancel` cleared); no worker exists at idle
- Artifact/checksum: none for this batch; the pre-release states the absence explicitly
- Known limitations: the rendered window still needs the maintainer/release smoke for visual confirmation; a cancel during a blocked socket read is bounded by the 120 s transport timeout; the OpenCode Go catalog, endpoints, limits, privacy terms, and authenticated models shape are externally mutable
- Wiki/repository history comparison: pending (B04-S07)

## Result and next batch

Batch B04 is released as `v0.0.1-alpha.5`: the minimal Linux conversation loop — explicit conversation state, an agent-first layout, typed streaming IPC against the live OpenCode Go transport, cancellation and close cleanup, action-oriented setup and failure states, and enforceable feature modules with accessibility and module gates — still without tools, Canvas, checkpoints, or project mutation. Rollback: revert the batch commits; after merge the merge commit stays unless a maintainer explicitly reverts it, and the published tag is never moved. This final record is completed by the documented post-merge commit on `main` because the merge commit, CI runs, tag, and release URL cannot exist before the merge.

Next batch: B05 — Linux MVP-0 hardening and release (`batch/b05-linux-mvp0-release`), starting with the end-to-end fake-provider journey.
