# Agent experience

## Product model

The user interacts with one visible **BrainRoot** agent. The UI renders a typed task model rather than treating chat as an unstructured transcript. Internal workers are an implementation detail and cannot independently acquire permissions.

## Task lifecycle

`DRAFT → PLANNING → AWAITING_APPROVAL | RUNNING → VALIDATING → SUCCEEDED | FAILED | CANCELLED`

Each task contains a user goal, short plan, explicit decisions, observable steps, owned resources, permission grants, checkpoint references, evidence, and a final result. Cancellation must propagate to tools and child processes.

## What the user sees

- What BrainRoot understood.
- A short plan and any decision that changes cost, safety, or outcome.
- Completed, active, queued, blocked, and failed steps.
- Concrete artifacts: preview, screenshot, test result, diff summary.
- A final explanation in user language plus optional technical detail.

The user does not see hidden chain-of-thought, raw protocol traffic, token-by-token internal deliberation, or repetitive command noise.

## Permission moments

Request permission at the narrowest meaningful action, not at generic “agent access.” Include purpose, scope, duration, and risk. Reuse a grant only when all four still match. Power Mode does not mean permanent unrestricted access.

## Progress and truthfulness

A step becomes complete only after its completion event and evidence. “Testing” means a test process actually started. “Preview ready” means the endpoint passed a readiness check and loaded. “Fixed” means the failure was rechecked. Unknown progress remains indeterminate; do not invent percentages.

## Checkpoint experience

Before a meaningful write set, BrainRoot records a checkpoint without asking the user to understand Git. At the end:

```text
BrainRoot changed 7 files and checked the main flow.
[Looks good] [Undo] [Show changes]
```

Undo previews affected files, protects unrelated user edits, and explains conflicts in ordinary language.

## Failure experience

Failures separate:

1. user message: what stopped and what BrainRoot can try;
2. recovery controls: Retry, Change approach, Restore, or Ask me;
3. technical detail: command, sanitized stdout/stderr, timing, exit status, and relevant files.

## Observability views

**Activity** is user-facing and records high-level actions and results. **Technical logs** are advanced, structured, bounded, redactable, and exportable with explicit user action. Neither may contain secrets by default.

## Multi-agent policy

The MVP runs one agent session. Later internal delegation is allowed only when benchmarks show a user benefit. The Agent Host remains the sole coordinator, permission broker, and lifecycle owner. The default UI may summarize “3 workers active” but never requires the user to manage a team of personas.

