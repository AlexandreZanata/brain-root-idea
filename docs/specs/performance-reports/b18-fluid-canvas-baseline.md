# Performance report: B18 fluid Canvas and phone-first Browser (baseline)

**Status:** Planned — no B18 runtime measurement executed
**Date/commit:** 2026-09-24; baseline `v0.0.13` (`main` `5f2835c`), phase head `e133db5`
**Owner:** maintainer / economical agent
**Related budget:** [performance contract](../../10-performance-budget.md) (startup, idle CPU, memory, one HOT view, cleanup, interaction budgets) · [B18 phase spec](../b18-frontend-fluidity-phone-browser.md)

## Claim

Question: does B18 — Browser/phone as the default Canvas destination, fluid divider, Swap sides, Canvas-chrome swipe, and phone-width native geometry — meet the responsiveness, native-work bound, startup/idle, lifecycle, accessibility, and security targets without regressing `v0.0.13`?

**Result: `UNKNOWN`.** No B18 behavior has been measured. The B17 numbers below are historical context from a different session and are **not** a controlled before/after comparison. Nothing in this report may be reported as a B18 pass.

## Environment

Fields to record exactly for the release-profile run at B18-S07:

- BrainRoot build/profile and lockfile: release build (`pnpm tauri build --no-bundle`), candidate commit, `src-tauri/Cargo.lock` and `pnpm-lock.yaml` sha256
- OS, version, architecture, patches: Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, x86_64 (B01 reference environment)
- CPU, RAM, storage, display: 13th Gen Intel Core i7-13620H (16 threads), 32,556,572 KiB RAM, 460 G root (record free space), Wayland session
- Power/thermal mode and background workload: record governor, AC state, uptime, load average
- WebView/runtime versions: WebKitGTK 2.52.6
- Project fixture and size: `tests/fixtures/b18-canvas/index.html` served over loopback; record file size and sha256
- Network condition: loopback only; no external network, no private site, no live profile or account
- Measurement tools and versions: `sh scripts/measure-linux.sh`, the debug deck harness, and process-tree sampling; record versions and raw commands

### B17 historical context (MEASURED on B17; not comparable until the same-session rerun)

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

None. Every B18 measurement is `UNKNOWN` until S07 executes the procedure above.

## Result against budget

- Budget: [performance contract](../../10-performance-budget.md) — startup ≤ 1.0 s p50, idle CPU `< 1 %`, RSS ≤ 150 MB, one HOT heavy view, zero orphans, interaction targets
- Measured result: `UNKNOWN` (B18)
- Pass / warn / fail / unknown: unknown
- Comparison revision: baseline `v0.0.13`; candidate head recorded at S07

## Resource cleanup

Not executed for B18. S07 records the 100-cycle child/PSS comparison and the post-close process/listener check.

## Interpretation

The B18 claim can only be settled at S07. Historical B17 data already shows the memory target failing and idle CPU being host-sensitive, so B18 must not advertise lightness without the same-session comparison. The local fixture isolates loopback content so viewport, page-gesture, bounds, and cleanup claims can be checked deterministically without external sites.
