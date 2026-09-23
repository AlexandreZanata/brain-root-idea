# Linux preview hosting probe: GtkFixed overlay (B09-S01)

**Status:** measured go for the GtkFixed overlay hosting path — decision input for the ADR 0006 review; not authorization to ship a preview

**Platform:** Linux only (reference environment)

**Reviewed:** 2026-09-23

**Related:** [Companion Browser plan](companion-browser-plan.md) · [preview origin policy](preview-origin-policy.md) · [Linux child-WebView probe](linux-webview-probe.md) · ADR 0006 · ADR 0011

## Question

B08-S01 measured the Tauri-managed child-WebView path as a no-go on geometry: Tauri builds children into the window's `gtk::Box`, so position and size are ignored. ADR 0006 therefore needs evidence for an alternative before any preview batch is assigned. This probe tests one candidate: a `wry` WebView created directly into a `gtk::Fixed` that overlays the Tauri window content, completely outside the Tauri webview manager (so it has no Tauri IPC channel at all).

## Method

A disposable probe binary (`src-tauri/src/bin/preview_hosting_probe.rs`) is built only with `--features probe`; the shipped app and its default build are unchanged. On `setup` it:

1. takes the window's `gtk::ApplicationWindow`, removes its `gtk::Box` child, puts that box into a `gtk::Overlay` as the main child, adds a `gtk::Fixed` as overlay child, and shows the result — the shell webview keeps filling the window, the fixed sits above it;
2. creates a `wry` WebView with `WebViewBuilder::new_with_web_context(...).with_url(...).with_bounds(...).build_gtk(&fixed)`, an allowlist navigation handler (fixture origin only), and a page-load handler;
3. drives a main-thread state machine that creates a second view at different bounds, reads back the GTK allocations, resizes the first view, attempts `file:///etc/passwd`, compares the shell box allocation, drops both views, and verifies the fixed has no children;
4. prints one bounded JSON report and exits.

Exact reproduction at the batch head:

```sh
pnpm build
cargo build --release --manifest-path src-tauri/Cargo.toml --features probe --bin preview_hosting_probe
timeout 300 ./src-tauri/target/release/preview_hosting_probe
```

## Environment

- Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, session `wayland` (`wayland-1`), WebKitGTK 2.52.6, scale factor 1.0 (`MEASURED`).
- Tauri `2.11.6`, `wry 0.55.1`, `gtk 0.18.2`.

## Results

Two consecutive release runs returned `"decision":"go"` with no failed checks and empty stderr; the numbers below are one of those runs, all `MEASURED` on the environment above.

| Check | Requested | Measured |
|---|---|---|
| first view allocation | (0, 0, 320, 200) | (0, 0, 320, 200) — exact |
| second view allocation | (340, 0, 200, 300) | (340, 0, 200, 300) — exact |
| resize allocation | (60, 30, 240, 180) | (60, 30, 240, 180) — exact |
| wry `bounds()` size | (320, 200) | (320, 200) |
| `file:///etc/passwd` | denied | denied (1 rejection), URL stayed on the fixture |
| shell `GtkBox` allocation | unchanged | unchanged before/after the overlay restructure |
| cleanup after dropping views | 0 fixed children | 0 children, clean exit 0, empty stderr |

Timings: create 53 ms, first page load 202 ms in the release probe. These are probe-scale numbers, not a product budget.

## Interpretation

- The GtkFixed overlay path removes the geometry blocker that B08-S01 measured: `wry` sizes and places the view through `Fixed::put` and `set_size_request`, and `set_bounds` reallocates exactly. Two views coexist at independent bounds.
- The preview view is created outside Tauri's webview manager, so it has no Tauri IPC channel by construction; the navigation handler is the enforcement point the preview origin policy describes.
- The overlay restructure leaves the Tauri-managed shell webview and its container allocation intact on this environment.
- Unknowns that must be proven before or during the preview slice: focus and keyboard routing between shell and preview, zoom/scale-factor behavior on non-1.0 displays, DPR/media behavior, repeated create/destroy over a longer soak, and crash handling of the preview view. None of these were measured here.

## Decision

- **Go for the GtkFixed overlay + direct wry hosting path.** The ADR 0006 review (B09-S02) should adopt it with the measured evidence above, the dependency review for `gtk`/`wry` (already transitive to Tauri; direct and optional behind the probe feature), and the ownership model from ADR 0011.
- The probe binary and feature stay disposable; the preview implementation moves the dependencies behind the production preview feature when CB-A lands, and the remaining unknowns above become acceptance checks in the preview slice.
