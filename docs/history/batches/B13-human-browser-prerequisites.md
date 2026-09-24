# B13 — Human Browser prerequisites (CB-B gate)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B13-Human-Browser-Prerequisites). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: Resolve the CB-B prerequisites before any Human Browser implementation is assigned — approve the explicit Human Browser policy and prove a persistent, isolated BrainRoot-owned profile — without shipping remote browsing or reordering the roadmap.
- Branch: `batch/b13-human-browser-prerequisites` (deleted after merge)
- Draft/final PR: [#84](https://github.com/AlexandreZanata/brain-root-idea/pull/84)
- Merge commit: `314275ee1d4ca1f6778fb13dba3af2a95f0a026e`
- Baseline commit: `065b813` (B12 released)
- Target/resulting version: `0.0.9` (annotated tag on the merge commit; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.9) without artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Any Human Browser UI, navigation, Deck switching, or remote content in the product.
- Importing bookmarks, history, preferences, or credentials; reading installed browser profiles.
- Any agent access or sharing flow.
- Reordering the roadmap: CB-B stays post-MVP; this batch only clears its prerequisites.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B13-S01 | [#83](https://github.com/AlexandreZanata/brain-root-idea/issues/83) | The Human Browser policy is approved as ADR 0013: fourth trust role with no BrainRoot IPC, arbitrary `http`/`https` after explicit user direction, denied schemes/popups/downloads/permission prompts, a BrainRoot-owned persistent profile separate from every other role and installed browsers, one remote HOT view with honest COLD semantics, no ambient sharing, and the implementation explicitly post-MVP; linked from `docs/08`, `docs/11`, and `docs/17` | `4ba7c94` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/83#issuecomment-5800518261) | The approval is prerequisite work only; no CB-B batch ID is assigned and the roadmap order is unchanged | Closed |
| B13-S02 | [#85](https://github.com/AlexandreZanata/brain-root-idea/issues/85) | The profile isolation probe measured `go`: two role profiles with separate `WebContext` data directories are isolated in both directions, persist across destroy/recreate, live under the application data directory, clean up (50 323 bytes measured), and record `no-go` on any failed check; recorded in `docs/specs/human-profile-isolation-probe.md` and `docs/17` | `fcf580a` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/85#issuecomment-5800577130) | Cookie/cache isolation, deletion/reset, crash recovery, and accessibility inside remote content remain for the implementation batch | Closed |
| B13-S03 | [#86](https://github.com/AlexandreZanata/brain-root-idea/issues/86) | Finalization: version `0.0.9` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.9` section; this history record created; PR #84 marked Ready and the full Linux gate run on the latest head; merge/tag deferred to maintainer approval | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/86) | The authenticated repository owner cannot approve its own PR; no tag is created before the merge commit exists on `main` | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B13-S01): the first Human Browser slice supports arbitrary `http`/`https` after explicit user direction, not a curated destination set; the isolation boundary (no IPC, separate profile, denied permissions) is what makes that acceptable, and a curated mode can be added later as a restriction.
- Decision (B13-S01): the profile is BrainRoot-owned under the application data directory, never an installed browser directory; agents have no access; deletion/reset are explicit user actions with confirmation.
- Decision (B13-S01): the policy is approved as a prerequisite while the implementation stays post-MVP (Phase 8); the roadmap order (Phases 3, 5, 6, 7) is unchanged unless the maintainer reorders.
- Decision (B13-S02): the CB-B profile isolation prerequisite is met with the same `wry`/WebKitGTK stack the product uses — separate role profiles are isolated and persistent, BrainRoot-owned, and clean up; the remaining evidence (cookies/cache, deletion/reset, crash, accessibility) belongs to the implementation batch.
- Decision (B13-S03): a prerequisites-only batch still changes the version exactly once at finalization (`0.0.9`) per `docs/19-release-and-versioning.md`, with no artifact. Resolution at merge: the authenticated owner cannot self-approve, so the merge used the documented single-maintainer administrator path; the annotated tag was created only after the merge commit existed on `main`.

## Failures and recovery

- None. Both microsteps passed on the first green run; no gate failed and no fixture or gate was weakened.

## Final gates

- Full CI: [`check-full-linux` success on the finalization head](https://github.com/AlexandreZanata/brain-root-idea/actions/runs/35903298949) (8 m 28 s, `4cca217`); post-merge `push` runs on `main` for `314275e` and the finalization commit are recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/86) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B13-Human-Browser-Prerequisites)
- Review: single-maintainer administrator bypass documented; merged at B13-S03
- Release: annotated `v0.0.9` on merge commit `314275e`; [pre-release](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.9) without artifact
- Security/privacy: `check-docs.sh` secret scan green (205 files); `check-security.sh` green with the dependency list unchanged; the policy denies risky schemes, popups, downloads, and permission prompts by default; the probe only used BrainRoot-owned paths and its local fixture
- Performance: probe-scale only; a fresh profile measured 50 323 bytes; no resident resource added
- Cleanup: the probe namespace was removed and no process, listener, or fixture server remained
- Artifact/checksum: none for this batch
- Known limitations: the Human Browser remains unimplemented; cookie/cache isolation, deletion/reset, crash recovery, accessibility inside remote content, curated mode, share flow, and retention UX stay open for the implementation batch

## Result and next batch

Batch B13 is released as [`v0.0.9`](https://github.com/AlexandreZanata/brain-root-idea/releases/tag/v0.0.9) on merge commit `314275e`: the Human Browser policy is approved with explicit boundaries, and the profile isolation prerequisite is measured `go` with a persistent BrainRoot-owned profile that is separate from the Preview, shell, and installed browsers. The Human Browser itself remains unimplemented and post-MVP. This final record was completed in one documented post-merge commit on `main` through the administrator bypass because the merge commit, CI run, tag, and release URL cannot exist before the merge. Next: the roadmap's next capability step (Phases 3/5/6/7) or the CB-B implementation batch once the maintainer schedules it. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier.
