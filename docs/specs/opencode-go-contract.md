# OpenCode Go contract

**Status:** Verified snapshot for batch B03 (reviewed 2026-09-22)
**Sources:** official OpenCode documentation, linked in [Sources](#sources)
**Mutability:** catalog, model IDs, endpoints, limits, prices, and privacy fields are **externally mutable**; product logic must discover them at runtime and must never hardcode them.

## Purpose

Records the OpenCode Go contract that batch B03 implements: authentication, client identity, model discovery, protocol endpoint families, usage limits, and per-model privacy/retention. Later microsteps re-verify the facts on their execution date; this document is the reviewed snapshot, not a promise that any value stays stable.

## Authentication

- Go requires an OpenCode API key obtained from OpenCode Zen; requests send it as `Authorization: Bearer <key>` (the Zen documentation shows exactly this header shape in its endpoint example).
- BrainRoot must own or explicitly receive its credential through its own secure flow (B03-S03). It must never read, copy, or import the OpenCode CLI auth file `~/.local/share/opencode/auth.json`, and no key may appear in source, project state, fixtures, logs, screenshots, issue evidence, Wiki pages, or CI artifacts.
- Authentication failures, an unavailable subscription, and unsupported models are normalized errors (B03-S05), never silent fallbacks.

## Client identity

The Go documentation states that a client should:

- send typical coding-agent traffic rather than synthetic bulk requests;
- identify itself with **its own user agent**, such as `my-coding-agent/1.0`, instead of a generic SDK or HTTP-library name — BrainRoot sends a BrainRoot-specific user agent;
- send a **stable session ID in `x-opencode-session`** for each conversation so routing and prompt caching can be optimized.

The session value is nonsecret, stable per conversation, and must not encode credentials, user identity, or file content.

## Model discovery

- `https://opencode.ai/zen/go/v1/models` returns the full list of currently available Go models and their metadata.
- Discovery happens at runtime through the Rust core; only nonsecret metadata is cached, with timestamp and expiry, and an explicit offline/stale state is shown when the endpoint is unreachable.
- The catalog observed on 2026-09-22 contained roughly thirty entries across the Grok, GLM, GPT, Kimi, LongCat, MiMo, MiniMax, Muse Spark, Qwen, DeepSeek, and Hy families. This list is an **external snapshot for documentation only** and must not be copied into product logic.

## Protocol endpoints

The Go page maps each model to one of three protocol endpoint families:

| Family | Endpoint | SDK style |
|---|---|---|
| OpenAI Responses | `https://opencode.ai/zen/go/v1/responses` | `@ai-sdk/openai` |
| OpenAI-compatible chat | `https://opencode.ai/zen/go/v1/chat/completions` | `@ai-sdk/openai-compatible` |
| Anthropic messages | `https://opencode.ai/zen/go/v1/messages` | `@ai-sdk/anthropic` |

- Not every model speaks the same wire protocol; the mapping is external and mutable.
- MVP-0 selects **one** currently available low-cost coding model and **one** protocol path through explicit criteria (availability, coding capability, retention policy, latency, limits); code must not assume every Go model shares a protocol and must not silently fall back to another model, protocol, paid Zen balance, or endpoint.
- Base URL, endpoint path, and model ID live in validated provider configuration, not in presentation code.

## Usage limits

- Go is a subscription with monthly dollar limits per model; the documentation describes 5-hour, weekly, and monthly windows as **20 % / 50 % / 100 %** of the monthly limit respectively, and token prices as per 1M tokens.
- Prices, limits, and model availability are externally mutable; the product surfaces usage-limit errors as plain-language states (B03-S05) and never assumes a specific allowance.

## Privacy and retention

- The Go page publishes a per-model table with **Model training** (`Not used` in the 2026-09-22 snapshot) and **Data retention** (for example 30 days for some models, 0 days for others).
- Retention and training policies are externally mutable and apply per model, not per subscription.
- BrainRoot exposes the current model's privacy/retention information before the first live use when the endpoint provides it, and never claims a policy the endpoint does not state.

## Mutable external facts

The following are **externally mutable** and must be re-verified on each relevant execution date: the model catalog and model IDs, per-model protocol endpoint mapping, base URLs, usage limits and prices, privacy and retention fields, client-identity guidance, and authentication header shape. A contradiction between this snapshot and the official documentation stops the microstep that discovers it (`docs/18` stop conditions) rather than being patched silently.

## Implementation obligations

- **B03-S02:** fetch and parse the current models endpoint through the core; filter only entries whose protocol MVP-0 supports; cache nonsecret metadata with timestamp/expiry and expose a stale/offline state; contract tests use captured minimal fixtures with unknown fields.
- **B03-S03:** Linux credential storage through an approved Secret Service integration or a documented, honest fallback; environment injection only for local development or the opt-in live test and never persisted; prove the key cannot cross the frontend contract or appear in logs.
- **B03-S04:** one validated Go protocol path with the BrainRoot user agent and a stable nonsecret `x-opencode-session` per conversation; configuration-based base URL and model ID; no silent fallback.
- **B03-S05:** normalize invalid key, unavailable subscription, unsupported model, limit reached, timeout, provider failure, malformed response, and network unavailable; expose privacy/retention before first live use when available.
- **B03-S06:** opt-in live smoke test, skipped without the documented environment variable, never run on untrusted contributions with secrets; a harmless fixed prompt; redact response bodies; record only safe timing/status evidence.

## Sources

- OpenCode Go guide: <https://opencode.ai/docs/go> (read 2026-09-22).
- OpenCode provider configuration: <https://opencode.ai/docs/providers> (read 2026-09-22).
- OpenCode Zen endpoints, authentication example, and privacy: <https://opencode.ai/docs/zen> (read 2026-09-22).
