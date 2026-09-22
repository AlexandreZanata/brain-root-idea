# ADR 0008: Event-driven zero-idle resource ownership

**Status:** Accepted  
**Date:** 2026-09-22

## Context

Agents, LSPs, terminals, dev servers, watchers, indexers, WebViews, automation, and local models can make an “idle” IDE expensive. Retrofitting lifecycle control after every feature owns hidden resources is unreliable.

## Decision

Every expensive resource is created on demand, registered with one owner, emits typed lifecycle events, has ACTIVE/IDLE/SUSPENDED/TERMINATED semantics appropriate to its kind, and has deterministic cancellation/cleanup. Boot activates only core and UI. The Resource Governor coordinates policy from events and deadlines; it does not poll each resource.

## Alternatives considered

- **Always-on tooling for instant response:** rejected as contrary to the core product promise.
- **Ad-hoc cleanup per feature:** rejected because ownership and leaks become invisible.
- **Aggressive immediate termination:** rejected because measured short reuse may justify bounded IDLE.
- **Periodic polling supervisor:** retained only as a low-frequency orphan watchdog, not normal state management.

## Consequences

All features must specify ownership, startup, readiness, cancellation, idle policy, reconstruction, and cleanup. Some actions may have cold-start latency; UX shows honest starting states. Platform resources may not support SUSPENDED and go directly to TERMINATED.

## Performance implications

This decision establishes the budgets of zero inactive agents/LSPs/PTYs and one HOT heavy WebView. It adds small registry/event overhead that must remain bounded. Policies are tuned using measurements, not intuition.

## Security implications

Short-lived, scoped processes reduce exposure. The Governor is not a security sandbox; process permissions are independently enforced. Cleanup must terminate process trees, not only parent handles.

## Reversibility

The invariant is intentionally difficult to reverse. Individual idle durations and lifecycle implementations are easily changeable behind manager contracts.

## References

[Performance contract](../10-performance-budget.md), [Resource Governor](../09-resource-governor.md).

