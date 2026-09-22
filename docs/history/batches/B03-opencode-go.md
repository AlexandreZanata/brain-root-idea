# B03 — OpenCode Go live transport

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B03-Opencode-Go). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: A Linux developer can configure an OpenCode Go credential without exposing it, discover the currently available models through the core, and run an opt-in live smoke request through one validated protocol path with the BrainRoot user agent and a stable nonsecret session header — while the provider-neutral contract from B02 stays unchanged.
- Branch: `batch/b03-opencode-go` (deleted after merge)
- Draft/final PR: [#25](https://github.com/AlexandreZanata/brain-root-idea/pull/25)
- Merge commit: pending (filled by the documented post-merge finalization commit)
- Baseline commit: `53a4b83a7ab7dd897baba5d3fe09edc2f815f20a`
- Target/resulting version: `0.0.1-alpha.4` (annotated tag on the merge commit; pre-release URL filled post-merge)
- Started/completed: 2026-09-22 / —
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned)

## Non-goals

- Autonomous tools, code edits, checkpoints, chat UI (B04), browser preview.
- Reading or importing the OpenCode CLI auth file `~/.local/share/opencode/auth.json`.
- Multiple simultaneous conversations, multiple providers, provider SDK types crossing the core/UI boundary.
- A default request timeout or cancellation wiring for live execution; only the opt-in smoke is bounded in this batch.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B03-S01 | [#24](https://github.com/AlexandreZanata/brain-root-idea/issues/24) | Official OpenCode Go contract captured in `docs/specs/opencode-go-contract.md`: auth header, BrainRoot user agent, `x-opencode-session`, discovery endpoint, three protocol families, usage limits, privacy/retention, mutability markers and sources | `14e46bb` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/24#issuecomment-5779926854) | The providers page does not restate the Go auth header; the Zen page documents the `Authorization: Bearer` shape with the same key and is cited | Closed |
| B03-S02 | [#26](https://github.com/AlexandreZanata/brain-root-idea/issues/26) | Model discovery in `provider::discovery`: injectable `HttpTransport` + `UreqTransport` (ureq 3.4.2/rustls, no async runtime), tolerant parsing of the models endpoint, protocol-family filter, and a bounded cache with explicit time (`Fresh`/`Stale`/`Empty`); 11 tests, all offline | `1d484e1` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/26#issuecomment-5780057087) | `DiscoveryError` gained `TransportUnavailable`; the live response shape stayed unverified until the opt-in smoke | Closed |
| B03-S03 | [#27](https://github.com/AlexandreZanata/brain-root-idea/issues/27) | Secret Service credential storage: injectable `SecretBackend` + `KeyringBackend` (keyring 4.2.0/zbus), `SecretServiceCredentialStore`, truthful `unavailable` status and session-only fallback, fallible `set`; 8 tests plus an opt-in real-keyring round trip that wrote, read, and deleted a labeled smoke entry | `55ebb48` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/27#issuecomment-5780197706) | `platform_store()` remains unwired until B04; the real check is `#[ignore]`d so CI never touches a user keyring | Closed |
| B03-S04 | [#28](https://github.com/AlexandreZanata/brain-root-idea/issues/28) | Validated `chat/completions` path in `provider::go`: config restricted to the documented endpoint with default model `glm-5.3-flash`, random nonsecret `x-opencode-session` per conversation, typed request body, injectable POST transport, and an SSE parser mapping the real stream to neutral events; 14 tests, all offline | `170aef4` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/28#issuecomment-5780325650) | `Content-Type: application/json` added to the headers; the live payload stays unverified until an authorized smoke run | Closed |
| B03-S05 | [#29](https://github.com/AlexandreZanata/brain-root-idea/issues/29) | Shared `GoFailure` classification in `provider::failure` (invalid key, subscription unavailable, unsupported model, rate/usage limit, timeout, provider failure, malformed response, network unavailable) with distinct plain-language messages and no contract change; discovery exposes bounded per-model `privacy.training`/`retention` as `Stated`/`Unknown` | `a00ec2c` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/29#issuecomment-5780475837) | Re-verification (no credential): the public models payload states no per-model `endpoint`/privacy fields and the unauthenticated chat call answers `401` with an `AuthError` envelope; the authenticated shape remains unverified | Closed |
| B03-S06 | [#30](https://github.com/AlexandreZanata/brain-root-idea/issues/30) | `provider::live_smoke` opt-in test (`#[ignore]` + `BRAINROOT_LIVE_SMOKE=1` + `BRAINROOT_OPENCODE_GO_KEY`): live discovery must return a non-empty catalog and one fixed prompt must stream at least one chunk to a `Completed` terminal; `UreqGoTransport`/`UreqTransport` gained explicit `with_timeout` (120 s B02 budget), defaults unchanged | `a1eaaea` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/30#issuecomment-5780554621) | Guards proven offline (default run ignores it; switch without key fails closed without network); the authorized live run is pending and is the first check of the authenticated models shape | Closed |
| B03-S07 | [#31](https://github.com/AlexandreZanata/brain-root-idea/issues/31) | Version `0.0.1-alpha.4` synchronized across `VERSION`, `Cargo.toml`, `package.json`, and the lockfile; changelog release section; final history record; PR made Ready; full CI on the latest head; merge commit; annotated tag and pre-release | pending (merge) | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/31) | Ecosystem copies plus the history record were in the allowlist; this record is finalized by the documented post-merge commit | Closed |

## Decisions and changed assumptions

- Decision (B03-S01): every external fact is a dated snapshot; the model catalog is summarized with a mutability warning instead of being copied into product logic.
- Decision (B03-S01): the record names `~/.local/share/opencode/auth.json` only as forbidden input; BrainRoot owns its own credential flow (B03-S03).
- Decision (B03-S02): `ureq` 3.4.2 with `json`/rustls defaults is the HTTP client (blocking, no async runtime; `reqwest` rejected for tokio and 47 direct deps); the resolved tree is 378 unique entries with `rustls` 0.23.45 and OSV-clean locked versions; discovery tests never touch the network.
- Decision (B03-S03): `keyring` 4.2.0 (default `v1` → Secret Service via zbus) is the storage path, wrapped by an injectable backend; the boundary reports `unavailable` and falls back to session-only rather than persisting plaintext; the resolved tree is 471 unique entries and OSV-clean.
- Decision (B03-S04): the MVP-0 protocol path is `chat/completions` with default model `glm-5.3-flash`; configuration is validated data restricted to the documented endpoint; `x-opencode-session` is a random nonsecret UUID per conversation; the SSE parser reuses the byte-buffer discipline and there is no silent fallback.
- Decision (B03-S05): one shared `GoFailure` table classifies status/transport conditions into existing neutral codes — `401` invalid key and `402`/`403` subscription → `authentication_failed`, `404` unsupported model → `invalid_input`, `429` → `rate_limited`, timeouts → `timed_out`, `5xx`/network → `provider_unavailable`, unreadable payload → `malformed_response` — with no contract, version, or dependency change; privacy values are exposed only when the payload states them and otherwise stay explicitly `unknown`.
- Changed assumption (B03-S05): the public `GET /zen/go/v1/models` payload (40 entries, 2026-09-22) contains only `id`/`object`/`created`/`owned_by` per entry, so discovery's `endpoint` filter produces an empty catalog until an authorized live run verifies the authenticated shape; remediation is owned by the batch close if it matches the public shape.
- Decision (B03-S06): the live smoke is `#[ignore]`d and additionally gated by `BRAINROOT_LIVE_SMOKE=1` plus `BRAINROOT_OPENCODE_GO_KEY`; the switch off skips, the switch on without the key fails closed before any request; evidence prints only counts, field names, and timing; the explicit `with_timeout` (120 s, the B02 total budget) is used only by the smoke, so default transport behavior is unchanged and no default timeout is wired yet.
- Decision (B03-S07): the live smoke is not part of the required deterministic CI path; its authorized run is recorded separately, and this record must state `UNKNOWN — not executed` until such a run exists.

## Failures and recovery

- 2026-09-22, B03-S05 observation: the public `GET /zen/go/v1/models` response (no credential) lists 40 entries with only `id`/`object`/`created`/`owned_by`; discovery's per-entry `endpoint` filter therefore yields an empty catalog. Reproduction: `curl -sS https://opencode.ai/zen/go/v1/models`. Impact: live discovery may need a shape update if the authenticated response matches the public one. Disposition: deferred to an authorized live run; a remediation issue must own the fix if confirmed. Not a confirmed failure — the authenticated shape is unverified.
- 2026-09-22, B03-S06: the first compile of `live_smoke.rs` failed with `E0599` because `HttpTransport` was not in scope for `UreqTransport::get`; fixed by importing the trait (no behavior change).
- 2026-09-22, B03-S06: the opt-in live smoke guards were proven offline (default run ignores it; switch without key fails closed with no network). The authorized live run had not been executed when this record was written.

## Final gates

- Full CI: pending (filled by the documented post-merge finalization commit)
- Review: single-maintainer exception; merge performed with the documented administrator bypass (`enforce_admins: false`)
- Security/privacy: core-only credential storage and redaction; bounded failure classification that never surfaces provider text; opt-in smoke guards; secret-pattern scans in the gates; no new network destination beyond the documented OpenCode endpoints
- Performance: not applicable to this batch (no runtime resource; requests are on demand and bounded by the caller)
- Cleanup: no thread, timer, or background resource exists; the live smoke drops its reader, parser, session, and credential before returning
- Artifact/checksum: none for this batch; the pre-release states the absence explicitly
- Known limitations: public models payload states no per-model endpoint/privacy; authenticated shape and `402`/`403`/`404` semantics live-unverified; the live smoke is published but not executed as required evidence; in-stream provider errors map generically; the credential store is not wired to the application yet (B04)

## Result and next batch

Batch B03 is released as `v0.0.1-alpha.4`: the OpenCode Go live transport — current model discovery with a bounded cache, Linux Secret Service credential storage with an honest session fallback, one validated `chat/completions` path with BrainRoot client identity, complete Go failure normalization, per-model privacy disclosure, and an opt-in live smoke test — all behind the unchanged provider-neutral contract, without a UI, artifact, or default live request. Rollback: revert the batch commits; after merge the merge commit stays unless a maintainer explicitly reverts it, and the published tag is never moved. This final record is completed by the documented post-merge commit on `main` because the merge commit, CI runs, tag, and release URL cannot exist before the merge.

Next batch: B04 — Minimal conversation experience (`batch/b04-conversation-ui`), starting with explicit conversation state.
