# MVP execution plan for economical agents

**Status:** Accepted execution plan  
**Initial target:** Linux  
**First experimental integration:** OpenCode Go model API  
**First product milestone:** MVP-0 Linux Model Loop  
**Workflow unit:** one GitHub issue per microstep, one branch and one pull request per batch

## 1. Purpose

This plan decomposes MVP-0 into work packets that a less-capable, economical coding agent can execute without inventing architecture. Each microstep is narrow, has a fixed file scope, deterministic validation, evidence requirements, and an explicit stop condition.

MVP-0 is a test foundation. It proves Linux shell, provider boundary, streaming conversation, cancellation, secret handling, cleanup, and project discipline. It does not yet let an agent edit a project or show a running application.

## 2. Non-goals

Do not add filesystem tools, shell tools, PTYs, LSPs, project indexing, dev servers, preview WebViews, Playwright, checkpoints, Code View, ACP, multiple providers, Windows, macOS, updater, cloud sync, plugins, or publishing during this plan.

Do not access OpenCode credentials from `~/.local/share/opencode/auth.json`. BrainRoot must own or explicitly receive its credential through its own secure flow. Never store a real key in the repository, issue, PR, Wiki, CI artifact, screenshot, or log.

## 3. Definitions

- **Microstep:** one independently verifiable GitHub issue and one focused commit.
- **Batch:** a coherent ordered set of microsteps implemented on one branch and reviewed through one PR.
- **Micro-gate from B17:** fast non-test scope, diff, secret, format/schema/documentation checks required before closing an implementation issue; test code is written but execution is deferred.
- **Batch/release gate:** complete Linux automated tests, relevant platform/manual probes and performance evidence, and human/high-capability review on the latest versioned batch head.
- **Evidence:** exact command, exit result, bounded output or artifact, changed-file list, and explicit acceptance checklist.
- **Remediation issue:** a new issue created when later work or final CI invalidates earlier evidence.

The release-only test cadence in [ADR 0014](adr/0014-release-only-test-cadence.md) applies from B17 onward. B00–B16 records and the historical detailed packets below describe the earlier, test-per-microstep process; do not rewrite their evidence.

The proposed next frontend-only Linux batch is [B18 fluid Canvas and phone-first Browser](specs/b18-frontend-fluidity-phone-browser.md). Its listed packets are planning candidates, not executable issues: apply this document's one-issue/one-commit and final-release-gate rules when opening the actual batch.

## 4. Mandatory repository workflow

### 4.1 Start a batch

1. Confirm the previous batch PR is merged into `main` and its Wiki/repository log is complete.
2. Pull the latest `main`; do not branch from another unmerged batch.
3. Create exactly one branch named `batch/bNN-short-name`, for example `batch/b02-provider-contract`.
4. Open one draft PR immediately: `[B02] Provider contract and deterministic fake`.
5. Add the batch objective, ordered issue checklist, allowed scope, excluded scope, target version, risks, and rollback to the PR using the batch template.
6. Open the first microstep issue. Later issues may be opened in advance only if their inputs are already known.

### 4.2 Execute one microstep

1. Read only the issue's required documents and current affected files.
2. Confirm every Definition of Ready checkbox. If one is false, comment `BLOCKED: <missing fact>` and stop.
3. Change only allowlisted files. A necessary out-of-scope file requires an issue edit approved before modification.
4. Do not add a dependency unless the issue explicitly authorizes it and includes the dependency-policy evidence.
5. Write/update the issue's automated test cases but do not execute them. Run only the exact fast **non-test** micro-gate commands from the issue: allowlisted diff, `git diff --check`, secret scan, and applicable format/schema/docs checks. Do not run unit, integration, E2E, soak, or full CI until the versioned release gate.
6. Compare fast-check output to the stated expected result. Record `IMPLEMENTED_UNVERIFIED` and the deferred test case IDs/commands; do not describe runtime behavior as verified.
7. Review `git diff`, list changed files, scan for secrets, and confirm no unrelated changes.
8. Commit once using `type(scope): summary (refs #N)`.
9. Push to the batch branch and update the draft PR checklist.
10. Post the fast-check evidence and deferred release tests on issue `#N`, manually link the issue to the batch PR, and update the issue row on the active Wiki batch page as `IMPLEMENTED_UNVERIFIED`.
11. Close the issue only after the PR checklist and Wiki row agree. Reopen it if the release gate later disproves its acceptance.

