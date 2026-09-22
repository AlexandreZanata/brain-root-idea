# ADR 0001: Tauri 2 as the desktop runtime

**Status:** Accepted for prototype  
**Date:** 2026-09-22

## Context

BrainRoot needs a cross-platform desktop shell, Rust integration, native process/filesystem access, multiple controlled visual surfaces, and low idle overhead. Bundling a Chromium runtime conflicts with the initial resource direction, but using system WebViews introduces platform variation.

## Decision

Prototype the MVP shell with Tauri 2, TAO, and WRY. Use the system WebView for the BrainRoot UI and keep privileged logic in Rust. Pin exact versions only during bootstrap after checking stable releases, OS requirements, MSRV, advisories, and compatibility. Phase 1 benchmarks and the browser spike are explicit validation gates.

## Alternatives considered

- **Electron:** mature tooling and consistent Chromium behavior, rejected as the default because bundled runtime and multi-process baseline are poorly aligned with the resource hypothesis.
- **Native UI per platform:** strongest native integration, rejected because three implementations would slow a small AI-developed project and fragment UX.
- **Flutter:** coherent rendering and desktop support, rejected because it adds another engine/runtime and weakens the direct Rust/web stack fit.
- **Wails or raw WRY:** plausible; Tauri currently provides the more complete capability, packaging, IPC, and plugin foundation. Raw WRY remains a fallback if Tauri overhead or constraints fail measurements.

## Consequences

The app depends on WebView2 (Windows), WKWebView (macOS), and WebKitGTK (Linux), so rendering and lifecycle differ. Tauri reduces bundled runtime size but does not itself guarantee low memory or startup. Cross-platform tests and capability differences are product requirements.

## Performance implications

Potentially smaller distribution and reuse of OS WebViews. Real process-tree CPU/RSS, startup, multi-WebView behavior, and memory release are UNKNOWN until measured. Failure of budgets can supersede this ADR.

## Security implications

Tauri capabilities narrow UI/core IPC; they are not a sandbox for preview pages or child commands. Arbitrary remote content must never receive shell capabilities. Core commands still validate every request.

## Reversibility

Moderate. Keeping product state, Agent Adapter, Tool Layer, and typed events independent of Tauri-specific frontend calls reduces migration cost. Validate before broad UI investment.

## References

[Tauri architecture](https://v2.tauri.app/concept/architecture/), [process model](https://v2.tauri.app/concept/process-model/), [Runtime Authority](https://v2.tauri.app/security/runtime-authority/), [WRY](https://github.com/tauri-apps/wry).

