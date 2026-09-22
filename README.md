# BrainRoot

> The user describes. The agent builds. The user watches, tests, and directs.

BrainRoot is an open-source, agent-first desktop environment for people who want to build software without first learning how to operate a traditional IDE. Its primary interface is a conversation beside a live **Companion Canvas**. Code, terminals, Git, and language tooling remain available as secondary tools.

## Why BrainRoot

Most AI coding products begin with an editor and add an assistant. BrainRoot begins with user intent, visible task progress, a running result, and safe recovery. It creates normal projects made of normal files: no proprietary project format and no deliberate lock-in.

BrainRoot is primarily for vibe coders and nontraditional developers. Experienced developers can reveal technical controls through an optional Developer Mode, but their workflows do not define the default experience.

## Product invariants

- Code is not the primary interface.
- Unused functionality must not consume meaningful resources.
- Technical complexity stays hidden until the user asks for it.
- Agent permissions never expand silently.
- Projects remain portable outside BrainRoot.
- Performance regressions are bugs.

## Status

**Phase 0 — architecture and product definition.** There is intentionally no application code yet. The current repository is the specification that future humans and coding agents must follow.

Start with:

- [Product vision](docs/00-product-vision.md)
- [MVP scope](docs/14-mvp-scope.md)
- [Technical architecture](docs/06-technical-architecture.md)
- [Security model](docs/11-security-and-permissions.md)
- [Performance contract](docs/10-performance-budget.md)
- [MVP microstep execution plan](docs/18-mvp-execution-plan.md)
- [Experimental release and versioning policy](docs/19-release-and-versioning.md)
- [Project history and Wiki protocol](docs/20-project-history-and-wiki.md)
- [Eight-line continuation prompt](docs/prompts/continue-economical-agent.md)
- [Rules for coding agents](AGENTS.md)
- [Architecture decisions](docs/adr/README.md)

## Contributing

Read `AGENTS.md` before proposing work. Implementation is organized into batch branches with one draft pull request and one GitHub issue per verified microstep. Follow [the execution plan](docs/18-mvp-execution-plan.md), keep changes reviewable, test user-visible behavior and resource cleanup, and add an ADR before changing a significant architectural decision. Do not skip ahead to the editor, Companion Deck, or plugin ecosystem.

## Open-source commitment

BrainRoot is licensed under the [Apache License 2.0](LICENSE), an OSI-approved license permitting commercial and noncommercial use, modification, and redistribution. Redistributions must preserve the license and the readable attribution in [NOTICE](NOTICE), including the BrainRoot project name and reference. Runtime dependencies must be open source and license-compatible.
