# B00 — Governance and delivery controls

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B00-Governance). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Ready to merge
- Objective: Establish the experimental delivery controls — license and attribution verification, issue and PR templates, labels/milestone/branch rules, history and Wiki, version/changelog controls, and the initial documentation CI.
- Branch: `batch/b00-governance`
- Draft/final PR: [#2](https://github.com/AlexandreZanata/brain-root-idea/pull/2)
- Baseline commit: `695bee3a41274f60044b9641852dfc1883ac0741`
- Target/resulting version: `0.0.1-alpha.1` (single source `VERSION`)
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
| B00-S04 | [#5](https://github.com/AlexandreZanata/brain-root-idea/issues/5) | Full Wiki structure published with the flat `Batch-B00-Governance` page; repository mirror created Active with validated bidirectional links | `7e75121` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/5#issuecomment-5776816232) | Nested Wiki URLs are not plain deep links; flat convention adopted and recorded in `docs/20` | Closed |
| B00-S05 | [#6](https://github.com/AlexandreZanata/brain-root-idea/issues/6) | Single machine-readable version source (`VERSION`), `CHANGELOG.md`, and an offline consistency check for the source, changelog sections, and future ecosystem copies | `b7c0034` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/6#issuecomment-5776927405) | Ecosystem copies do not exist yet; the check is dormant and enforces once B01 scaffolds Cargo/npm/Tauri | Closed |
| B00-S06 | [#7](https://github.com/AlexandreZanata/brain-root-idea/issues/7) | Initial Linux CI workflow produces the required `check-full-linux` check (docs links/sections, secrets, integrity, version consistency, license/NOTICE); version, changelog, runbook, and records finalized; PR merged with the documented administrator exception; annotated tag and pre-release `v0.0.1-alpha.1` | `<merge>` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/7) | Single-maintainer approval exception documented; post-merge record commit carries the merge/tag/CI links | Closed |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision/ADR: [ADR 0010](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/adr/0010-project-license.md) (Apache-2.0 with NOTICE attribution) unchanged.
- Decision (B00-S03, maintainer): `main` protection is active with administrator bypass (`enforce_admins: false`) because the repository has a single maintainer identity; recorded as a gap in the runbook, together with the required Linux check deferred to B00-S06.
- Decision (B00-S04): Wiki pages use flat names (`Batch-Bxx-Name`, `Release-vX.Y.Z`) because plain nested URLs return 404; recorded in `docs/20-project-history-and-wiki.md`.
- Decision (B00-S05, maintainer): the root `VERSION` file is the single machine-readable version source (`UNRELEASED` until B00-S06), enforced by `scripts/check-version-consistency.sh`; recorded in `docs/19-release-and-versioning.md`.
- Decision (B00-S06, maintainer): the required check is named `check-full-linux`; the approval gate is satisfied through the documented administrator bypass because the repository has one maintainer identity; a GitHub pre-release is created for `v0.0.1-alpha.1`.
- Decision (B00-S06): the initial CI excludes `scripts/check-governance-templates.sh` because the runner image does not provide PyYAML to the default `python3`; B01-S04 adds it after B01-S01 freezes the environment. No installation was added.
- Evidence: `LICENSE` is byte-identical to the canonical Apache-2.0 text, sha256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.
- Result: pinned by `scripts/check-license-artifacts.sh` for CI and release packaging.

## Failures and recovery

- 2026-09-22, B00-S01: the Wiki git repository did not exist until the maintainer created the first page through the web UI; the microstep row was recorded immediately after (accepted process limitation).
- 2026-09-22, B00-S04: nested Wiki page URLs are not directly reachable; the page was renamed to the flat `Batch-B00-Governance` and the convention recorded. Historical evidence comments are intentionally unchanged. See the [Wiki Failure Log](https://github.com/AlexandreZanata/brain-root-idea/wiki/Failure-Log).
- 2026-09-22, B00-S06: the runner image does not provide PyYAML to the default interpreter (yamllint and ansible are pipx-isolated), so `scripts/check-governance-templates.sh` cannot run in the initial CI without an installation; the check stays local until B01-S04 wires it after the environment is frozen.

## Final gates

- Full CI: `check-full-linux` on the batch head and the post-merge `push` run on `main` — links recorded in issue #7
- Review: single-maintainer exception; merged with the documented administrator bypass (no second identity exists)
- Security/privacy: `scripts/check-docs.sh` secret-pattern scan and manual diff review; no credential or authorization header in any artifact
- Performance: not applicable for B00
- Cleanup: not applicable (no runtime process); temporary fixtures removed
- Artifact/checksum: none for this batch; the pre-release states the absence explicitly
- Known limitations: nested Wiki URLs return 404 (flat convention in use); the governance template check is not in the initial CI; administrator bypass remains documented

## Result and next batch

Batch B00 delivers the governance and delivery controls: verified license/attribution artifacts, repository templates with structural validation, documented labels/milestone/branch rules, synchronized Wiki and repository history, a single version source with changelog controls, and the first Linux CI check required on `main`. Version `0.0.1-alpha.1` is the batch output; there is no user-visible application capability yet.

Next batch: B01 — minimal measured Linux shell (`batch/b01-linux-shell`), starting with the Linux reference environment and dependency evidence. The merge commit, tag, release, and CI links are finalized in the documented post-merge record commit because they cannot exist before the merge.
