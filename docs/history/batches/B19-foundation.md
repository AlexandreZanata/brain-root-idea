# B19 — Foundation, shared controls, shell, edge, and the Lean pivot

> Batch record for [B19-S08](https://github.com/AlexandreZanata/brain-root-idea/issues/127), extended in place when the maintainer's [Lean YAGNI pivot](https://github.com/AlexandreZanata/brain-root-idea/issues/128) was added to the same branch. B18 merged into `main` at `1dbf7de`; this record covers the versioned `0.0.15` gate that ran afterwards.
>
> **Process deviation, stated rather than hidden:** the pivot's fourteen commits were written before any per-microstep issue existed, contrary to `docs/18-mvp-execution-plan.md` §4.2. Issue #128 is a tracking issue opened afterwards, not a Definition-of-Ready issue. The scope was expanded on the same branch instead of a separate batch, so this record covers two deliveries and the batch's original non-goals no longer fully describe what shipped. The branch was pushed and opened as Draft PR [#129](https://github.com/AlexandreZanata/brain-root-idea/pull/129) on 2026-09-25, then marked Ready so the required check runs on the final head, and the matching Wiki page was published; no merge and no tag were created by this task.

## Header

- Status: Release gate — `CI_PENDING` on the submitted head `7feda2c` (no merge, tag, or release in this task)
- Objective: (a) give the frontend one token scale, explicit button primitives, and harmonized shell surfaces without changing behavior, providers, or trust boundaries; (b) add an on-demand coding agent — a Rust-owned `opencode serve --pure` sidecar with sessions, a live model catalog, agent/cost modes, a turn ledger, and idle enforcement — under the release-gate test cadence of ADR 0014.
- Branch: `batch/b19-foundation`, pushed to `origin`, 25 commits ahead of `main` (based on B18 tip `f4919f4`; `main` advanced to `1dbf7de`, and `git diff f4919f4 1dbf7de` is MEASURED empty, so an update from `main` is purely topological and adds no content)
- Baseline: B18 tip for the B19 slices; the pivot added new surfaces, so cross-release comparison is by same-session measurement only
- Target version: `0.0.15` (set at this gate)
- Draft/final PR: [#129](https://github.com/AlexandreZanata/brain-root-idea/pull/129) — opened Draft, then Ready for the release gate; `CI_PENDING`
- Wiki batch page: [Batch-B19-Foundation](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B19-Foundation) — created 2026-09-25 together with the [Batch Index](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-Index), [Current Status](https://github.com/AlexandreZanata/brain-root-idea/wiki/Current-Status), and [Release Index](https://github.com/AlexandreZanata/brain-root-idea/wiki/Release-Index) reconciliation (B18 had merged without its Wiki row being updated)
- Artifact: `BrainRoot_0.0.15_amd64.deb`, bytes 3,537,182, sha256 `ec15ec7cb2cfc2dcc782055a4e17a73629753a9f4158fce9984b3929152a4a01`
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, Wayland, WebKitGTK 2.52.6, rustc 1.96.0)

## Non-goals (as originally written for the B19 slices)

- New providers, agents, tools, dependencies, fonts, icon packs, frameworks, animations.
- Any change to the provider/core protocol, permissions, persistence, or trust boundaries.
- Palette-hue changes (contrast was never measured; only documented gaps).
- Migration of B20-owned surfaces (turns, cards, suggestions, Canvas, Browser, Preview controls).
- Per-microstep test execution (ADR 0014): tests written per microstep, executed at this gate.

**Correction:** the first two bullets above were written before the pivot and are no longer accurate for this branch. The pivot added an agent, a new process, new outbound destinations, and new IPC commands without going through a permission-boundary change. That is recorded as an unresolved blocker in `docs/17-open-questions.md`, not as an accepted exception.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B19-S00 | [#119](https://github.com/AlexandreZanata/brain-root-idea/issues/119) | Adopts the Codex plan verbatim plus a provisional B18-tip rebaseline (inventory, measured baseline, gaps) | `f144107` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/119#issuecomment-5815747310) | Override: branch from B18 tip, no PR/Wiki, provisional numbers | Closed, docs only |
| B19-S01 | [#120](https://github.com/AlexandreZanata/brain-root-idea/issues/120) | Freezes the 20-file visual inventory with cited defects, TARGET after-states, owner column, and reference mapping | `5adcc2b` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/120#issuecomment-5815811384) | Same override; screenshots unavailable (reason recorded) | Closed, docs only |
| B19-S02 | [#121](https://github.com/AlexandreZanata/brain-root-idea/issues/121) | Semantic token foundation in `theme.css` (spacing/type/10-6px radii/heights/focus) with in-file dedupe; static token/contrast cases written and registered | `47b538e` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/121#issuecomment-5815983629) | Same override; palette hues untouched; per-variant heights deferred to S03 | Closed, `IMPLEMENTED_UNVERIFIED` |
| B19-S03 | [#122](https://github.com/AlexandreZanata/brain-root-idea/issues/122) | Button/icon primitives: 40 px base, 36 px compact tab/rail, group hook, long-label containment, square icon variant (no consumers yet); no surface migrated | `12b3dd4` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/122#issuecomment-5816087641) | Same override; Icon.svelte verified unchanged | Closed, `IMPLEMENTED_UNVERIFIED` |
| B19-S04 | [#123](https://github.com/AlexandreZanata/brain-root-idea/issues/123) | Composer on the group hook (equal Send/Cancel peers), field on body type, scoped spacing tokenized; handlers byte-identical | `f8f7640` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/123#issuecomment-5816132370) | Same override; TextArea verified unchanged; Browser controls left for B20 | Closed, `IMPLEMENTED_UNVERIFIED` |
| B19-S05 | [#124](https://github.com/AlexandreZanata/brain-root-idea/issues/124) | Header on display/supporting tokens with scale spacing; markup and failure copy byte-identical; brand-mark geometry kept | `e2f007e` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/124#issuecomment-5816213437) | Same override; ThemeToggle verified unchanged; compact-height decision deferred to S08 probe | Closed, `IMPLEMENTED_UNVERIFIED` |
| B19-S06 | [#125](https://github.com/AlexandreZanata/brain-root-idea/issues/125) | Narrow rail buttons fixed to content-sized chips (`flex: 0 0 auto; width: auto`); components verified unchanged | `65f242d` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/125#issuecomment-5816256430) | Same override | Closed, `IMPLEMENTED_UNVERIFIED` |
| B19-S07 | [#126](https://github.com/AlexandreZanata/brain-root-idea/issues/126) | Edge contract audited item-by-item; workspace/resizer chrome tokenized; drag lifecycle byte-identical | `3bf4b19` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/126#issuecomment-5816310369) | Same override; PanelResizer verified unchanged | Closed, `IMPLEMENTED_UNVERIFIED` |
| B19-S08 | [#127](https://github.com/AlexandreZanata/brain-root-idea/issues/127) | Provisional verification gate: full suite, soak, absolute measurements; no versioning | `223d2d6` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/127) | Version/PR/Wiki/Ready deferred post-merge | Closed, provisional |
| B19-S08 (finalization) | none — local task | Versioned `0.0.15` gate: Clippy and the governor/active-turn race fixed, `LICENSE`/`NOTICE` added to the `.deb`, full Linux suite green, soak and measurements re-run, version/changelog/docs synchronized, PR opened | `c52f917`, `7feda2c` | this record; [PR #129](https://github.com/AlexandreZanata/brain-root-idea/pull/129) | No issue was created for the gate remediation; the pivot deviation is disclosed above | Pushed, `CI_PENDING`, in review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Pivot scope added to the same branch (issue #128)

Fourteen commits, oldest first, none with a preceding microstep issue. Deferred test IDs and rollback notes live in the per-slice specs under `docs/specs/R*.md`.

| Unit | Commits | Outcome | Spec |
|---|---|---|---|
| Plan + R1 spike | `7758aeb`, `b4721c8` | Lean YAGNI plan; pinned `opencode 1.18.31` with measured RSS, `/doc`, and the finding that `/config/providers` returns the provider key inline (mitigation: identifiers only, never forward the payload) | `21-lean-yagni-pivot.md`, `R1-harness-spike.md` |
| Agent Host | `fbd2482` | `opencode serve --pure` as a Rust-owned child: start/health/stop, filtered catalog, session send over SSE | `R2-agent-host-s01.md`, `R2-agent-send-s04.md` |
| Tabs + host badge + send path | `300b5da`, `dfc5f72` | Session tabs, redacted host status, per-submit send-path choice with the legacy loop as fallback | `R2-agent-tabs-s02.md`, `R2-models-s03.md`, `R2-agent-path-s05.md` |
| Catalog merge | `27aa887` | Public OpenRouter context/prices merged onto sidecar ids, 6 h TTL, honest `stale`, `UNKNOWN` instead of a guess | `R3-catalog-s01.md` |
| Token discipline | `6785b8e`, `0bb59da`, `78d2215` | Pinned minimal system prompt, Plan/Build selection, Fast/Balanced/Max profiles over live prices | `R4-token-s01.md`, `R4-agent-toggle-s02.md`, `R4-profiles-s03.md` |
| Turn ledger | `0bbc117` | Per-turn tokens and cost read from the sidecar session, best-effort | `R4-cost-s04.md` |
| Governor | `4346bc1`, `389e08e`, `dfc5f72` | Single-thread idle enforcement, visible policy and auto-stop count, report-only preview/browser clocks | `R5-governor-s01.md`, `R5-governor-s02.md`, `R5-clocks-s03.md` |
| Gate remediation + artifact | `d8cfeb6`, `26676f3` | Three defects fixed (invalid JSON fixture, TS syntax in an `.mjs` test, focus-attribute violations); `.deb` cut with checksum | `R5-release-s03.md` |

## Decisions and changed assumptions

- Decision (#119): maintainer override recorded — B19 work proceeds pre-merge as docs-then-CSS slices with deviations disclosed per issue; rebase + freeze + PR + Wiki follow the B18 merge.
- Decision (S02): tokenize at TARGET values; palette hues frozen; per-variant control heights belong to S03.
- Decision (S03): group/icon hooks provided with zero migrated consumers; surfaces adopt per packet.
- Decision (S08): gate runs provisional without versioning — bumping past unreleased `0.0.14` pre-merge would collide; `0.0.15` reserved for the post-merge versioned gate.
- Decision (finalization): `0.0.15` is cut on the local head now that B18 is merged into `main` at `1dbf7de`. The maintainer chose a combined B19 + pivot delivery, so this record covers both and states the process deviation instead of presenting it as the planned workflow.
- Decision (finalization): an unfinished turn is authoritative over the idle budget. The earlier "`last_used` updates on send/status" claim only held for turns shorter than 60 s, so `stop_if_idle` now refuses to stop while a send is in flight; corrected in `docs/specs/R5-governor-s01.md`.
- Decision (finalization): `LICENSE` and `NOTICE` ship inside the `.deb` through `bundle.linux.deb.files`, closing the recorded ADR 0010 packaging gap. The `0.0.14` artifact recorded earlier is superseded by the `0.0.15` re-cut.
- Decision (finalization): the agent permission-boundary blocker stays **open** and is disclosed in `docs/17-open-questions.md`, `docs/11`, the README, and the changelog. No Safe Mode claim is made for the agent path.

## Failures and recovery

- No gate failures: all 42 frontend tests passed on first execution (including the five never-run theme cases), the full suite passed, and the soak stayed bounded. No remediation was needed.
- Transient SSH push retry at S05 and S07 (`access rights` error, success on immediate retry); no history impact.

## Versioned gate at `0.0.15` (application head `0bbc117` plus the finalization changes)

- `sh scripts/check-full-linux.sh`: **OK** — fmt, clippy `-D warnings`, Rust tests (172 passed, 0 failed, 3 ignored by design), svelte-check 0/0, vite production build, Tauri release build, security, modules, accessibility, fake-provider e2e (cargo 2 passed, frontend 53 passed), process smoke (readiness observed, clean close, no listeners), docs, version consistency, license artifacts.
- `pnpm run test:frontend`: 53 passed, 0 failed.
- Release soak (ignored, explicit): 1 passed — 500 + 500 fake conversations and 200 cancellations, RSS 34,920 → 35,636 KiB (bounded), threads 2 → 2.
- Performance, absolute (`BRAINROOT_MEASURE_SKIP_BUILD=1`, host load 5.16 — busy): startup n=10 median **1.269 s**, p95 1.309 s (range 1.249–1.310) → the ≤ 1.0 s p50 TARGET is **missed**; settled idle CPU median 0.103 % across 5 windows (p95 2.055 %) → `< 1 %` median passes; process-tree PSS median 249.3 MB (brainroot 86.3 / WebKitWebProcess 145.1 / WebKitNetworkProcess 18.4) → the 150 MB target **fails as since B01**; binary 8,226,216 B; JS 102,259 B (gzip 34,389); CSS 20,416 B (gzip 4,041). The host was busy and this head adds the whole agent surface, so no cross-release regression is claimed from these absolutes.
- Defects found and fixed by this gate (tracked here, commit-scoped): two `clippy -D warnings` failures (`manual_div_ceil`, `let_unit_value`) that had stopped the gate before its builds; the governor could stop the sidecar during an unfinished turn because `last_used` is set only at send start; and the `.deb` omitted `LICENSE`/`NOTICE`. Each has a unit test or artifact check, and the governor's corrected guarantee is recorded in `docs/specs/R5-governor-s01.md`.
- Wall-clock budget: the ≤ 1.0 s startup TARGET is missed on this busy host while the added agent surface and a heavier code path are also in play; this needs a quiet-host re-measurement, not an excuse.
- Deferred/manual: peer-box equality, keyboard/focus order, measured contrast beyond static ratios, narrow behavior, 100-cycle soak, the interactive sidecar round trip, and the 65 s idle auto-stop probe — all `UNKNOWN` (no GUI automation and no real credential in this task); helper-level and unit cases passed in-suite.
- Security/privacy: the sidecar password is ephemeral and in-memory, `agent_host_status` has no secret field, the providers payload is filtered to identifiers with a serialization test, and the outbound allowlist permits only the loopback sidecar origin plus the public OpenRouter catalog. **The agent has no workspace or permission boundary** — see the blocker in `docs/17-open-questions.md`.
- CI: `CI_PENDING` on the submitted head `7feda2c`; the Ready-for-review push triggered `linux-full` run 36149987256 ([PR #129](https://github.com/AlexandreZanata/brain-root-idea/pull/129)). The local full Linux suite is green on this head. The required check's result is recorded only when known — no polling happened inside the task.
- Artifact/checksum: `BrainRoot_0.0.15_amd64.deb` with the recorded sha256 above, now including `LICENSE`/`NOTICE`. Clean-room install/remove was **not** executed (requires root).
- Known limitations: this record covers two deliveries on one branch; phone width is not device emulation; WebView focus escape out of scope; Files/Terminal/Changes stay planned entries; ADR 0015 remains Proposed.

## Result and next batch

The branch is versioned at `0.0.15`, pushed, and green on the local head, but it is **not yet mergeable**: the submitted head `7feda2c` is `CI_PENDING`, the security blocker below is unresolved, the B19 scope was widened by the pivot without a separate batch, and review is outstanding. Immediate next steps, in order: (1) close the agent permission-boundary blocker or explicitly scope it out of the advertised capability — an unmediated agent must not merge into an advertised capability while this stands; (2) take one latest-head CI snapshot (pending means report and stop, failed means a tracked remediation issue, green means proceed); (3) obtain review, then merge and tag only after green latest-head CI and resolved conversations — the Wiki batch page is published and its index/status pages reconciled. Note that an update from `main` needs no content resolution (see the Header). Next product batch: B20 conversation/Canvas polish, only after this merges.

Rollback: revert the finalization commit and the three scoped remediation edits; `main` and the B18 merge commit are untouched. The artifact in `target/` is ignored by git.
