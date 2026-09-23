# Linux child-WebView lifecycle probe (B08-S01)

**Status:** measured spike result — no-go on child-view geometry with the current stack; not authorization to implement preview

**Platform:** Linux only (reference environment)

**Reviewed:** 2026-09-23

**Related:** [Companion Browser plan](companion-browser-plan.md) · [ADR 0006](../adr/0006-browser-webview-strategy.md) · [Linux reference environment](linux-reference-environment.md)

## Question

The adopted Companion Browser plan requires a platform probe before any CB-A preview issue is assigned: on the exact supported Linux display stacks, can child WebViews be positioned, resized, isolated, destroyed, and recreated reliably through the current Tauri/wry stack? If not, the plan requires stopping and reviewing ADR 0006 instead of quietly introducing bundled Chromium or another always-on process.

## Method

A disposable probe binary (`src-tauri/src/bin/webview_probe.rs`) is built only with `--features probe`, which enables `tauri/unstable` and leaves the shipped application and its default build unchanged. The probe starts a fixture HTTP server on `127.0.0.1` with an OS-assigned port, hides the application window, and drives phases through the Tauri API:

1. `create_destroy` — 100 create → load → destroy cycles of a child WebView (`WebviewBuilder` + `Window::add_child`) at logical 480 × 320, sampling `VmRSS` and child processes every 10 cycles.
2. `resize` — create one child, `set_size(640 × 200)`, read back the physical size.
3. `layout` — create two children simultaneously and read their physical size and position.
4. `isolation` — two children with distinct `data_directory`s: profile A writes `localStorage`, a fresh profile B reads it, then A is recreated and reads it again.
5. `crash_recovery` — force-close a loaded child and recreate it, timing the reload.
6. `navigation` — attempt `file:///etc/passwd` navigation and confirm the fixture URL is still loaded.
7. `cleanup` — close all children and count remaining child processes.

Exact reproduction on `main` at the batch head:

```sh
pnpm build
cargo build --release --manifest-path src-tauri/Cargo.toml --features probe --bin webview_probe
timeout 600 ./src-tauri/target/release/webview_probe --cycles 100
```

## Environment

- Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, session `wayland` (`wayland-1`), WebKitGTK `2.52.6` (`MEASURED`).
- Tauri `2.11.6` with `unstable`, `tauri-runtime-wry 2.11.4`, `wry 0.55.1`.
- Scale factor 1.0; host window inner size 760 × 520 physical.

## Results

All numbers below are `MEASURED` on the environment above; no threshold is claimed because this probe has no prior budget.

| Phase | Result |
|---|---|
| create/load/destroy | 100/100 cycles completed, 0 failures |
| create latency | min 1 ms, p50 1 ms, max 4 ms |
| load latency | min 97 ms, p50 571 ms, max 1530 ms |
| destroy latency | 50 ms (measurement floor: 50 ms settle after `close()`) |
| RSS | 167.2 MB before cycles → 190.0 MB after; +244 KB/cycle average while one child process stayed alive |
| child processes | 1 during cycles; 1 remained after all views closed |
| resize | requested 640 × 200 logical; physical stayed 720 × 433 — **not honored** |
| layout (2 children) | requested 480 × 320 each; actual 720 × 217 and 720 × 216 at position (0, 0) — **geometry not honored**; children split the window box |
| profile isolation | pass: fresh profile read `empty`, recreated profile read `alpha` |
| crash recovery | pass: forced close then recreate loaded in 582 ms |
| navigation denial | pass: `file://` navigation was ignored while the fixture URL stayed loaded |
| cleanup | probe process exited 0, no probe process remained; one shared WebKit process outlived all closed views |

## Findings

- **Child geometry is not supported by the current public stack on this environment.** Tauri's runtime attaches child WebViews to the window's default `gtk::Box` (`tauri-runtime-wry-2.11.4`, `create_webview` → `default_vbox`), and wry only applies bounds when the parent container is a `GtkFixed` (`wry-0.55.1`, `webkitgtk::add_to_container` and `WebView::set_bounds`); with a `GtkBox` the child is packed with expand/fill and `set_size`/`set_position` become no-ops. Under Wayland there is no native X11 child-window path either. Two children therefore stack and split the window instead of sitting at requested positions.
- Create/destroy, localhost navigation, the navigation allowlist hook, separate profile storage, and forced-close recovery all work on this stack, so a preview slice is not blocked on those capabilities.
- Memory grew ~24 MB across 100 cycles with a constant child-process count; whether that is allocator growth or a leak is `UNKNOWN` and needs a longer soak before any budget claim.
- One shared WebKit process remains after the last view closes; the app-level idle story (one-HOT ownership) must account for that process, not assume zero WebKit processes at idle.

## Decision

- **No-go for CB-A as specified while position and size of a child view are required.** Per the plan's stop rule, do not work around this with a bundled engine or an always-on process.
- Next: review ADR 0006 with the options this probe surfaced — a `GtkFixed`-based hosting path with its own ownership model, a native-window preview surface, upstream Tauri/wry support for child-view bounds, and the X11 native-child path — before assigning preview implementation batches. The remaining CB-A prerequisites (origin policy and dev-server lifecycle contract) are independent of this result and proceed in B08.
- The probe binary and its Cargo feature are disposable; the shipped application does not include them or any dependency change.
