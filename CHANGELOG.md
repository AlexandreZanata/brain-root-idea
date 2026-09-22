# Changelog

All notable changes to BrainRoot are recorded here. The format is human-readable and uses the categories from `docs/19-release-and-versioning.md`. Versions follow Semantic Versioning 2.0.0 and stay below `1.0.0` during the experiment.

Batch B00 (governance and delivery controls) is tracked by [pull/2](https://github.com/AlexandreZanata/brain-root-idea/pull/2) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B00-Governance).

Batch B01 (measured Linux shell) is tracked by [pull/9](https://github.com/AlexandreZanata/brain-root-idea/pull/9) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B01-Linux-Shell).

## Unreleased

No unreleased changes yet.

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
