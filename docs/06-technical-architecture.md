# Technical architecture

## Architectural style

BrainRoot begins as a **modular desktop monolith**. One installed application contains a Tauri 2 shell, a Rust core, and a Svelte/TypeScript interface. External agents, shells, dev servers, LSPs, search, and browser automation are child processes started only when requested.

```text
Svelte UI + primary shell WebView
          │ typed commands/events; least privilege
Rust Core ├─ Workspace / Checkpoints / Persistence
          ├─ Agent Host / Tool & Permission Layer
          ├─ Process & Browser Managers
          └─ Resource Governor / Event Bus
                    │
      on-demand child processes and content WebViews
```

This is a logical map, not a requirement for one crate per box.

## Initial technology decisions

- Tauri 2 with WRY/TAO for the desktop shell; see ADR 0001.
- Rust for privileged core and lifecycle state; see ADR 0002.
- Svelte with TypeScript for the interface; see ADR 0003.
- CodeMirror 6 only when optional Code View enters scope; see ADR 0004.
- ACP-capable Agent Adapter boundary with one provider in the MVP; see ADR 0005.
- System WebViews with one heavy content view active by default; see ADR 0006.
- SQLite for global structured state plus a small project-local `.brainroot/`; see ADR 0007.
- Event-driven, zero-idle lifecycle; see ADR 0008.

Versions are intentionally not pinned until the implementation bootstrap task verifies the then-current stable releases, MSRV, OS requirements, advisories, and compatible lockfile.

## Module boundaries

### App Shell

Owns desktop windows, menus, global shortcuts, update entry points, and UI lifecycle. It does not execute arbitrary project operations.

### Workspace Manager

Opens explicitly selected roots, canonicalizes allowed paths, identifies project capabilities lazily, and owns project-level cancellation.

### Agent Host

Starts one adapter, negotiates capabilities, normalizes events, owns the session, cancels it, and cleans it up.

### Tool Layer and Permission Manager

Expose small typed operations, validate inputs, apply grants, redact outputs, and record audit events. Agents never invoke arbitrary core internals.

### Process Manager

Owns child process groups, stdio/PTY handles, readiness, cancellation, escalation, and cleanup. Every process has a resource owner.

### Browser Manager

Separates shell, preview, human, and agent automation contexts; owns navigation policy and WebView lifecycle.

### Checkpoint Manager

Captures changed state, summarizes impact, and restores without exposing Git semantics or overwriting unrelated work.

### Context Engine

Discovers context lazily through paths, ripgrep, dependency edges, and optional parsing. It does not index or embed every repository by default.

### Persistence

Stores global structured state transactionally and project-portable metadata explicitly. Secrets use OS credential storage.

### Resource Governor and Event Bus

Track resource state and route typed domain events. The Event Bus is in-process, bounded, and observable—not a distributed message broker.

## Boundary rules

- Frontend input is untrusted at Rust commands.
- All paths are canonicalized and checked against allowed roots at time of use.
- IDs, events, and persisted records are versioned typed schemas.
- Long work is cancellable and cannot block the UI event loop.
- Providers cannot directly own UI state or permission policy.
- Process and WebView handles never leak to presentation code.

## Minimal event model

`ProjectOpened`, `TaskCreated`, `AgentStarted`, `AgentProgressed`, `PermissionRequested`, `PermissionResolved`, `FilesChanged`, `ProcessStarted`, `ProcessExited`, `PreviewReady`, `BrowserNavigated`, `TestStarted`, `TestFinished`, `CheckpointCreated`, `ResourceStateChanged`, `AgentFinished`, and `TaskFailed`.

Events carry stable IDs, monotonic sequence per task, timestamp, source, sanitized user summary, optional technical payload reference, and schema version. Persist selected high-level events for session resume; keep high-volume output in bounded logs.

## Technical evidence ledger (reviewed 2026-09-22)

### Tauri 2 and WRY

