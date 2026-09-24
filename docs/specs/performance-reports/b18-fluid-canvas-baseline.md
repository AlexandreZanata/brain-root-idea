# Performance report: B18 fluid Canvas and phone-first Browser (release gate)

**Status:** Measured 2026-09-24 at B18-S07; interactive-only steps remain `UNKNOWN`
**Date/commit:** 2026-09-24; baseline `v0.0.13` (`main` `5f2835c`), candidate `211af91`
**Owner:** maintainer / economical agent
**Related budget:** [performance contract](../../10-performance-budget.md) (startup, idle CPU, memory, one HOT view, cleanup, interaction budgets) · [B18 phase spec](../b18-frontend-fluidity-phone-browser.md)

## Claim

Question: does B18 — Browser/phone as the default Canvas destination, fluid divider, Swap sides, Canvas-chrome swipe, and phone-width native geometry — meet the responsiveness, native-work bound, startup/idle, lifecycle, accessibility, and security targets without regressing `v0.0.13`?

**Result: MEASURED where the environment allowed; `UNKNOWN` where GUI automation is required.** The B17 numbers below are now a same-session comparison (same host and session, release builds, governor performance, AC online), not historical context — except the interactive-only steps, which stay `UNKNOWN`.

## Environment

Fields to record exactly for the release-profile run at B18-S07 (recorded):

- BrainRoot build/profile and lockfile: release build (`pnpm tauri build --no-bundle`), candidate commit `211af91`, B17 rerun commit `5f2835c` (separate worktree, identical lockfiles)
- OS, version, architecture, patches: Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, x86_64 (B01 reference environment)
- CPU, RAM, storage, display: 13th Gen Intel Core i7-13620H (16 threads), 32,556,572 KiB RAM, 460 G root, Wayland session (`:1`/`wayland-1`)
- Power/thermal mode and background workload: governor `performance`, AC online, uptime 1 week 1 day, load average 3.28 (B18 run) and 4.88 (B17 rerun)
- WebView/runtime versions: WebKitGTK 2.52.6
- Project fixture and size: `tests/fixtures/b18-canvas/index.html` served over loopback (fixture not navigated — no GUI automation; size/clamp covered by unit cases)
- Network condition: loopback only; no external network, no private site, no live profile or account
- Measurement tools and versions: `sh scripts/measure-linux.sh` (both runs), debug deck harness (`BRAINROOT_DECK_FIXTURE=1`), process-tree sampling

### B17 same-session rerun values (MEASURED 2026-09-24, superseding the older session below for comparison)

- Startup: n=10, median 0.856 s, p95 0.866 s, range 0.826–0.867 s (host load 4.88).
- Settled idle: CPU median 0.069 % across 5 windows; RSS/PSS medians from the rerun: PSS 249.3 MB.
- Bundle: binary 8,081,576 B; JS 82,068 B (gzip 28,529 B); CSS 14,635 B (gzip 3,225 B).

### Older B17 session values (superseded for comparison; preserved in the B17 record)

- Startup: n=10, median 0.907 s, p95 1.033 s, range 0.826–1.053 s.
- Settled idle: CPU median 1.724 % (windows 4–5 at 0.103 % and 0.069 %); RSS median 446.7 MB, PSS median 266.3 MB (brainroot 101.8 MB, WebKitWebProcess 146.3 MB, WebKitNetworkProcess 18.6 MB). The `< 1 %` idle budget and the 150 MB memory target were not met on a busy host.
- Bundle: binary 8,081,576 B; JS 82,068 B (gzip 28,529 B); CSS 14,635 B (gzip 3,225 B).
- Deck one-HOT: `decision: go`, switches 0–12 ms, children 3 → 6 after two cycles → 4 after cleanup.
- Source: [B17 interaction baseline](../b17-interaction-baseline.md).

## Current behavior inventory (source inspection; no runtime claim)

| Behavior | Current implementation | Reference |
|---|---|---|
| Canvas starts on the Preview destination | `let active = $state<CanvasTab>("preview")` | `src/lib/CanvasPanel.svelte:17` |
| Preview starts in the Desktop preset | `let preset = $state<ViewportPresetId>("desktop")` | `src/lib/PreviewPanel.svelte:36` |
| Human Browser WebView exists only after an explicit address | `onGo` → `humanShow(parsed, bounds)` | `src/lib/HumanBrowserPanel.svelte:159` |
| Divider is a native vertical range input | `<input type="range" aria-orientation="vertical">` | `src/lib/PanelResizer.svelte:17-26` |
| Chat width is clamped to 280–560 px | `clampPanelWidth(agentWidth, 280, 560)` | `src/App.svelte:82` |
| Workspace grid stacks below 1080 px | grid columns plus the `max-width: 1080px` media query | `src/App.svelte:331,337`, `src/lib/theme.css:549` |
| Bounds are scheduled at `requestAnimationFrame`; native calls are async | `scheduleBounds` + `ResizeObserver` | `src/lib/PreviewPanel.svelte:80,91,166`, `src/lib/HumanBrowserPanel.svelte:45,103` |

## Procedure (same-session before/after at B18-S07)

S07 compares the `v0.0.13` baseline and the final B18 head **on the same Linux host and session**, with release builds, debug tooling off, loopback fixture only, and no private or external site. Raw samples, p50/p95/range, and process roles are recorded; no value is invented.

Fixture serving command (release gate only; not executed in B18-S01):

