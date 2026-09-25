# Agent architecture

## One product agent, replaceable runtime

BrainRoot presents one agent identity while the Rust core ultimately hosts one external coding-agent integration. The product task model, permissions, tools, checkpoints, and UI do not depend on provider-specific message formats. MVP-0 first validates a narrower OpenCode Go model transport on Linux; that transport is not described as an autonomous coding agent.

```text
Product Task Model
       ↕ normalized events/actions
Agent Host ─ Permission Broker ─ Tool Layer
       ↕ AgentAdapter
ACP adapter or provider-specific adapter
       ↕ stdio / negotiated transport
External agent process
```

## Agent Host responsibilities

- Discover and start the configured adapter on demand.
- Negotiate protocol/capabilities and reject incompatible versions.
- Create, resume where supported, cancel, and terminate sessions.
- Convert provider updates into stable product task events.
- Broker tool requests through permission policy.
- Enforce workspace identity and session ownership.
- Bound output, redact secrets, persist high-level session history, and clean up.
- Mark restoration as unavailable when an agent cannot resume rather than simulating it.

The Agent Host does not remain alive with an agent when no session needs it.

## Adapter contract

An adapter exposes `start`, `capabilities`, `new_session`, `send`, `cancel`, `close`, and an event stream. Capability flags include streaming text, structured tool calls, permission requests, task plan updates, session resume, images, diff/edit metadata, and MCP forwarding. Unsupported features degrade explicitly.

ACP is preferred for the later coding-agent integration because it is open and already supports multiple agents, but BrainRoot owns a thin product adapter because ACP does not define BrainRoot's UX, checkpoint, process, or security model. MVP-0 uses a provider-neutral model transport for OpenCode Go with no tools. Direct ACP compatibility remains a later acceptance decision after a focused agent spike.

## MVP-0 provider subset

The Linux Model Loop implements only model discovery/configuration, one conversation/session identity, user text input, streamed assistant text, cancellation, normalized provider errors, and cleanup. Credentials and HTTP requests stay in the Rust core. Provider-specific endpoint shapes do not cross into Svelte state. Filesystem, shell, project context, tool calls, checkpoints, and autonomous loops remain unavailable.

## Tool Layer

Initial conceptual operations:

- `filesystem.read`, `filesystem.write`, `filesystem.search`, `filesystem.list`
- `git.status`, `git.diff`, `checkpoint.create`, `checkpoint.restore`
- `shell.run`, `shell.stop`
- `project.start`, `project.stop`
- `browser.open_preview`, `browser.inspect`, `browser.screenshot`, `browser.console`, `browser.network`, `browser.click`, `browser.type`
- `tests.run`

Each request includes task, workspace, purpose, typed arguments, permission class, timeout, cancellation token, and idempotency/retry semantics where relevant. Each result is size-bounded, typed, sanitized, and references larger artifacts rather than flooding the protocol.

## Context Engine

Discovery sequence:

1. user goal and current task state;
2. explicit files and recent changes;
3. cheap path and text search with ripgrep;
4. related imports/configuration;
5. Tree-sitter symbols only for the relevant languages/files;
6. LSP only when its capability justifies startup;
7. embeddings only after a benchmark shows better task outcomes per cost.

No automatic full-repository scan, embedding pipeline, vector database, or always-on indexer.

## Trust boundaries

Agent messages are untrusted proposals. The Tool Layer, not the provider, resolves filesystem roots, permissions, shell policy, browser identity, network policy, secrets, and destructive-action rules. A subagent inherits no authority beyond the parent task and cannot bypass the Agent Host.

## Cancellation and failure

Cancellation flows from task → adapter → active tool calls → owned processes. A grace period is followed by force termination of the owned process group. Partial output is marked incomplete; resource cleanup and checkpoint integrity run even after protocol failure.

## Lean pivot implementation evidence (2026-09-25, `0.0.15`)

**MEASURED (code and gates):** the first real agent integration exists as `features::agent_host`. It owns the sidecar process, generates one ephemeral basic-auth password per run (`uuid v4`), holds it only in memory with a redacting `Debug` impl, probes `/global/health` until ready, spawns one worker thread per turn on a fresh session, and terminates the child with a 5 s grace on stop, idle, or window close. The product event envelope is versioned (`contract_version = 1`) and normalized to `started`/`text_chunk`/`completed`/`failed`/`cancelled`, with bounds on the delta (64 KiB), the turn (2 MiB), and the stream (10 min). A minimal, deliberately stable system prompt travels on every send to keep the sidecar KV-cache warm.

**What this is not.** It is not an implementation of the adapter contract on this page. There is no capability negotiation, no session resume, no tool mediation, no permission brokering, and no checkpoint integration on this path — the sidecar's own `plan`/`build` modes are passed through as data and nothing in BrainRoot enforces them. ACP is still unimplemented, and the Tool Layer, workspace roots, and permission records described above remain designs. The agent therefore currently runs with the user's own authority; see [open questions](17-open-questions.md).