Use `Refs #N`, not `Closes #N`, because the issue is intentionally closed when its microstep passes before the batch PR merges. If later work changes or breaks that behavior, reopen the issue or create a remediation issue and link both.

### 4.3 Finish a batch

1. Every planned issue is closed with evidence and appears in the PR checklist.
2. Working tree is clean; no untracked result or local-only fix is allowed.
3. Update the repository batch history and matching GitHub Wiki page.
4. Apply the single batch version change and changelog entry where the version policy requires it.
5. Rebase is not required. Update the batch branch from `main` using the repository's chosen non-destructive policy and resolve any conflicts explicitly.
6. Run the complete Linux automated suite only at this versioned batch/release gate by marking the draft PR Ready for review. Collect required manual/platform and performance evidence at this gate. Record `CI_PENDING`, the exact head SHA, PR/CI links if available, and the next action in the PR and Wiki. **End this task without waiting for CI or review.**
7. In a later task, inspect the PR, latest head, required checks, and review once. If still pending, leave the PR open and end that check; do not poll. Independent research, specs, and issue preparation may proceed, but a dependent implementation batch does not start from an unmerged predecessor.
8. If the final CI fails, open a remediation issue in the same batch or reopen the responsible issue. Fix it as a normal microstep, push, record the new head SHA, and return to `CI_PENDING` without waiting. Never patch an untracked failure.
9. Only when the latest-head CI passes, required review approves, conversations are resolved, and Wiki/history links agree, merge with a merge commit so the one-commit-per-issue history remains visible. Delete the batch branch after merge.
10. Create the annotated tag/release only after the merge commit exists on `main`. Complete any post-merge history/Wiki evidence in a separately tracked task when the merge SHA is known.

### 4.4 CI behavior

From B17 onward, no automated tests run per implementation issue. Fast non-test micro-gates catch scope, formatting, and secret mistakes; they do not certify behavior. The full suite runs only when the versioned batch PR becomes Ready, and again after a release-gate remediation push while it is non-draft. Draft PRs do not run test jobs. A failed release candidate may require another final-head run; it is still a release-gate run, not per-microstep testing.

The final batch submission and final merge are separate tasks. Do not run `gh pr checks --watch`, a sleep/poll loop, or repeated CI status queries during one task. Mark the PR `CI_PENDING` and hand off. A future task makes one non-blocking status snapshot: pending means report the state and stop; failed means track remediation; green means inspect review, conversations, history/Wiki, and the exact head before merge. A stale green result from a prior commit is not sufficient.

While CI runs, useful non-dependent work includes fixture research, measured spike design, issue drafting, docs, and review preparation. Do not treat pending CI as approval to merge or to build a dependent batch from an unmerged branch. A new code batch still starts from current `main` after its predecessor merges.

The required check must be generated by a supported PR event against the latest commit; a standalone manual workflow is not sufficient as the only protected-branch check. `main` requires the named Linux full check, at least one approval, resolved conversations, no force push, and no deletion.

The full-CI trigger is `pull_request` when a PR becomes ready for review and on subsequent updates while it is not Draft. Do not repeat the full suite on `push` to `main` after the same final-head gate; release evidence records the merge/tag separately. Do not use `pull_request_target` to execute untrusted branch code with secrets. The OpenCode Go live smoke remains outside untrusted PR CI.

Fast non-test documentation, schema, secret, or formatting checks may run during draft pushes, but no automated test job runs. They do not replace the final full gate and the agent does not block the next microstep waiting for them.

