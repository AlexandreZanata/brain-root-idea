# Feature: <user-facing capability>

**Status:** Discovery | Proposed | Accepted | Building | Released  
**Roadmap phase:** <phase>  
**Decision owner:** <owner>

## Problem and user

Who has the problem, what are they trying to do, and why does the current experience fail?

## Desired experience

Write the primary journey in user language, including first use, progress, success, failure, recovery, and optional technical details.

## Non-goals

List tempting adjacent capabilities that are deliberately excluded.

## Functional requirements

Use stable requirement IDs (`FR-1`, `FR-2`). Keep implementation out unless it is a constraint.

## UX and accessibility

Default versus Developer Mode, terminology, focus/keyboard, screen reader, contrast, reduced motion, empty/loading/error states.

## Architecture and contracts

Owning modules; typed commands/events; persisted schemas; adapters; platform differences; migration strategy. Link ADRs rather than duplicating decisions.

## Resource lifecycle

For each process, PTY, watcher, LSP, WebView, browser, listener, and artifact specify owner, start trigger, readiness, idle behavior, cancellation, termination, and crash cleanup.

## Security and privacy

Trust boundaries, permission classes, path/origin/network scope, secrets, prompt injection, logs/artifacts, Safe/Power Mode differences, and platform enforcement.

## Performance budget

Metrics with TARGET/MEASURED/UNKNOWN labels, fixtures, measurement method, and regression threshold.

## Test strategy

Unit, contract, integration, E2E, accessibility, security, platform, performance, and cleanup coverage.

## Rollout and reversibility

Feature flag or migration if needed, compatibility, failure rollback, data deletion/export, and how to remove the feature cleanly.

## Acceptance criteria

Concrete, observable, and testable checks including Definition of Done.

## Open questions

Classify BLOCKER, IMPORTANT, or LATER; name an owner and resolution evidence.

