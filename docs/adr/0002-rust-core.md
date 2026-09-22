# ADR 0002: Rust owns privileged core state

**Status:** Accepted  
**Date:** 2026-09-22

## Context

BrainRoot coordinates untrusted agent requests with files, processes, WebViews, persistence, permissions, and cleanup. Splitting authority across frontend state and ad-hoc helper processes would make enforcement and lifecycle difficult to reason about.

## Decision

Rust owns workspace scope, permission decisions, Agent Host lifecycle, process/resource handles, checkpoint coordination, persistence access, and the canonical task/resource state when appropriate. The frontend renders projections and submits typed intent. Long work runs asynchronously and is cancellable; the UI event loop is never blocked.

## Alternatives considered

- **TypeScript/Node backend:** larger shared-language surface, rejected because it would add a persistent runtime and weaken the single native authority model.
- **Frontend-owned state with Tauri commands:** rejected because privileged lifecycle and policy could diverge from UI state.
- **Several local services:** rejected as premature operational complexity.

## Consequences

Core APIs require explicit typed schemas and Rust expertise. UI prototypes may take slightly more boundary design, but tests can exercise policy without a WebView. Logical modules stay in a small number of crates until separation is justified.

## Performance implications

Avoids a mandatory Node sidecar and enables efficient process/lifecycle control. Rust can still be slow through poor design; allocations, locks, async runtime, and binary size remain measured concerns.

## Security implications

Creates one enforceable privileged boundary. It also concentrates risk, so the core must be small, deny by default, validate frontend/provider input, and avoid unsafe code without an ADR.

## Reversibility

Low for the privileged core, deliberately. Individual services can later move out of process behind the same contracts if isolation evidence justifies it.

## References

[Tauri process model](https://v2.tauri.app/concept/process-model/), [Tauri Runtime Authority](https://v2.tauri.app/security/runtime-authority/).

