# Performance report: BrainRoot Linux shell warm start and settled idle

**Status:** Measured
**Date/commit:** 2026-09-22 · `f62923114d62d25f910c375188ea57d23506ed3e` (`Cargo.lock` sha256 `db0bd279…`, `pnpm-lock.yaml` sha256 `3e544218…`)
**Owner:** BrainRoot maintainer (B01-S06)
**Related budget:** `docs/10-performance-budget.md` — warm launch p50, idle CPU, total RSS, bundle accounting

## Claim

On the single reference environment below, with the production profile and no project activity: warm start to the instrumented ready marker is **MEASURED at p50 0.282 s** (TARGET ≤ 1.0 s, pass); settled idle CPU is **MEASURED at median 0.172 %** (TARGET < 1 %, pass); total memory is **MEASURED at 198.5 MB proportional (PSS) and 418.9 MB summed VmRSS** (TARGET ≤ 150 MB, **fail**); no process, listener, or WebView survives a close (TARGET 0 orphans, pass). These labels apply only to this environment; they are not generalized to other hardware, Windows, or macOS.

## Environment

- BrainRoot build/profile: production (`pnpm tauri build --no-bundle`, `release` profile, debug tooling off); `Cargo.lock` and `pnpm-lock.yaml` hashes above.
- OS, version, architecture, patches: Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, x86_64.
- CPU, RAM, storage, display: 13th Gen Intel Core i7-13620H, 16 logical CPUs, 32,556,572 KiB RAM (~31.0 GiB), 460 GB root filesystem with 53 GB free; Wayland session.
- Power/thermal mode and background workload: laptop on AC (`ACAD online=1`, battery present), CPU governor `performance`; uptime 6 d 21 h; load average `2.65 2.76 3.61` immediately before the run; no reboot was possible, so this prior state is documented instead.
- WebView/runtime versions: WebKitGTK 2.52.6 (`webkit2gtk-4.1`).
- Project fixture and size: none — the shell has no project open (the first MVP-0 milestone state); frontend bundle sizes are listed below.
- Network condition: no network request occurs during startup or idle; measurements were taken without network activity from the app.
- Measurement tools and versions: `scripts/measure-linux.sh` (POSIX `sh` + Python 3.12 stdlib), `/proc` sampling, `stat`/`gzip`, `sha256sum`; `hyperfine` and `perf` are not installed.

## Procedure

1. Build the production binary with `pnpm tauri build --no-bundle` (debug tooling off).
2. **Startup runs:** launch the binary in its own process group, poll its stderr every 20 ms for the readiness marker `brainroot: health contract v… served`, and record the wall time (15 s timeout). Ten sequential runs; the app is closed through its process group after each run and the next run starts only after every process is gone.
3. **Steady-state windows:** launch, wait for readiness, settle 10 s, then sample every 1 s for 30 s. Sampling includes the main process and every descendant (`WebKitWebProcess`, `WebKitNetworkProcess`): CPU % from `/proc/<pid>/stat` utime+stime deltas divided by `CLK_TCK` and the measured interval; memory from `VmRSS` (`/proc/<pid>/status`) and `Pss` (`/proc/<pid>/smaps_rollup`). Five fresh sessions.
4. **Cleanup cycles:** three open/close cycles, each asserting that no BrainRoot or WebKit process remains.
5. **Bundle accounting:** binary bytes; every `dist/` asset uncompressed and gzip bytes.
6. Statistics: median; p95 by linear interpolation (`statistics.quantiles(n=100, method="inclusive")`); minimum and maximum. Raw samples are below.

Reproduction: `BRAINROOT_MEASURE_RAW=/tmp/brainroot-perf-raw.txt sh scripts/measure-linux.sh` (add `BRAINROOT_MEASURE_SKIP_BUILD=1` to reuse a current binary).

## Raw results

Warm start to the readiness marker (seconds), 10 samples:

| Run | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| Seconds | 0.262 | 0.282 | 0.282 | 0.262 | 0.282 | 0.282 | 0.303 | 0.282 | 0.282 | 0.282 |

Startup summary: median **0.282 s**, p95 **0.293 s**, min **0.262 s**, max **0.303 s**.

Settled steady-state windows (30 samples each; CPU is the window mean, memory the median):

