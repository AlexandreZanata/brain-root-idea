# Capability roadmap

No dates are promised. A phase advances only when its user outcome and quality gates are met. Security, accessibility, documentation, cleanup, and relevant performance measurements apply to every phase.

Implementation follows the issue/batch workflow in `18-mvp-execution-plan.md`. Linux is the proving platform. Windows follows the Linux MVP-0 baseline; macOS follows Windows. Platform work is not parallelized until the interfaces and lifecycle behavior are stable enough to avoid multiplying rework.

## Phase 0 — product and architecture foundation

**Goal/outcome:** a future contributor or agent can explain the product, constraints, decisions, risks, and next slice. **Dependencies:** none. **Acceptance:** this documentation set is reviewed; blockers have owners; license is selected before external contributions. **Performance:** budgets and protocol exist, with numbers labeled TARGET. **Out:** application code.

## Phase 1 — measured native shell and project opening

**Goal/outcome:** launch a minimal Linux desktop shell, establish typed UI/core health, and collect the first trustworthy baseline. Project opening follows after the MVP-0 model loop unless a batch explicitly includes it. **Dependencies:** Phase 0 license and Linux support choices. **Acceptance:** typed UI/core boundary, clean exit, deterministic tests, Linux packaging smoke test. **Performance:** measured startup/idle baseline and bundle accounting on the Linux reference environment. **Out:** terminal, dev server, editor, extra WebView, Windows, macOS.

## Phase 2 — task UI and one agent integration

**Goal/outcome:** first prove a minimal Linux conversation loop through the OpenCode Go model API, then evolve it into structured BrainRoot tasks and one coding-agent integration. **Dependencies:** shell, provider transport spike, secure credential decision. **Acceptance for MVP-0:** model discovery/configuration, streaming, cancellation, deterministic fake-provider CI, session header, understandable failures, no leaked secret. **Acceptance for later Agent Host:** on-demand start/cancel/terminate, capability negotiation, task events, and session metadata. **Performance:** no inactive request/agent process and bounded stream memory. **Out:** multiple providers/workers and general tools.

## Platform expansion after Linux MVP-0

### Windows minimum environment

Port only the proven shell, provider transport, credential storage, tests, cleanup, and packaging. Record WebView2, Credential Locker, process, and installer differences. Do not start full Windows product parity in the same batch.

### macOS minimum environment

Port the same proven boundary after Windows. Record WKWebView, Keychain, entitlements, App Sandbox, signing, and notarization constraints. Do not weaken Safe Mode globally to make one platform easier.

## Phase 3 — safe workspace tools and process lifecycle

**Goal/outcome:** agent can inspect and change the selected project and run bounded commands with clear permission moments. **Dependencies:** Agent Host, permission model. **Acceptance:** file/search/write, command process groups, redaction, cancellation, audit events, hostile path tests. **Performance:** no automatic index; no orphan process/PTY. **Out:** broad home access, interactive terminal UI, Power Mode automation.

## Phase 4 — localhost Companion Canvas

**Goal/outcome:** BrainRoot starts a project and shows the real app. **Dependencies:** Process Manager and cross-platform WebView spike. **Acceptance:** one dev server, readiness, localhost navigation, reload, failure/recovery, origin isolation, focus escape. **Performance:** one active content WebView; measured create/destroy and memory. **Out:** remote browser, Deck, Phone.

The adjacent layout, Swap action, and responsive preview presets are specified in [the Companion Browser plan](specs/companion-browser-plan.md); they do not authorize remote browsing in this phase.

## Phase 5 — observe/test/fix loop

**Goal/outcome:** the agent can run the app, observe key behavior, find a failure, fix, and recheck with evidence. **Dependencies:** preview and on-demand Playwright. **Acceptance:** screenshot, DOM/console/network evidence, bounded artifacts, deterministic E2E fixture, clear test state. **Performance:** automation is absent after task; no background browser. **Out:** autonomous exhaustive QA or device farm.

## Phase 6 — checkpoints and safe Undo

**Goal/outcome:** users confidently accept or restore agent work without Git knowledge. **Dependencies:** mature file mutation events and checkpoint spike. **Acceptance:** Git/non-Git cases, preexisting edits, create/delete/rename, conflict UX, integrity and recovery tests. **Performance:** incremental storage/latency measured on fixture sizes. **Out:** branch management UI or history rewriting.

## Phase 7 — optional Code View and Developer Mode

**Goal/outcome:** inspect changed code, diff, logs, commands, and processes without changing the default workflow. **Dependencies:** successful core loop. **Acceptance:** lazy CodeMirror loading, basic edit/search/highlight/diff navigation, accessible mode switch. **Performance:** zero CodeMirror/language cost before opening; LSP remains optional. **Out:** extension API, debugger, full IDE parity.

## Phase 8 — Companion Deck

**Goal/outcome:** switch deliberately among Preview, Docs, and Browser destinations. **Dependencies:** proven Browser Manager isolation/lifecycle. **Acceptance:** one HOT heavy card, COLD reconstruction, snapshots/placeholders, navigation policy, profile boundaries. **Performance:** inactive cards consume no live WebView by default. **Out:** perfect session restoration or social-specific features.

This phase includes a separately gated, user-operated Human Browser before any [portable browser-data import](specs/companion-browser-plan.md). Bookmarks may start from explicit HTML exports; history/preferences need their own compatibility and privacy gates. Passwords, cookies, sessions, and extensions are not promised.

## Phase 9 — Companion Phone and spatial navigation

**Goal/outcome:** use phone/9:16, square, landscape, desktop, and full Canvas modes with swipe/drag and accessible controls. **Dependencies:** Deck. **Acceptance:** gesture conflict handling, keyboard/buttons, focus, reduced motion, responsive fixtures. **Performance:** transitions remain responsive without multiplying WebViews. **Out:** mobile BrainRoot app.

The phone-width view is a responsive desktop-app presentation, not a claim of mobile browser-engine equivalence; see [the interaction contract](specs/companion-browser-plan.md).

## Phase 10 — advanced Resource Governor

**Goal/outcome:** measured policies balance fast resume with low CPU/RAM across agents, tooling, and WebViews. **Dependencies:** real lifecycle data. **Acceptance:** ownership graph, ACTIVE/IDLE/SUSPENDED/TERMINATED policy, platform-specific hibernation only where proven, leak dashboard. **Performance:** budgets enforced with reproducible results. **Out:** local-model orchestration unless separately approved.

## Future, not sequenced

BrainRoot Replay may record high-level prompts, task states, changed-file summaries, previews, tests, and results for debugging and opt-in demo creation. It must exclude hidden reasoning, secrets, human browser data, and unapproved source content.
