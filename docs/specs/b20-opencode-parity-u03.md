# B20-U3 — Session timeline parity: stops on a contract gap

- **Status:** **STOPPED before implementation**, per this stage's own stop condition. Specification and event-coverage inventory delivered.
- **Issue:** [#132](https://github.com/AlexandreZanata/brain-root-idea/issues/132)
- **Pin:** `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769`, read-only.
- **Sources read:** `pages/session/timeline/rows.ts` (10,997 B), `timeline/model.ts` (4,157 B), `timeline/projection.ts` (2,848 B), `pages/session/message-gesture.ts` (654 B). The two large rendering files (`pages/session.tsx` 80,351 B, `timeline/message-timeline.tsx` 76,833 B) were not read: the row model decides what can be rendered, and the row model is fully specified in `rows.ts`.

## 1. Why this stage stops

The issue's stop conditions include:

> The timeline requires an IPC or provider payload change — stop; that is a separate issue and decision.

It does. Measured, not inferred.

## 2. What the pin's timeline actually is

The pin does not render a turn as "prompt plus answer". It projects a **row list** from a parts-based data model. `TimelineRowMap` (`rows.ts:11-33`) defines exactly nine row kinds:

| Row kind | Payload it needs |
|---|---|
| `UserMessage` | the user message + an anchor flag |
| `TurnGap` | spacing between turns; nothing else |
| `Error` | `{ text }` from the assistant message |
| `AssistantPart` | a grouped `PartGroup` of `Part[]` — text, tool, and file parts, filtered by `renderable(part, showReasoning)` |
| `Thinking` | reasoning parts, shown while `status.type === "busy"` |
| `DiffSummary` | `diffs: SummaryDiff[]` from file parts |
| `TurnDivider` | `label: "compaction" \| "interrupted"` — a `compaction` part |
| `CommentStrip` | comment metadata parts on the user message |
| `Retry` | an assistant message that can be retried |

Every one of those except `UserMessage`, `TurnGap`, and `Error` is derived from **parts** (`rows.ts:112-145`, `186-194`), and the projection's own signature takes `getMessageParts: (messageID) => Part[]` as a required input (`projection.ts:11-22`). The session index comes from the server as `SessionMessageInfo[]`, and `status.type === "busy"` plus `MessageAbortedError` drive the thinking and interrupted states.

### The "message gestures" correction

My issue described gestures as message-level actions (retry, fork). `message-gesture.ts` is something else: a **scroll boundary** helper. `normalizeWheelDelta` converts `deltaMode` 1 (lines → ×40) and 2 (pages → × root height), and `shouldMarkBoundaryGesture` decides whether a wheel gesture is sitting on the top or bottom boundary of the scroller. It is about scroll physics, not about acting on a message. Retry and fork live in the row model, not in a gesture module.

## 3. Event-coverage inventory (the stage's step 2)

Our contracts carry five event types and nothing else:

- `src/conversation.ts:14-31` — `ConversationEvent`: `started`, `text_chunk`, `completed`, `cancelled`, `failed`.
- `src/agentHost.ts:15-20` — `AgentStreamEvent`: the same five, plus a session id.
- `src-tauri/src/features/agent_host/mod.rs:104-121` — `AgentStreamEvent` in Rust, serialized as the versioned envelope.

Against the nine row kinds:

| Row kind | Can we populate it today? | Why |
|---|---|---|
| `UserMessage` | **yes** | we hold the prompt |
| `TurnGap` | **yes** | layout only |
| `Error` | **yes** | `failed` carries a normalized error |
| `AssistantPart` | no | needs parts; we hold one concatenated `text` per turn |
| `Thinking` | no | needs reasoning parts |
| `DiffSummary` | no | needs file/diff parts |
| `TurnDivider` | no | needs a `compaction` part or an `MessageAbortedError` signal |
| `CommentStrip` | no | needs comment metadata parts |
| `Retry` | no | needs a message identity the backend can replay |

**Three of nine.**

## 4. The data exists upstream and we discard it on purpose

This is the decisive finding, and it is proven by our own passing test suite rather than by reading OpenCode.

`src-tauri/src/features/agent_host/mod.rs:484-538` classifies each sidecar SSE frame. It recognises exactly three upstream event names:

- `message.part.delta` — accepted **only** when `field` is absent or `"text"`; every other field returns `Ignored`.
- `session.idle` → done.
- `session.error` → failed.
- everything else → `_ => FrameOutcome::Ignored`.

And the test `frames_ignore_foreign_sessions_and_non_text` (`mod.rs:1510-1527`) asserts, currently passing:

```rust
let reasoning = serde_json::json!({
    "id": "evt_4", "type": "message.part.delta",
    "data": {"sessionID": "ses_abc", "messageID": "msg_1",
             "partID": "prt_2", "field": "reasoning", "delta": "hmm"}
});
assert_eq!(frame(reasoning), FrameOutcome::Ignored);
```

A reasoning delta arrives from the sidecar with its `partID` and `messageID`, and we drop it by design. The same `message.part.delta` channel is where tool and file parts flow. So this is not "the backend does not provide the data" — it is a deliberate filter at our boundary that could be lifted.

The relay already handles both sidecar generations (`data` and `properties` envelopes), so the plumbing to widen is small; the widening is the problem, not the transport.

## 5. Why this is a separate decision and not a U3 task

Delivering the six missing row kinds requires all of the following, in order:

1. New `FrameOutcome` variants and `classify_frame` arms for non-text parts, with the same bounding and session filtering the text path has.
2. New `AgentStreamEvent` variants — that is **a change to the versioned IPC contract**, which is currently `AGENT_CONTRACT_VERSION = 1` (`mod.rs:32`), mirrored in `src/agentHost.ts:13` and enforced by a strict equality check at `src/agentHost.ts:43`. A payload change means a version bump and a compatibility story.
3. New frontend event types and validation, with bounded sizes for every new field (tool output and diffs are far larger than a text delta; `MAX_DELTA_CHARS` is 64 KiB for text alone).
4. Only then the row model and the rendering this stage was written for.

Steps 2 and 3 are exactly what the issue's Forbidden scope calls out ("No change to the provider contract, IPC payloads… **If a payload must change, stop — that is a separate decision**"), and step 2 is literally the stop condition. The frontend cannot work around it: reading the sidecar's stream directly would make the frontend a second privileged backend (architecture invariant) and the sidecar's password deliberately never crosses IPC.

**Deliberately not done:** building the nine-row UI against a stream that can only fill three rows. That would have produced either an empty shell or fabricated tool output, diffs, and reasoning — both forbidden by the issue ("no fabricated tool output, diff, cost, or reasoning content") and by the repository's UX invariant against faking capability.

## 6. Named follow-up

The contract extension is filed as its own issue against this batch, because it is a payload and versioning decision rather than a UI stage: it names the new event variants, the bounding rules, and the `AGENT_CONTRACT_VERSION` bump, and it must land before U3 can be re-opened.

- **Follow-up:** [#138](https://github.com/AlexandreZanata/brain-root-idea/issues/138) — *[B20-U3a] Extend the agent event contract with non-text parts (payload + versioning decision)*, labels `type:rust,type:frontend,risk:high,agent:economical,batch:B20`.
- **Deferred case IDs:** `B20-U3a-T01`…`T05` are recorded on that issue.
- Its first task is capturing the **real** part shapes from the pin or a live capture — no guessing the envelope, which is how the current filter came to drop `field: "reasoning"` in the first place.

Until then, U3 stays stopped. The `0.0.16` version, the batch budget, and the surfaces owned by U1 and U2 are unaffected: no file outside this spec changed.

## 7. What is verified today

Nothing about the timeline is implemented, so nothing is claimed. The three populatable rows (`UserMessage`, `TurnGap`, `Error`) already render in `Turn.svelte` from the B19 work and were not touched.

## 8. Verification plan when U3 is re-opened

Unchanged from the issue, and still to be honoured: the five deferred case IDs `B20-U3-T01`…`T05`, svelte-check 0/0, the three gates, the allowlist diff, and the `docs/22` §3 bundle budget, with the honest-placeholder inventory re-derived against whatever the extended contract actually carries.
