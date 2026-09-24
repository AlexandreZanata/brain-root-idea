# Freebuff release reference and BrainRoot interaction plan

**Status:** proposed for the next Linux experience batch; no application code or new dependency is authorized by this document

**Reference reviewed:** 2026-09-24

## Evidence and limits

- The published [Freebuff Desktop v0.0.142 pre-release](https://github.com/CodebuffAI/codebuff-community/releases/tag/freebuff-desktop-v0.0.142) is the fixed product reference. It was published on 2026-09-24 with Linux, Windows, and macOS assets. The Linux x86_64 AppImage is 180,126,952 bytes **as release metadata**, not a measured RAM, startup, or interaction benchmark.
- The nearest public [source snapshot before that release](https://github.com/CodebuffAI/freebuff/tree/3180425d0b41feb1359a31abcf39ab569f241c95) is commit `3180425d0b41feb1359a31abcf39ab569f241c95` (2026-09-24 03:03:40 UTC). Its public tree does **not** contain the Freebuff Desktop source. Timing alone does not prove that this snapshot built the release. We can inspect the published product notes and public shared code, not assert Desktop internals or copy an unpublished architecture.
- A separate, older [Codecane CLI prerelease v1.0.420-beta.185](https://github.com/CodebuffAI/freebuff/tree/5f154c58d13460ae43c343205d1729d32ac519e5) exposes a React/OpenTUI terminal UI. Its [chat store](https://github.com/CodebuffAI/freebuff/blob/5f154c58d13460ae43c343205d1729d32ac519e5/cli/src/state/chat-store.ts), [message renderer](https://github.com/CodebuffAI/freebuff/blob/5f154c58d13460ae43c343205d1729d32ac519e5/cli/src/hooks/use-message-renderer.tsx), and [send/stream hook](https://github.com/CodebuffAI/freebuff/blob/5f154c58d13460ae43c343205d1729d32ac519e5/cli/src/hooks/use-send-message.ts) are inspectable. They demonstrate separation of conversation state, rendering, streaming, collapse, and focus. They do **not** prove Desktop's implementation. The stream hook batches updates; BrainRoot must measure its own suitable coalescing policy rather than copying its 48 ms timer.

This is inspiration, not a fork or dependency decision. Do not import Electron, React, OpenTUI, Bun, their state libraries, the Freebuff service, advertisements, model catalog, parallel agent runtime, or visual assets into BrainRoot. Freebuff's public description of [parallel local agents](https://github.com/CodebuffAI/freebuff) is a product choice, not a reason to start extra BrainRoot processes. BrainRoot remains Apache-2.0, Tauri/Rust/Svelte, Linux-first, agent-first, and Canvas-dominant. Any actual source reuse would need separate license/NOTICE review; this plan proposes none.

## Experience to build, uniquely BrainRoot

The user describes intent in chat, sees a short truthful plan, watches compact progress, and manipulates the result beside the conversation. The Canvas—not a terminal transcript or file tree—is the primary work surface. Existing Preview, Human Browser, and Deck roles remain isolated.

1. **Stable two-surface shell.** Keep the conversation and Canvas side by side with the existing divider and layout presets. Preserve selection, focus, and viewport across ordinary chat updates. No large layout shift when a response starts, finishes, or errors.
2. **Compact task narrative.** Show one current action, a concise result, and optional expandable detail. Collapse old tool/agent activity by default without hiding failures or approval requests. Use BrainRoot language (Build, Preview, Test, Undo); raw commands and provider events stay in Technical details.
3. **Streaming without visual churn.** Normalize provider events once at the typed boundary. Derive a small presentational view model from existing events; coalesce only while a stream is active and flush immediately on terminal/error/cancel. Avoid whole-thread rerenders, constant timers, and unbounded text accumulation. Keep the input responsive and support explicit cancel.
4. **Predictable control.** Keyboard and pointer actions agree; Enter/Shift+Enter, cancel, focus escape from WebViews, scroll anchoring, and “jump to latest” are tested at release. Never steal a user's reading position to follow new output.
5. **Immediate visual result.** Preserve the last valid Canvas frame while a destination loads; show real state (loading/stale/failed), not fictitious progress. Preview ↔ Human Browser switching retains the one-HOT default and tells the user when a COLD page must reload.
6. **Professional restraint.** One visual hierarchy, few persistent controls, contextual actions, accessible contrast/semantics, reduced motion, and no default IDE chrome. Do not mimic Freebuff branding or claim browser-engine/mobile-device parity.

## Architecture mapping and performance contract

- **Rust core:** existing conversation, preview, Deck, Human Browser, and resource ownership remain authoritative. No new privileged frontend access, duplicated state machine, background service, or always-on agent.
- **Svelte presentation:** small components and typed view models, selected subscriptions/props, bounded message rendering, and event-driven state updates. Prefer existing Svelte facilities; any new dependency requires the repository dependency review and a measured benefit.
- **Stream path:** one event subscription per active owner; bounded queue and cleanup on cancel, close, or navigation. A proposed batching change must preserve ordering, exactly one terminal event, error visibility, and screen-reader announcements. It must not increase perceived latency merely to reduce renders.
- **Browser/Canvas:** keep one HOT heavy content WebView by default. Reuse the current Preview/Browser lifecycle and origin policies rather than embedding remote content in the shell.
- **Evidence:** compare before/after on the Linux reference environment for input feedback, first visible stream content, stream CPU/render work, settled idle CPU/RSS, app startup, Canvas switch latency, and WebView/process cleanup. Use `TARGET`, `MEASURED`, or `UNKNOWN` and raw samples per `docs/10-performance-budget.md`. Release asset size is not runtime performance evidence.

## Proposed next batch: B17 interaction and performance polish

One branch, one draft PR, one version at its end. These are candidate microsteps, **not executable issues** until each issue has the exact starting SHA, allowlist, acceptance, rollback, and release-test cases. Limit the batch to existing Linux behavior; defer new browser import, multi-agent execution, a bundled engine, new providers, and Windows/macOS parity. If a candidate needs a new trust boundary or dependency, split it and obtain a decision first.

1. **B17-S01 — Freeze reference and baseline.** Record the release/snapshot distinction above, current BrainRoot UX friction from a reproducible fixture, existing B12/B16 performance evidence, and the before/after measurement procedure. If a disposable, approved Linux sandbox is available, visually audit the exact Desktop v0.0.142 binary with no real profile or credentials; otherwise label Desktop-specific UI observations `UNKNOWN` rather than inventing them. No UI change. A single lightweight BrainRoot baseline capture is allowed; it is not a test-suite run.
2. **B17-S02 — Presentational state contract.** Specify and implement a pure mapping from existing typed conversation events to concise user-facing states; no provider/core protocol change. Write deterministic unit cases for release execution, including error/cancel/reordered input.
3. **B17-S03 — Bounded streaming presentation.** Coalesce visible updates only during active streaming, flush terminal/error/cancel synchronously, and release listener/timer ownership. Write tests for ordering, bound, cancel, and cleanup; benchmark at the release gate.
4. **B17-S04 — Progressive disclosure.** Show current action/result prominently and collapse old technical detail without hiding approval/failure. Keep Canvas dominant. Write UI/accessibility cases for expanded/collapsed state and screen-reader semantics.
5. **B17-S05 — Composer, focus, and scroll.** Improve keyboard path, cancellation affordance, focus return/escape, and reading-position preservation. Write keyboard/focus cases; do not change privileged permissions.
6. **B17-S06 — Canvas transition polish.** Reuse existing one-HOT Deck policy, last useful frame, stale/loading/retry states, and resource cleanup. Write deterministic Preview ↔ Browser transition cases; no second live heavy WebView.
7. **B17-S07 — Versioned release gate.** Synchronize version/changelog/history/Wiki, run the complete Linux checks on the latest PR head once via Ready-for-review CI, collect release-profile performance/security/accessibility/cleanup evidence, and request review. End the task with `CI_PENDING`; a later task handles green/failure without polling. Any failure creates a remediation issue and a new final-head run before merge/tag.

## Validation cadence from B17 onward

- **Each implementation microstep:** write or update relevant tests, but do **not run automated unit, integration, E2E, soak, or full CI tests**. Record exact tests deferred to the release gate. Perform only fast, non-test checks: allowed-file diff, `git diff --check`, secret scan, and narrowly applicable format/schema/documentation checks. Record `IMPLEMENTED_UNVERIFIED`, not “tested” or “passed”. Static checks can stop a bad commit; they do not prove runtime behavior.
- **At the version/release boundary:** move the PR from Draft to Ready once all planned issues and version files are complete. Required CI executes the complete suite on the latest head. Release-only local/manual probes cover real WebView, lifecycle, accessibility, and performance where CI cannot. CI failures create tracked remediation; after a fix, rerun the release gate. Never merge on stale/missing checks or hide a failure.
- **Asynchronous handoff:** do not wait, poll, or watch CI in the submitting task. Record PR/head/run and next action, then stop. A future task checks once and either closes on pending, opens remediation on failure, or merges/tags after required green checks and approval.

This cadence trades earlier defect detection for shorter microstep turns. The quality bar moves to the version gate; it is **not removed**. Security-critical boundaries remain fail-closed and cannot be asserted safe from an unexecuted test. See ADR 0014 and `docs/13-testing-strategy.md`.
