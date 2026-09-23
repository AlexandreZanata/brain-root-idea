# Preview quality measurements (B12-S01)

**Status:** measured closure of the remaining CB-A acceptance checks — focus, page zoom, scale/DPR, and bounds soak; keyboard routing stays `UNKNOWN`

**Reviewed:** 2026-09-23

**Related:** [Companion Browser plan](companion-browser-plan.md) · [ADR 0012](../adr/0012-linux-preview-hosting.md) · [Linux preview hosting probe](linux-preview-hosting-probe.md)

## Question

ADR 0012 listed four checks that the preview slice had to measure before the capability could be called complete: focus/keyboard routing, zoom/scale behavior beyond factor 1, DPR/media behavior, and repeated create/destroy or resize soak. This document records those measurements with the disposable debug harness, which exercises the production preview module.

## Method

The debug-only fixture harness (`BRAINROOT_PREVIEW_FIXTURE=1`, `#[cfg(debug_assertions)]` only) starts the owned dev server, shows the preview view at the fixed rectangle (40, 80, 480 × 320), and then measures through the production module:

1. **Scale/allocation:** the GTK scale factor and the preview widget allocation, compared with the requested logical rectangle.
2. **DPR:** `window.devicePixelRatio` read from the preview content through a script evaluation.
3. **Focus:** the preview is focused and the GTK widget focus plus the host window active state are read.
4. **Zoom:** `zoom(2.0)` then `zoom(1.0)`, reading `window.innerWidth` in CSS pixels between steps.
5. **Soak:** 100 alternating `set_bounds` updates with per-update latency, plus `VmRSS` and child-process counts before and after.

Reproduction:

```sh
pnpm build
cargo build --manifest-path src-tauri/Cargo.toml
BRAINROOT_PREVIEW_FIXTURE=1 ./src-tauri/target/debug/brainroot            # Wayland reference session
GDK_BACKEND=x11 GDK_SCALE=2 BRAINROOT_PREVIEW_FIXTURE=1 ./src-tauri/target/debug/brainroot
```

## Results

All numbers are `MEASURED`; both runs exited `0` and left no process, listener, or fixture server behind.

| Check | Wayland, scale 1 | XWayland, `GDK_SCALE=2` |
|---|---|---|
| scale factor seen by the app | 1 | 2 |
| `window.devicePixelRatio` | 1.0 | 2.0 |
| requested rectangle (logical) | (40, 80, 480, 320) | (40, 80, 480, 320) |
| allocation (GTK3 logical) | (40, 80, 480, 320) — exact | (40, 80, 480, 320) — exact, no scale drift |
| preview widget focused | true | true |
| host window active | true | true |
| `innerWidth` base → zoom 2.0 → restore | 480 → 240 → 480 | 480 → 240 → 480 |
| soak updates | 100 | 100 |
| soak latency p50 / max | 0 µs / 31 µs | 0 µs / 26 µs |
| RSS before → after | 206 448 → 206 468 KB (+20 KB) | 188 440 → 188 456 KB (+16 KB) |
| child processes before → after | 5 → 5 | 5 → 5 |

## Interpretation

- **Geometry is scale-correct.** GTK3 reports allocations in logical pixels; the requested rectangle is preserved exactly at scale 1 and at scale 2, and the content sees the matching DPR (1.0 / 2.0). The preview does not double-scale or drift when the display scale changes.
- **Page zoom works and is reversible.** Doubling the zoom halves the CSS viewport width and restoring it returns the original value; the widget bounds are unaffected.
- **Focus is reachable.** The preview widget accepts GTK focus and the host window is active when measured.
- **Bounds churn is cheap and stable.** 100 resizes cost microseconds each with no child-process change and only tens of kilobytes of RSS growth, which is within run-to-run noise at this scale.
- **Keyboard event routing into the preview and focus traversal between shell and preview remain `UNKNOWN`.** They require synthetic input on the reference stack, which is not available; keyboard, screen-reader, and reduced-motion behavior inside the preview still needs the maintainer's rendered check.

## Decision

- The measured checks close the corresponding ADR 0012 acceptance items for the supported environments above: scale/DPR, zoom, focus (widget-level), and the bounded resize soak.
- The remaining `UNKNOWN` items are recorded in `docs/17-open-questions.md` and stay explicit; no claim is made about keyboard routing or screen-reader behavior inside preview content.
