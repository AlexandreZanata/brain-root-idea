# ADR 0006: Isolated browser roles and one HOT heavy WebView

**Status:** Accepted for prototype  
**Date:** 2026-09-22

## Context

The Canvas needs localhost preview and later web destinations, but heavy pages may exceed the rest of the app's resource use. Tauri/WRY use different OS engines and do not expose a uniform, reliable tab hibernation API.

## Decision

Separate shell, preview, Human Browser, and Agent Browser contexts. The MVP has one on-demand localhost preview WebView and one on-demand Playwright automation context. Later Deck behavior allows one HOT heavy content WebView by default. Portable inactivity means capture a permitted placeholder/state, destroy, and reconstruct. Platform-specific WARM behavior is optional after measurement.

## Alternatives considered

- **Several always-live WebViews:** smooth switching, rejected for uncontrolled RAM/CPU and attack surface.
- **Iframes inside the shell:** rejected due embedding restrictions, isolation, focus, and privileged-origin risk.
- **Bundled Chromium tabs:** more consistent lifecycle, rejected for current resource and distribution goals.
- **Assume native suspend everywhere:** contradicted by current Tauri/WRY platform support.

## Consequences

Some pages lose in-memory state when switched away. Snapshots may become stale and authenticated restoration varies. UX must communicate reload without pretending perfect suspension.

## Performance implications

Caps default live heavy views and should return memory after destruction, subject to engine behavior. Measure creation, destruction, switching, memory release, and leaks per OS.

## Security implications

Remote/preview views receive no shell capabilities. Human and agent profiles/cookies remain separate. Navigation, downloads, new windows, permissions, and agent inspection are policy controlled.

## Reversibility

Moderate. Browser Manager contracts permit a bundled engine or platform-specific implementation later without changing task/Canvas concepts.

## References

[Tauri process model](https://v2.tauri.app/concept/process-model/), [Tauri Webview API background-throttling notes](https://github.com/tauri-apps/tauri/blob/dev/packages/api/src/webview.ts), [WRY](https://github.com/tauri-apps/wry).

