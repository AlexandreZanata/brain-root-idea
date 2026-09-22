# Product principles

## 1. Code is secondary

Conversation, task state, result, testing, and recovery form the main interface. Code View is an escape hatch and inspection tool.

## 2. Show the thing being built

The Companion Canvas dominates the workspace. Prefer a working preview and observable evidence over long textual claims.

## 3. One understandable agent

The default UI presents BrainRoot, not an organization chart of specialist agents. Internal workers may exist when justified; advanced users may inspect “workers active.”

## 4. Progressive disclosure

Default Mode shows intent, progress, result, tests, decisions, and checkpoints. Developer Mode reveals code, diffs, terminal, Git, processes, protocols, and raw logs. Both represent the same underlying task state.

## 5. Zero idle by design

Anything unused should be stopped, suspended, or absent. Every resource has an owner and cleanup rule. Event-driven state is preferred to polling.

## 6. Safety is a system property

Prompts are not enforcement. Permissions are checked in the core and, where practical, an operating-system or container boundary. Safe Mode is default; Power Mode is explicit, scoped, time-bounded where possible, and auditable.

## 7. Recovery before confidence theater

Agents will make mistakes. Create recoverable checkpoints around meaningful work, show the scope of changes, and make Undo comprehensible.

## 8. Real projects, no lock-in

BrainRoot operates on normal files and can use existing project conventions. `.brainroot/` stores only portable, nonsecret project metadata. The project remains usable without BrainRoot.

## 9. Honest evidence

Separate plan, action, result, and uncertainty. Do not expose private chain-of-thought. Do not claim a test, screenshot, benchmark, or restore succeeded without evidence.

## 10. Small before generic

One excellent agent integration, one active preview, and one coherent task flow precede provider matrices, decks, marketplaces, extension systems, and generalized internal frameworks.

## 11. Open by construction

BrainRoot uses Apache-2.0 with a NOTICE attribution that preserves the BrainRoot name and project reference in redistributed derivative works. Accept only license-compatible open-source runtime dependencies, document third-party notices, and make core local functionality independent of proprietary services. Remote models may be optional providers, not architectural owners.

## Anti-drift checklist

Reject or redesign a feature when it makes the default UI resemble a file explorer, requires users to learn infrastructure unnecessarily, keeps hidden resources alive, couples project data to BrainRoot, grants broad ambient authority, or adds a framework without a current user capability.
