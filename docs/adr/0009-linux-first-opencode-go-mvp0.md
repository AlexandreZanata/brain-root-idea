# ADR 0009: Linux-first MVP-0 with OpenCode Go model gateway

**Status:** Accepted for experiment  
**Date:** 2026-09-22

## Context

The complete BrainRoot MVP spans native shell, agent orchestration, safe tools, preview, validation, and checkpoints across three desktop platforms. Implementing all dimensions before testing the product boundary would make an economical coding agent handle too much uncertainty at once. The project lead selected Linux as the first proving platform and OpenCode Go as the first low-cost model source.

OpenCode Go exposes several model protocols and a changing model catalog. Its current guidance asks third-party coding clients to identify themselves with a specific user agent and send a stable `x-opencode-session` value per conversation. A model API is not a complete coding agent protocol.

## Decision

Create an explicitly experimental **MVP-0 Linux Model Loop** before the full product MVP. It validates a minimal Tauri/Svelte/Rust Linux shell, secure provider configuration, current model discovery, one selected protocol/model path, streaming, cancellation, errors, and cleanup.

Keep the transport behind BrainRoot's adapter/provider boundary. Do not expose OpenCode-specific response types to the task UI or claim that the raw model gateway implements Agent Host, tools, project modification, ACP, Canvas, or checkpoints. Use deterministic fake-provider contract tests; live Go tests are opt-in and secret-safe.

## Alternatives considered

- **Build the complete cross-platform MVP immediately:** rejected because it multiplies platform, agent, browser, security, and workflow risk.
- **Integrate the OpenCode CLI as the first agent:** still viable later, but the selected first experiment is the Go model API and should not inherit CLI auth/process behavior implicitly.
- **Call one endpoint directly from the frontend:** rejected because it exposes credentials and provider-specific types to an untrusted UI boundary.
- **Hardcode one current model forever:** rejected because Go's catalog, limits, protocols, and privacy properties can change.

## Consequences

The first tagged artifact is a test foundation, not a usable coding IDE. Linux receives deliberate attention first; Windows and macOS remain roadmap work. A provider transport abstraction and mock server are necessary earlier than originally planned.

## Performance implications

The experiment produces the first Linux startup, idle, streaming-memory, cancellation, and cleanup measurements. No model connection exists at idle. Network latency and provider time are reported separately from local UI latency.

## Security implications

Credentials stay in the Rust/provider boundary and OS credential storage when available, never frontend state, project files, Wiki, logs, or CI. The UI sends user intent, not authorization headers. Model choice must surface current privacy/retention information rather than assuming all Go models have the same policy.

## Reversibility

High. OpenCode Go is one provider behind an internal boundary. The experiment can be removed or replaced without changing the product task model. Linux-first affects sequencing, not the long-term cross-platform goal.

## References

[OpenCode Go guide](https://opencode.ai/docs/go), [OpenCode provider setup](https://opencode.ai/docs/providers), [Agent architecture](../07-agent-architecture.md), and [MVP scope](../14-mvp-scope.md).
