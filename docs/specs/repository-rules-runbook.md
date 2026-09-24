# Repository rules runbook

**Status:** Active
**Applies to:** `AlexandreZanata/brain-root-idea`
**Last verified:** 2026-09-22 (issue #4, B00-S03; required check added in issue #7, B00-S06)

## Purpose

Records the GitHub label scheme, milestone, and `main` branch protection that the BrainRoot delivery workflow in `docs/18-mvp-execution-plan.md` depends on, including the known gaps and the exact commands to re-apply and verify each rule. GitHub settings live outside the repository, so this file is the durable operational copy and must be updated whenever a rule changes.

## Required labels

Every microstep issue carries `batch:Bxx`, exactly one `type:*`, exactly one `risk:*`, and `agent:economical`. `status:blocked` is added only while a stop condition exists. These 18 labels are required and were verified against the live repository on 2026-09-22.

| Label | Color | Description |
|---|---|---|
| `batch:B00` | `0E7490` | Batch B00 - governance and delivery controls |
| `batch:B01` | `0E7490` | Batch B01 - minimal measured Linux shell |
| `batch:B02` | `0E7490` | Batch B02 - provider-neutral contract and deterministic fake |
| `batch:B03` | `0E7490` | Batch B03 - OpenCode Go live transport |
| `batch:B04` | `0E7490` | Batch B04 - minimal conversation experience |
| `batch:B05` | `0E7490` | Batch B05 - Linux MVP-0 hardening and release |
| `type:docs` | `0075CA` | Documentation-only change |
| `type:build` | `5319E7` | Build system, tooling, or packaging change |
| `type:frontend` | `1D76DB` | Frontend Svelte/TypeScript change |
| `type:rust` | `DEA584` | Rust core change |
| `type:test` | `0E8A16` | Tests and verification change |
| `type:security` | `B60205` | Security or permission boundary change |
| `type:release` | `FBCA04` | Release, versioning, or distribution change |
| `risk:low` | `0E8A16` | Low risk microstep |
| `risk:medium` | `FBCA04` | Medium risk microstep |
| `risk:high` | `B60205` | High risk microstep |
| `agent:economical` | `6F42C1` | Executed by an economical agent under docs/18 |
| `status:blocked` | `D73A4A` | Stop condition active: missing fact, decision, or permission |

## Default labels

GitHub's default labels (`accessibility`, `bug`, `documentation`, `duplicate`, `enhancement`, `good first issue`, `help wanted`, `invalid`, `question`, `wontfix`) are also present. They are not part of the required workflow set and are intentionally left untouched; classification uses the required labels above.

## Milestone

| Title | State |
|---|---|
| `MVP-0 Linux Model Loop` | open |

One milestone covers the Linux MVP-0 batches. A later platform or release milestone is created only when `docs/18` schedules it.

## Branch protection

`main` uses classic branch protection, configured on 2026-09-22 (issue #4, B00-S03).

| Rule | Value |
|---|---|
| Require a pull request before merging | enabled |
| Required approving reviews | 1 |
| Dismiss stale reviews on new commits | enabled |
| Require conversation resolution | enabled |
| Enforce for administrators | disabled (documented gap) |
| Required status checks | `check-full-linux` (`strict: true`), added by B00-S06 |
| Allow force pushes | disabled |
| Allow deletions | disabled |

Applied payload:

```json
{
  "required_status_checks": null,
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "required_conversation_resolution": true,
  "allow_force_pushes": false,
  "allow_deletions": false
}
```

## Gaps and pending work

- **Administrator bypass:** `enforce_admins` is disabled because the repository has a single maintainer identity; the required approval is therefore not enforced for the administrator. The rule exists and applies to any other collaborator. Revisit when a second reviewer identity exists.
- **Required status check:** `check-full-linux` is required on `main` (`strict: true`) and is produced by `.github/workflows/ci.yml` on `pull_request` (non-draft) and `push` to `main`. The initial check runs the documentation/secret/integrity gate, the version consistency gate, and the license/NOTICE artifact gate. `scripts/check-governance-templates.sh` is not part of this workflow because the runner image does not provide PyYAML to the default interpreter; B01-S04 adds it after B01-S01 freezes the Linux reference environment.
- **Single-maintainer approval:** with one identity, `required_approving_review_count: 1` cannot be satisfied by another person, which is why the administrator bypass above is required for the batch to merge. This is recorded as a gap, not as success.
- **Classic protection versus rulesets:** classic branch protection is used because it is simple and reversible; if repository needs grow (multiple actors, bypass lists), migrate to a ruleset and update this runbook.
- **Evidence drift:** GitHub settings cannot be enforced from the repository. `docs/20-project-history-and-wiki.md` requires a maintainer-visible comparison at batch close; future automation may clone settings read-only.

**B17 policy update (2026-09-24):** the bullet above records the original B00 workflow. Under ADR 0014, `check-full-linux` stays required by branch protection but is generated only for a non-draft, versioned batch `pull_request`; the duplicate `push`-to-`main` full run is removed. Verify the repository's required-check settings still name this unique job before adopting the updated workflow.

## Apply and verify

```sh
# Re-apply protection (payload file contains the JSON from "Branch protection")
gh api --method PUT repos/AlexandreZanata/brain-root-idea/branches/main/protection --input protection.json

# Verify protection
gh api repos/AlexandreZanata/brain-root-idea/branches/main/protection \
  --jq '{approvals: .required_pull_request_reviews.required_approving_review_count, dismiss_stale: .required_pull_request_reviews.dismiss_stale_reviews, enforce_admins: .enforce_admins.enabled, conversation_resolution: .required_conversation_resolution.enabled, allow_force_pushes: .allow_force_pushes.enabled, allow_deletions: .allow_deletions.enabled, required_status_checks: .required_status_checks}'

# Verify labels and milestone
gh api repos/AlexandreZanata/brain-root-idea/labels --paginate
gh api repos/AlexandreZanata/brain-root-idea/milestones
```

Enabled by B00-S06 through the full protection replacement (`PUT /branches/main/protection`) because the status-check sub-resource did not exist while checks were unset; once enabled, use the sub-resource to change checks:

```sh
gh api --method PATCH repos/AlexandreZanata/brain-root-idea/branches/main/protection/required_status_checks \
  -f strict=true -F contexts[]=check-full-linux
```

## Evidence

- Applied and verified in issue #4 ([B00-S03](https://github.com/AlexandreZanata/brain-root-idea/issues/4)); exported protection settings and label/milestone checks are recorded in the issue evidence comment without credentials.
- Required check added and verified in issue #7 ([B00-S06](https://github.com/AlexandreZanata/brain-root-idea/issues/7)); the first green `check-full-linux` run and the post-merge `main` run are recorded there.
