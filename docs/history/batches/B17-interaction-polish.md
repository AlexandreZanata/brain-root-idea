# B17 — Interaction and performance polish

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B17-Interaction-Polish). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: Make the conversation surface read as one compact, truthful task narrative, stream without visual churn, keep keyboard/focus/scroll predictable, and switch the Canvas between Preview and Browser with honest loading/COLD states — under the release-gate test cadence of ADR 0014.
- Branch: `batch/b17-interaction-polish`
- Draft/final PR: [#99](https://github.com/AlexandreZanata/brain-root-idea/pull/99)
- Merge commit: `e25f32f51c06bcf12b2eb4ab6ad233f4e373bdde`
- Baseline commit: `0f8cdb5` (B16 released)
- Target/resulting version: `0.0.13` (no artifact)
- Started/completed: 2026-09-24 / 2026-09-24
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned)
- Repository history mirror: this file (created at batch close)

## Non-goals

- New providers, agents, tools, dependencies, browser-data import, multi-agent execution, bundled browser engines, Windows/macOS parity.
- Any change to the provider/core conversation protocol, permissions, persistence, or trust boundaries.
- Per-microstep automated test execution (ADR 0014): tests are written with each microstep and executed only at this release gate.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B17-S00 | [#98](https://github.com/AlexandreZanata/brain-root-idea/issues/98) | Incorporates the release-gate governance and Freebuff plan from PR #62: ADR 0014, the release-reference plan, synchronized issue/PR/Wiki templates, `AGENTS.md`, roadmap/open questions, and the PR-only CI trigger; PR #62 closed as incorporated | `d16824f` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/98#issuecomment-5813177820) | No runtime change; PR #62 closed without merging by design | Closed |
| B17-S01 | [#100](https://github.com/AlexandreZanata/brain-root-idea/issues/100) | Freezes the Freebuff v0.0.142 reference and its evidence limits, the reproducible fake-provider fixture, the friction inventory FR-1..FR-7, the existing B01/B05/B12/B15/B16 evidence, and the before/after procedure for this release gate | `54a84b2` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/100#issuecomment-5813223672) | No runtime claim; Desktop UI stays `UNKNOWN` without a sandbox audit | Closed |
| B17-S02 | [#101](https://github.com/AlexandreZanata/brain-root-idea/issues/101) | Adds the pure presentational contract (`taskStatus`, `presentTurn`, `acceptsEvent` mirroring the Rust transition table) and applies it in the App so reordered/late/duplicate events are ignored | `c5f130f` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/101#issuecomment-5813274633) | Static checks green; runtime cases executed at this gate | Closed |
| B17-S03 | [#102](https://github.com/AlexandreZanata/brain-root-idea/issues/102) | Adds the bounded stream buffer (one flush per frame, 64 KiB cap, synchronous terminal flush, dispose releases the frame callback) and routes chunks through it in the App | `4039f6f` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/102#issuecomment-5813316844) | Static checks green; ordering/bound/cancel cases executed at this gate | Closed |
| B17-S04 | [#103](https://github.com/AlexandreZanata/brain-root-idea/issues/103) | Adds per-turn status chips and word-bounded collapsing of old long answers with an accessible Show full answer / Show less toggle; failures, cancellations, and technical details stay visible | `9983bed` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/103#issuecomment-5813353547) | Static checks green; expanded/collapsed accessibility cases remain a manual probe | Closed |
| B17-S05 | [#104](https://github.com/AlexandreZanata/brain-root-idea/issues/104) | Makes the composer predictable (Enter submits, Shift+Enter newline, IME safe, Escape cancels with focus return) and preserves the reading position with a Jump to latest control | `07cf77f` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/104#issuecomment-5813403228) | Static checks green; keyboard/scroll probe remains manual; WebView focus escape stays out of scope | Closed |
| B17-S06 | [#105](https://github.com/AlexandreZanata/brain-root-idea/issues/105) | Announces Canvas destination changes honestly (Preview view reloads; Browser page reloads and may lose state) via `canvasTransition`, fixes the `stopped` status line, and surfaces stop failures as a real alert; one-HOT unchanged | `a687a70` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/105#issuecomment-5813454587) | Static checks green; one-HOT/cleanup executed at this gate via the deck harness | Closed |
| B17-S07 | [#106](https://github.com/AlexandreZanata/brain-root-idea/issues/106) | Release gate: version `0.0.13` synchronized; changelog section; this record created; complete Linux suite executed on the application head; baseline probes executed; PR marked Ready, CI green on `d48b5f4`, merged with a merge commit, tagged `v0.0.13`, and released | `d48b5f4`, `e25f32f` (merge) | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/106#issuecomment-5813623285) | Merge used the documented single-maintainer administrator bypass; CI passed before merge | Closed |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B17-S00): the planning/governance PR #62 is incorporated into this versioned batch instead of merging alone, as its own handoff required.
- Decision (B17-S00): from B17, draft pushes run no test jobs; the complete Linux suite runs once when the batch PR becomes Ready.
- Decision (B17-S02): the frontend mirrors the Rust transition table for presentation acceptance but does not become a second state machine; the core remains authoritative.
- Decision (B17-S03): coalescing is per animation frame with a 64 KiB byte cap and a synchronous terminal flush; perceived latency must not increase, and the first-content measurement remains `UNKNOWN` until a future interactive probe.
- Decision (B17-S06): the COLD reload/state-loss rule is announced in product language at the switch; no snapshot/hibernation work is added and the B15 one-HOT destroy/recreate policy is unchanged.

## Failures and recovery

- The B17-S04 implementation first tripped a Svelte 5 warning (`state_referenced_locally` on `turn.id`); the response id became a `$derived` value and the check returned to zero warnings. No gate was weakened.

## Final gates

- Local release gate on the application head `a687a70` (finalization changes are version/docs only): `sh scripts/check-full-linux.sh` passed — Rust fmt/clippy/tests (151 passed, 3 ignored), svelte-check, production frontend build, Tauri release build, security/modules/accessibility gates, fake-provider e2e, process smoke (readiness observed; main and child processes exited; no listeners remained), docs, version consistency, license artifacts.
- CI: required `check-full-linux` passed on the submitted head `d48b5f4` ([run 35996141045](https://github.com/AlexandreZanata/brain-root-idea/actions/runs/35996141045)); the PR was merged with a merge commit after the single-maintainer administrator bypass.
- Review: single-maintainer administrator bypass documented; merged at B17-S07
- Release: annotated `v0.0.13` on merge commit `e25f32f`; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.13) without artifact
- Performance (MEASURED on the reference host, `BRAINROOT_MEASURE_SKIP_BUILD=1 sh scripts/measure-linux.sh`, commit `a687a70`, load average 3.66): startup n=10 median 0.907 s, p95 1.033 s (TARGET ≤ 1.0 s p50); idle CPU median 1.724 % across 5 windows (windows 4–5 at 0.103 % and 0.069 %; the `< 1 %` budget is exceeded on this busy host and is reported, not excused); PSS median 266.3 MB (the 150 MB target still fails, as since B01); binary 8,081,576 bytes; frontend JS 82,068 B (gzip 28,529 B) and CSS 14,635 B (gzip 3,225 B).
- Cleanup/one-HOT (MEASURED): `BRAINROOT_DECK_FIXTURE=1` returned `decision: go` with switches 0–12 ms, children bounded at 6 after two cycles and 4 after cleanup, and no leftover process or listener.
- Deferred/manual: first visible stream content, input-feedback latency, and the keyboard/focus/scroll probes remain `UNKNOWN` without interactive automation; the buffer's coalescing/bound/order/dispose cases passed in the frontend suite.
- Security/privacy: no privileged or protocol change; fast secret scan green per microstep and in the full gate.
- Artifact/checksum: none for this batch
- Known limitations: presentation-only polish; no new capability; WebView focus escape, COLD snapshots, and portable import remain for later batches.

## Result and next batch

Batch B17 is released as [`v0.0.13`](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.13) on merge commit `e25f32f`: the conversation reads as a compact task narrative with bounded streaming, predictable composer/focus/scroll, and honest Canvas transitions under the ADR 0014 release-gate cadence. The installed application on the maintainer machine was refreshed from this head (`~/.local/bin/brainroot`, sha256 `5378a55f…`, verified by the process smoke). Next: portable import, phone presentation, or the roadmap's deferred phases. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier.