## 5. Work-packet design for a less-capable agent

Every issue must contain:

- one observable outcome;
- background limited to facts needed for that outcome;
- exact starting commit and batch branch;
- required reading, normally no more than five files;
- an allowlist of files/directories that may change;
- a forbidden-scope list;
- exact implementation sequence with no architectural choice left implicit;
- exact fast non-test commands and expected evidence;
- automated release-test cases to write now and execute at the version gate, including negative/failure states;
- security, cleanup, and performance checks;
- rollback instructions;
- a Definition of Ready and Definition of Done.

Target size is one behavior, one focused commit, and normally no more than three production files plus tests/docs. Split an issue when it mixes API design and UI, adds more than one dependency, changes more than one trust boundary, or cannot be validated with a focused command.

### Stop conditions

The agent stops without improvising when:

- required input, API behavior, model availability, or expected output differs from the issue;
- a new dependency, permission, network destination, persistent field, or privileged command is needed;
- a secret appears in a diff, log, fixture, screenshot, or artifact;
- the fast non-test gate fails or a release-gate test fails and cause is not proven;
- a change outside the allowlist is necessary;
- two attempted fixes fail the same acceptance criterion;
- current official documentation contradicts the issue;
- the task would weaken an invariant or require an ADR.

## 6. Labels, milestones, branches, and commits

Required issue labels:

- `batch:B00` through `batch:B05`;
- exactly one of `type:docs`, `type:build`, `type:frontend`, `type:rust`, `type:test`, `type:security`, `type:release`;
- exactly one of `risk:low`, `risk:medium`, `risk:high`;
- `agent:economical`;
- `status:blocked` only while a stop condition exists.

Use milestone `MVP-0 Linux Model Loop`. Issue titles follow `[B02-S03] Implement cancellable provider request`. Batch branches use `batch/b02-provider-contract`. Commits use Conventional Commit types and include `(refs #N)`.

Keep a batch at ten or fewer microstep issues because GitHub's manual PR/issue Development linking is limited. The planned batches contain at most seven; split any batch that would exceed the limit.

## 7. Batch overview and release map

```text
B00 Governance and delivery controls       → v0.0.1-alpha.1
B01 Minimal measured Linux shell           → v0.0.1-alpha.2
B02 Provider-neutral contract, fake first  → v0.0.1-alpha.3
B03 OpenCode Go live transport             → v0.0.1-alpha.4
B04 Minimal conversation experience        → v0.0.1-alpha.5
B05 Linux MVP-0 hardening and release       → v0.0.1
```

Version tags are batch/release outputs, never microstep outputs. See `19-release-and-versioning.md`.

## 8. Detailed microsteps

### Batch B00 — Governance and delivery controls

**User-visible outcome:** none; contributors and agents have a safe, repeatable path.  
**Branch:** `batch/b00-governance`  
**PR:** `[B00] Establish experimental delivery controls`  
**Out of scope:** application scaffold, provider calls, credentials.

#### B00-S01 — Audit license metadata and distribution attribution

- The maintainer has selected Apache-2.0 plus a NOTICE attribution; ADR 0010 records the decision.
- Verify `LICENSE` is the unmodified Apache 2.0 text and `NOTICE` contains the BrainRoot name and canonical project reference.
- Ensure source and binary packaging plans include LICENSE and NOTICE, and project metadata uses SPDX `Apache-2.0`.
- Validate README, dependency policy, contribution terms, and release checklist agree; do not create additional attribution restrictions outside NOTICE.
- Exit: a packaging test or manifest check proves both files will accompany distributed artifacts.

#### B00-S02 — Install GitHub issue and PR templates

- Convert the repository templates in `docs/specs/` into one microstep issue form and one batch PR template under `.github/`.
- Require batch/step ID, allowlist, forbidden scope, commands, expected evidence, security, cleanup, rollback, and checklists.
- Validate YAML/frontmatter and render preview; no CI workflow yet.

