# B21-S01 — Agent event contract version 2 (non-text parts)

**Status:** In progress — spec written before code, per `docs/18` §4.2.
**Issue:** [#138](https://github.com/AlexandreZanata/brain-root-idea/issues/138) · **Batch:** B21 · **ADR:** [0017](../adr/0017-agent-event-contract-v2.md)
**Scope:** the boundary only. No UI, no rendering, no new dependency.

## 1. Why this exists

B20-U3 stopped because the session timeline cannot be built against a stream that carries one concatenated `text` string per turn: only **3 of the pin's 9** timeline row kinds are populatable (`docs/specs/b20-opencode-parity-u03.md` §3). The data is not missing upstream — our classifier drops it by design (`classify_frame` accepts `message.part.delta` only when `field` is absent or `"text"`).

## 2. Captured part shapes — the evidence this issue demanded

No payload below is inferred from a field name. Each was read from one of two sources:

- **Pin source:** `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769` — `packages/schema/src/session-message.ts` (`Part` union, `ToolState`, `AssistantTool`), `packages/schema/src/session-event.ts` (the `session.next.*` durable event family), `packages/app/src/context/global-sync/event-reducer.ts` (which events the app actually consumes, and that `message.part.delta` appends `properties.delta` onto `part[properties.field]`).
- **Installed sidecar:** `opencode 1.18.33` (`~/.opencode/bin/opencode`, the binary the Agent Host spawns). Its embedded schema declares the same v1 union and additionally the `session.next.*` family; every `session.next.*` name measured **3 occurrences** in the binary, which is the definition-side count, not an emit-site count. That ambiguity is exactly why the v1 family — whose emit sites are visible (`message.part.updated` ×27, `session.diff`, `session.status`) — is the authority here.

### 2.1 The part union (v1 tree, `session-message.ts`)

```text
Part = text | reasoning | tool | file | agent | snapshot | patch | subtask
     | retry | step-start | step-finish | compaction
```

| Part | Fields the timeline needs | Bound in this contract |
|---|---|---|
| `text` | `text` (streamed via `message.part.delta`, `field: "text"`) | already bounded: 64 KiB per delta, 2 MiB per turn |
| `reasoning` | `text` (streamed via `message.part.delta`, `field: "text"` with a reasoning `partID`) | 64 KiB per delta, 256 KiB per turn, 8 KiB per part in the live state |
| `tool` | `id`, `tool` (name), `callID`, `state.status`, `state.input`, `state.title`/`state.output`/`state.error` | 8 KiB per tool input, 8 KiB per tool output/error |
| `file` | `mime`, `filename`, `url` | 2 KiB per field |
| `patch` | `hash`, `files[]` | 2 KiB per path, 64 paths |
| `session.diff` | `{file, additions, deletions, status}` per entry | 64 entries, 2 KiB per path |

Tool state variants measured in the binary (`ToolState`, discriminated by `status`):

```text
pending   { input, raw }
running   { input, title?, metadata?, time.start }
completed { input, output, title, metadata, time{start,end,compacted?}, attachments? }
error     { input, error, metadata?, time{start,end} }
```

`AssistantTool` carries `type: "tool"`, `id`, `name`, `provider.executed`, `state`, `time`. `message.part.updated` carries the **whole part** (`properties.part`); `message.part.delta` carries only `{sessionID, messageID, partID, field, delta}` and requires the part to already exist (the app reducer drops a delta whose part it has not seen).

## 3. Contract v2 — what changes

`AGENT_CONTRACT_VERSION` moves **1 → 2**. Version 1 is not silently widened; see [ADR 0017](../adr/0017-agent-event-contract-v2.md) for the compatibility decision.

New variants (Rust `AgentStreamEvent` / TS `AgentStreamEvent`), all tagged `snake_case`, all carrying `session`:

| Variant | Fields | Source frame |
|---|---|---|
| `reasoning_delta` | `text` | `message.part.delta` with a reasoning part (`field` absent or `"text"`; the part type decides) |
| `tool_event` | `phase` (`called` \| `progress` \| `success` \| `failed`), `tool_id`, `name`, `state`, `input`, `output`, `error`, `title` | `message.part.updated` with `part.type == "tool"` |
| `file_event` | `path`, `mime`, `status` (`added` \| `modified` \| `deleted` \| `unknown`) | `session.diff` entries and `patch` parts |
| `turn_divider` | `reason` (`compaction`) | `compaction` part |

Everything else is unchanged: `started`, `text_chunk`, `completed`, `failed`, `cancelled`.

### 3.1 Deliberate non-goals inside the contract

- **No `session.next.*` variants.** The durable family is newer than the pinned contract, its emit sites could not be proven from the binary, and building on it would be guessing. Recorded as a follow-up, not implemented.
- **No `Retry` row.** It needs a message identity the backend can replay; that is a session-management capability, not an event field.
- **No `CommentStrip` row.** The pinned comments are editor metadata with no BrainRoot source.
- **No tool `metadata`/`structured` passthrough.** Arbitrary nested maps from an untrusted process are not forwarded; only the bounded, named fields above.

## 4. Bounding rules

Every new field states its maximum and what happens past it. The rule is **truncate, never reject the frame** (matching `MAX_DELTA_CHARS`'s existing behaviour), because a rejected frame would silently lose a tool's terminal state.

| Field | Maximum | Past the bound |
|---|---|---|
| `reasoning_delta.text` | 64 KiB chars | truncated |
| `tool_event.input` | 8 KiB chars | truncated |
| `tool_event.output` | 8 KiB chars | truncated |
| `tool_event.error` | 8 KiB chars | truncated |
| `tool_event.title` | 200 chars (existing `MAX_NAME_CHARS`) | truncated |
| `tool_event.name` / `tool_id` | 200 chars | truncated |
| `file_event.path` | 2 KiB chars | truncated |
| `file_event.mime` | 200 chars | truncated |

Per-turn ceilings, applied in the worker exactly like the existing 2 MiB text ceiling:

- reasoning: **256 KiB chars** per turn, then one `failed` with `ResponseTooLarge`;
- tool/file events: **512 events** per turn, then one `failed` with `ResponseTooLarge`.

The session filter (`frame_session != session_id → Ignored`) runs **before** classification for every variant, unchanged.

## 5. Compatibility

- The frontend's `isAgentEventEnvelope` keeps a strict `contractVersion === AGENT_CONTRACT_VERSION` equality, so a v2 core and a v1 frontend never half-agree; the mismatch is reported as an unexpected event and the turn fails loudly.
- `AgentSendAccepted.contract_version` carries the same constant, so a send is refused before a stream starts if the two sides disagree.
- The legacy `conversation.ts` contract (`CONVERSATION_CONTRACT_VERSION = 1`) **does not move**. Decision: the extended events are agent-host-only. The legacy loop is the no-sidecar fallback and keeps its five-event shape; nothing in this change reads or writes it.

## 6. Implementation map

| File | Change |
|---|---|
| `src-tauri/src/features/agent_host/mod.rs` | `AGENT_CONTRACT_VERSION = 2`; four new `AgentStreamEvent` variants; `FrameOutcome` extended with the non-text outcomes; `classify_frame` arms for `message.part.updated`, reasoning deltas, `session.diff`, and `patch` parts; per-turn ceilings in `run_send_worker`; tests |
| `src/agentHost.ts` | `AGENT_CONTRACT_VERSION = 2`; the four variant types; `isAgentEventEnvelope` validation for each, including a bounded-string check |
| `src/agentHost.test.mjs` | accept/reject cases per variant; version-mismatch case |
| `docs/adr/0017-agent-event-contract-v2.md` | the versioning decision |
| `docs/07-agent-architecture.md` | the contract evidence paragraph moves to version 2 |
| `docs/specs/b20-opencode-parity-u03.md` | the stop note points at the landed contract |
| `scripts/check-modules.sh` | no new public symbol is required (the new variants are fields of an already-registered enum), verified rather than assumed |

No new dependency. No change to permissions, the sidecar lifecycle, secrets, persistence, or network destinations.

## 7. Verification

Fast non-test micro-gate (this issue, ADR 0014):

1. `cd src-tauri && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings` → clean.
2. `pnpm run check` → svelte-check 0 errors, 0 warnings.
3. `sh scripts/check-docs.sh && sh scripts/check-security.sh && sh scripts/check-accessibility.sh` → clean.
4. `git diff --name-only <start>...HEAD` → only the allowlisted paths.
5. `pnpm build` → CSS gzip ≤ 35 KB, JS gzip ≤ 60 KB, MEASURED values recorded.

Deferred to the versioned release gate (ADR 0014), case IDs as filed on the issue:

- `B20-U3a-T01` each new variant round-trips through Rust serialization and TS validation;
- `B20-U3a-T02` a foreign session's part is ignored for every new variant;
- `B20-U3a-T03` each field truncates at its bound rather than rejecting the frame;
- `B20-U3a-T04` a malformed part is ignored, not fatal;
- `B20-U3a-T05` a version-1 envelope is rejected deterministically by the v2 validator.

## 8. Rollback

Revert the commit. The contract returns to version 1, `classify_frame` ignores non-text parts again, and the timeline stays stopped. No persisted state, no migration, no artifact.
