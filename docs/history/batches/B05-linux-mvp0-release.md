# B05 — Hardening and release Linux MVP-0

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B05-Linux-MVP0-Release). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Release candidate
- Objective: A reproducible experimental Linux build demonstrates the complete MVP-0 loop — launch → Ready → configured fake → discover → prompt → stream → complete → second prompt → cancel → close — with security/privacy, performance/soak, packaging, documentation, and release evidence, while the product itself still has no tools, project mutation, or Canvas.
- Branch: `batch/b05-linux-mvp0-release` (deleted after merge)
- Draft/final PR: [#44](https://github.com/AlexandreZanata/brain-root-idea/pull/44)
- Merge commit: pending (finalized at B05-S07)
- Baseline commit: `430f8743a3535360b51c922a4067b8669c99f639`
- Target/resulting version: `0.0.1` (set at B05-S06; annotated tag on the merge commit)
- Started/completed: 2026-09-22 / —
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned)

## Non-goals

- New product capability: tools, shell, project mutation, Canvas/preview, checkpoints, Code View.
- Windows, macOS, updater, marketplace, cloud sync, collaboration, telemetry, local models.
- Multiple providers or simultaneous conversations; universal Linux compatibility.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B05-S01 | [#43](https://github.com/AlexandreZanata/brain-root-idea/issues/43) | Fake-provider end-to-end journey in `features::conversation::e2e` (completion, cancellation, neutral error, joined workers) plus `scripts/e2e-linux.sh` with the frontend bounds and the launch/readiness/close smoke; wired into `check-full-linux` | `8003634` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/43#issuecomment-5783085641) | `model_id` needed `#[cfg(test)]` to keep release builds warning-free; the product journey is in-process because no UI automation is approved | Closed |
| B05-S02 | [#45](https://github.com/AlexandreZanata/brain-root-idea/issues/45) | Offline security gate (`check-security.sh`) for destinations, frontend surface, telemetry, capabilities, log hygiene, and dependency allowlists; injectable live transport with oversized/malformed/failed-response tests; OSV audit clean | `1c3551f` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/45#issuecomment-5783205969) | Two documented test-only URL fixtures; OSV requires network and is recorded as dated evidence, not a gate | Closed |
| B05-S03 | [#46](https://github.com/AlexandreZanata/brain-root-idea/issues/46) | Release soak (500+500 conversations, 200 cancellations; stable threads/RSS; dispatch 0 µs) and `scripts/soak-linux.sh` open/close cycles; report `b05-mvp0-soak` with startup pass, idle CPU warn, memory fail | `a926c14` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/46#issuecomment-5783414924) | Measurements are evidence, not CI; the memory TARGET fails and stays failing; live cancel latency `UNKNOWN` | Closed |
| B05-S04 | [#47](https://github.com/AlexandreZanata/brain-root-idea/issues/47) | Documented Debian artifact (`BrainRoot_0.0.1-alpha.5_amd64.deb`) built by `scripts/package-linux.sh`; runtime deps, checksum, contents, install/run/remove, and limitations recorded | `08b0a99` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/47#issuecomment-5783542149) | The deb is not bit-reproducible (per-build checksum); install/remove are verified at B05-S07 | Closed |
| B05-S05 | [#48](https://github.com/AlexandreZanata/brain-root-idea/issues/48) | Documentation and history synchronized: README status, MVP-0 status, measured performance status, architecture/security evidence, open questions, changelog, and this record | pending (B05-S05 commit) | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/48) | The record is finalized with merge/CI/tag/checksum at B05-S07 | Closed |
| B05-S06 | release candidate | pending | — | — | — | Planned |
| B05-S07 | merge, tag, release | pending | — | — | — | Planned |

## Decisions and changed assumptions

- Decision (B05-S01): the product journey runs in-process against `ConversationSession` because no UI automation is approved; the process-level launch/readiness/close is `scripts/smoke-linux.sh`, run by `scripts/e2e-linux.sh` when a display exists or `BRAINROOT_E2E_SMOKE=1`. The configured fake is deterministic, so the cancel step uses a controllable runner through the same product API.
- Decision (B05-S02): release security is enforced offline by `scripts/check-security.sh` (destinations, frontend surface, telemetry, capabilities, log hygiene, dependency allowlists) plus an injectable live transport so oversized/malformed/failed responses are tested without network access; OSV advisories are dated evidence.
- Decision (B05-S03): the soak measures the core in-process and stays out of CI; the release report labels startup/cancellation/cleanup as measured passes, idle CPU a warn, memory a failing TARGET, and live cancellation latency `UNKNOWN`.
- Decision (B05-S04): one packaging format only — Tauri's Debian bundle for the Ubuntu 24.04 family x86_64 — with the artifact outside Git and its checksum recorded per build; no universal Linux claim.
- Changed assumption (B04-S03/B05): the authenticated models payload omits per-model endpoints; discovery accepts the shape (B04-R01) and the live smoke passes end to end (`models=33`, chat `chunks=1`, normal terminal).

## Failures and recovery

- 2026-09-22, B05-S01: `model_id` was dead code in release builds and broke `clippy -D warnings`; the accessor is now `#[cfg(test)]` with a fully qualified type.
- 2026-09-22, B05-S03: idle CPU windows exceeded the budget on a host in active use; recorded as a warn, not a pass, and not generalized.
- 2026-09-22, B05-S03/S04: the total-memory TARGET fails (426.2 MB summed RSS / 241.6 MB PSS) and the Debian artifact is not bit-reproducible (timestamped archive); both are recorded as accepted limitations rather than hidden.

## Final gates

- Full CI: pending (B05-S06/S07)
- Review: pending; single-maintainer administrator bypass documented
- Security/privacy: `check-security.sh` gate and OSV audit clean on 2026-09-22; secret scans clean; no telemetry or unapproved destination
- Performance: [b05-mvp0-soak](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/specs/performance-reports/b05-mvp0-soak.md) — startup and cancellation pass; idle CPU warn; memory TARGET fails; live cancellation latency UNKNOWN
- Cleanup: the e2e journey and soak assert zero active requests and joined workers; eight open/close cycles left no process, listener, or extra WebView
- Artifact/checksum: `BrainRoot_0.0.1-alpha.5_amd64.deb`, 3,384,838 bytes, sha256 `ea4ca51b…` (per-build; `docs/specs/b05-linux-artifact.md`); the `0.0.1` artifact is rebuilt and attached at B05-S07
- Known limitations: unsigned experimental Debian artifact for the Ubuntu family x86_64; memory TARGET fails; live cancellation latency bounded by the transport read; the rendered window still has no UI automation

## Result and next batch

In progress: batch B05 is hardening the proven MVP-0 loop for the first reproducible experimental release (`0.0.1`) with security, performance, packaging, and documentation evidence. This record is finalized at B05-S07 with the merge commit, CI runs, tag, release URL, and release artifact checksum. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier.
