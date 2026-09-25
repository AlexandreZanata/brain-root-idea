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

**MVP-0 Linux Model Loop — experimental, hardening for release.** The minimal Linux shell, the provider-neutral contract, the deterministic fake, the OpenCode Go transport, Linux Secret Service credentials, and the minimal conversation loop (prompt, streamed answer, cancel, understandable failures) are implemented and released through `v0.0.1-alpha.5`. The first reproducible test release is `0.0.1` (batch B05), packaged as a Debian artifact for the Ubuntu 24.04 family on x86_64.

The workspace shell, the localhost Companion Canvas preview, the Human Browser, and the frontend interaction phases followed in the `0.0.x` line. The current experimental line is **`0.0.15`**: batch B19 gave the frontend one token scale and explicit control primitives, and the maintainer's [Lean YAGNI pivot](docs/21-lean-yagni-pivot.md) added an on-demand coding agent — a Rust-owned `opencode serve --pure` sidecar with session tabs, a live model picker with context and prices, Plan/Build modes, Fast/Balanced/Max cost profiles, a per-turn token/cost line, and a 60-second idle auto-stop ([#128](https://github.com/AlexandreZanata/brain-root-idea/issues/128)).

> **The agent has no workspace or permission boundary yet.** Starting the sidecar gives `opencode` the same filesystem and process authority as your user account; there is no approved project root, per-action permission prompt, or containment boundary on that path. It only starts on an explicit action, but this is **not** Safe Mode. See [open questions](docs/17-open-questions.md) and the [security model](docs/11-security-and-permissions.md), which still describe the target, not this path.

Try it on the [reference environment](docs/specs/linux-reference-environment.md):

- Build and run: `pnpm install && pnpm tauri build --no-bundle`, then run `src-tauri/target/release/brainroot`.
- Package: `sh scripts/package-linux.sh` → `BrainRoot_…_amd64.deb` ([artifact notes](docs/specs/b05-linux-artifact.md)).
- Gates: `sh scripts/check-fast.sh` and `sh scripts/check-full-linux.sh`; end-to-end journey: `sh scripts/e2e-linux.sh`.
- Live provider smoke: opt-in and local only (`src-tauri/src/provider/live_smoke.rs`), never run in untrusted CI.

**External service warning:** OpenCode Go terms, model catalog, limits, prices, endpoints, and privacy/retention fields are externally mutable and can change at any time. BrainRoot discovers the catalog at runtime, never hardcodes it, keeps the key in the OS credential store, and never makes a provider request at idle.

Start with:

- [Product vision](docs/00-product-vision.md)
- [MVP scope](docs/14-mvp-scope.md)
- [Technical architecture](docs/06-technical-architecture.md)
- [Security model](docs/11-security-and-permissions.md)
- [Performance contract](docs/10-performance-budget.md)
- [First measured Linux baseline](docs/specs/performance-reports/b01-linux-baseline.md) and [MVP-0 soak](docs/specs/performance-reports/b05-mvp0-soak.md)
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
