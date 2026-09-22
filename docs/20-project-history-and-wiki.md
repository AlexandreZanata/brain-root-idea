# Project history and GitHub Wiki protocol

## Purpose

BrainRoot is a Build in Public experiment. Its history must explain what was attempted, what evidence passed, what failed, what changed our mind, and which artifact resulted. Commit history alone is not sufficient.

## Two synchronized records

- **Repository history (`docs/history/`)** is the reviewed, versioned, durable source of truth merged with code.
- **GitHub Wiki** is the readable public operations journal updated while a batch is in progress.

The Wiki must never be the only copy of a release-critical decision because it is versioned separately from the main repository. At batch close, the final Wiki page and repository batch record must contain the same issue/result matrix and link to each other.

## Required Wiki structure

```text
Home
Current-Status
Batch-Index
Batches/B00-Governance
Batches/B01-Linux-Shell
...
Release-Index
Releases/v0.0.1-alpha.1
Decisions
Experiments
Failure-Log
Performance-Index
```

Wiki capabilities and page naming are initialized in B00. If nested names are not rendered as expected, use flat names such as `Batch-B00-Governance` and record the convention. Observed on 2026-09-22 (B00-S04): a page stored as `Batches/B00-Governance.md` is served only at `.../wiki/Batches%2FB00-Governance` and at GitHub's normalized `.../wiki/B00-Governance`, while the plain nested URL `.../wiki/Batches/B00-Governance` returns 404. BrainRoot therefore uses the flat convention `Batch-Bxx-Name` for batch pages and `Release-vX.Y.Z` for release pages.

## Update cadence

### At batch start

- Create the batch page from the template.
- Record objective, branch, draft PR, target version, ordered issues, baseline commit, risks, and explicit non-goals.
- Update Current Status and Batch Index.

### At every microstep close

- Append or update exactly one issue row: issue, outcome, commit, validation summary, deviations, and status.
- Link the issue evidence comment and batch PR.
- Record a failure in Failure Log immediately when it teaches something or changes the plan; do not wait for a successful narrative.
- Do not copy raw provider output, authorization headers, environment dumps, personal paths, or secrets.

### At batch close

- Add final CI run, review, merge commit, version/tag, artifact/checksum if any, measured results, known limitations, unresolved questions, and next batch.
- Create/update the release page.
- Copy the final normalized record into `docs/history/batches/Bxx-<name>.md` through the batch PR.
- Verify bidirectional repository/Wiki links and update Current Status.

## Required history content

Every batch record answers:

1. What user or project capability was targeted?
2. What was explicitly excluded?
3. Which issues and commits implemented it?
4. Which validations actually ran, on which environment?
5. What failed, was retried, or was deferred?
6. What security, privacy, performance, and cleanup evidence exists?
7. Did research contradict an assumption?
8. What version/artifact resulted?
9. What remains unknown?
10. What is the next smallest batch?

## Evidence rules

Link to bounded logs or CI runs instead of copying thousands of lines. Preserve exact commands and exit status. Screenshots support UI evidence but never replace semantic assertions. Performance claims link a report following `specs/performance-test-template.md`. External service facts include source URL and review date.

Sanitize all evidence. Never include API keys, authorization headers, cookies, full environment output, personal browser state, private prompts, user data, or provider response bodies that might contain project content.

## Failure history

Failure Log entries contain date, batch/issue, observed behavior, expected behavior, safe reproduction, impact, disposition, and prevention. Use one of: fixed in issue, accepted limitation, superseded decision, provider incident, or unresolved blocker.

Do not erase a failed experiment after the solution works. The failed path is part of the project's architectural memory and Build in Public narrative.

## Drift handling

Final batch CI cannot directly prove the remote Wiki is synchronized without additional GitHub credentials. The finalization issue therefore requires a maintainer-visible Wiki link and a manual comparison. Future automation may clone the Wiki repository read-only and compare normalized metadata, but it must not receive broader repository authority than needed.

If the Wiki is unavailable, the repository history is completed, the batch remains unmergeable unless a maintainer explicitly records a temporary exception, and the Wiki-sync task remains open. Never claim the Wiki was updated when it was not.

