# ADR 0015 (draft): opencode serve --pure as default sidecar

**Status:** Proposed  
**Date:** 2026-09-25

## Context

BrainRoot needs a real coding agent without building a harness. R1 spike pins `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769` (`opencode 1.18.31`) and measures: full sidecar ~435MB RSS at 3s, `--pure` ~311MB, `/doc` 479KB, `/config/providers` 33KB, unauthenticated `/doc`/`/config` HTTP 401, `/config/providers` leaks the provider key inline. The sidecar alone exceeds the UI idle budget, so it must never be resident. See `docs/specs/R1-harness-spike.md` and `docs/21-lean-yagni-pivot.md`.

## Decision

Run `opencode serve --pure --hostname 127.0.0.1` as an on-demand child of the Rust Agent Host with an ephemeral `OPENCODE_SERVER_PASSWORD` per session, killed on idle 60s/session end/close. Full plugin set only via explicit allowlist. Never forward raw provider payloads to the frontend or logs; expose only IDs plus configured/not-configured.

## Alternatives considered

- **Full `opencode serve` default:** rejected for R1 — costs ~120MB extra RSS with no product need proven.
- **Embed OpenCode UI bundle / Electron desktop:** rejected — brings Solid/Electron runtime into the idle budget.
- **dsh web as default:** rejected — Node-resident Cordis stack, developer preview breaking changes; keep `dsh --profile acp` optional behind a flag.
- **Custom Rust agent loop as product path:** rejected — keep MVP-0 transport only as deterministic fake for tests.

## Consequences

Agent Host owns spawn/health/auth/sse/kill and maps server events to product events. Provider key handling needs a filter plus a denial test. Pin must be re-verified when opencode changes auth, `/doc`, or provider schema.

## Performance implications

Sidecar is measured separately from the ≤150MB UI idle budget. Use `scripts/measure-rss.sh`, `scripts/perf-baseline.sh`, `scripts/perf-soak.sh` for every change touching spawn, plugins, streaming, or idle policy. Termination must leave zero orphans.

## Security implications

Basic-auth password is ephemeral and never persisted. Unauthenticated server access must fail closed (401 observed). Provider secrets stay in Rust/keychain; redaction is release-gated.

## Reversibility

High: swap `--pure` for allowlisted full, or swap sidecar for another ACP server, without changing the product event model. Pin rollback is one SHA change.

## References

[opencode server](https://opencode.ai/docs/server), [opencode repo](https://github.com/anomalyco/opencode), [Models.dev](https://models.dev), [OpenRouter docs](https://openrouter.ai/docs), [pi.dev](https://pi.dev), [dsh acp-app](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/bundle/acp-app/README.md), `docs/specs/R1-harness-spike.md`.
