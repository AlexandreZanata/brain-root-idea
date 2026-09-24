# ADR 0012: Linux preview hosting with a GtkFixed overlay

**Status:** Accepted  
**Date:** 2026-09-23

## Context

ADR 0006 chose isolated WebView roles and one HOT heavy content view, but left the Linux hosting mechanism to a probe: "one on-demand preview child WebView if the platform implementation proves reliable". B08-S01 measured the Tauri-managed child-WebView path and found it unreliable for the product layout: Tauri builds children into the window's `gtk::Box`, so position and size are ignored on the Wayland reference environment ([child-WebView probe](../specs/linux-webview-probe.md)).

B09-S01 measured an alternative and returned `go` twice in release: a `wry` WebView created directly into a `gtk::Fixed` that overlays the shell content inside the same Tauri window. Two views held exact requested bounds, a resize reallocated exactly, `file://` navigation was denied, the Tauri-managed shell allocation stayed unchanged, and dropping the views left zero children and a clean exit ([hosting probe](../specs/linux-preview-hosting-probe.md)).

The surrounding contracts are already accepted: the [preview origin policy](../specs/preview-origin-policy.md) (loopback, BrainRoot-owned ports, fail-closed), ADR 0011 (the Rust core owns the one dev server per project, with stop-on-close and bounded output), ADR 0008 (zero idle), and the plan's rule that the preview receives no privileged core API. What remains unmeasured is focus/keyboard routing between shell and preview, zoom/scale behavior beyond factor 1, DPR/media behavior, create/destroy soak, and preview crash handling.

## Decision

1. On Linux, the Preview role is a `wry` WebView created directly into a `gtk::Fixed` that is an **overlay child** of the single Tauri window; the shell webview keeps filling the window as the overlay's main child. The preview is **not** registered in Tauri's webview manager, so it has no Tauri IPC channel by construction.
2. At most one preview view exists at a time. It is owned by the Rust core, created on demand for preview, and destroyed on preview close, project close, or application exit, following the ADR 0011 ownership rules.
3. Geometry comes from the shell: the Svelte shell reports the Canvas slot rectangle in logical coordinates through the typed IPC boundary; the Rust core applies `Fixed::put`, `set_size_request`, and wry `set_bounds`. The shell is the trusted surface and the preview never controls its own placement.
4. Every preview view is created with the approved origin allowlist enforced in the navigation handler, an isolated `WebContext` data directory owned by BrainRoot, and no privileged IPC. Denied navigations fail closed and surface the visible blocked state defined by the policy.
5. The first slice previews the owned dev server only, with lifecycle, port registration, and cleanup from ADR 0011.
6. This mechanism is Linux-specific (GTK). Windows and macOS keep ADR 0006's probe-first rule and require their own hosting decision before parity is promised.
7. The preview slice must measure the remaining unknowns before the capability is called usable: focus and keyboard routing between shell and preview, zoom/scale factor other than 1, DPR/media behavior, repeated create/destroy soak without leaks, and preview crash handling with recovery.
8. `gtk` and `wry` become direct dependencies only behind the production preview feature when it lands; B09-S01 records their probe-only optional review and the security gate's dependency set is extended deliberately at that point.

## Alternatives considered

- **Tauri-managed child WebView:** measured no-go on geometry (B08-S01).
- **Separate native preview window:** Wayland gives clients no absolute positioning, so the preview cannot be placed in the Canvas slot.
- **Bundled Chromium or a second engine:** forbidden by the Companion Browser plan; it would add a resident engine and hide the baseline cost.
- **Wait for upstream Tauri/wry bounds support:** no path in Tauri 2.11.6; it would stall the roadmap without a decision.
- **X11 native-child path:** would force the app onto XWayland, contradicting the frozen Wayland reference environment.
- **GTK-level reparenting of a Tauri-managed webview:** possible in principle but keeps the view inside the Tauri manager (an IPC surface) and fights Tauri's layout ownership; rejected in favor of a view outside the manager.

## Consequences

BrainRoot owns a small amount of low-level GTK composition (overlay + fixed) on Linux, isolated in one Rust module and covered by the probe evidence. The preview has no IPC surface by construction, which is stronger than capability scoping, and the shell's own IPC boundary is unchanged. The shell must report the Canvas rectangle and keep ownership of layout decisions. The preview feature adds optional direct dependencies for the Linux target that are already present transitively; Windows and macOS builds are unaffected.

## Performance implications

One preview view at a time, created on demand; the probe measured 51–53 ms create and 150–202 ms load at probe scale on the reference environment, which is not a product budget. No preview or dev server exists at idle (ADR 0008, ADR 0011). The preview slice must measure create/destroy latency, memory return, and the soak before any "light" or "fast" claim, and the existing failing process-tree memory TARGET is not hidden by the new view.

## Security implications

The preview view has no Tauri IPC channel, so remote content cannot invoke core commands even if a capability were misconfigured. The approved origin allowlist is enforced at creation and on every navigation, including redirects; `file:`, `javascript:`, `data:`, popups, downloads, and permission prompts are denied by default. The profile lives in a BrainRoot-owned data directory separate from the shell and from any installed browser, and the preview never reads installed browser profiles. The GTK composition stays inside the application window and introduces no new network destination, resident engine, or privileged process.

## Reversibility

Medium-high. The hosting mechanism is an internal implementation choice: a future upstream Tauri/wry geometry API or a platform-specific alternative can replace it without changing the product contract, the origin policy, or ADR 0011. The probe binaries remain disposable and the optional dependencies can be removed with the preview feature.

## References

[Companion Browser plan](../specs/companion-browser-plan.md), [child-WebView probe](../specs/linux-webview-probe.md), [preview hosting probe](../specs/linux-preview-hosting-probe.md), [preview origin policy](../specs/preview-origin-policy.md), [browser architecture](../08-browser-architecture.md), ADR 0006, ADR 0008, and ADR 0011.