#### B00-S03 — Create labels, milestone, and branch rules runbook

- Record exact label names/colors/descriptions and milestone.
- Configure or document `main` protection: PR required, Linux full check required once it exists, one approval, resolved conversations, no force push/deletion.
- Evidence is exported settings/screenshots with no sensitive repository data.
- If repository plan/permissions cannot enforce a rule, record the gap rather than claiming success.

#### B00-S04 — Initialize project history and GitHub Wiki

- Create Wiki Home, Current Status, Batch Index, Release Index, Decisions, Experiments, and Failure Log.
- Add B00 page from the Wiki template and link the repository history record.
- Validate bidirectional links. Wiki is public narrative; reviewed repository history remains the durable source.

#### B00-S05 — Establish version and changelog controls

- Add the single version source chosen by `19-release-and-versioning.md`, `CHANGELOG.md`, and a deterministic version-consistency check design.
- Do not duplicate a version manually across files without an automated check.
- Set the batch target to `0.0.1-alpha.1` only at finalization.

#### B00-S06 — Initial CI, documentation gate, and batch close

- Create the initial GitHub Actions check for link, required-section, secret-pattern, version-consistency, and repository-integrity validation. It must attach to the PR and become required on `main`.
- Keep the PR Draft during microsteps; mark it Ready only after every B00 issue is complete so the first full documentation CI runs on the final head.
- Update B00 repository/Wiki logs and apply version/changelog.
- Merge only after gate and review; tag `v0.0.1-alpha.1`.

### Batch B01 — Minimal measured Linux shell

**User-visible outcome:** BrainRoot opens a minimal Linux window and exits cleanly.  
**Branch:** `batch/b01-linux-shell`  
**PR:** `[B01] Create the measured Linux shell`  
**Out of scope:** project opening, provider, chat, second WebView, terminal.

#### B01-S01 — Freeze the Linux reference environment and dependency evidence

- Record distribution/version, kernel, architecture, display protocol, WebKitGTK, CPU, RAM, storage, Rust, Node/package manager, and build packages.
- Verify current stable Tauri/Svelte versions, licenses, maintenance, advisories, MSRV, transitive footprint, and official Linux prerequisites.
- Produce a dependency decision record; do not install optional UI/state libraries.

#### B01-S02 — Scaffold the smallest Tauri/Rust/Svelte/TypeScript shell

- Generate only the minimal application files using the approved versions.
- Window shows product name and a static “Experimental Linux setup” state.
- No filesystem, shell, network, dialog, updater, analytics, or broad Tauri capabilities.
- Micro-gate: frontend typecheck/build and Rust check succeed on the reference environment.

#### B01-S03 — Add a typed core/UI health contract

- Define one versioned health command/result with no generic JSON map.
- Validate normal response and malformed frontend input rejection.
- UI displays Ready only after the Rust result; no polling.

#### B01-S04 — Establish deterministic test and check scripts

- Add repository-owned `check-fast` and `check-full-linux` entry points using only already approved toolchains.
- Fast gate covers format/type/unit tests for affected modules.
- Full gate covers formatting, lint/static analysis, all tests, frontend production build, Rust release build, documentation links, version consistency, and secret scan.
- A deliberate failing fixture proves each gate returns nonzero; remove the fixture before closing.

#### B01-S05 — Validate Linux launch and clean exit

- Run a release-profile smoke test that waits for the typed Ready state and closes normally.
- Confirm no owned child process, listener, timer, or extra WebView remains.
- Record actual platform limitations rather than adding workarounds outside scope.

#### B01-S06 — Capture the first Linux performance baseline

- Follow `10-performance-budget.md`: release build, environment, repeated startup samples, settled idle CPU/RSS, binary and frontend asset sizes.
- Label results MEASURED only for the reference environment; targets may fail without blocking the experimental shell unless the failure indicates a defect.

#### B01-S07 — Full Linux CI, history, and batch close

