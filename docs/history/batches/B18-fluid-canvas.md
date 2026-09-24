# B18 — Fluid Canvas and phone-first Browser

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B18-Fluid-Canvas). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Release gate — `CI_PENDING` (no merge, tag, or release in the S07 task)
- Objective: Open the Companion Canvas on the Browser destination in a phone-width viewport, resize it through an edge-only fluid divider, move it with Swap sides and Canvas-chrome swipe, keep native Browser/Preview geometry correct under a bounded latest-wins policy, and measure the result — under the release-gate test cadence of ADR 0014.
- Branch: `batch/b18-fluid-canvas`
- Draft/final PR: [#108](https://github.com/AlexandreZanata/brain-root-idea/pull/108)
- Merge commit: pending (recorded by the merge task)
- Baseline commit: `5f2835c` (B17 released, `v0.0.13`)
- Target/resulting version: `0.0.14` (no artifact)
- Started/completed: 2026-09-24 / 2026-09-24
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, Wayland, WebKitGTK 2.52.6, rustc 1.96.0)
- Repository history mirror: this file (created at batch close)

## Non-goals

- New providers, agents, tools, dependencies, browser-data import, bundled browser engines, Windows/macOS parity.
- Any change to the provider/core conversation protocol, permissions, persistence, or trust boundaries.
- A blank native WebView at boot (separate unresolved core/performance decision); the empty Browser still creates no WebView or network request.
- Per-microstep automated test execution (ADR 0014): tests are written with each microstep and executed only at this release gate.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B18-S00 | [#107](https://github.com/AlexandreZanata/brain-root-idea/issues/107) | Adopts the fluid-Canvas/phone-Browser phase plan and cross-references the owning documents; no runtime change or claim | `e133db5` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/107#issuecomment-5814083140) | Planning only | Closed |
| B18-S01 | [#109](https://github.com/AlexandreZanata/brain-root-idea/issues/109) | Freezes the behavior baseline and the local loopback fixture (`tests/fixtures/b18-canvas/index.html`) with the B18 measurement procedure; all runtime results `UNKNOWN` | `29089f0` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/109#issuecomment-5814203537) | No product-code change | Closed |
| B18-R01 | [#110](https://github.com/AlexandreZanata/brain-root-idea/issues/110) | Corrects the pending S03 contract: drag directly from the pane edge, no visible thumb/knob/rail, keyboard and focus access preserved | `ece82d2` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/110#issuecomment-5814235924) | Docs only | Closed |
| B18-S02 | [#111](https://github.com/AlexandreZanata/brain-root-idea/issues/111) | Phone-first entry: Browser selected at launch, Preview defaults to phone, Human Browser slot is a centered phone-width native rectangle; empty Browser creates no WebView or network request | `a0a828d` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/111#issuecomment-5814304938) | Static checks green; U01..U04 deferred to this gate | Closed |
| B18-S03 | [#113](https://github.com/AlexandreZanata/brain-root-idea/issues/113) | Edge-only fluid divider: 24 px transparent hit area, pointer capture with one-rAF coalescing, final commit on release, restore on cancel/blur/lost capture, teardown cleanup; semantic range preserved | `5978b5c` | [issue](https://github.com/AlexandreZanata/brain-root-idea/issues/113) | Static checks green; U05/U06/P01 deferred to this gate | Closed |
| B18-R02 | [#114](https://github.com/AlexandreZanata/brain-root-idea/issues/114) | Freebuff-inspired empty destination menu (Browser available; Files/Terminal/Changes honest planned entries) and restrained surface/control rounding; no fake capability | `ea61544` | [issue](https://github.com/AlexandreZanata/brain-root-idea/issues/114) | Static checks green; U13 deferred to this gate | Closed |
| B18-S04 | [#115](https://github.com/AlexandreZanata/brain-root-idea/issues/115) | Explicit Swap sides: visual chat/Canvas reversal via CSS order with DOM order preserved, no Canvas/WebView remount, polite side announcement | `462cb7f` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/115#issuecomment-5814601077) | Static checks green; U07/U08 deferred to this gate | Closed |
| B18-S05 | [#116](https://github.com/AlexandreZanata/brain-root-idea/issues/116) | Header/chrome swipe with 48 px (24 px flick) thresholds and vertical rejection, reusing `selectTab` with button/keyboard parity; page body and WebView untouched | `12872a6` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/116#issuecomment-5814735797) | Static checks green after a justified svelte-ignore; U09/U10/U11 deferred to this gate | Closed |
| B18-S06 | [#117](https://github.com/AlexandreZanata/brain-root-idea/issues/117) | Latest-wins native geometry: shared `createBoundsSync` pump (one in flight plus one pending, dedupe, local counters), stale-status guard, visible Retry on both panels, teardown cancels | `211af91` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/117#issuecomment-5814877994) | Static checks green; U12/P02/P03 deferred to this gate | Closed |
| B18-S07 | [#118](https://github.com/AlexandreZanata/brain-root-idea/issues/118) | Release gate: version `0.0.14` synchronized; changelog section; this record created; complete Linux suite executed on the application head; baseline probes executed; PR marked Ready with a `CI_PENDING` handoff | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/118) | Merge/tag deferred to a later task per ADR 0014 | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B18-R01): the divider has no visible thumb, knob, or rail; the full pane boundary is the drag affordance with the semantic range kept for keyboard and assistive technology.
- Decision (B18-S02): Browser selected at launch means no remote fetch and no native content WebView until the first explicit address; phone width is responsive presentation, never device emulation.
- Decision (B18-S04): side swapping is visual-only CSS order so component and WebView instances survive; DOM/keyboard order stays rail → chat → resizer → Canvas and its confusing potential is a release-gate probe, not a claim.
- Decision (B18-S05): the header swipe is a pointer-only shortcut with a justified compiler ignore and no invented ARIA semantics; parity lives in the tab buttons.
- Decision (B18-S06): bounds traffic is serialized and deduped in one shared pure helper both panels already import; stale bounds results never overwrite navigation state; failures show Retry.
- Decision (B18-S07): the task ends with `CI_PENDING`; a later task checks CI/review once and handles merge or remediation.

## Failures and recovery

- The B18-S05 implementation first tripped a Svelte warning (`a11y_no_static_element_interactions` on the header div); resolved with a justified `svelte-ignore` (parity via tab buttons) instead of an ARIA role hack. The first wording of that comment tripped the accessibility gate's `tabindex` pattern match and was reworded; no gate was weakened.
- The B18-S06 implementation first tripped `svelte-check` on an unused generic (`BoundsSync<T>`); simplified to a non-generic type. No gate was weakened.

## Final gates

- Local release gate on the application head `211af91` (finalization changes are version/docs only): `sh scripts/check-full-linux.sh` passed — Rust fmt/clippy/tests (151 passed, 0 failed, 3 ignored: live smoke and Secret Service round-trip by design), svelte-check 0/0, frontend production build, Tauri release build, security/modules/accessibility gates, fake-provider e2e (cargo e2e 2 passed, 37 frontend tests passed), process smoke (readiness observed; main and child processes exited; no listeners remained), docs, version consistency, license artifacts.
- Deferred unit cases executed at this gate via `pnpm run test:frontend` (37 passed, 0 failed): the S02 phone-default case and all five S06 bounds-sync cases (burst latest-wins, duplicate suppression, failure retry, dispose safety).
- Release soak (ignored, run explicitly): `cargo test --release soak -- --ignored` passed — 1200 fake conversations, RSS 34,180 → 35,280 KiB (bounded), threads 2 → 2.
- Performance (MEASURED on the reference host, `BRAINROOT_MEASURE_SKIP_BUILD=1 sh scripts/measure-linux.sh`, commit `211af91`, governor performance, AC online, load average 3.28): startup n=10 median 1.007 s, p95 1.028 s (TARGET ≤ 1.0 s p50 missed by 7 ms on this busy host; reported, not excused); idle CPU median 1.345 % across 5 windows (windows 1–2 elevated at 10.134 % and 4.896 % while settling; windows 3–5 at 0.034 %, 1.345 %, 0.069 %); the `< 1 %` budget is exceeded and is reported, not excused; PSS median 252.1 MB (brainroot 93.5 MB, WebKitWebProcess 140.0 MB, WebKitNetworkProcess 18.0 MB); the 150 MB target still fails, as since B01; binary 8,085,800 bytes; frontend JS 87,614 B (gzip 30,205 B) and CSS 16,667 B (gzip 3,559 B).
- Same-session before/after (B17 `5f2835c` vs B18 `211af91`, same host and session, release builds, governor performance, AC online): startup median 0.856 s → 1.007 s (ranges 0.826–0.867 vs 0.967–1.028, no overlap; host load 4.88 vs 3.28); idle-CPU median 0.069 % → 1.345 % (B18 windows 1–2 still settling at 10.134 %/4.896 %, windows 3–5 at 0.034 %/1.345 %/0.069 %); PSS median 249.3 MB → 252.1 MB (+1 %); binary +4,224 B, JS +5,546 B, CSS +2,032 B. No regression is claimed from one session pair — but the startup TARGET (≤ 1.0 s p50) passes on B17 and misses by 7 ms on B18, and the idle-CPU budget passes on B17 and fails on B18's median; both are reported for maintainer review, not excused. Full table in the baseline document.
- Cleanup/one-HOT (MEASURED, debug deck harness `BRAINROOT_DECK_FIXTURE=1`): `decision: go` with no reasons; switches 0–4 ms; children 3 → 5/6 across phases → 6 after the second cycle → 4 after cleanup (same bounded pattern as B17).
- Deferred/manual: divider/swipe frame timing (P01), live fixture navigation and `innerWidth` (U02, P02), 100 drag/swap/tab cycles (P03), and the keyboard/focus-order probes (U05..U11, U13) remain `UNKNOWN` — no GUI automation exists in this batch's scope; the helper-level bounds cases passed in the frontend suite.
- Security/privacy: no privileged or protocol change; fast secret scan green per microstep and in the full gate.
- CI: `CI_PENDING` on the submitted head (recorded by the S07 task; result, merge SHA, and tag recorded only when known).
- Artifact/checksum: none for this batch.
- Known limitations: frontend-only batch; phone width is not device emulation; WebView focus escape remains out of scope; Files/Terminal/Changes stay planned entries.

## Result and next batch

Batch B18 is submitted as `0.0.14` on PR [#108](https://github.com/AlexandreZanata/brain-root-idea/pull/108) (merge, tag `v0.0.14`, and release handled by a later task after green CI and review): the Canvas opens phone-first on Browser, resizes through an edge-only divider, swaps sides without remounting, accepts header swipes with button parity, and keeps native geometry under a bounded latest-wins policy with visible retry — all under the ADR 0014 release-gate cadence. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier. Next: the maintainer's roadmap phases (portable import, phone presentation refinements, or the B19 proposal).
