# B07 — Companion Browser plan integration (documentation only)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B07-Companion-Browser-Plan). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: Integrate the maintainer-adopted Companion Browser plan into the reviewed documentation set — the plan file under `docs/specs/` and cross-references in the owning documents — without implementing any browser capability and without turning the CB-A…CB-D candidates into executable issues.
- Branch: `batch/b07-companion-browser-plan` (deleted after merge)
- Draft/final PR: [#60](https://github.com/AlexandreZanata/brain-root-idea/pull/60)
- Merge commit: `334f28ef3ecf6473bb44fc615588853e3d878741`
- Baseline commit: `0e759cb98b5e0472288d1da4d2f25244f126a47b`
- Target/resulting version: `0.0.3` (annotated tag on the merge commit; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.3) without artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: not applicable (documentation-only; offline `bash`/`sh` gates)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Any browser, WebView, preview, dev-server, Playwright, or navigation implementation.
- Any application code, test, script, workflow, dependency, capability, IPC, permission, or ADR change.
- Treating the CB-A…CB-D candidates as executable issues or assigning batch IDs.
- Remote browsing, profile/data import, Deck switching, or phone presentation.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B07-S01 | [#59](https://github.com/AlexandreZanata/brain-root-idea/issues/59) | The adopted Companion Browser plan (`41836fd`) is integrated byte-identical into `docs/specs/companion-browser-plan.md` and referenced from `docs/05`, `docs/08`, `docs/15`, and `docs/17`, applied to current `main` content instead of copying branch-era file versions | `2e3b18b` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/59#issuecomment-5795618891) | None | Closed |
| B07-S02 | [#61](https://github.com/AlexandreZanata/brain-root-idea/issues/61) | Finalization: version `0.0.3` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.3` section; this history record created; PR #60 marked Ready, merged with a merge commit through the documented single-maintainer administrator bypass, branch deleted, annotated `v0.0.3` tagged with a pre-release, and this record completed on `main` | `0f7a488`, `334f28e` (merge) | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/61) | Merge and release used the documented single-maintainer administrator bypass; no artifact for this batch | Closed |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B07-S01): the plan is integrated as documentation only. The CB-A…CB-D candidate lists remain planning inputs; the CB-A prerequisites become executable issues only after B07 merges and the relevant architecture spike is accepted, as a separate follow-up step tracked on the batch PR.
- Decision (B07-S02): a documentation-only batch still changes the version exactly once at finalization (`0.0.3`) per `docs/19-release-and-versioning.md`, with no artifact. Resolution at merge: the authenticated owner cannot self-approve, so the merge used the documented single-maintainer administrator path in `docs/specs/repository-rules-runbook.md`; the annotated tag was created only after the merge commit existed on `main`.

## Failures and recovery

- None. B07-S01 integrated the plan on the first pass; no gate failed and no fixture or gate was weakened.

## Final gates

- Full CI: [`check-full-linux` success on the finalization head](https://github.com/AlexandreZanata/brain-root-idea/actions/runs/35867403771) (8 m 5 s, `0f7a488`); post-merge `push` runs on `main` for `334f28e` and the finalization commit are recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/61) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B07-Companion-Browser-Plan)
- Review: single-maintainer administrator bypass documented; merged at B07-S02
- Release: annotated `v0.0.3` on merge commit `334f28e`; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.3) without artifact
- Security/privacy: `check-docs.sh` secret scan green (180 files) and `check-security.sh` green; no runtime surface or destination added
- Performance: not applicable (documentation-only batch)
- Cleanup: no process, listener, timer, PTY, or network resource created
- Artifact/checksum: none for this batch
- Known limitations: the Companion Browser itself remains unimplemented; no preview, Human Browser, import, Deck, or phone capability exists, and CB batch IDs are not assigned

## Result and next batch

Batch B07 is released as [`v0.0.3`](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.3) on merge commit `334f28e`: the adopted Companion Browser plan and its cross-references are reviewable on `main`, with the local-preview-first boundary and the excluded import categories stated explicitly. The browser capability itself remains out of scope. This final record was completed in one documented post-merge commit on `main` through the administrator bypass because the merge commit, CI run, tag, and release URL cannot exist before the merge. Next: a separate follow-up step turns the CB-A prerequisites — owned dev-server/process lifecycle, approved localhost origin policy, and the Linux WebView probe — into executable issues per `docs/18-mvp-execution-plan.md` §5, assigning batch IDs only after this batch merges and the spike decision. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier.
