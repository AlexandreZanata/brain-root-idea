# Performance report: BrainRoot MVP-0 Linux release soak

**Status:** Measured
**Date/commit:** 2026-09-22 · `1c3551f5d240f4c4b90d990c84d458527e4f8eb6` (B05-S03 worktree; the release binary contains only non-test code)
**Owner:** BrainRoot maintainer (B05-S03)
**Related budget:** `docs/10-performance-budget.md` — warm launch p50/p95, settled idle CPU, total RSS, task cancellation acknowledgement, orphan cleanup

## Claim

On the single reference environment below, with the production profile and no project activity: warm start to the ready marker is **MEASURED at p50 0.463 s / p95 0.475 s** (TARGET ≤ 1.0 s, pass); settled idle CPU is **MEASURED at median 0.034 %** with two of five windows at 1.724 % and 1.965 % (TARGET < 1 %, **warn** — the median passes, the p95 does not); total memory is **MEASURED at 241.6 MB proportional (PSS) / 426.2 MB summed VmRSS** (TARGET ≤ 150 MB, **fail**, worse than the B01 baseline); the conversation core soaks **500 + 500 fake conversations and 200 cancellations with joined workers, stable thread count, and ~0.5 MiB RSS growth**; cancellation acknowledgement is **MEASURED at 0 µs dispatch**, with the full deterministic cancel cycle at 11 µs median / 13 µs p95 / 73 µs max (TARGET < 100 ms, pass); live-path cancellation latency remains **UNKNOWN** (bounded by the 120 s transport read). These labels apply only to this environment.

## Environment

- BrainRoot build/profile: production (`pnpm tauri build --no-bundle`, `release` profile, debug tooling off); `Cargo.lock` sha256 `5f555d18…`, `pnpm-lock.yaml` sha256 `3e544218…`; binary 7,903,784 bytes; assets `index.html` 393 B (gzip 279), `index-7QAJX4LH.js` 45,863 B (gzip 17,335), `index-CJlYcmYR.css` 3,559 B (gzip 1,066).
- OS, version, architecture, patches: Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, x86_64.
- CPU, RAM, storage, display: 13th Gen Intel Core i7-13620H, 16 logical CPUs, 32,556,572 KiB RAM (~31.0 GiB), 460 GB root filesystem with 35 GB free; Wayland session.
- Power/thermal mode and background workload: laptop on AC, CPU governor `performance`; the host was in active use during the run (browser and editor), which the idle windows reflect.
- WebView/runtime versions: WebKitGTK 2.52.6.
- Project fixture and size: none — the shell has no project open.
- Network condition: no network request during startup or idle; the live smoke is separate and opt-in.
- Measurement tools and versions: `scripts/measure-linux.sh` and `scripts/soak-linux.sh` (POSIX `sh` + Python 3.12 stdlib), `/proc` sampling, `stat`/`gzip`, `sha256sum`; no `hyperfine` or `perf`.

## Procedure

1. Build the production binary with `pnpm tauri build --no-bundle` (debug tooling off).
2. **Startup:** ten sequential launches, polling stderr every 20 ms for the readiness marker `brainroot: health contract v… served`; each app closed through its process group before the next run.
3. **Idle windows:** five fresh sessions, 10 s settling, then 30 s of 1 s samples over the whole process tree: CPU from `/proc/<pid>/stat` deltas, memory from `VmRSS` and `Pss`.
4. **Cleanup cycles:** three open/close cycles; any leftover process fails the run; plus five additional cycles through `soak-linux.sh` (via `smoke-linux.sh`, which also asserts one web and one network child, no listener, and clean `SIGTERM` exit).
5. **Core soak (`soak-linux.sh`, release):** `cargo test --release soak -- --ignored --nocapture` runs 500 completed fake conversations, 200 cancellations through the product session, then 500 more conversations, printing RSS (`/proc/self/status`), thread count (`/proc/self/task`), and cancel dispatch/cycle latencies.
6. Statistics: median; p95 by linear interpolation (`statistics.quantiles(n=100, method="inclusive")`); minimum/maximum. Raw files: `/tmp/opencode/b05-measure-raw.txt` (measure) and the soak stdout above (bounded; not committed).