- Update batch/Wiki logs and changelog; set `0.0.1-alpha.2`.
- Mark PR Ready and require `check-full-linux` on the latest head.
- Create remediation issues for failures; merge and tag only after green CI and review.

### Batch B02 — Provider-neutral contract and deterministic fake

**User-visible outcome:** none beyond a developer test; the provider boundary works without a real API key.  
**Branch:** `batch/b02-provider-contract`  
**PR:** `[B02] Add provider contract and deterministic fake`  
**Out of scope:** real OpenCode request, production credential, final chat UI.

#### B02-S01 — Define provider-neutral MVP-0 types

- Define model descriptor, conversation/session ID, user message, request, stream event, completion, cancellation, and normalized error types.
- No OpenCode endpoint names or raw SDK types cross the core/UI boundary.
- Contract tests cover schema version, unknown event, missing field, oversized field, and invalid state transition.

#### B02-S02 — Implement deterministic fake provider

- Fake supports model list, streamed text chunks, completion, auth failure, rate limit, server error, malformed event, delayed event, and never-ending stream.
- Tests use fixed seeds/data and no internet or wall-clock sleeps; use controllable time where required.

#### B02-S03 — Implement bounded, cancellable provider execution

- Add one request owner and cancellation token with connect/read/total timeouts and response-size limits.
- Cancellation emits exactly one terminal event and releases handles.
- Any HTTP/runtime dependency requires the issue's dependency-policy review.

#### B02-S04 — Normalize streaming and errors

- Map the fake wire data to provider-neutral stream events and plain-language user errors with optional technical codes.
- Bound chunk/event queues so a fast provider cannot grow memory without limit.
- Tests cover split UTF-8/JSON frames, duplicate terminal events, malformed content, cancellation race, and backpressure.

#### B02-S05 — Define the credential boundary without storing a real key

- Introduce a core-only credential-reference interface and fake in-memory implementation.
- Frontend can learn configured/not-configured only; it cannot retrieve a credential.
- Logs/debug representations redact credential and authorization headers.

#### B02-S06 — Provider contract integration test

- From a test UI intent, exercise fake discovery → request → chunks → complete and request → cancel.
- Assert idle state has no network request or background provider worker.

#### B02-S07 — Full Linux CI, history, and batch close

- Update logs/changelog; set `0.0.1-alpha.3`; run latest-head full CI, review, merge, and tag.

### Batch B03 — OpenCode Go live transport

**User-visible outcome:** a developer can configure Go, discover current models, and run an opt-in live smoke request.  
**Branch:** `batch/b03-opencode-go`  
**PR:** `[B03] Integrate OpenCode Go transport`  
**Out of scope:** autonomous tools, reading OpenCode CLI auth files, multiple simultaneous conversations.

#### B03-S01 — Capture the current OpenCode Go contract

- Verify official provider and Go docs on the execution date.
- Record model-list endpoint, supported protocol endpoints, required auth, BrainRoot-specific User-Agent, stable `x-opencode-session`, limits, and privacy/retention fields.
- Mark catalog, models, limits, prices, and privacy as externally mutable; never copy a current model list into product logic.

#### B03-S02 — Implement model discovery

- Fetch and parse the current Go models endpoint through the core.
- Filter only entries whose protocol MVP-0 explicitly supports.
- Cache only nonsecret metadata with timestamp/expiry; offer a clear offline/stale state.
- Contract tests use captured minimal fixtures and unknown fields.

#### B03-S03 — Implement Linux credential storage and test fallback

- Prefer an approved Linux Secret Service integration after a dedicated dependency/security review.
- Allow environment injection only for local development/opt-in live tests; never persist it.
- If no Secret Service is available, show a truthful unavailable/session-only choice rather than plaintext storage.
- Prove the key cannot cross the frontend contract or appear in logs.

#### B03-S04 — Implement one validated Go protocol path

