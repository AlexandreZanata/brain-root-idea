# ADR 0017: Agent event contract version 2 with non-text parts

**Status:** Accepted

**Date:** 2026-10-02

## Context

The Agent Host publishes one versioned product event envelope, `agent_event`, currently `AGENT_CONTRACT_VERSION = 1` in `src-tauri/src/features/agent_host/mod.rs`, mirrored in `src/agentHost.ts` and enforced by strict equality on both sides. Version 1 carries exactly five variants — `started`, `text_chunk`, `completed`, `failed`, `cancelled` — and `classify_frame` deliberately discards every sidecar frame that is not a text delta, a session idle, or a session error.

Batch B20 stopped the session timeline (U3) on that boundary: of the pinned OpenCode timeline's nine row kinds, only three (`UserMessage`, `TurnGap`, `Error`) can be populated from a stream that carries one concatenated string per turn. The measured inventory and the passing test that proves we discard a `field: "reasoning"` frame are recorded in `docs/specs/b20-opencode-parity-u03.md`.

The follow-up issue ([#138](https://github.com/AlexandreZanata/brain-root-idea/issues/138)) demanded two things before any code: the **real** part shapes, captured rather than guessed, and a recorded versioning decision. Both were produced in `docs/specs/b21-agent-event-contract.md`: the pin's `Part`/`ToolState`/`session.diff` schemas plus the installed `opencode 1.18.33` binary's embedded declarations. That capture also showed a newer `session.next.*` durable event family whose emit sites could not be proven from the binary — so it is explicitly *not* the basis for this decision.

## Decision

Extend the product contract to **version 2**, additively:

1. **Four new variants** — `reasoning_delta`, `tool_event` (phases `called`/`progress`/`success`/`failed`), `file_event`, and `turn_divider` — carrying only named, bounded fields. No arbitrary `metadata`/`structured` maps from the untrusted sidecar are forwarded.
2. **A version bump, not a widening.** `AGENT_CONTRACT_VERSION` becomes `2` on both sides, and both sides keep strict equality. A version-1 frontend paired with a version-2 core fails loudly at the envelope instead of silently ignoring fields it cannot render; the same constant travels on `AgentSendAccepted`, so the disagreement is visible before a stream starts.
3. **Bounds stated per field, and truncation rather than rejection.** Every new field has a documented maximum (8 KiB for tool input/output/error, 64 KiB for a reasoning delta, 2 KiB for a path, 200 chars for names), and per-turn ceilings (256 KiB of reasoning, 512 tool/file events) end the turn with `ResponseTooLarge` exactly as the existing 2 MiB text ceiling does. A rejected frame would silently lose a tool's terminal state; truncation is visible.
4. **The session filter stays ahead of classification** for every variant, unchanged.
5. **The legacy `conversation.ts` contract does not move.** It stays at `CONVERSATION_CONTRACT_VERSION = 1` and remains the no-sidecar fallback. The extended events are agent-host-only.

## Alternatives considered

- **Widen version 1 in place (add fields, keep the number).** Rejected. A version number that does not change while the payload does cannot be validated by either side, which is the one property the versioned envelope exists to provide.
- **A second event name (for example `agent_part_event`) next to `agent_event`.** Rejected: it splits the session's stream across two channels with two lifecycles, and the frontend would have to correlate them. One versioned envelope with a version bump is the smaller, more testable change.
- **Build on the `session.next.*` durable family.** Rejected for this batch: the family exists in the installed binary, but its emit sites could not be proven from that binary, and the pinned contract does not declare it. Choosing it would be guessing, which is what this issue's Definition of Ready forbids. Recorded as a follow-up for a future batch that can capture a live frame.
- **Forward the sidecar's raw `part` JSON with a size cap only.** Rejected: it would put an unbounded, untyped, untrusted structure into the product contract, and every consumer would need its own defensive parsing. The named-field shape keeps the contract reviewable and keeps provider-specific structure on the far side of the Agent Adapter boundary (ADR 0005).
- **Do nothing and let the timeline render placeholders forever.** Rejected: three of nine rows is a shell, not the pinned capability, and the repository's UX invariant forbids shipping a surface that pretends.

## Consequences

- The session timeline (U3, [#132](https://github.com/AlexandreZanata/brain-root-idea/issues/132)) can be re-opened against real data: reasoning, tool cards with identity and state, and file/diff rows now cross the boundary.
- The contract has a second version to maintain. Because both sides move together inside one application, there is no long-lived dual-version support burden; the cost is the discipline of bumping the constant whenever the payload changes.
- Rows still unavailable — `Retry` (needs a replayable message identity), `CommentStrip` (editor comment metadata), and the `session.next.*`-only signals — remain named non-goals rather than implied capabilities.
- `docs/07-agent-architecture.md` records the contract as version 2 with its bounds; `docs/specs/b20-opencode-parity-u03.md` points its stop note at the landed contract.

## Performance implications

No new thread, task, timer, poll loop, or dependency: the work happens inside the existing relay on the already-open SSE stream, and the classifier is a pure function over an already-parsed frame. Bounds reduce, not increase, the worst-case memory a turn can hold. The measured evidence is the versioned gate's bundle-budget line (CSS ≤ 35 KB gzip, JS ≤ 60 KB gzip) plus the existing 1,000-conversation soak, which still applies because the stream path is the one it exercises.

## Security implications

The sidecar is untrusted input. This change widens what it can put into the UI process, so the mitigations are explicit: every field is bounded and truncated; no nested free-form maps are forwarded; the session filter still drops foreign sessions before classification; the secret-scan gate and the "no credential on this path" rule are unchanged; and the new payloads carry no authorization material by construction. Tool output and file paths can contain project content, which is the same exposure the text path already has — the difference is that it is now labelled as tool output rather than silently concatenated into the answer. The agent's missing workspace/permission boundary (ADR 0016, `docs/17-open-questions.md`) is untouched by this decision and stays the open blocker.

## Reversibility

High. Reverting the single commit restores version 1 and the previous filter; nothing is persisted, no migration exists, and no external consumer depends on version 2. The decision itself can be superseded by a later ADR without rewriting this record.

## References

- Issue [#138](https://github.com/AlexandreZanata/brain-root-idea/issues/138) · Batch B21 · blocked stage [#132](https://github.com/AlexandreZanata/brain-root-idea/issues/132)
- Spec: [`docs/specs/b21-agent-event-contract.md`](../specs/b21-agent-event-contract.md)
- Stop record: [`docs/specs/b20-opencode-parity-u03.md`](../specs/b20-opencode-parity-u03.md)
- Pin: `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769` — `packages/schema/src/session-message.ts`, `packages/schema/src/session-event.ts`, `packages/app/src/context/global-sync/event-reducer.ts`
- Sidecar: `opencode 1.18.33` (installed binary, read-only schema extraction)
- ADR 0005 (Agent Adapter boundary), ADR 0014 (release-only test cadence), ADR 0016 (uncontained agent experiment)
