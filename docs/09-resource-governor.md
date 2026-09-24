# Resource Governor

## Purpose

The Resource Governor is the central registry and policy engine for expensive runtime resources. It knows what exists, who owns it, why it is active, and how it stops. It coordinates managers; it does not directly implement every resource.

Tracked kinds include agents, PTYs, commands, dev servers, LSPs, file watchers, indexers, automation browsers, content WebViews, and future local models.

## Resource record

Each record contains `resource_id`, kind, owning project/task, lifecycle state, reason, start/last-use timestamps, dependencies, estimated or measured cost, idle policy, health, cleanup handle, and platform metadata. A resource without an owner is a bug.

## State machine

```text
TERMINATED ─ request → ACTIVE ─ inactivity → IDLE
     ↑                    ↑                   │
     └──── stop/error ────┴─ resume ← SUSPENDED
```

- **ACTIVE:** currently producing user value.
- **IDLE:** alive briefly because near-term reuse is likely; no active work.
- **SUSPENDED:** platform-specific low-resource state with verified restore semantics.
- **TERMINATED:** no live process, PTY, watcher, or WebView; reconstruct from metadata.

Not every resource supports SUSPENDED. In that case the policy moves from IDLE to TERMINATED.

## Default boot state

Only BrainRoot Core and its primary UI are active. Agent, terminal, dev server, LSP, indexer, browser automation, and extra content WebViews are absent.

## Policy examples

- Agent starts for a user task; closes after session end or explicit retention policy.
- A TypeScript LSP starts only for a capability requiring it; idle timeout is evidence-based and termination-safe.
- Dev server starts for preview/test; remains while the preview is actively used, then stops or asks if stopping would be surprising. The preview-scoped ownership, port registration, termination, readiness, and idle rules are fixed by [ADR 0011](adr/0011-dev-server-lifecycle.md).
- Playwright starts for QA and terminates after artifacts are collected.
- Only one heavy Deck WebView is HOT by default; inactive cards become COLD/DEAD on the portable path.

Timeout values are configuration backed by measurements, not embedded product assumptions.

## Event-driven operation

Managers emit creation, activity, health, exit, and cleanup events. The Governor schedules the nearest lifecycle deadline with one timer and reacts to ownership and UI visibility changes. It must not poll each resource. OS process-exit notifications and WebView callbacks are preferred.

## Safety and cleanup

Project close and application exit walk the ownership graph leaf-first: cancel work, close protocol sessions, terminate process groups, close PTYs, destroy WebViews, remove listeners/watchers, flush bounded state, then mark resources terminated. A watchdog may detect orphans but is not the normal control path.

## User experience

Default Mode shows plain-language status only when useful. Developer Mode may display resource kind, state, owner, age, memory estimate, and Stop/Restart controls. The Governor never kills active user work solely to hit a cosmetic number without warning.

## Measurement

Track transition latency, failed cleanup, orphan count, time idle before reuse, memory before/after termination, and unnecessary restarts locally. No remote telemetry is implied.

