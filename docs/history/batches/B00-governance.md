# B00 — Governance and delivery controls

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B00-Governance). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Active
- Objective: Establish the experimental delivery controls — license and attribution verification, issue and PR templates, labels/milestone/branch rules, history and Wiki, version/changelog controls, and the initial documentation CI.
- Branch: `batch/b00-governance`
- Draft/final PR: [#2](https://github.com/AlexandreZanata/brain-root-idea/pull/2) (Draft)
- Baseline commit: `695bee3a41274f60044b9641852dfc1883ac0741`
- Target/resulting version: `UNRELEASED` (B00-S05 introduces the version source; B00-S06 sets `0.0.1-alpha.1`)
- Started/completed: 2026-09-22 / —
- Supported test environment: Linux reference environment, not frozen yet (B01-S01)

## Non-goals

- Application scaffold, Tauri/Svelte code, provider calls, credentials, network destinations.
- Filesystem tools, shell tools, PTYs, LSPs, dev servers, WebViews, Playwright, checkpoints, Code View, ACP, multiple providers, Windows, macOS, updater, cloud sync, plugins, publishing.
- Version numbers before B00-S05/S06.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B00-S01 | [#1](https://github.com/AlexandreZanata/brain-root-idea/issues/1) | Offline manifest check pins the unmodified Apache-2.0 `LICENSE` and the BrainRoot `NOTICE`, failing closed on tamper, missing artifact, or SPDX metadata drift | `8846393` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/1#issuecomment-5776486688) | Bootstrap ordering: draft PR opened after the first branch commit because GitHub requires a diff | Closed |
| B00-S02 | [#3](https://github.com/AlexandreZanata/brain-root-idea/issues/3) | Microstep issue form and batch PR template installed under `.github/`, validated offline by a structural check that fails on malformed YAML, missing fields, missing sections, and checklist drift | `dc2d068` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/3#issuecomment-5776610244) | Live GitHub render preview only after the batch merges (default-branch templates); structural conformance used until then | Closed |
| B00-S03 | [#4](https://github.com/AlexandreZanata/brain-root-idea/issues/4) | Repository rules runbook records the 18-label scheme, milestone, and configured `main` protection (PR required, 1 approval, conversation resolution, no force push/deletion); admin bypass and the missing Linux check documented as gaps | `7176661` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/4#issuecomment-5776689307) | Administrator bypass intentional for a single maintainer; required status check deferred to B00-S06 | Closed |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision/ADR: [ADR 0010](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/adr/0010-project-license.md) (Apache-2.0 with NOTICE attribution) unchanged.
- Decision (B00-S03, maintainer): `main` protection is active with administrator bypass (`enforce_admins: false`) because the repository has a single maintainer identity; recorded as a gap in the runbook, together with the required Linux check deferred to B00-S06.
- Decision (B00-S04): Wiki pages use flat names (`Batch-Bxx-Name`, `Release-vX.Y.Z`) because plain nested URLs return 404; recorded in `docs/20-project-history-and-wiki.md`.
- Evidence: `LICENSE` is byte-identical to the canonical Apache-2.0 text, sha256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.
- Result: pinned by `scripts/check-license-artifacts.sh` for CI and release packaging.

## Failures and recovery

- 2026-09-22, B00-S01: the Wiki git repository did not exist until the maintainer created the first page through the web UI; the microstep row was recorded immediately after (accepted process limitation).
- 2026-09-22, B00-S04: nested Wiki page URLs are not directly reachable; the page was renamed to the flat `Batch-B00-Governance` and the convention recorded. Historical evidence comments are intentionally unchanged. See the [Wiki Failure Log](https://github.com/AlexandreZanata/brain-root-idea/wiki/Failure-Log).

## Final gates

- Full CI: pending (B00-S06)
- Review: pending
- Security/privacy: pending (B00-S06)
- Performance: not applicable for B00
- Cleanup: not applicable (no runtime process)
- Artifact/checksum: none
- Known limitations: nested Wiki URLs return 404; flat naming convention in use

## Result and next batch

In progress. Next microstep: B00-S05 — establish version and changelog controls. The batch merges only after every B00 issue closes with evidence, full Linux CI is green on the latest head, review approves, and this record matches the Wiki batch page.
