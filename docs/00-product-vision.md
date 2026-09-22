# Product vision

## North star

BrainRoot is an agent-first IDE for people who want to build software, not learn how to operate a traditional IDE.

> The user describes. The agent builds. The user watches, tests, and directs.

## Problem

Modern AI coding tools still ask people to navigate files, terminals, Git, package managers, logs, and editor conventions. This makes code the control surface even when the user's real skill is describing outcomes and judging results.

BrainRoot moves the control surface to intent, progress, and a running product. It should let a person build useful software without opening a code editor, while keeping the resulting project real, portable, inspectable, and editable elsewhere.

## Product promise

1. Describe a goal in ordinary language.
2. See a concise, comprehensible plan and live task progress.
3. Watch the application appear and run in the Companion Canvas.
4. Test it directly and request changes.
5. Accept the result, inspect details, or restore a checkpoint.

## What BrainRoot is

- A local-first desktop environment with one visible BrainRoot agent.
- A task-oriented interface backed by real project files and ordinary developer tools.
- A safe orchestrator for agents, processes, preview, testing, and recovery.
- An open-source project designed for low idle resource use and no deliberate lock-in.

## What BrainRoot is not

- An editor with a chatbot attached.
- A clone of VS Code, Cursor, Windsurf, Zed, or JetBrains.
- A no-code website builder or proprietary project format.
- A permanently running fleet of agents, servers, LSPs, indexers, and browsers.
- A cloud requirement, model provider, marketplace, or social network.

## Success signals

- A first-time nontechnical user can complete the MVP journey without opening code or a terminal.
- The product explains errors in terms of the user's goal and offers optional technical detail.
- Closing a task releases all task-owned resources.
- A project opens normally in external tools after BrainRoot is removed.
- Restoring an unwanted agent change is obvious and dependable.
- Resource targets are measured continuously rather than claimed from intuition.

## A possible first session

The user opens BrainRoot and sees “What do you want to build?” with **New project** and **Open project**. They choose New project and write “Create a simple expense tracker.” BrainRoot summarizes the intended result, asks only decisions that materially change it, creates a checkpoint, builds in small stages, starts the project only when preview is needed, opens localhost in the Canvas, tests important interactions, and presents the result. The user says “Make it darker.” The Canvas updates. No editor, terminal, package manager, branch, or container terminology is required.

## Product decision test

For every proposal ask:

1. Does it make intent, observation, testing, or direction easier?
2. Can a user who does not understand code use it?
3. Does it keep projects portable?
4. Does it consume resources only while useful?
5. Does it expose new risk or complexity that can remain behind BrainRoot?

If the answer is unfavorable and the benefit is marginal, do not build it.

