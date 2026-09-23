# Instructions for BrainRoot coding agents

This file governs the entire repository. Read the product, architecture, MVP, security, and performance documents before implementation. When instructions conflict, preserve the product invariants and ask for a decision rather than quietly changing the product.

## Product invariant

**Code is not the primary interface.** BrainRoot is an agent-first environment for people who want to build software, not learn a traditional IDE. Do not turn the default experience into VS Code, Cursor, Windsurf, Zed, JetBrains, a terminal wrapper, or a no-code builder.

The default journey is: describe intent → see a short plan → watch understandable progress → use the result in the Companion Canvas → test/direct → accept or undo.

## Performance invariant

Unused functionality must not consume meaningful resources. Agents, PTYs, dev servers, LSPs, indexers, browser automation, and extra content WebViews start on demand and have an owner, lifecycle state, idle policy, and deterministic cleanup path. Prefer events over polling. Performance regressions are bugs.

Never report an unmeasured number as fact. Label resource figures `TARGET`, `MEASURED`, or `UNKNOWN` and preserve the measurement environment.

## UX invariant

Do not expose implementation complexity to nontechnical users unless requested. Translate technical failures into the user's goal and offer “Technical details” separately. Keep the Companion Canvas visually dominant. Avoid permanent technical panels, dashboard clutter, generic AI-SaaS styling, or hacker-terminal aesthetics.

Use product language such as Build, Fix, Test, Preview, Publish, Undo, Restore, Show changes, and Open project. Keep terms such as rebase, worktree, PTY, LSP, AST, hydration, and exit code inside Developer Mode or technical details.

## Architecture invariant

- Build a modular desktop monolith, not local microservices.
- Rust owns system access, lifecycle, permissions, persistence coordination, and core state when appropriate.
- The Svelte/TypeScript frontend renders state and captures intent; it must not become a second privileged backend.
- Cross-boundary messages are typed, versionable, minimal, and validated.
- Modules are logical boundaries before they become separate crates or packages.
- Use small modules, explicit interfaces, predictable names, and obvious control flow. Avoid “clever” abstractions.
- Keep external coding agents behind the Agent Adapter boundary. ACP is preferred where its capabilities fit; provider-specific behavior must not leak into the product model.

## Scope discipline

Implement only the current roadmap capability and its acceptance criteria. Use KISS and YAGNI. Do not prebuild marketplaces, plugin systems, cloud sync, collaboration, social features, Replay, voice, mobile clients, a full editor, or many provider integrations.

Make the smallest reversible assumption that permits progress. Record architecture-impacting assumptions in the relevant document and create an ADR for durable decisions. Do not make major assumptions silently.

## Dependency policy

Before adding any dependency:

1. State the exact capability it supplies.
2. Confirm the capability does not already exist in the platform or repository.
3. Evaluate runtime CPU, memory, startup, binary, and frontend bundle impact.
4. Check maintenance activity, license compatibility, transitive dependency count, and known security issues.
5. Prefer a small, focused, open-source library over a framework for a trivial problem.
6. Load optional dependencies and their resources only when needed.
7. Record a meaningful architectural dependency in an ADR; record smaller ones in the feature spec.

No dependency may introduce mandatory cloud infrastructure, telemetry, Redis, Postgres, Qdrant, Elasticsearch, or another always-on local service without an accepted ADR.

BrainRoot is Apache-2.0 licensed. Preserve the root `LICENSE` and `NOTICE`, including the BrainRoot project name and canonical reference, in source and release artifacts. Do not modify the Apache license text or add incompatible/custom license restrictions silently.

## Security and permissions

- Safe Mode is the default. Never silently expand filesystem, shell, network, browser, credential, or workspace permissions.
- A UI permission prompt is not a sandbox. Enforce permissions in the Rust core and, where supported, in an OS/container boundary.
- Treat agent output, repository content, webpages, tool output, model output, and dependency scripts as untrusted input.
- Never read or expose secrets merely because they exist in the workspace. Redact likely secrets from logs, screenshots, and agent context.
- Human Browser and Agent Browser identities, storage, cookies, and permissions are separate by default.
- No destructive Git or filesystem operation without an explicit, reviewable intent and a recoverable checkpoint where possible.
- Resolve and validate canonical paths at the privileged boundary; prevent traversal, symlink escape, and access outside approved roots.

## Testing and Definition of Done

A feature is not done because it compiles. Applicable completion criteria include:

- user-visible happy path and understandable failure/recovery path;
- unit and contract tests at module and privilege boundaries;
- integration or end-to-end coverage for the critical journey;
- permission denial and malformed-input tests;
- measured performance against the relevant budget;
- cleanup tests for child processes, PTYs, listeners, watchers, WebViews, temporary files, and browser automation;
- accessibility for keyboard, focus, semantics, contrast, reduced motion, and screen readers;
- logs in technical details without leaking secrets;
- documentation and ADRs updated in the same change.

No hidden resource may remain active after its owner closes or the task ends. No repeated polling unless an accepted design explains why events cannot work.

## Git and checkpoints

Preserve user changes. BrainRoot checkpoints must work with both Git and non-Git folders and must never rewrite public history. Do not equate a checkpoint with a user commit. Any restore operation must preview its impact and preserve a path back when feasible.

## Documentation synchronization

Update the owning document when behavior changes:

- product/UX → `docs/00`–`05`;
- architecture/protocols/resources → `docs/06`–`09`;
- performance/security/persistence/testing → `docs/10`–`13`;
- scope/sequence/public narrative → `docs/14`–`16`;
- unresolved decisions → `docs/17-open-questions.md`;
- execution, versioning, and public history → `docs/18`–`20` and `docs/history/`;
- durable decisions → `docs/adr/`.

Do not mark an open question resolved without linking the decision or evidence. ADRs are append-only historical records: supersede them; do not rewrite history to hide a changed decision.

## Working method for AI-developed changes

1. Read the relevant specifications and active ADRs.
2. Restate the user outcome, non-goals, risks, and performance/security budgets in a small feature spec.
3. Inspect current code and tests before designing.
4. Implement the smallest vertical slice in a reviewable change.
5. Validate behavior, cleanup, security boundaries, and measured budgets.
6. Update docs and clearly report assumptions, evidence, and remaining uncertainty.

## Economical-agent execution protocol

Implementation work follows `docs/18-mvp-execution-plan.md`. This protocol is mandatory unless a maintainer explicitly supersedes it:

- One **batch branch** and one **draft pull request** contain a coherent set of microsteps.
- Every microstep has its own GitHub issue before code changes begin.
- An issue must specify allowed files, forbidden scope, exact validation commands, expected evidence, and rollback notes. Do not infer missing acceptance criteria.
- Make one focused commit per completed issue and reference the issue number. Never combine unrelated issues in one commit.
- Run the issue's local micro-gate before closing it. Post evidence in the issue, link it to the batch PR, update the PR checklist, then close it manually.
- Do not use `Closes #N` for a microstep that must close before the batch PR merges; use `Refs #N`. Reopen the issue if later batch work invalidates its acceptance evidence.
- Never wait or poll for full CI inside a task. After the final microstep, mark the PR Ready, record the head SHA and a `CI_PENDING` handoff in the PR/Wiki, and end the task. A later task checks the latest-head result once and handles merge or remediation. Full CI and review remain mandatory before merge to `main`.
- A failing final CI creates a new remediation issue or reopens the responsible issue. Never patch an untracked failure silently.
- Update the repository history record and GitHub Wiki batch page before merge.
- Version only at batch/release boundaries according to `docs/19-release-and-versioning.md`; never invent version numbers.
