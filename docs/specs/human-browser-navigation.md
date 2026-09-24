# Human Browser engine and navigation policy (B14-S01)

**Status:** measured engine slice — the Canvas surface and Deck remain open

**Reviewed:** 2026-09-23

**Related:** [ADR 0013](../adr/0013-human-browser-policy.md) · [Human profile isolation probe](human-profile-isolation-probe.md) · [ADR 0012](../adr/0012-linux-preview-hosting.md) · [Companion Browser plan](companion-browser-plan.md)

## Question

CB-B's first candidates ask for a typed browser role with navigation decisions and negative policy tests (B1), then minimal human navigation (B3). This record covers the engine half: the typed policy, the isolated remote view hosted in the shared Canvas overlay, and the navigation commands, validated by a debug harness on the reference environment.

## Typed policy

`decide_navigation(url)` is the only authority for what the Human Browser may load:

- **Allow:** `http` and `https` without embedded credentials.
- **Open external:** `mailto:`, `tel:`, `sms:`, and `magnet:` — surfaced as an explicit user action, never loaded in the view.
- **Deny:** every other scheme (`file:`, `javascript:`, `data:`, `blob:`, `about:`, `view-source:`, `chrome:`, `ftp:`, …) with the code `human_scheme_denied`; URLs with userinfo with `human_userinfo_denied`; unparsable input with `human_url_invalid`.
- Popups and `target="_blank"` are denied and recorded as `human_popup_denied`; downloads are denied and recorded as `human_download_denied`.

The negative corpus lives in `src-tauri/tests/human-navigation-corpus.json` and every entry is asserted by the unit test.

## View hosting

The Human Browser view is a `wry` WebView built into the shared Canvas `GtkFixed` overlay (extracted to `features/canvas_host.rs` so Preview and Human Browser share one host), outside the Tauri webview manager, so it has no Tauri IPC by construction. It uses the persistent `human-profile` under the application data directory, records its title, URL, and history state for the shell, and keeps at most one view.

## Harness result

`BRAINROOT_HUMAN_FIXTURE=1` (debug builds only) serves a local two-page fixture, then drives the production module: show `/a`, navigate to `/b`, back, forward, reload, attempt a `file://` navigation, and hide.

```json
{"decision":"go","page_a_reached":true,"page_b_reached":true,"back_ok":true,"forward_ok":true,
 "denial_code":"human_scheme_denied","final_url":"http://127.0.0.1:<fixture>/b","hidden":true,"reasons":[]}
```

All `MEASURED` on the frozen Pop!_OS 24.04 reference environment (Wayland, WebKitGTK 2.52.6); the app exited `0` and left no process or listener behind. History navigation uses `webkit2gtk` `can_go_back`/`can_go_forward`/`go_back`/`go_forward`, the only source for that state that wry does not surface.

## Canvas surface (B14-S02)

The Browser tab in the Canvas provides the address entry, Back/Forward/Reload controls with truthful `aria-disabled` states, the page title and current address, and a `role="alert"` line for blocked links. Geometry is synchronized from the slot element (`ResizeObserver` plus `requestAnimationFrame`), the view is shown on the first Go and re-bounded while live, and switching Canvas tabs unmounts the surface, which hides its view so one content slot stays visible. Link-driven changes arrive through the typed `human-browser-status` event; the shell does not poll. The address entered by the user is normalized (a bare host gains `https://`) and the engine policy remains the authority for what loads.

## Open for the next microsteps

- Deck switching, permission prompts, downloads, deletion/reset, cookie/cache isolation, crash recovery, and accessibility inside remote content remain open; the policy is not changed by this record.
