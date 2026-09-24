# B17 interaction baseline and before/after procedure

**Status:** frozen reference and inventory; no UI change and no measured BrainRoot runtime claim in this record

**Date:** 2026-09-24

**Related:** [Freebuff release reference and BrainRoot interaction plan](freebuff-inspired-experience-plan.md) · [ADR 0014](../adr/0014-release-only-test-cadence.md) · [performance contract](../10-performance-budget.md) · [B01 baseline](performance-reports/b01-linux-baseline.md) · [B05 soak](performance-reports/b05-mvp0-soak.md) · [preview quality](preview-quality-measurements.md) · [deck switching](deck-switching.md)

## Frozen reference

- **Published product reference:** Freebuff Desktop v0.0.142 pre-release, published 2026-09-24 with Linux, Windows, and macOS assets. The Linux x86_64 AppImage is 180,126,952 bytes **as release metadata**, not a measured RAM, startup, or interaction benchmark.
- **Source limit:** the nearest public snapshot before that release (`3180425d0b41feb1359a31abcf39ab569f241c95`, 2026-09-24 03:03:40 UTC) does **not** contain the Freebuff Desktop source. Timing alone does not prove provenance. Desktop-specific internals, implementation, and UI behavior are `UNKNOWN`.
- **Inspectable patterns only:** an older Codecane CLI prerelease (`5f154c58d13460ae43c343205d1729d32ac519e5`) exposes a React/OpenTUI terminal UI whose chat store, message renderer, and send/stream hook demonstrate separation of conversation state, rendering, streaming, collapse, and focus. It does not prove Desktop behavior.
- **No sandbox audit was performed for this record.** A visual audit of the exact Desktop v0.0.142 binary in a disposable, approved Linux sandbox with no real profile or credentials remains optional and unexecuted; until it happens, any Desktop-specific observation stays `UNKNOWN`.
- **No reuse:** no Freebuff code, assets, runtime, service, model catalog, or dependency enters BrainRoot. This is inspiration, not a fork or a compatibility claim.

## Reproducible BrainRoot fixture

- `BRAINROOT_PROVIDER_MODE=fake` (`src-tauri/src/main.rs:8`) runs the deterministic fake provider: no credential, no network, fixed chunks, and a normal terminal event.
- Existing fixture toggles for the Canvas: `BRAINROOT_PREVIEW_FIXTURE`, `BRAINROOT_HUMAN_FIXTURE`, `BRAINROOT_DECK_FIXTURE` (`src-tauri/src/main.rs:32-40`), all `#[cfg(debug_assertions)]` harnesses.
- Frontend fixture data lives in `src/conversation.test.mjs`; the B17 presentation tests extend it. No live account or wall-clock sleep is used.

## Current friction inventory (code inspection; no runtime claim)

| ID | Friction | Where |
|---|---|---|
| FR-1 | Status is shown only as one composer line; successful turns carry no visible status, and there is no compact "current action → result" narrative | `src/App.svelte:97-112`, `src/lib/Turn.svelte:7-25` |
| FR-2 | Every chunk replaces the whole turn array and re-renders the list; no coalescing or per-frame bound | `src/conversation.ts:135-145`, `src/App.svelte:232-235` |
| FR-3 | Enter inserts a newline instead of submitting; Shift+Enter semantics are undefined; only the Send button submits | `src/lib/ConversationPanel.svelte:72-88`, `src/lib/TextArea.svelte` (no key handling) |
| FR-4 | The history body has no scroll anchoring and no "jump to latest"; new content can move the reading position | `src/lib/ConversationPanel.svelte:54-70` |
| FR-5 | Switching Preview ↔ Browser unmounts the previous panel and mounts the next with no transition status; the COLD/reload message exists only inside the Browser placeholder | `src/lib/CanvasPanel.svelte:44-50`, `src/lib/HumanBrowserPanel.svelte:268-272` |
| FR-6 | `stopped` renders an empty status line, and `errorDetail` is set but never displayed because it only renders when `phase === "failed"` | `src/lib/PreviewPanel.svelte:56-72`, `:228`, `:245-247` |
| FR-7 | Accessibility invariants must survive the polish: no native `disabled`, no `tabindex`, no outline removal, no `user-select: none`, and `role="alert"`/`role="status"`/`aria-live="polite"` must remain | `scripts/check-accessibility.sh` |

## Existing performance evidence (before)

