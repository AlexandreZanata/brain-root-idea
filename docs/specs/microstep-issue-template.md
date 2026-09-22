# [Bxx-Syy] <single observable outcome>

## Metadata

- Batch: `Bxx`
- Batch branch: `batch/bxx-name`
- Batch PR: `<draft PR URL>`
- Starting commit: `<full SHA>`
- Target milestone: `MVP-0 Linux Model Loop`
- Risk: low | medium | high
- Agent profile: economical

## Outcome

One sentence describing what is observably true when this issue is done.

## Required reading

- `<file or official source>` — why it is needed

Do not add broad reading “for context.” Give the agent only the files needed for this microstep.

## Facts and fixed decisions

- Fact:
- Decision already made:
- Assumption that must be tested:

## Allowed changes

- `<exact file or narrow directory>`

## Forbidden scope

- No unrelated refactor.
- No new dependency unless explicitly listed below.
- No change to permissions, persistence, network destinations, or public contracts unless explicitly listed.
- Add issue-specific exclusions.

## Exact implementation sequence

1. `<mechanical step>`
2. `<mechanical step>`
3. `<mechanical step>`

## Dependency authorization

- New dependency allowed: yes | no
- If yes: capability, existing alternatives, runtime/bundle/startup impact, transitive count, maintenance, license, advisories, lazy-load behavior, and removal path.

## Validation commands and expected evidence

1. Command: `<exact command>`
   Expected: `<specific observable result>`
2. Command: `<exact command>`
   Expected: `<specific observable result>`
3. Changed-file command: `git diff --name-only <starting SHA>...HEAD`
   Expected: only allowlisted paths.

## Negative/security/cleanup checks

- [ ] Required failure state tested.
- [ ] No secret or authorization header in diff/log/artifact.
- [ ] No resource remains active after completion/cancel/close, when applicable.
- [ ] No unrelated test regressed.

## Rollback

Explain how to remove only this microstep without discarding other user/batch work.

## Stop conditions

List issue-specific conditions in addition to `docs/18-mvp-execution-plan.md`. The agent comments `BLOCKED` and stops when any condition is met.

## Definition of Ready

- [ ] Starting SHA and branch verified.
- [ ] Batch PR exists and is Draft.
- [ ] Required input and expected output are unambiguous.
- [ ] Allowed/forbidden scope is complete.
- [ ] Commands are available on the documented environment.

## Definition of Done

- [ ] Outcome and every acceptance check pass.
- [ ] Evidence comment posted without secrets.
- [ ] One focused commit created with `(refs #N)`.
- [ ] Issue linked to batch PR and PR checklist updated.
- [ ] Wiki batch row updated.
- [ ] Issue manually closed.