Reproduction: `BRAINROOT_MEASURE_RAW=/tmp/b05-measure-raw.txt sh scripts/measure-linux.sh` and `sh scripts/soak-linux.sh`.

## Raw results

Warm start to the readiness marker (seconds), 10 samples: 0.484, 0.463, 0.463, 0.443, 0.463, 0.463, 0.463, 0.464, 0.444, 0.443 → **median 0.463, p95 0.475, min 0.443, max 0.484**.

Settled idle windows (whole process tree): mean CPU 0.034 %, 1.724 %, 1.965 %, 0.034 %, 0.034 %; median RSS 425.6–435.9 MB, median PSS 241.0–251.2 MB over the five windows → **CPU median 0.034 %, p95 1.917 %, max 1.965 %; RSS median 426.2 MB, PSS median 241.6 MB**.

Per-process class medians: `brainroot` PSS 98.3 MB / RSS 177.3 MB; `WebKitWebProcess` PSS 125.7 MB / RSS 201.5 MB; `WebKitNetworkProcess` PSS 17.8 MB / RSS 47.5 MB (150 samples each).

Core soak (`soak: conversations=500 cancelled=200 second_batch=500`): RSS 3,888 KiB → 4,360 KiB (after conversations) → 4,364 KiB (after cancellations) → 4,408 KiB (after the second batch), i.e. **~520 KiB growth for 1,200 requests**; threads 2 → 2; cancel dispatch 0 µs median/p95/max; full cancel cycle 11 µs median, 13 µs p95, 73 µs max; 500 conversations in 18 ms; 200 cancellations in 7 ms.

Open/close: 3 measure cycles plus 5 soak cycles, all with no leftover process, listener, or child WebView.

## Result against budget

- Warm launch p50: TARGET ≤ 1.0 s → **MEASURED 0.463 s, pass** (B01 baseline was 0.282 s; this host was busier).
- Idle CPU: TARGET < 1 % → **MEASURED median 0.034 %, warn**: two of five windows exceeded the budget on a host in active use; not a proven regression, not a pass.
- Total RSS: TARGET ≤ 150 MB → **MEASURED 426.2 MB summed / 241.6 MB PSS, fail** (B01 was 418.9 MB / 198.5 MB PSS); the shell plus WebKit processes remain far above the target.
- Cancellation acknowledgement: TARGET < 100 ms → **MEASURED 0 µs dispatch, 11 µs median full cycle, pass** for the deterministic fake; the live path is UNKNOWN.
- Orphan processes/listeners: TARGET 0 → **MEASURED 0 across 8 cycles, pass**.

## Resource cleanup

After every deterministic conversation and cancellation, `is_active()` is false and every worker handle joins; the thread count returns to its pre-soak value; RSS grows by ~0.5 MiB across 1,200 requests (allocator noise, no monotonic trend between batches). The process smoke asserts one web and one network child, no remaining listener, and clean exit after `SIGTERM`, repeated eight times.

## Interpretation

The release candidate keeps startup, cancellation, and cleanup comfortably inside their budgets on this host, and the core shows no thread or request leak under repeated fake conversations. The memory TARGET still fails and is worse than B01 (241.6 MB vs 198.5 MB PSS; the difference sits in `brainroot` and WebKit classes and is plausibly driven by the added conversation UI and WebKit state), and the idle CPU budget is only a median pass on a busy host. Both are recorded as failing/warned, not adjusted. Next experiments: rerun on a quiet machine for a clean CPU distribution, and investigate WebKit/`brainroot` PSS growth; live-path cancellation latency needs a slow-stream fixture and remains UNKNOWN.
