# Performance contract

## Status vocabulary

- **TARGET:** desired engineering constraint, not yet proven.
- **MEASURED:** reproducible result with build, hardware, OS, procedure, and sample data.
- **UNKNOWN:** insufficient evidence.

As of 2026-09-22, the numbers below are a mix of **TARGET** and **MEASURED**: the Linux reference environment has a measured shell baseline and an MVP-0 release soak (startup and cancellation pass; idle CPU is a median pass with windows above budget on a busy host; the total-memory target **fails**). Live-path cancellation latency remains **UNKNOWN**. See [the B01 baseline](specs/performance-reports/b01-linux-baseline.md) and [the MVP-0 soak](specs/performance-reports/b05-mvp0-soak.md). Technology reputation is not a benchmark.

## Initial budgets

| Scenario | Metric | Budget | Status |
|---|---:|---:|---|
| Core + UI, no project activity | idle CPU after settling | `< 1%` | TARGET |
| Core + UI, no heavy content WebView | total RSS | `≤ 150 MB` | TARGET |
| Same, aspirational | total RSS | `≤ 100 MB` | TARGET / stretch |
| Warm application launch to interactive shell | p50 | `≤ 1.0 s` on reference modern hardware | TARGET |
| Warm application launch | p95 | UNKNOWN until prototype | UNKNOWN |
| Heavy content | simultaneously HOT views | `1` default | TARGET |
| No active task | agent processes | `0` | TARGET |
| No active language capability | LSP processes | `0` | TARGET |
| No active terminal/command | PTYs | `0` | TARGET |
| No preview/test need | dev servers / automation browsers | `0 / 0` | TARGET |
| Ordinary project open | automatic full index or embeddings | none | TARGET |
| Owned resource termination | orphan child processes | `0` | TARGET |

The memory budget excludes user project processes and heavy content only when reported separately; the UI must always show both BrainRoot-owned and project-owned cost in benchmark output.

## Interaction budgets

- UI input feedback: target within one 60 Hz frame for local state.
- Layout preset switch: target visual response under 100 ms, excluding WebView reconstruction.
- Task cancellation acknowledgement: target under 100 ms; actual child termination is separately measured.
- Preview and WebView create/destroy targets remain UNKNOWN until the platform probe.

## Measurement protocol

1. Use release builds with debug tooling off.
2. Record commit, dependency lockfile, OS/build, CPU, RAM, storage, display, power mode, WebView/runtime version, project fixture, and network condition.
3. Reboot or document prior system state; collect at least 10 startup runs and 5 steady-state windows.
4. Define interactive by an instrumented shell-ready event, not visual judgment.
5. Wait a documented settling period for idle metrics and report median, p95, range, and raw samples.
6. Measure process-tree RSS/working set/private memory and CPU with platform-native tools; never compare unlike metrics without a label.
7. Separate core/UI, content WebViews, agent, project processes, LSP, and automation.
8. Test open/close cycles for leaks and confirm resources return near baseline.

Use [the performance test template](specs/performance-test-template.md) for each result.

## CI and release gates

Phase 1 establishes a Linux reference environment and versioned benchmark fixture. Windows and macOS baselines are added in their later minimum-environment milestones; Linux measurements must not be generalized to them. Pull requests affecting startup, WebViews, process management, frontend bundle, provider streaming, or idle behavior include before/after evidence. Early CI warns on statistically meaningful regression; it becomes blocking after baselines are stable. A major regression requires a written exception with owner, reason, and expiry.

## Bundle/dependency accounting

Track installer size, unpacked size, Rust binary size, frontend compressed/uncompressed assets, JS evaluated at startup, dependency count, and licenses. Lazy loading counts only when it also avoids initialization and resource ownership.

## Local observability, not hidden telemetry

Useful future metrics include startup, RSS, CPU, WebView memory, agent latency, task success, rollback use, crashes, and cleanup failures. Development benchmarks can collect them locally. Any remote product telemetry requires a separate opt-in privacy decision and public schema.

## Pivot measurement status (2026-09-25, `0.0.15`)

**MEASURED** on the reference host (Pop!_OS 24.04 LTS, Wayland, WebKitGTK 2.52.6, release build, governor `performance`, AC online, load average 5.16, `BRAINROOT_MEASURE_SKIP_BUILD=1 sh scripts/measure-linux.sh`):

| Metric | Measured | Budget | Verdict |
|---|---:|---:|---|
| Startup to readiness, n=10 median (p95; range) | 1.269 s (1.309; 1.249–1.310) | ≤ 1.0 s p50 | **miss** |
| Settled idle CPU, 5 windows median (p95; range) | 0.103 % (2.055; 0.034–2.276) | < 1 % | pass (median), windows above budget |
| Process-tree PSS median | 249.3 MB (brainroot 86.3 / WebKitWebProcess 145.1 / WebKitNetworkProcess 18.4) | ≤ 150 MB | **fail since B01** |
| Release binary | 8,226,216 B | — | informational |
| Frontend JS / CSS | 102,259 B gzip 34,389 / 20,416 B gzip 4,041 | — | informational |

**Separate budget rule for the pivot:** when the sidecar is running it costs roughly 300–480 MB by itself, which is far above the whole-application target. Sidecar memory is therefore always reported as its own role and never added to the shell budget to make either number look better; the mitigation is lifecycle, not size — it is on-demand and is stopped after 60 s idle, on explicit Stop, or on window close.

The host was busy (load 5.16) and this head adds the entire agent surface, so these absolutes are not a controlled cross-release comparison. The failures above are recorded as failures, and no TARGET is adjusted to match them.
