# B19 — Foundation, shared controls, shell, and edge (provisional record)

> Provisional batch record for [B19-S08](https://github.com/AlexandreZanata/brain-root-idea/issues/127). No batch PR, Wiki page, version, tag, or release exists yet: B18 PR #108 was still OPEN (CI green, review pending) when this ran, so verification executed under a recorded maintainer override with versioning/PR/Ready explicitly deferred to a post-merge follow-up. Finalize through the batch PR once it exists.

## Header

- Status: Provisional verification gate (no version bump, no Ready handoff)
- Objective: Give the frontend one token scale, explicit button primitives, and harmonized shell surfaces (composer, header, rail, edge/workspace) without changing behavior, providers, or trust boundaries — under the release-gate test cadence of ADR 0014.
- Branch: `batch/b19-foundation` (based on B18 tip `f4919f4`, not `main` — rebase follows the B18 merge)
- Baseline: B18 tip (provisional; freeze to the merge SHA post-merge)
- Target version: none assigned (stays `0.0.14` inherited from the B18 tip; `0.0.15` reserved for the post-merge versioned gate)
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, Wayland, WebKitGTK 2.52.6, rustc 1.96.0)

## Non-goals

- New providers, agents, tools, dependencies, fonts, icon packs, frameworks, animations.
- Any change to the provider/core protocol, permissions, persistence, or trust boundaries.
- Palette-hue changes (contrast was never measured; only documented gaps).
- Migration of B20-owned surfaces (turns, cards, suggestions, Canvas, Browser, Preview controls).
- Per-microstep test execution (ADR 0014): tests written per microstep, executed at this gate.

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
| B19-S08 | [#127](https://github.com/AlexandreZanata/brain-root-idea/issues/127) | Provisional verification gate (this record): full suite, soak, absolute measurements; no versioning | pending commit | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/127) | Version/PR/Wiki/Ready deferred post-merge | Open |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (#119): maintainer override recorded — B19 work proceeds pre-merge as docs-then-CSS slices with deviations disclosed per issue; rebase + freeze + PR + Wiki follow the B18 merge.
- Decision (S02): tokenize at TARGET values; palette hues frozen; per-variant control heights belong to S03.
- Decision (S03): group/icon hooks provided with zero migrated consumers; surfaces adopt per packet.
- Decision (S08): gate runs provisional without versioning — bumping past unreleased `0.0.14` pre-merge would collide; `0.0.15` reserved for the post-merge versioned gate.

## Failures and recovery

- No gate failures: all 42 frontend tests passed on first execution (including the five never-run theme cases), the full suite passed, and the soak stayed bounded. No remediation was needed.
- Transient SSH push retry at S05 and S07 (`access rights` error, success on immediate retry); no history impact.

## Final gates (provisional — application head `3bf4b19`)

- `pnpm run test:frontend`: 42 passed, 0 failed (37 inherited + 5 theme cases).
- `sh scripts/check-full-linux.sh`: OK — fmt/clippy/tests (151 passed, 0 failed, 3 ignored by design), svelte-check 0/0, vite + Tauri release builds, security/modules/accessibility, fake-provider e2e (cargo 2 passed, frontend 42 passed), smoke (readiness observed; clean exit; no listeners), docs, version consistency, license artifacts.
- Release soak (ignored, explicit): 1 passed — 1200 fake conversations, RSS 34,072 → 35,124 KiB (bounded), threads 2 → 2.
- Performance, absolute (`BRAINROOT_MEASURE_SKIP_BUILD=1`, host load 7.15 — very busy): startup n=10 median 0.796 s, p95 0.839 s (TARGET ≤ 1.0 s p50 pass); idle CPU median 0.103 % across 5 windows (`< 1 %` pass); PSS median 276.6 MB (one early window sampled 7.8 MB before WebKit spawn; 150 MB target fails as since B01); binary 8,085,672 B; JS 87,604 B (gzip 30,206); CSS 18,238 B (gzip 3,706, +1.5 KB tokens/group/icon rules vs B18). Host-noise dominates cross-run deltas — compare against the post-merge same-session B18/B19 pair, not against these absolutes.
- Deferred/manual: peer-box equality, keyboard/focus order, measured contrast beyond static ratios, narrow behavior, 100-cycle soak — all `UNKNOWN` (no GUI automation); helper-level bounds cases passed in-suite.
- Security/privacy: no privileged or protocol change; secret scan green per microstep and in the full gate.
- CI: none (no batch PR exists by design); full CI runs when the post-merge PR becomes Ready.
- Artifact/checksum: none for this batch.
- Known limitations: provisional base (B18 tip, not merged); phone width is not device emulation; WebView focus escape out of scope; Files/Terminal/Changes stay planned entries.

## Result and next batch

Batch B19-foundation is verified but unreleased and unversioned by design: tokens, primitives, and four harmonized shell surfaces sit on the provisional branch with green gates and measured absolutes. Post-merge follow-up (tracked, in order): rebase onto `main`, freeze numbers to the merge SHA, open the batch Draft PR + Wiki page, cut `0.0.15`, re-run the versioned gate with the same-session B18/B19 comparison, mark Ready with `CI_PENDING`. Rollback: delete the branch and close the issues; `main` and the B18 branch are untouched. Next: B20 conversation/Canvas polish per the plan, only after B19 merges.
