# Changelog

All notable changes to BrainRoot are recorded here. The format is human-readable and uses the categories from `docs/19-release-and-versioning.md`. Versions follow Semantic Versioning 2.0.0 and stay below `1.0.0` during the experiment.

Batch B00 (governance and delivery controls) is tracked by [pull/2](https://github.com/AlexandreZanata/brain-root-idea/pull/2) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B00-Governance).

Batch B01 (measured Linux shell) is tracked by [pull/9](https://github.com/AlexandreZanata/brain-root-idea/pull/9) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B01-Linux-Shell).

## Unreleased

No unreleased changes yet.

## 0.0.4

Batch B08 (Companion Browser prerequisites, CB-A gate) is tracked by [pull/64](https://github.com/AlexandreZanata/brain-root-idea/pull/64) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B08-Preview-Prerequisites).

### Documentation

- The Preview origin policy is approved: only canonical loopback origins on BrainRoot-owned dev-server ports may load, fail-closed, with the denial rules, the Rust-owned enforcement contract, and the negative test corpus for the preview slice ([#65](https://github.com/AlexandreZanata/brain-root-idea/issues/65)).
- The owned dev-server lifecycle is approved as ADR 0011: one declared-command server per project, an assigned loopback port registered for the preview, process-group termination that never kills foreign processes, no automatic restart loop, stop-on-close idle policy, and untrusted bounded output ([#66](https://github.com/AlexandreZanata/brain-root-idea/issues/66)).
- A disposable Linux child-WebView probe and its measurement record document what the current Tauri/wry stack supports before any preview implementation is assigned ([#63](https://github.com/AlexandreZanata/brain-root-idea/issues/63)).

### Known limitations

- Child-view position and size are not honored on the Wayland reference stack (`GtkBox` hosting), so the Companion Canvas preview implementation waits for the ADR 0006 review; memory growth over 100 probe cycles is unclassified, and focus/WARM behavior is unmeasured.
- The probe binary exists only behind the optional `probe` Cargo feature and is not part of the shipped app.

## 0.0.3

Batch B07 (Companion Browser plan integration, documentation only) is tracked by [pull/60](https://github.com/AlexandreZanata/brain-root-idea/pull/60) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B07-Companion-Browser-Plan).

### Documentation

- The maintainer-adopted Companion Browser plan now lives in `docs/specs/companion-browser-plan.md` and is referenced from the Companion Canvas, browser architecture, roadmap, and open-questions documents, without implementing any browser capability and without turning the CB-A…CB-D candidates into executable issues ([#59](https://github.com/AlexandreZanata/brain-root-idea/issues/59)).

### Known limitations

- The Companion Browser is not implemented; no preview, Human Browser, import, Deck, or phone capability exists, and future batch IDs are not assigned.

## 0.0.2

Batch B06 (workspace shell in the product visual language) is tracked by [pull/53](https://github.com/AlexandreZanata/brain-root-idea/pull/53) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B06-Workspace-Shell).

### Added

- A reusable monochrome component set on one token source (`src/lib/theme.css`): `Icon`, `Button`, `Badge`, `NavItem`, `SuggestionItem`, `Turn`, `WelcomeCard`, `EmptyState`, `ActionCard`, `ThemeToggle`, `AppHeader`, `WorkspaceRail`, `ConversationPanel`, `CanvasPanel`, plus `TextArea` and `PanelResizer`, with a persisted light/dark theme toggle and the accessibility gate covering the whole frontend ([#54](https://github.com/AlexandreZanata/brain-root-idea/issues/54), [#57](https://github.com/AlexandreZanata/brain-root-idea/issues/57)).
- A resizable agent panel (280–560 px, in-memory only) with a native, keyboard-operable vertical handle that is hidden when the workspace stacks below 1080 px ([#57](https://github.com/AlexandreZanata/brain-root-idea/issues/57)).

### Changed

- The workspace shell follows the requested product visual language: left section rail, agent panel with welcome and suggestion actions, bottom composer with the configured-model chip, and a dominant Canvas with tabs; every future surface is `aria-disabled` and labeled MVP-1 instead of pretending to be live ([#52](https://github.com/AlexandreZanata/brain-root-idea/issues/52)).
- Dark follows the Codex monochrome palette with no blue, light follows the reference palette with the blue accent, and tabs use link-style underlines; the workspace stacks below 1080 px and compacts below 640 px ([#55](https://github.com/AlexandreZanata/brain-root-idea/issues/55), [#56](https://github.com/AlexandreZanata/brain-root-idea/issues/56)).

### Known limitations

- Future surfaces remain disabled placeholders; the rendered window still has no UI automation; the panel width is not persisted across restarts.

## 0.0.1

Batch B05 (Linux MVP-0 hardening and release) is tracked by [pull/44](https://github.com/AlexandreZanata/brain-root-idea/pull/44) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B05-Linux-MVP0-Release).

Batch B05 (Linux MVP-0 hardening and release) is tracked by [pull/44](https://github.com/AlexandreZanata/brain-root-idea/pull/44) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B05-Linux-MVP0-Release).

### Added

- A deterministic fake-provider end-to-end journey: 500 + 500 fake conversations and 200 cancellations with joined workers, driver latency, and RSS/thread reporting, plus an open/close cycle soak ([#46](https://github.com/AlexandreZanata/brain-root-idea/issues/46)).
- An offline security gate covering destination allowlists, the frontend network surface, telemetry markers, Tauri capabilities, log hygiene, and the direct-dependency allowlists, with an injectable live transport so oversized, malformed, and failed responses are tested without network access ([#45](https://github.com/AlexandreZanata/brain-root-idea/issues/45)).
- An end-to-end script that runs the product journey, the rendered-output bounds, and the launch/readiness/close process smoke ([#43](https://github.com/AlexandreZanata/brain-root-idea/issues/43)).
- One documented Linux artifact: a Tauri Debian package with its runtime dependencies, checksum, and install/run/remove notes ([#47](https://github.com/AlexandreZanata/brain-root-idea/issues/47)).

### Changed

- Release-facing documentation now reflects the implemented Linux MVP-0, including the mutable OpenCode Go terms, catalog, limits, endpoints, and privacy, and the failure/recovery history.

### Known limitations

- The total-memory TARGET fails on the reference environment (426.2 MB summed RSS / 241.6 MB PSS against 150 MB); idle CPU is a median pass with windows above budget on a busy host; live-path cancellation latency is `UNKNOWN` ([report](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/specs/performance-reports/b05-mvp0-soak.md)).
- The Debian artifact is unsigned, experimental, Ubuntu/Debian-family x86_64 only, and its checksum is per build.

## 0.0.1-alpha.5

Batch B04 (minimal Linux conversation loop) is tracked by [pull/33](https://github.com/AlexandreZanata/brain-root-idea/pull/33) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B04-Conversation-UI).

### Added

- An explicit conversation state machine — `EMPTY`, `READY`, `SENDING`, `STREAMING`, `CANCELLING`, `SUCCEEDED`, `FAILED` — that rejects late chunks, double submits, duplicate terminals, and completion after cancel ([#32](https://github.com/AlexandreZanata/brain-root-idea/issues/32)).
- An agent-first layout: the Focus preset (30 % agent / 70 % Canvas), a labeled prompt field, and the dominant "Preview comes in MVP-1." Canvas placeholder with keyboard, focus, zoom, semantics, contrast, and reduced-motion checks ([#34](https://github.com/AlexandreZanata/brain-root-idea/issues/34)).
- A typed, versioned conversation IPC (`conversation_send`, `conversation_event`) that streams neutral events to the Svelte surface with bounded rendered history and duplicate-submit prevention ([#35](https://github.com/AlexandreZanata/brain-root-idea/issues/35)).
- Cancellation end to end: the state moves to `CANCELLING` immediately, propagates through the shared token, emits exactly one `cancelled` terminal, and the window close path cancels the same way ([#37](https://github.com/AlexandreZanata/brain-root-idea/issues/37)).
- Understandable setup and failure states: not configured, no compatible model, invalid credential, network unavailable, limit reached, timeout, provider error, and malformed response each explain the next action, with the neutral error code available as optional technical detail ([#40](https://github.com/AlexandreZanata/brain-root-idea/issues/40)).
- Feature modules with an enforceable interface: `features::conversation` (state, wire, catalog, runner, runtime, interface) and `features::health`, guarded by `scripts/check-modules.sh` in both gates ([#39](https://github.com/AlexandreZanata/brain-root-idea/issues/39)).

### Changed

- Model discovery accepts the live models payload shape, where entries state no per-model endpoint; an unstated endpoint is a candidate, a stated unsupported endpoint is still filtered, and the configured default model resolves without a fallback ([#36](https://github.com/AlexandreZanata/brain-root-idea/issues/36)).
- Failure and setup messages now end with the next action, and the credential status is read once at mount instead of being inferred from send failures.

### Fixed

- The conversation history no longer announces every streamed chunk; `Send`/`Cancel` keep keyboard focus via `aria-disabled`; the disabled button label contrast was raised from 4.15:1 to 7.11:1 ([#41](https://github.com/AlexandreZanata/brain-root-idea/issues/41)).

### Security

- The OpenCode Go key stays in the Rust core and never crosses the IPC boundary; the accessibility verification scanned 118 files and the built bundle with no key material, credential, or authorization header found ([#41](https://github.com/AlexandreZanata/brain-root-idea/issues/41)).
- The live smoke ran locally with a maintainer key (environment only) and recorded counts and timing only.

### Documentation

- The OpenCode Go contract re-verification, the batch decisions, and the failure/rollback notes are recorded in the Wiki batch page and `docs/history/batches/B04-conversation-ui.md`.

### Known limitations

- The rendered window still needs the maintainer/release smoke for visual confirmation; no UI automation is approved.
- A cancel issued while a socket read is blocked is released when the transport returns, bounded by the 120 s total timeout.
- The OpenCode Go catalog, endpoints, limits, privacy terms, and authenticated models shape are externally mutable; the live smoke passed on 2026-09-22 with a local key (`models=33`, chat `chunks=1`, normal terminal).
- No package, artifact, or checksum exists for this pre-release.

## 0.0.1-alpha.4

Batch B03 (OpenCode Go live transport) is tracked by [pull/25](https://github.com/AlexandreZanata/brain-root-idea/pull/25) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B03-Opencode-Go).

### Added

- OpenCode Go model discovery in the Rust core: the documented models endpoint fetched through an injectable transport, protocol-family filtering, a bounded nonsecret cache with explicit `Fresh`/`Stale`/`Empty` states, and tolerant parsing of unknown fields ([#26](https://github.com/AlexandreZanata/brain-root-idea/issues/26)).
- Linux credential storage through an approved Secret Service integration with an injectable backend, a truthful `unavailable` status, and a session-only fallback instead of plaintext persistence; the key cannot cross the frontend contract ([#27](https://github.com/AlexandreZanata/brain-root-idea/issues/27)).
- One validated `chat/completions` path with a BrainRoot-specific user agent, a random nonsecret `x-opencode-session` per conversation, a typed request body, and an SSE parser that maps the live stream to provider-neutral events with no silent fallback to another model, protocol, endpoint, or balance ([#28](https://github.com/AlexandreZanata/brain-root-idea/issues/28)).
- A shared Go failure classification: invalid key, unavailable subscription, unsupported model, rate/usage limit, timeouts, provider failures, malformed responses, and network failures become distinct plain-language states using the existing neutral error codes ([#29](https://github.com/AlexandreZanata/brain-root-idea/issues/29)).
- Per-model privacy/retention disclosure captured from the models payload when it states it, bounded and explicitly `unknown` otherwise, so no policy is claimed that the endpoint does not state ([#29](https://github.com/AlexandreZanata/brain-root-idea/issues/29)).
- An opt-in live smoke test that discovers the live catalog and streams one harmless fixed prompt to a normal terminal event; it is ignored by default and additionally requires `BRAINROOT_LIVE_SMOKE=1` plus a locally exported `BRAINROOT_OPENCODE_GO_KEY` ([#30](https://github.com/AlexandreZanata/brain-root-idea/issues/30)).

### Changed

- Discovery and chat HTTP clients gained an explicit `with_timeout` option used by the opt-in smoke (the B02 120 s total budget); default transport behavior is unchanged and no default timeout is wired yet.

### Security

- The OpenCode Go key stays in the Rust core: environment injection is limited to the opt-in live test, never persisted, and never printed; logs, test output, and evidence contain no credential, authorization header, or provider response body.
- Failure classification never reads or surfaces a provider error body, and the live smoke records only counts, field names, and timing.

### Documentation

- The current OpenCode Go contract (endpoints, client identity, session guidance, usage limits, per-model privacy, and mutability) is captured in `docs/specs/opencode-go-contract.md` ([#24](https://github.com/AlexandreZanata/brain-root-idea/issues/24)); the batch record and Wiki page document the re-verification findings.

### Known limitations

- The public models payload observed on 2026-09-22 states no per-model `endpoint` or privacy field; the authenticated shape and the `402`/`403`/`404` status semantics remain live-unverified. Discovery therefore yields an empty catalog on the observed public shape until an authorized live run confirms or a remediation updates it.
- The opt-in live smoke was not executed as part of this batch's evidence; it is not part of the required deterministic CI path.
- In-stream provider errors map to the generic `provider_unavailable`, and the credential store is not wired to the application yet (B04).
- No package, artifact, or checksum exists for this pre-release.

## 0.0.1-alpha.3

### Added

- A versioned provider-neutral contract in the Rust core: model descriptor, conversation ID, user message, request, stream events, completion, cancellation, and normalized errors with documented size limits and a request-state transition function ([#16](https://github.com/AlexandreZanata/brain-root-idea/issues/16)).
- A deterministic fake provider that speaks a documented wire format and scripts success, authentication failure, rate limiting, server error, malformed frames, delayed events, and a never-ending stream as data — no clock, thread, or network ([#18](https://github.com/AlexandreZanata/brain-root-idea/issues/18)).
- A pull-based bounded execution layer: one request owner with a shared cancellation token, connect/read/total timeouts, a response-size limit, and exactly one terminal event per request ([#19](https://github.com/AlexandreZanata/brain-root-idea/issues/19)).
- Byte-level streaming normalization: arbitrary chunk boundaries including split UTF-8, bounded buffers and queues with explicit backpressure, neutral event mapping, and plain-language errors that never surface provider text ([#20](https://github.com/AlexandreZanata/brain-root-idea/issues/20)).
- A core-only credential boundary: `CredentialStatus` is the only serializable type, credential values and authorization headers redact themselves, and the `provider_status` app command lets the UI learn configured/not-configured only ([#21](https://github.com/AlexandreZanata/brain-root-idea/issues/21)).
- A provider contract integration test that drives credential pre-flight, discovery, request, streaming, completion, cancellation, and a plain-language failure end to end ([#22](https://github.com/AlexandreZanata/brain-root-idea/issues/22)).

### Security

- No credential is stored, logged, or serialized; provider-supplied text is never surfaced; the provider modules contain no thread, timer, clock, or network client.

### Known limitations

- The provider boundary is not wired to the UI yet (B04) and there is no live transport (B03); the staged `dead_code` allowances remain until the UI consumes the modules.
- No package, artifact, or checksum exists for this pre-release.

## 0.0.1-alpha.2

### Added

- A minimal Linux shell: a Tauri 2 + Rust binary with a Svelte 5 + Vite + TypeScript frontend, checked with TypeScript 7.0.2 through `svelte-check --tsgo`, opening a `BrainRoot` window with the `Experimental Linux setup` state and no IPC permissions ([#10](https://github.com/AlexandreZanata/brain-root-idea/issues/10), [reference environment](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/specs/linux-reference-environment.md)).
- One versioned typed `health` contract with runtime validation and rejection of malformed input, unknown fields, and unsupported versions; the window shows `Ready` only after the core result, with a single request and no polling ([#11](https://github.com/AlexandreZanata/brain-root-idea/issues/11)).
- Repository-owned deterministic gates: `check-fast` (Rust formatting, unit tests, frontend types) and `check-full-linux` (plus clippy, production frontend build, production Tauri build, documentation links, version consistency, and license/secret checks), with a Rust toolchain pinned through `rust-toolchain.toml` and the full gate running in CI ([#12](https://github.com/AlexandreZanata/brain-root-idea/issues/12)).
- A release smoke test that proves launch, typed readiness, exactly one web and one network process, clean shutdown, and zero leftover processes or listeners ([#13](https://github.com/AlexandreZanata/brain-root-idea/issues/13)).

### Performance

- First measured baseline on the reference environment: warm start p50 **0.282 s** (target ≤ 1.0 s) and settled idle CPU median **0.172 %** (target < 1 %); frontend bundle ≈31 KB uncompressed; cleanup returns to zero processes. Report: [b01-linux-baseline](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/specs/performance-reports/b01-linux-baseline.md) ([#14](https://github.com/AlexandreZanata/brain-root-idea/issues/14)).

### Known limitations

- Memory target fails: the shell alone reaches ≈198.5 MB proportional (PSS) / 418.9 MB summed VmRSS against the 150 MB target; recorded as failing, not adjusted.
- Idle CPU stays a stability warning until more quiet-machine windows are measured, after one 1.310 % window in an earlier run.
- Without approved UI automation, the window-close path and the rendered `Ready` label are not asserted automatically; `SIGTERM` and the readiness marker stand in.
- `scripts/check-governance-templates.sh` is still outside CI because the runner lacks PyYAML for the default interpreter.
- No package or artifact is produced; there is no checksum and this is a pre-release.

## 0.0.1-alpha.1

### Added

- Repository governance for the experimental delivery workflow: verified Apache-2.0 `LICENSE` and BrainRoot `NOTICE` with an offline artifact check ([#1](https://github.com/AlexandreZanata/brain-root-idea/issues/1)), a microstep issue form and batch pull request template with a structural check ([#3](https://github.com/AlexandreZanata/brain-root-idea/issues/3)), a repository rules runbook with `main` protection ([#4](https://github.com/AlexandreZanata/brain-root-idea/issues/4)), and the project history and Wiki structure ([#5](https://github.com/AlexandreZanata/brain-root-idea/issues/5)).
- A single machine-readable version source (`VERSION`) with a deterministic consistency check (`scripts/check-version-consistency.sh`).

### Documentation

- Batch B00 repository history mirror and the Wiki operations pages, including the flat page-naming convention.

### Known limitations

- No application code, CI, provider integration, or user-visible capability exists yet; the repository is the specification for MVP-0.
