# UX and information architecture

## Primary workspace

The stable hierarchy is intentionally small:

```text
App chrome: project identity + global actions
├── Agent surface: intent, task progress, decisions, result
└── Companion Canvas: live preview or selected visual destination
Optional: compact status/resource strip
Hidden until requested: Code View, terminal, logs, Git, processes
```

Do not place a permanent file explorer in the primary left surface.

## Layout presets

- **Focus:** Agent 30% / Canvas 70%; default for active building.
- **Watch:** Agent 15% / Canvas 85%; compact task summary remains visible.
- **Chat:** Agent 50% / Canvas 50%; useful for clarification and planning.
- **Full Canvas:** Canvas fills the window; a small agent affordance shows state and reopens chat.

A single draggable divider may refine the ratio. Presets provide fast recovery and accessible keyboard commands. Layout persists per project without persisting sensitive page data.

## Default Mode

Show the current task, understandable stages, agent questions, preview, test result, change summary, checkpoint, Undo, and Show changes. Limit simultaneous attention targets. Technical errors become goal-oriented messages, for example: “The project preview stopped. BrainRoot is checking why.”

## Developer Mode

May reveal Code View, structured diff, terminal, logs, Git state, child processes, LSP state, raw commands, stdout/stderr, timings, exit status, and worker details. Developer Mode is not a separate product and does not bypass permission enforcement.

## Task presentation

```text
Build authentication
Planning   ✓ Understand current project  ✓ Confirm existing auth pattern
Building   ✓ Login screen                ● Connect backend
Testing    ○ Sign-in flow                ○ Error recovery
```

Only externally useful actions, decisions, results, and blockers appear. Never display private chain-of-thought or an endless stream of model narration.

## First-run experience

1. Ask “What do you want to build?”
2. Offer New project and Open project.
3. Explain Safe Mode in one sentence at the first consequential permission.
4. Confirm only ambiguous choices with material product impact.
5. Show the first preview automatically when ready.
6. End with result, tests performed, change count, and Accept / Undo / Show changes.

The proposed [B18 phone-first Browser phase](specs/b18-frontend-fluidity-phone-browser.md) refines step 5 for the post-MVP Canvas: Browser/phone is the initial destination; a newly ready project Preview may open automatically only if the user has not already selected a different destination in that session. A startup Browser tab is not permission to fetch a remote page without an address.

## Language system

Primary: Build, Fix, Test, Preview, Publish, Undo, Restore, Show changes, Open project, Needs your approval, and Technical details.

Technical-detail only: checkout, rebase, stash, worktree, PTY, LSP, AST, hydration, stack trace, and exit code.

## Interaction requirements

- Every gesture has a click and keyboard alternative.
- Focus never disappears into a WebView without a reliable escape shortcut.
- Destructive actions state what will change and how recovery works.
- Progress is state-based, not a fake percentage.
- Reduced-motion mode replaces large transitions with direct state changes.
