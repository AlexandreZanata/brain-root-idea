# ADR 0016: Ship the uncontained agent as an experiment

**Status:** Accepted for experiment

**Date:** 2026-09-25

## Context

Batch B19 and the maintainer's Lean YAGNI pivot add an on-demand coding agent to the experimental line: the Rust Agent Host owns `opencode serve --pure` as a child process, the Canvas drives it through session tabs, and the Resource Governor stops it after 60 s idle. The `0.0.15` release gate verified the frontend system, the suite, the soak, and the absolute measurements.

That same gate recorded a blocker. The agent has **no approved workspace root, no per-action permission decision, no tool-layer mediation, and no containment boundary**. Once started it inherits the user's own filesystem and process authority, so the deny-by-default policy, permission record, and revocable grants described in `docs/11-security-and-permissions.md` do not apply to this path. The only mitigation actually in place is that the sidecar starts solely on an explicit user action. The blocker is tracked in `docs/17-open-questions.md`, and [ADR 0015](draft-0015-opencode-serve-default.md) is still **Proposed**, so the sidecar's status as default architecture is also undecided.

The maintainer had three defensible options: hold the batch until a boundary exists, scope the agent out of `0.0.15` and ship only the frontend work, or merge with the gap open and declared. This record captures the choice made and the conditions attached to it.

## Decision

Merge the batch and tag `0.0.15` with the gap **open and declared**, under these conditions:

- Every surface that describes the agent states that it is experimental and uncontained: the README, the changelog, `docs/11-security-and-permissions.md`, the Wiki batch page, and the release page.
- No document, artifact, UI string, or release note claims Safe Mode, sandboxing, or a permission boundary for agent actions.
- The sidecar keeps its explicit-user-action start and its bounded lifetime: 60 s idle auto-stop, window close, and explicit Stop, with no orphan process.
- The blocker stays open in `docs/17-open-questions.md` with an owner until a canonicalized approved root, a permission moment for writes and commands, denial tests, and either an OS/container boundary or a truthful written statement of the enforcement floor exist.
- [ADR 0015](draft-0015-opencode-serve-default.md) stays **Proposed**. This record does not accept the sidecar as the product's default architecture; it only accepts shipping the implementation while that decision is unresolved.
- `v0.0.15` is an annotated tag on the batch merge commit on `main`, created only after the required latest-head CI is green, per `docs/19-release-and-versioning.md`. The tag is never placed on a branch head.

## Alternatives considered

- **Hold the batch until the permission boundary exists:** the strongest security posture and the option that keeps the documented model true. Rejected because it blocks a large, already-gated frontend delivery on separately schedulable work, and because the agent cannot start without a deliberate user action.
- **Scope the agent out of `0.0.15` and ship only tokens, shell, and edge:** keeps every advertised capability honest at the cost of discarding a working implementation and its measurements. Rejected by the maintainer, but it remains the recommended path if the gap is still open at a future release where the agent is described as anything more than experimental.
- **Merge without recording the gap:** rejected outright. It would make the security documentation false, which is worse than the gap itself.
- **Start the agent at boot or make it always resident:** rejected. It would remove the single existing mitigation and contradict the event-driven zero-idle architecture in [ADR 0008](0008-zero-idle-architecture.md).

## Consequences

An uncontained agent is now part of the `0.0.x` experimental line, and every release page published while this stays true must repeat that it is uncontained. The security documentation's target model and the shipped behavior now differ on one path, which is acceptable only while the difference is stated. A prompt injection in fetched content, a malicious repository, or an agent mistake can read or modify anything the user's account can. Closing the blocker requires a new ADR; superseding this one is how that happens, and this record is not edited to hide the change.

## Performance implications

MEASURED at this gate: the sidecar costs roughly 300–480 MB RSS while alive, measured separately from the ≤ 150 MB UI idle budget, and is stopped on 60 s idle, window close, or explicit Stop. The uncontained posture itself changes neither CPU nor memory. Any future containment layer — a container, a sandbox helper, or a mediated tool process — adds startup, memory, and latency cost that must be measured and reported against the existing budgets before it is adopted, not assumed to be free.

## Security implications

This decision knowingly accepts a security gap; it does not resolve one. There is no enforcement floor for agent actions today, so the honest statement is that the agent runs with the user's authority and no BrainRoot boundary constrains it. The mitigations that do hold and are release-gated: the basic-auth password is ephemeral, memory-only, redacted in `Debug`, and absent from every IPC payload by construction; the provider payload is reduced to identifiers, with a test asserting no key, `apiKey`, or `Authorization` material is serialized; the outbound allowlist permits only the loopback sidecar origin plus the public OpenRouter catalog; and the sidecar is never started implicitly. Explicit start is a speed bump, not a boundary, and disclosure is not a mitigation — it only prevents the repository from claiming a safety property it does not have.

## Reversibility

High. Disabling the agent start path, or gating it behind Developer Mode, is a small change in the Rust Agent Host and one frontend entry point, with no migration or persisted state to unwind. A published tag is immutable: if this decision is reversed after tagging, the correction is a superseding ADR plus a new version, never a moved tag.

## References

- [Open questions: agent workspace and permission boundary](../17-open-questions.md)
- [Security and permissions](../11-security-and-permissions.md)
- [ADR 0015 (draft): opencode serve --pure as default sidecar](draft-0015-opencode-serve-default.md)
- [ADR 0008: event-driven zero-idle resource ownership](0008-zero-idle-architecture.md)
- [ADR 0014: automated tests run at the versioned release gate](0014-release-only-test-cadence.md)
- [Release and versioning](../19-release-and-versioning.md)
- [B19 batch record](../history/batches/B19-foundation.md)