- **B01 baseline:** startup p50 0.282 s, idle CPU median 0.172 %; RSS ≈198.5 MB PSS fails the 150 MB target ([b01-linux-baseline.md](performance-reports/b01-linux-baseline.md)).
- **B05 soak:** repeated fake conversations; startup and cancellation pass, the memory TARGET still fails ([b05-mvp0-soak.md](performance-reports/b05-mvp0-soak.md)).
- **B12 preview quality:** scale/DPR 1.0 and 2.0, zoom 480 → 240 → 480 CSS px, widget focus, and a 100-update bounds soak (p50 0 µs, max 31 µs, RSS +20 KB) ([preview-quality-measurements.md](preview-quality-measurements.md)).
- **B15 deck switching:** Preview ↔ Browser switches measured 0–3 ms at the view layer with one cached `WebContext` per role ([deck-switching.md](deck-switching.md)).
- **B16 permissions:** deny-only permission handler and confirmed clear-data path ([human-browser-permissions.md](human-browser-permissions.md)).

## Before/after procedure for the release gate (B17-S07)

Labels follow `docs/10-performance-budget.md`: `TARGET` is a desired constraint, `MEASURED` requires a reproducible result with environment and raw samples, and everything else is `UNKNOWN`.

1. **Input feedback (local state):** TARGET within one 60 Hz frame. Measured by manual release-profile observation; `TARGET`/`UNKNOWN` until recorded.
2. **First visible stream content:** with `BRAINROOT_PROVIDER_MODE=fake`, measure from `conversation_send` resolution to the first rendered chunk over at least 10 runs; report p50, p95, and range. TARGET: p95 ≤ 200 ms on the reference environment.
3. **Stream render work:** count conversation-list updates while a deterministic fixture stream is active; assert coalescing keeps updates ≤ 60/s and the buffered text stays bounded. Executed by the B17-S03 frontend test at the release gate.
4. **Settled idle CPU/RSS:** `sh scripts/measure-linux.sh` on the release build; compare against the B01/B05 numbers with the same environment labels and process-tree accounting.
5. **Canvas switch latency:** `BRAINROOT_DECK_FIXTURE=1` debug harness; compare against the B15 0–3 ms view-layer result.
6. **Cancellation and close cleanup:** `sh scripts/smoke-linux.sh` plus the deck harness; assert no owned process, listener, WebView, or temporary artifact remains after cancel, close, and switch.
7. **Bounded output and journey:** `sh scripts/check-full-linux.sh` (frontend unit tests, Rust tests, e2e journey, production build) on the latest release-gate head.

Never promote a `TARGET` or `UNKNOWN` figure to `MEASURED` without the recorded environment and raw samples.

## Release-gate results (2026-09-24, B17-S07)

Measured on the B01 reference environment with the release build at application head `a687a70` (the finalization commit changes version and documentation only). Host state: load average 3.66, uptime 1 week 1 day, `performance` governor, AC online.

- **Complete Linux suite:** `sh scripts/check-full-linux.sh` passed — Rust fmt/clippy/tests (151 passed, 3 ignored), svelte-check (0 errors, 0 warnings), production frontend build, Tauri release build, security/modules/accessibility gates, fake-provider e2e (31 frontend tests passed), process smoke (readiness observed; main and child processes exited; no listeners remained), docs, version consistency, license artifacts.
- **Startup (MEASURED):** n=10, median 0.907 s, p95 1.033 s, range 0.826–1.053 s. The p50 target of ≤ 1.0 s passes on this host.
- **Settled idle CPU/RSS (MEASURED):** 5 windows; CPU median 1.724 %, p95 8.206 %, min 0.069 % (windows 4–5 at 0.103 % and 0.069 %); RSS median 446.7 MB, PSS median 266.3 MB (classes: brainroot 101.8 MB, WebKitWebProcess 146.3 MB, WebKitNetworkProcess 18.6 MB). The `< 1 %` idle budget is exceeded on this busy host and the 150 MB memory target still fails; both are reported, not excused, and neither is attributed to B17 without a same-session before sample.
- **Bundle (MEASURED):** binary 8,081,576 bytes; `dist/index.html` 929 B; JS 82,068 B (gzip 28,529 B); CSS 14,635 B (gzip 3,225 B).
- **Canvas one-HOT and cleanup (MEASURED):** `BRAINROOT_DECK_FIXTURE=1` returned `decision: go`; switches 0–12 ms; children 3 baseline → 6 after two cycles → 4 after cleanup; RSS 196.7 MB → 200.5 MB; no leftover process or listener.
- **First visible stream content, input feedback, and the keyboard/focus/scroll probes:** `UNKNOWN` — no interactive automation ran at this gate; the buffer coalescing/order/bound/dispose cases passed in the frontend suite as logic evidence only.