- Select one currently available low-cost coding model/protocol using explicit criteria: availability, coding/tool capability needed for the experiment, retention/training policy, latency, and usage limits.
- Put base URL/model ID in validated provider configuration, not presentation code.
- Build authentication, BrainRoot-specific User-Agent, and one stable nonsecret `x-opencode-session` per conversation.
- Do not silently fall back to another model, protocol, paid Zen balance, or endpoint.

#### B03-S05 — Map Go-specific failures and privacy metadata

- Normalize invalid key, subscription unavailable, unsupported model, rate/usage limit, timeout, provider failure, malformed response, and network unavailable.
- Expose current model privacy/retention information before first live use when available.

#### B03-S06 — Add opt-in live smoke test

- Test is disabled/skipped without the documented environment variable and never runs on untrusted contributions with secrets.
- Send a harmless fixed prompt, verify at least one chunk and a normal terminal event, then destroy request/session resources.
- Redact response bodies from default CI logs and record only safe timing/status evidence.

#### B03-S07 — Full Linux CI, history, and batch close

- Fake-provider tests remain the required deterministic CI path; live smoke is separately evidenced by an authorized maintainer.
- Update logs/changelog; set `0.0.1-alpha.4`; full CI, review, merge, and tag.

### Batch B04 — Minimal conversation experience

**User-visible outcome:** a Linux user can enter a prompt, watch a streamed answer, cancel, and understand failures.  
**Branch:** `batch/b04-conversation-ui`  
**PR:** `[B04] Add the minimal Linux conversation loop`  
**Out of scope:** code edits, tools, markdown ecosystem, conversation branching, model marketplace.

#### B04-S01 — Implement explicit conversation state

- Use `EMPTY`, `READY`, `SENDING`, `STREAMING`, `CANCELLING`, `SUCCEEDED`, and `FAILED` with legal transitions.
- Reducer/state tests reject late chunks, double submit, duplicate terminal events, and completion after cancel.

#### B04-S02 — Build the minimal agent-first layout

- Agent surface accepts one prompt; the larger Canvas region remains a clear “Preview comes in MVP-1” placeholder.
- No file tree, terminal, technical dashboard, provider marketing, or editor.
- Keyboard order, visible focus, zoom, semantics, contrast, and reduced motion are validated.

#### B04-S03 — Connect submit and streaming display

- UI sends provider-neutral intent to Rust and renders normalized chunks.
- Disable accidental duplicate submit while a request owns the conversation.
- Bound rendered history/content for the experiment and test long output.

#### B04-S04 — Implement cancellation and close cleanup

- Cancel changes state immediately, propagates to provider execution, and ignores late events.
- Closing window/application during streaming cancels and releases resources.
- Integration test asserts zero active request after cancel and close.

#### B04-S05 — Implement understandable setup and failure states

- Cover not configured, no compatible model, invalid credential, network unavailable, limit reached, timeout, provider error, malformed response, and cancelled.
- Default text explains the user's next action; sanitized technical code/detail is optional.

#### B04-S06 — Accessibility and secret-safety verification

- Keyboard-only prompt/send/cancel/retry, screen-reader status announcements without chunk spam, focus recovery, selection/copy, and secret/log scans.

#### B04-S07 — Full Linux CI, history, and batch close

- Update logs/changelog; set `0.0.1-alpha.5`; run full latest-head CI, review, merge, and tag.

### Batch B05 — Linux MVP-0 hardening and release

**User-visible outcome:** a reproducible experimental Linux build demonstrates the complete MVP-0 loop.  
**Branch:** `batch/b05-linux-mvp0-release`  
**PR:** `[B05] Harden and release Linux MVP-0`  
**Out of scope:** new product capability.

#### B05-S01 — End-to-end fake-provider journey

- Automate launch → Ready → configured fake → discover → prompt → stream → complete → second prompt → cancel → close.
- Assert user states, technical errors, bounded output, and cleanup.

#### B05-S02 — Security and privacy gate

