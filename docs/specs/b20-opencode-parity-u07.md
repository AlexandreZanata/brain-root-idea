# B20-U7 — Panel parity: file tabs, review, side panel, COLD terminal mirror

Issue [#136](https://github.com/AlexandreZanata/brain-root-idea/issues/136) · branch `batch/b20-opencode-parity` · pin `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769` (read-only) · `risk:high` (WebView lifecycle, memory ceiling)

Outcome: the Canvas side presets take the OpenCode panel shapes — file tabs, a review tab, a side panel — as COLD resources that open on demand and release everything on close, with exactly one HOT heavy view at a time, plus a terminal panel that honestly **mirrors turn output** instead of pretending a shell exists.

## 1. Pin sources (MEASURED byte sizes, verified against the git tree)

| Pin source | Bytes | Authority for |
|---|---|---|
| `pages/session/session-side-panel.tsx` | 40,776 | The side panel: `Tabs` strip `[Review] [Context] [file tabs…]` + an open-file trigger + a sticky `+` opener (the pin's `dialog-select-file`), drag-reorder of file tabs, middle-click/`closeButton` close, content region per tab |
| `pages/session/file-tabs.tsx` | 24,394 | The file view per tab (`FileTabContent`/`SessionFileView`): `FileVisual` (icon + `text-14-medium truncate`, **italic when the tab is temporary**), line comments, file search, scroll sync |
| `components/session/session-sortable-tab-v2.tsx` | 2,392 | File tab chrome: `FileVisual` + `IconButton icon="close-small" class="h-5 w-5"`, close on click, close on middle-click, dbl-click pins a temporary tab |
| `pages/session/review-tab.tsx` | 5,781 | The review tab: `SessionReview` with `diffStyle: "unified" \| "split"`, scroll restore, `title`/`empty` slots, line-comment actions |
| `pages/session/terminal-panel.tsx` / `terminal-panel-v2.tsx` | 11,820 / 13,853 | Terminal panel: `aside role="region"` labelled `terminal.title`, `h-10` header with tab/handoff labels, `rounded-[10px]`, `--v2-elevation-raised`, vertical `ResizeHandle` (`min=100`, `max=60%` viewport, `collapseThreshold=50` → close). **PTY-backed xterm** — the part that cannot exist here |
| `pages/session/file-tab-scroll.ts` | 1,813 | Tab-strip scroll sync: wheel → ±50 px horizontal, `MutationObserver(childList)`, rAF-coalesced update, teardown removes all three |

Per `docs/22` §2, U7's authority is the **v1 tree** (`pages/session.tsx` + its panels); the v2 files are read where they refine the same shape (terminal header, tab chrome).

## 2. Honest-state inventory (sequence step 2 — what does not exist)

| Pin capability | BrainRoot reality | What ships |
|---|---|---|
| Workspace/project listing → file tabs with real files | **No workspace listing exists** (same gap recorded in #133 and #135) | The file-tabs strip + content **shape**, honest empty state, **no fabricated path** |
| Diffs (`FileDiffInfo`/`SnapshotFileDiff`) → review tab | **No diff source exists** (the agent emits text turns, not diffs) | The review-tab **shape**, honest empty state; the unified/split toggle is **not rendered** while there are zero diffs — a control with nothing to act on is a fake control |
| PTY + Process Manager → real terminal | **No PTY, no Process Manager**; a real shell is a separate batch with its own permission design | `TerminalMirror.svelte`: the terminal-panel shape mirroring **real turn output** (`agent_event` / `conversation_event` text chunks), labelled as a mirror. No process, no PTY, no shell prompt |
| File tree side panel (`fileBrowserState` mode) | No workspace listing | Not ported; recorded in §5 |

## 3. What ships (behaviors, ported or recorded)

**Canvas presets** (`CanvasPanel.svelte`): the strip gains `Files`, `Review`, `Terminal` alongside the existing `Preview`/`Browser` heavy views; the B19 planned placeholders (`Components`, `Logs`, `AI Notes`) stay exactly as they are — honest `MVP-1` declarations are not U7's to remove. The body is driven by the registry alone (`openPanels(registry)[0]`): one `{#if}` slot, so the released view unmounts — its teardown runs — before the next mounts. With eight presets the strip wraps whole tabs at narrow window widths (a scoped `.br-tabs` rule in `CanvasPanel.svelte`; `theme.css` is untouched) — the browser pass caught it breaking labels into one character per line.

**`SidePanel.svelte`** (the side-panel shape): `role="region"` aside on the pin's geometry — `bg-v2-background-bg-base` (`--v2-background-bg-base`), `border-radius: 10px`, `--v2-elevation-raised`, overflow clipped — with a sticky header slot and a content slot. Files and Review presets render inside it.

**`FileTabs.svelte`** (the file tabs shape): the per-file tab strip (pin's `SortableTabV2` chrome: name, close revealed on hover and pinned on the active tab, middle-click closes) and the content region. With no workspace there are **zero tabs**; the strip renders its shape and the content shows the honest empty state. Wheel-scrolling the strip is ported (`file-tab-scroll.ts` behavior) with full teardown on destroy.

**`ReviewPanel.svelte`** (the review tab shape): title header + content slot on the pin's geometry (`pr-3`/`px-3`/`pl-3` rhythm from `review-tab.tsx`'s `classes`). Empty state only while no diff source exists.

**`TerminalMirror.svelte`** (the terminal mirror): the pin's `aside role="region"` + `h-10` header + monospace output body. It subscribes to the same streams App.svelte consumes (`AGENT_EVENT_NAME` envelope validated with `isAgentEventEnvelope`, `conversation_event`) and folds events into a bounded ledger of turns (`MIRROR_MAX_TURNS = 20`, `MIRROR_MAX_TEXT_CHARS = 200 KiB`, oldest dropped first; a lone over-budget turn keeps its tail). Text appends to **that session's running turn** — interleaved sessions keep independent turns in one ledger — and turn boundaries (`started`/`completed`/`cancelled`/`failed`) close the turn; a failed turn keeps its text and records the error. The header **names it a mirror** (`TERMINAL_MIRROR_LABEL = "Terminal — turn output mirror"`, note "This panel mirrors the turn output. There is no shell here yet."), so nothing on screen can be read as an interactive shell. Teardown: both `listen` unlistens run on destroy (and on the race where the listener arrives after disposal), before the component is gone.

**`src/panels.ts`** (the registry, pure): panel ids `preview | browser | files | review | terminal`; kinds `heavy` (WebView) / `light` (DOM); phases `closed | cold | hot`; `openPanel` returns the ids that **must tear down** when a second heavy view would go HOT — the one-HOT rule of [ADR 0006](../adr/0006-browser-webview-strategy.md) is enforced in the transition function and asserted by tests, not merely observed. Light panels open as `cold` and never claim `hot`.

**Lifecycle table** (owner = `CanvasPanel`'s preset switch, which unmounts the previous panel; each panel's cleanup runs in its own destroy path):

| Panel | Kind | Opens | Phase while active | Teardown on close/switch | HOT? |
|---|---|---|---|---|---|
| `preview` | heavy | preset select | `hot` | existing `onMount` cleanup: `hidePreview()`, event unlisten, resize listener removal, `boundsSync.dispose()` | yes, alone |
| `browser` | heavy | preset select | `hot` | existing `onDestroy`: `humanHide()` + unlisten/listener/`boundsSync` cleanup | yes, alone |
| `files` | light | preset select | `cold` | component unmount; wheel listener removed by its own teardown — the scroll chase runs from reactivity, so there is no observer/rAF to leak | no |
| `review` | light | preset select | `cold` | component unmount | no |
| `terminal` | light | preset select | `cold` | event unlisten on destroy | no |

`PreviewPanel.svelte`, `HumanBrowserPanel.svelte`, `src/preview.ts`, `src/humanBrowser.ts` need **no changes**: their existing destroy paths already satisfy the registry contract (verified line-by-line before this spec). Their allowlist entry permits the change; it does not require one.

## 4. Divergences from the pin, recorded

| Pin behavior | Decision | Reason |
|---|---|---|
| One side-panel strip mixing `[Review] [Context] [file tabs…]` | The Canvas preset strip carries `Files`/`Review`; the file strip carries only file tabs | The Canvas strip is the host-level navigator (Preview/Browser are BrainRoot-only); nesting a second strip that repeats the same destinations would hide them. The pin's per-file tab chrome is ported intact |
| `Context` tab (`SessionContextUsage`, context-in-tabs) | Not ported | No context-usage accounting exists; U2/U5 record the same no-invention rule |
| Terminal as a bottom-docked, vertically resizable panel with PTY tabs | Terminal is a Canvas preset filling the Canvas body | No PTY means no tab list to resize; the chat↔Canvas resize already exists (`PanelResizer.svelte`, unchanged). Collapse-to-close has nothing to collapse to |
| Unified/split diff-style toggle in the review tab | Rendered only when a diff source exists | A toggle that changes nothing with zero diffs is a fake control |
| `+` opener launching `dialog-select-file` | Not ported | No workspace listing (the #133 gap); an opener with nothing to open is a fake control |
| Drag-reorder of file tabs | Not ported yet | Zero tabs to reorder without a workspace; the pin's middle-click close and temporary-tab rules are ported where they apply |
| Nerd Font mono in the terminal | System mono stack (U1's recorded decision) | No web font ships; the Nerd Font gap stays open with the terminal reality |
| Tab strip geometry in `ui-tabs.css` (hidden scrollbars, fixed 48 px bar) | Ported to `FileTabs.svelte`; the Canvas preset strip keeps BrainRoot's `br-tabs` chrome | The Canvas strip is host chrome (B18), not the pin's file strip; only the 48 px file strip takes the pin's geometry. Narrow-width label wrapping is fixed scoped in `CanvasPanel.svelte` |

## 5. Acceptance mapping (deferred IDs live on #136; nothing here is claimed as run)

- `B20-U7-T01` — 20 open/close cycles return process-tree memory to within ±5 MB: the browser-level soak is run during the stage (MEASURED JS heap before/during/after, environment recorded); the process-tree release soak `cargo test --release soak -- --ignored` stays deferred at the release gate.
- `B20-U7-T02` — one HOT heavy view at a time, switching destroys the previous: enforced in `panels.ts` transitions (registry refuses a second `hot`), rendered by a single `{#if}` slot in `CanvasPanel`, asserted in `src/panels.test.mjs`.
- `B20-U7-T03` — no WebView, listener, interval, or temp file survives close: each panel's destroy path is named in §3's table; source assertions freeze the absence of stray `setInterval`/`addEventListener` without removal.
- `B20-U7-T04` — the terminal mirror is labelled a mirror and never spawns a process or PTY: visible label + `aria-label`, and a source assertion that `TerminalMirror.svelte` contains no `invoke`, `spawn`, `pty`, or shell command.
- `B20-U7-T05` — file tabs and review render honest empty states with no fabricated path: source + runtime checks that no invented path, filename, or diff appears.
- `B20-U7-T06` — keyboard access and focus order match existing Canvas behavior: new presets are the same `Button variant="tab"` controls the existing strip uses.

Budgets (docs/22 §3): CSS gzip ≤ 35 KB, JS gzip ≤ 60 KB (MEASURED at the stage), no new dependency.