```sh
python3 -m http.server <port> --bind 127.0.0.1 --directory tests/fixtures/b18-canvas
# then navigate to http://127.0.0.1:<port>/
```

1. **Startup:** 10 runs to the readiness marker; report p50, p95, range.
2. **Settled idle CPU/PSS:** 5 windows after settling; report the shell, WebKitWebProcess, WebKitNetworkProcess, and fixture-server roles separately with median, p95, and range.
3. **Active Human Browser cost:** repeat step 2 with the fixture loaded in the phone slot; report the delta and the fixture server separately.
4. **Pointer-to-paint frames:** divider drag and Canvas-chrome swipe event-to-paint samples; p50/p95, missed-frame distribution, and the reference display's frame budget (60 Hz = 16.7 ms).
5. **Bounds IPC count:** instrumented count of `preview_set_bounds` / `human_browser_set_bounds` during one drag; assert at most one in flight plus one pending latest rectangle, no duplicate call, and the exact final rectangle applied after release.
6. **Browser/Preview phone `innerWidth`:** read the fixture's viewport readout (actual native width; 390 CSS px target clamped to the Canvas area); confirm no page reload solely from a width change.
7. **100 resize/swap/tab cycles:** child-process counts and PSS before/after, final bounds correctness, and zero orphaned resources.
8. **Cleanup:** after close, no `brainroot`, WebKit, or fixture-server process and no listener remains.

Deferred release cases: **B18-P00** fixture serves locally and renders the load marker and size readout; **B18-P01** same-session B17 vs B18 startup/idle/active-WebView samples; **B18-P02** 100 geometry/swap/tab cycles show final bounds and no leak; **B18-P03** 100 drags/switches plus idle cleanup.

## Raw results

### Same-session comparison (B17 `5f2835c` vs B18 `211af91`)

| Metric | B17 baseline | B18 candidate | Budget | Verdict |
|---|---|---|---|---|
| Startup to readiness, n=10 median (p95; range) | 0.856 s (0.866; 0.826–0.867) | 1.007 s (1.028; 0.967–1.028) | ≤ 1.0 s p50 | B17 pass; **B18 miss by 7 ms** |
| Settled idle CPU, 5-window median | 0.069 % | 1.345 % (windows 10.134/4.896/0.034/1.345/0.069) | < 1 % | B17 pass; **B18 fail on median** |
| Process-tree PSS median | 249.3 MB (brainroot/WebKit split not re-recorded) | 252.1 MB (brainroot 93.5, WebKitWebProcess 140.0, WebKitNetworkProcess 18.0) | ≤ 150 MB | **Fail since B01**, both builds |
| Release binary bytes | 8,081,576 | 8,085,800 (+4,224) | — | Informational |
| Frontend JS bytes (gzip) | 82,068 (28,529) | 87,614 (30,205) | — | Informational |
| Frontend CSS bytes (gzip) | 14,635 (3,225) | 16,667 (3,559) | — | Informational |

Host load differed between runs (4.88 during B17, 3.28 during B18), so no cross-build regression is claimed from this single pair — but budget verdicts are per-build facts: B18 misses the startup target it previously met and fails the idle-CPU median while B17 passed it. Both misses are reported for maintainer review, not excused and not attributed to any single microstep.

### One-HOT deck probe (B18 candidate, debug harness)

`BRAINROOT_DECK_FIXTURE=1` on the candidate tree: `decision: go`, no reasons; switches 0–4 ms; children 3 → 5/6 across phases → 6 after the second cycle → 4 after cleanup — the same bounded pattern as B17 (3 → 6 → 4).

### Interactive-only steps (no GUI automation in scope)

- Divider/swipe event-to-paint frames and missed-frame distribution (procedure step 4): `UNKNOWN`.
- Bounds IPC count during a live drag (procedure step 5): `UNKNOWN` at the IPC level; the helper-level burst/dedupe/retry/dispose cases passed in the frontend suite.
- Fixture `innerWidth` and reload-free width changes (procedure step 6): `UNKNOWN`.
- 100 resize/swap/tab cycles with child/PSS comparison (procedure step 7): `UNKNOWN`; the debug deck harness covers two core-level switch cycles with cleanup above.
- Post-close process/listener check (procedure step 8): covered by the smoke test in the full gate (main and child processes exited; no listeners remained).

## Result against budget

- Budget: [performance contract](../../10-performance-budget.md) — startup ≤ 1.0 s p50, idle CPU `< 1 %`, RSS ≤ 150 MB, one HOT heavy view, zero orphans, interaction targets
- Measured result (B18): startup miss by 7 ms; idle-CPU median fail (settling windows included); memory fail since B01; one-HOT `go`; zero orphans in smoke and deck cleanup
- Pass / warn / fail / unknown: fail (startup, idle CPU, memory) / unknown (interaction timing, 100-cycle soak)
- Comparison revision: baseline `v0.0.13` (`5f2835c`); candidate `211af91`

## Resource cleanup

Measured for B18: the full-gate smoke run closed the release binary with main and child processes exited and no listeners remaining; the debug deck harness returned to 4 children after cleanup with `decision: go`. The 100-cycle frontend soak stays `UNKNOWN` (procedure step 7).

## Interpretation

The B18 claim can only be settled at S07. Historical B17 data already shows the memory target failing and idle CPU being host-sensitive, so B18 must not advertise lightness without the same-session comparison. The local fixture isolates loopback content so viewport, page-gesture, bounds, and cleanup claims can be checked deterministically without external sites.