**FACT:** Tauri uses Rust plus OS WebViews and routes IPC through a core process; desktop engines are WebView2 on Windows, WKWebView on macOS, and WebKitGTK on Linux. Capabilities authorize UI-to-core calls but do not sandbox arbitrary child processes. [Architecture](https://v2.tauri.app/concept/architecture/), [process model](https://v2.tauri.app/concept/process-model/), and [Runtime Authority](https://v2.tauri.app/security/runtime-authority/).  
**ASSUMPTION:** A system-WebView shell can meet BrainRoot's idle/startup targets and child-view needs.  
**DECISION:** Use Tauri 2 for the prototype, gated by Phase 1 and browser lifecycle measurements.  
**OPEN QUESTION:** Exact stable version, Linux support floor, multi-WebView reliability, updater, packaging, and sandbox interaction.

### Rust

**FACT:** Tauri's core runs in Rust and centralizes global state/IPC.  
**ASSUMPTION:** One Rust authority will be simpler and lighter than a permanent Node or multi-service backend.  
**DECISION:** Rust owns privileged state, policy, persistence coordination, and resource handles.  
**OPEN QUESTION:** Async runtime, crate boundaries, MSRV, and any narrowly justified `unsafe` use.

### Svelte and TypeScript

**FACT:** Svelte is compiler-based and is designed to emit concise browser work. [Svelte](https://svelte.dev/).  
**ASSUMPTION:** It will keep the small task/Canvas UI understandable for AI contributors without a large UI runtime.  
**DECISION:** Use stable Svelte + TypeScript for the prototype without a component mega-library.  
**OPEN QUESTION:** Exact toolchain/version and measured bundle/startup behavior.

### CodeMirror 6

**FACT:** CodeMirror is modular; state, view, commands, and languages can be selected separately. [System guide](https://codemirror.net/docs/guide/).  
**ASSUMPTION:** Lazy loading a small subset will satisfy basic Code View without IDE-scale cost.  
**DECISION:** Defer CodeMirror until Phase 7 and load it only when Code View opens.  
**OPEN QUESTION:** Minimal package/language set and large-file limits.

### ripgrep and Tree-sitter

**FACT:** Tree-sitter is an incremental parsing library designed for editor-time updates. [Tree-sitter](https://tree-sitter.github.io/tree-sitter/). [ripgrep](https://github.com/BurntSushi/ripgrep) is a focused text-search tool, not a semantic index.  
**ASSUMPTION:** Search-first lazy discovery is sufficient for the MVP and cheaper than automatic whole-repository indexing.  
**DECISION:** Use path/text search first; start Tree-sitter or LSP capabilities only for a concrete need.  
**OPEN QUESTION:** Which languages and task types justify parsing or LSP startup, based on outcome benchmarks.

### Agent Client Protocol

**FACT:** ACP is an open editor/agent interoperability protocol with real Codex and other integrations; capabilities and provider-owned authentication vary. [ACP](https://zed.dev/acp).  
**ASSUMPTION:** It can reduce provider coupling without dictating BrainRoot's task or permission model.  
**DECISION:** Keep an internal Agent Adapter contract and prefer ACP when the first full coding-agent spike meets requirements. MVP-0 may exercise a narrower provider transport without pretending it is ACP or an agent.  
**OPEN QUESTION:** First full coding agent, direct ACP versus bridge, protocol version, resume/cancellation/tool coverage, and packaging.

### OpenCode Go model gateway

**FACT:** OpenCode Go currently exposes a model catalog and multiple protocol endpoints; its third-party client guidance requires a client-specific User-Agent and stable `x-opencode-session` per conversation. Catalog, limits, endpoints, and privacy properties may change. [OpenCode Go](https://opencode.ai/docs/go).  
**ASSUMPTION:** A single Linux model loop is a useful low-cost experiment for the UI/core/provider boundary before a complete coding agent exists.  
**DECISION:** MVP-0 integrates one validated Go model/protocol behind a provider-neutral boundary with fake-first tests, streaming, cancellation, and core-only credentials.  
**OPEN QUESTION:** Selected model/protocol at execution time, Linux Secret Service path, subscription onboarding, live-test policy, and whether the experiment later delegates to an OpenCode/ACP agent.

### Git and checkpoints

**FACT:** Git exposes stable low-level commands for alternative porcelain interfaces and supports restoring tree content. [Git commands](https://git-scm.com/docs/git).  
**ASSUMPTION:** Git object storage can back checkpoints for Git projects without creating user-visible commits or rewriting history.  
**DECISION:** Present checkpoint/restore semantics; never expose Git as the required UX and support a non-Git fallback.  
**OPEN QUESTION:** Object/ref strategy, ignored/untracked/large files, dirty baselines, retention, and conflict recovery.

### SQLite

**FACT:** SQLite is embedded and serverless; WAL improves some concurrency patterns but adds files and checkpoint responsibilities. [Application format](https://www.sqlite.org/appfileformat.html) and [WAL](https://www.sqlite.org/wal.html).  
**ASSUMPTION:** One core-owned database will keep resume state simpler than JSON piles without meaningful idle overhead.  
**DECISION:** Use SQLite for global structured state; do not choose WAL without measurement.  
**OPEN QUESTION:** Rust crate, migration layer, journal mode, backup, corruption recovery, and encryption scope.

### Playwright

**FACT:** Playwright exposes DOM, console, network, screenshot, and trace evidence. [Debugging](https://playwright.dev/docs/debug) and [network](https://playwright.dev/docs/network).  
**ASSUMPTION:** An on-demand process can provide the MVP observe/test/fix loop with acceptable cold-start cost.  
**DECISION:** Use it as a disposable Agent Browser candidate, never an always-on service.  
**OPEN QUESTION:** Browser installation/distribution size, startup latency, sandboxing, artifact retention, and per-project version conflicts.

### PTY and process control

**FACT:** Rust libraries such as `portable-pty` abstract POSIX and Windows PTY implementations. [Documentation](https://docs.rs/portable-pty/latest/portable_pty/).  
**ASSUMPTION:** MVP commands may not need an interactive PTY; pipes could be smaller and safer for most tasks.  
**DECISION:** Put both behind Process Manager; add a PTY only for a demonstrated interaction.  
**OPEN QUESTION:** Library, ConPTY/POSIX behavior, process-group termination, encoding, resize, and shell discovery.

### Credential storage and sandboxing

**FACT:** macOS Keychain and Windows Credential Locker store small secrets; Linux mechanisms and availability vary. OS sandbox mechanisms also differ materially. See [security and permissions](11-security-and-permissions.md).  
**ASSUMPTION:** A small abstraction can provide honest capability detection without a lowest-common-denominator security claim.  
**DECISION:** Secrets stay out of project/SQLite state; Safe Mode combines core authorization with the strongest practical platform containment.  
**OPEN QUESTION:** Linux Secret Service fallback and exact child-process containment/distribution strategy per OS.

### MVP-0 implementation evidence (2026-09-22)

**MEASURED (code and gates):** the Linux model loop is implemented as a modular Rust monolith. `features::conversation` owns the explicit state machine, the versioned IPC wire types, the model catalog selection, the provider runners, and the one-worker runtime; `features::health` owns the typed shell readiness contract; `provider/` remains the adapter subsystem (frozen neutral contract, deterministic fake, bounded execution, normalizer, Go discovery/transport, Secret Service credential boundary). Offline gates enforce module boundaries, accessibility, and security invariants, and the opt-in live smoke verified the real `chat/completions` path on the reference environment.

**DECISION:** the MVP-0 runtime is one on-demand `std::thread` per active blocking request with typed Tauri events; this does not settle the later agent/process concurrency stack (`docs/17-open-questions.md`).

**EXTERNAL MUTABILITY:** the OpenCode Go catalog, endpoints, limits, prices, privacy/retention fields, and the authenticated models payload can change; the transport discovers and validates at runtime, and the dated snapshot lives in `docs/specs/opencode-go-contract.md`. See [the artifact notes](specs/b05-linux-artifact.md) and [the MVP-0 soak](specs/performance-reports/b05-mvp0-soak.md).

### Lean pivot implementation evidence (2026-09-25, `0.0.15`)

**MEASURED (code and gates):** the pivot described in [the Lean YAGNI plan](21-lean-yagni-pivot.md) adds two Rust modules and a typed frontend surface. `features::agent_host` owns `opencode serve --pure` as an on-demand child: start/status/stop, a 10 s authenticated readiness probe, model listing filtered to identifiers, `send`/`cancel_send` over a fresh sidecar session with a single-flight worker on the `/event` stream, an agent selector (`plan`/`build`), and a per-session token/cost read. `features::agent_host::catalog` fetches the public OpenRouter listing and merges context length and per-million prices onto the sidecar ids with a six-hour TTL and an honest `stale` flag. `features::governor` becomes the single-thread idle enforcer for the sidecar. The frontend adds `src/agentHost.ts` (versioned guards plus invoke wrappers), session tabs, and a native model picker.

**BOUNDARY STATUS — the module map above is aspirational for this path.** The Agent Host talks to the sidecar directly; requests do **not** pass through the Tool Layer or Permission Manager, the sidecar receives no approved project root, and there is no per-action permission decision. `features::conversation` stays frozen as the offline fallback used when the sidecar is stopped. This is recorded as a blocker in [open questions](17-open-questions.md); it is not a claim that the architecture invariant is met. [ADR 0015](adr/draft-0015-opencode-serve-default.md) remains Proposed.