- Test secret redaction, frontend contract exclusion, artifacts, logs, screenshots, malformed provider data, oversized responses, URL allowlist, TLS-only production endpoint, and dependency audit.
- Confirm no telemetry or unapproved network destination.

#### B05-S03 — Performance and soak gate

- Measure startup, idle CPU/RSS, request memory, cancellation latency, and return-to-baseline after repeated fake conversations.
- Run repeated open/close and request/cancel cycles; record raw environment/results.
- Do not turn failed TARGET values into MEASURED success.

#### B05-S04 — Produce one documented Linux test artifact

- Select one packaging format supported by current Tauri/Linux evidence; do not promise universal Linux compatibility.
- Record runtime dependencies, checksum, install/run/remove steps, known limitations, and supported reference environment.

#### B05-S05 — Complete documentation and history

- Update README status, MVP scope, architecture evidence, security limitations, performance report, open questions, changelog, repository batch log, Wiki batch page, Wiki release page, and failure log.
- Document that OpenCode Go terms, catalog, limits, endpoints, and privacy can change.

#### B05-S06 — Release candidate and final CI

- Set version `0.0.1`, ensure every version surface matches, and freeze scope.
- Run the full Linux CI on the latest head. Any failure becomes a remediation issue; no direct “quick fix.”
- Require final review against every MVP-0 quality gate.

#### B05-S07 — Merge, tag, and release

- Merge only after CI/review/history/Wiki gates.
- Create annotated `v0.0.1` tag and GitHub pre-release with checksum, supported environment, privacy warning, known limitations, and links to issues/PR/Wiki.
- Verify the published artifact from a clean Linux environment before marking the release complete.

## 9. Required full Linux CI at batch end

The final required check should eventually include, in stable order:

1. repository cleanliness and version consistency;
2. Markdown/local-link checks and ADR structure;
3. frontend formatting, typecheck, unit tests, and production build;
4. Rust formatting, lint/static analysis, unit and contract tests;
5. fake-provider integration and cleanup tests;
6. Tauri Linux release build;
7. secret-pattern and dependency/license/advisory checks;
8. MVP-0 fake end-to-end test when it exists;
9. artifact manifest and checksum for release batches.

Tests that need a real paid credential are never required for untrusted PR CI. An authorized opt-in live smoke is recorded separately without exposing secret or response content.

## 10. Batch acceptance invariant

A batch is complete only when code, tests, docs, issue evidence, PR checklist, full CI, repository history, Wiki page, changelog/version, and cleanup evidence agree. A green build with missing history or a closed issue with invalidated behavior is incomplete.

## 11. Future platform expansion

After `v0.0.1`, platform work uses separate batches and never shares a branch with new product capabilities.

### Windows minimum environment

Create ordered batches for platform evidence/toolchain, minimal shell build, Credential Locker implementation, fake-provider contract, opt-in Go smoke, conversation E2E, performance/cleanup, and one documented artifact. Reuse provider-neutral fixtures; write Windows-specific tests for WebView2, paths, process termination, credential behavior, installer, and uninstall. Do not accept “works on Linux” as evidence.

### macOS minimum environment

Start only after Windows findings are merged and interfaces remain stable. Create ordered batches for Xcode/toolchain evidence, shell build, Keychain, entitlements/App Sandbox feasibility, fake/live provider path, conversation E2E, performance/cleanup, signing/notarization decision, and one documented artifact. Never weaken the cross-platform Safe Mode contract to bypass an entitlement constraint.

Each platform gets its own reference environment, MEASURED reports, Wiki batch pages, release notes, and version increments below `1.0.0`.

## 12. References

- [OpenCode Go current guide](https://opencode.ai/docs/go)
- [OpenCode provider configuration](https://opencode.ai/docs/providers)
- [GitHub issue/PR linking](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue)
- [GitHub required status checks](https://docs.github.com/en/pull-requests/reference/status-checks)
- [GitHub protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches)
- [Semantic Versioning](https://semver.org/)