| Window | Mean CPU % | Median RSS MB | Median PSS MB |
|---|---:|---:|---:|
| 1 | 0.207 | 418.6 | 198.1 |
| 2 | 0.000 | 419.2 | 198.8 |
| 3 | 0.034 | 418.9 | 198.5 |
| 4 | 0.172 | 421.2 | 200.5 |
| 5 | 0.448 | 418.7 | 198.4 |

Window summaries: CPU median **0.172 %**, p95 **0.400 %**, min **0.000 %**, max **0.448 %**; RSS median **418.9 MB**, p95 **420.8 MB**, min **418.6 MB**, max **421.3 MB**; PSS median **198.5 MB**, p95 **200.2 MB**, min **198.1 MB**, max **200.5 MB**.

Memory split (median of 150 samples per class, MB):

| Process | RSS | PSS |
|---|---:|---:|
| `brainroot` (core/UI host) | 174.8 | 76.2 |
| `WebKitWebProcess` (webview) | 196.1 | 104.9 |
| `WebKitNetworkProcess` | 48.0 | 17.5 |

Bundle accounting:

| Artifact | Bytes | Gzip bytes |
|---|---:|---:|
| `src-tauri/target/release/brainroot` | 4,434,616 | — |
| `dist/index.html` | 393 | 276 |
| `dist/assets/index-BgV6EIr7.js` | 30,882 | 11,984 |
| `dist/assets/index-CDXTXvlt.css` | 548 | 325 |

Cleanup cycles: 3 of 3 completed; after every cycle no `brainroot` or WebKit process remained; the smoke test separately confirms no listening socket and no extra WebView.

Repeatability note: the same procedure ran three times within an hour. Startup medians were 0.262 s, 0.282 s, and 0.282 s; idle CPU medians were 0.103 %, 0.069 %, and 0.172 %, with one window at 1.310 % in the first run (above the 1 % target); summed RSS stayed within 418–421 MB. Startup is stable; idle CPU is close to the budget and needs more windows before it is declared stable.

## Result against budget

- **Warm launch p50 ≤ 1.0 s:** measured **0.282 s** — **pass**.
- **Warm launch p95:** budget was UNKNOWN before the prototype; now **0.293 s MEASURED** on this environment.
- **Idle CPU after settling < 1 %:** measured **0.172 % median** — **pass** for this run; one window in an earlier run measured 1.310 %, so treat this as **warn** until more windows confirm stability.
- **Total RSS ≤ 150 MB:** measured **198.5 MB PSS / 418.9 MB summed VmRSS** — **fail**. Summed VmRSS double-counts shared library pages (the three processes map the same WebKit stack), so PSS is the fairer comparison; both numbers exceed the target.
- **Agent / LSP / PTY / dev server / automation browser processes:** **0** — pass by construction for this shell and confirmed by the cleanup checks.
- **Orphan child processes:** **0** — pass.

Comparison revision: first baseline; no previous measured revision exists. Statistical caveats: 10 startup samples and 5 idle windows on a machine with background load ~2.7–3.6 and no reboot before measurement; percentiles use linear interpolation; memory metrics are process-tree aggregates.

## Resource cleanup

After every startup run, steady-state window, and cleanup cycle, the main process and both WebKit children exited; `scripts/smoke-linux.sh` additionally verifies no listening socket remains. No temporary log, fixture, or measurement artifact was left behind by the procedure. The repeated-cycle check (3 cycles) found no accumulating process.

## Interpretation

The data supports: a small, fast-starting shell (p50 0.282 s to the typed ready marker), effectively idle CPU (0.17 % median) with no background ownership, and a very small frontend bundle (≈31 KB uncompressed, ≈12.6 KB gzip) plus a ≈4.2 MB Rust binary. It does **not** support claims about first paint or visual readiness (the marker is a contract round-trip, not a paint event), cold start, other hardware, or other platforms; the WebKit RSS figures include shared pages and must not be compared with private-memory metrics without a label.

The memory TARGET fails: the shell alone reaches ≈198 MB PSS, dominated by WebKitGTK's webview (~105 MB) and the core/UI host (~76 MB). This is the expected cost of a system-WebView architecture rather than a defect, but the 150 MB target was aspirational and is not met at the shell stage. Next experiments: (1) repeat idle windows on a quiet machine to settle the CPU budget; (2) capture PSS with a project open and a preview WebView to see the second large component; (3) evaluate whether the memory target should be revised by an ADR-level decision instead of being treated as a regression.
