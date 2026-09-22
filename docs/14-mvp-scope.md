# MVP scope

## Delivery stages

BrainRoot is delivered platform-first rather than pretending immediate parity:

1. **MVP-0 Linux Model Loop:** experimental setup for testing the native shell and OpenCode Go model API.
2. **MVP-1 Linux Product Loop:** the full open → describe → build → preview → test → undo vertical slice below.
3. **Windows minimum environment:** port the proven capability with explicit platform differences.
4. **macOS minimum environment:** port only after Linux and Windows lifecycle/security findings are documented.

MVP-0 is not marketed as the complete BrainRoot product and is not evidence that Agent Host, tools, Canvas, or checkpoints are finished.

## MVP-0 Linux Model Loop

A Linux user can launch the minimal BrainRoot shell, configure an OpenCode Go credential without committing or logging it, discover currently available Go models, select one compatible model, send one prompt, receive a streamed response, cancel an active request, see understandable network/auth/rate-limit errors, and close the app with no model request or child resource left active.

MVP-0 explicitly excludes filesystem tools, shell tools, project mutation, autonomous agent loops, browser preview, checkpoints, Code View, Windows, and macOS. The API transport remains behind the Agent Adapter/provider boundary so this experiment does not redefine the final agent architecture.

## MVP outcome

A user can open an existing local web project, ask one supported agent for a bounded change, watch truthful progress, preview localhost in the Companion Canvas, run an observable validation, inspect a plain-language change summary, undo to a checkpoint, and resume basic task history—without opening code or a terminal.

Starting a brand-new project from an empty folder is desirable but enters the MVP only after the existing-project vertical slice is reliable; project scaffolding introduces ecosystem and network policy that should not block the core loop.

## Included capabilities

1. Select and open one workspace folder with Safe Mode scope.
2. Show one task-oriented Agent surface and persist basic session history.
3. Start one well-supported Agent Adapter on demand and terminate it cleanly.
4. Allow scoped file search/read/write and noninteractive command execution through the Tool Layer.
5. Create a checkpoint before meaningful writes and summarize changed files.
6. Start/stop one project dev server with readiness and failure detection.
7. Display one localhost preview in the Companion Canvas.
8. Run one on-demand Playwright validation path with screenshot/console evidence.
9. Present result, test state, Looks good / Undo / Show changes.
10. Restore the checkpoint without overwriting unrelated edits.
11. Offer minimal optional Code View: open changed file, highlighting, selection, search, basic edit, and diff navigation—only after the main loop works.

## Explicitly out of scope

Companion Deck/Phone, remote/social browsing, publish/deploy, marketplace, cloud sync, multiplayer, voice, mobile app, extension ecosystem, complete IDE/editor, many providers, visible agent teams, local models, automatic full indexing/embeddings, Replay, remote workspaces, hidden telemetry, and unattended production operations.

## MVP acceptance journey

1. Fresh launch stays within idle budgets and has no agent/LSP/PTY/dev server/automation browser.
2. User opens a known fixture and describes a UI change.
3. BrainRoot explains the plan and any needed permission in plain language.
4. A checkpoint is created; the single agent performs scoped work.
5. BrainRoot starts preview, waits for readiness, loads it, validates interaction, and reports evidence.
6. The user can use the preview, view a concise summary, and Undo.
7. Undo returns the fixture to its exact pre-task state while preserving unrelated changes.
8. Closing the project leaves no owned child process, PTY, listener, watcher, or extra WebView.
9. Reopening shows the high-level task history and last result; unsupported provider resume is labeled honestly.

## MVP-1 quality gates

- Critical journey first passes on the documented Linux reference environment. Windows and macOS receive separate minimum-environment milestones after the Linux behavior is proven.
- No remote content has privileged Tauri IPC.
- All numeric claims follow the performance measurement protocol.
- Core flow works with a deterministic fake agent in CI.
- No proprietary service is required to build/test the core; a remote model may be required only for the chosen live-agent experience.
- Project and dependency licenses are compatible with the open-source commitment.

## MVP-0 Linux quality gates

- Fresh launch has no model request, agent, LSP, PTY, dev server, automation browser, or extra content WebView.
- All tests except the explicitly opt-in live-provider smoke test work without a paid service or real credential.
- The provider integration has a deterministic fake HTTP server covering success, streaming, cancellation, timeout, malformed response, authentication failure, rate limit, and server failure.
- The OpenCode Go key is never placed in source, project state, fixtures, logs, screenshots, issue evidence, Wiki pages, or CI artifacts.
- Requests use a BrainRoot-specific user agent and a stable nonsecret `x-opencode-session` per conversation, as required by the current Go guidance.
- Endpoint/model compatibility is discovered or configured; code does not assume every Go model uses the same wire protocol.
- Linux build, unit/contract tests, security checks, cleanup checks, and the final batch CI pass on the latest commit.
- Performance results are labeled TARGET, MEASURED, or UNKNOWN and include the Linux environment.
