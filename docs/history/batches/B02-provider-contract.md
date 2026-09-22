# B02 — Provider contract and deterministic fake

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B02-Provider-Contract). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Ready to merge
- Objective: The Rust core owns one versioned, provider-neutral contract and a deterministic fake provider that can list models, stream text, complete, cancel, and produce every normalized failure without a real API key, network, or background worker.
- Branch: `batch/b02-provider-contract`
- Draft/final PR: [#17](https://github.com/AlexandreZanata/brain-root-idea/pull/17)
- Merge commit: pending (recorded post-merge)
- Baseline commit: `46431ff14ae39b316fbef3f984792c74b2ba7c02`
- Target/resulting version: `0.0.1-alpha.3` (annotated tag on the merge commit; pre-release)
- Started/completed: 2026-09-22 / —
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, rustc 1.96.0 pinned)

## Non-goals

- Real OpenCode request, endpoint, model catalog, production credential, or any network destination.
- Final chat UI, conversation branching, code edits, tools, checkpoints.
- Multiple providers, autonomous loops, Windows, macOS.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B02-S01 | [#16](https://github.com/AlexandreZanata/brain-root-idea/issues/16) | Versioned provider-neutral contract with validating newtypes, request/validation, stream events, completion, cancellation, normalized errors, limits, and a pure request-state transition; 14 tests | `3e3607a` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/16#issuecomment-5779053209) | `cargo fmt --check` failed on the first gate run; the module carries a documented `#[allow(dead_code)]` until the fake consumes it | Closed |
| B02-S02 | [#18](https://github.com/AlexandreZanata/brain-root-idea/issues/18) | Deterministic fake provider: fixed two-model list and scripted wire scenarios (success, auth, rate limit, server error, malformed frame, delayed, never-ending) with time as data; 10 tests | `355e007` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/18#issuecomment-5779120877) | No clock, thread, network, or randomness exists in the module | Closed |
| B02-S03 | [#19](https://github.com/AlexandreZanata/brain-root-idea/issues/19) | Pull-based bounded execution: shared cancellation token, connect/read/total timeouts, response limit, exactly one terminal, script released on termination; 12 tests | `9c16cb6` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/19#issuecomment-5779204194) | `cargo fmt --check` failed on the first gate run; the secret scan matched a variable named `token` | Closed |
| B02-S04 | [#20](https://github.com/AlexandreZanata/brain-root-idea/issues/20) | Byte-level normalizer: streaming JSON decode across UTF-8/JSON splits, 64 KiB buffer and 64-event queue with typed backpressure, neutral mapping, plain-language errors, one terminal; 14 tests | `8e3a2f9` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/20#issuecomment-5779277257) | `ErrorCode` gained `timed_out` and `response_too_large`; `QueueFull` consumes nothing so retry is safe | Closed |
| B02-S05 | [#21](https://github.com/AlexandreZanata/brain-root-idea/issues/21) | Core-only credential boundary: status is the only serializable type, values/headers/references redact themselves, in-memory fake store, `provider_status` app command; 10 tests | `102ad29` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/21#issuecomment-5779342097) | The only secret is an obviously fake test literal; no capability or real store exists | Closed |
| B02-S06 | [#22](https://github.com/AlexandreZanata/brain-root-idea/issues/22) | Contract integration test: credential pre-flight, discovery, request, streaming to completion, cancellation, plain-language failure, idle with no background work; 4 tests | `7e1b414` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/22#issuecomment-5779485224) | The executor's synthetic terminal after a wire `end`/`error` frame is ignored by the consumer once the normalizer finished (first terminal wins) | Closed |
| B02-S07 | [#23](https://github.com/AlexandreZanata/brain-root-idea/issues/23) | Version `0.0.1-alpha.3` synchronized across `VERSION`, `Cargo.toml`, `package.json`, and the lockfile; changelog release section; full CI on the latest head; PR merged with the documented administrator exception; annotated tag and pre-release | `<merge>` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/23) | Version copies were included in the allowlist from the start, learning from B01-S07 | Ready |

## Decisions and changed assumptions

- Decision (B02-S01): the contract lives in `src-tauri/src/provider/contract.rs`, is provider-neutral, uses `deny_unknown_fields` on every struct, and documents size limits (message 16 KiB, model ID 256 B, display name 200 B, conversation ID 128 B, stream event 64 KiB, accumulated response 2 MiB).
- Decision (B02-S02): the fake speaks a documented wire format (JSON frames) with `Delay`/`Hang` as scripted data; no clock trait is needed.
- Decision (B02-S03): execution is pull-based and deterministic with no thread, timer, async runtime, or dependency; cancellation is exactly-once through the owner or the shared token.
- Decision (B02-S04): frames are self-delimiting JSON; the normalizer buffers bytes with `serde_json` streaming, bounds to 64 KiB and 64 events, never surfaces provider text, and lets the first terminal win.
- Decision (B02-S05): the credential boundary is core-only; `CredentialStatus` is the sole serializable type and `provider_status` is the first provider app command (no capability needed).
- Decision (B02-S06): the integration test lives in a `#[cfg(test)]` module of the binary crate; the consumer stops at the first terminal; `dead_code` allowances stay until the B04 UI consumes the modules.

## Failures and recovery

- 2026-09-22, B02-S01/S03: `cargo fmt --check` failed on the first gate run in both microsteps; formatted before the gates passed (no behavior change).
- 2026-09-22, B02-S06: the first integration run failed with `AlreadyTerminal` because the executor emits a synthetic `Completed` after a wire `end`/`error` frame; the consumer now stops forwarding at the first terminal, mirroring the UI's late-event rule. No semantic change to the executor or normalizer.
- 2026-09-22, B02-S05: the secret scan matched the identifier in `let token = CancellationToken::new();`; a variable name, not a credential.

## Final gates

- Full CI: `check-full-linux` on the batch head and the post-merge `push` run on `main` — recorded in issue #23
- Review: single-maintainer exception; merged with the documented administrator bypass
- Security/privacy: core-only credentials, redacted debug output, no provider text surfaced, no network, bounded buffers and queues
- Performance: not applicable to this batch (no runtime resource)
- Cleanup: no worker, thread, or timer exists; integration tests leave nothing running
- Artifact/checksum: none for this batch; the pre-release states the absence explicitly
- Known limitations: boundary not wired to the UI yet (B04), no live transport (B03), staged `dead_code` allowances remain

## Result and next batch

In progress. Next microstep: B02-S07 finalization. Batch merges only after every B02 issue closes with evidence, full Linux CI is green on the latest head, and this record matches the Wiki batch page.
