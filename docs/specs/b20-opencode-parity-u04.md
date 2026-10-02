# B20-U4 — Composer v2 parity

- **Status:** Implemented on `batch/b20-opencode-parity` as `IMPLEMENTED_UNVERIFIED`; runtime acceptance deferred to the versioned release gate (ADR 0014).
- **Issue:** [#133](https://github.com/AlexandreZanata/brain-root-idea/issues/133)
- **Pin:** `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769`, read-only.
- **Authority:** `packages/session-ui/src/v2/components/prompt-input/index.tsx` (**27,989 B**, the actual v2 shape) and its app-side wiring `packages/app/src/components/prompt-input-v2.tsx` (21,690 B), `pages/session/composer/prompt-model-selection.ts` (4,703 B), `pages/session/use-composer-commands.tsx` (3,255 B). The 61,170 B `components/prompt-input.tsx` (v1) was **not** read as a reference, per the batch decision.
- **Read to fix behavior, not to imitate code:** `v2/components/prompt-input/machine.ts` (10,420 B — the key reducer), `interaction.ts` (17,173 B — the effect layer), `types.ts`, `store.ts`; `components/prompt-input/contracts.ts`, `placeholder.ts`. `submit.ts` (20,350 B) and `context-items.tsx` were opened only to confirm what the submit path and the attachment strip own.

## 1. What the pin's composer actually is

Measured from the v2 source, not guessed:

| Piece | Pin geometry |
|---|---|
| Form | `min-h-[96px] w-full overflow-clip rounded-xl bg-v2-background-bg-base` + `shadow-[var(--v2-elevation-raised)]`; a dashed `--v2-icon-icon-info` border replaces the shadow while a file drag is active |
| Editor | `min-h-[60px] max-h-[180px] overflow-y-auto px-4 pt-4 pb-2 text-[13px] font-[440] leading-5`, `whitespace-pre-wrap` |
| Control row | `flex h-11 items-center px-2`; left group `flex min-w-0 flex-1 items-center gap-1`; submit at the right |
| Submit | `size-7 rounded-md p-[6px]` (28 px), `variant="primary"`, `--v2-elevation-button-contrast`, and a **two-layer gradient** `linear-gradient(180deg, alpha-light-20 → transparent), linear-gradient(90deg, bg-contrast → bg-contrast)` |
| Add menu | `IconButtonV2` `plus`, `ghost-muted`, `size="large"`; popover `min-width: 180px` with `Attach · separator · Commands (/) · Context (@) · Shell (!)`, `placement="top-start"`, `gutter=6` |
| Selects | `ButtonV2 ghost-muted size normal`, `max-w-[220px] justify-start`, label `truncate capitalize leading-5`, trailing `chevron-down`; menu is a radio group |
| Model button | same trigger shape, `height: 28px`, provider icon at `opacity-40` rising to 100 on hover, `font-weight: 440` |
| Popover | `absolute inset-x-0 -top-2 z-40 max-h-80 -translate-y-100 flex flex-col overflow-auto rounded-xl bg-v2-background-bg-base p-2 shadow-[var(--v2-elevation-raised)]`, plus `onMouseDown preventDefault` so a click never blurs the editor |
| Popover row | `flex w-full items-center gap-2 rounded-md px-2 py-1`, active = `bg-v2-overlay-simple-overlay-hover`, label `shrink-0`, description `truncate text-v2-text-text-muted`, keybind right-aligned muted |
| Empty state | `px-2 py-1 text-v2-text-text-muted` — the popover still opens and says so |
| Command search | only in the `command-menu` variant: a bare `text-[13px]` input, `placeholder="/"`, autofocused on the next frame |

## 2. Behavior matrix (ported exactly)

**Triggers**, both regexes copied:

- context: `value.slice(0, cursor).match(/(?:^|\s)@([^\s@]*)$/)` → the `@` popover, anchored to the *last* `@` after a space or start.
- command: `value.match(/^\/(\S*)$/)` → the palette, but **only when the whole draft is one slash token**. `look at /mod` does **not** open it. This is the pin's behavior and an easy thing to get wrong.
- `!` → shell mode. **Not ported:** there is no PTY (U7 owns the terminal), so a `!` command would be a fake shell. Untouched, so `!` stays ordinary text.
- `command-menu`: the `+` menu's *Commands* item. If the draft is empty it appends `/` and opens the inline palette; if the draft is non-empty it opens the same list as a menu with its own search field.

**Keys** (the pin's reducer, `machine.ts:keyDown`):

| Key | Open | Closed |
|---|---|---|
| `Escape` | close, refocus the editor, **no submit and no cancel** | falls through to BrainRoot's `cancel` |
| `ctrl+g` | close, refocus the editor | falls through |
| `Enter` (not composing) | select the active item; if none is active, consume the key and do nothing | submit |
| `Tab` | same as `Enter` | natural tab order |
| `ArrowDown` / `ctrl+n` | next, wrapping | not handled |
| `ArrowUp` / `ctrl+p` | previous, wrapping | not handled |

Selection with nothing active starts at index 0 going down and at the last index going up; the wrap is `(current + direction + length) % length`. On a filter change the active id becomes the first result, but survives if it is still in the list. Selecting a command rewrites the trigger: inline replaces the leading `/query`, the menu variant prepends `label + " "` to the existing draft. After any selection the popover closes and focus returns to the editor.

**Space** does not close the `@` popover (the query simply stops matching). The pin re-filters on every input event and scrolls the active row into view with `scrollIntoView({ block: "nearest" })`.

## 3. What BrainRoot does not have — and does not fake

| Pin capability | Decision | Why |
|---|---|---|
| Image/file attachments, drag-and-drop strip, paste-to-attach | **Not built** | No attachment pipeline and no workspace; the strip, the drop overlay, and the `Attach` menu item are absent rather than inert |
| Shell mode (`!`), `Shell` menu item | **Not built** | Needs the Process Manager with a PTY (U7); a shell-backed composer with no shell is a false capability |
| Tool toggles, model variant select | **Not built** | BrainRoot has neither tool permissions nor model variants; a toggle that toggles nothing is a fake control |
| `@` file/directory/MCP-resource listing | **Shell only, empty state "No workspace is open yet"** | The honest-empty-state decision recorded at batch open; a real list needs a project-listing backend and its own decision |
| Update loader, per-tab spinner | **Not built** | Already deferred by U2 |
| The pin's slash catalog (`share`, `unshare`, `new`, `undo`, `redo`, `compact`, `fork`, `export`, `open`, `terminal`, `mcp`, `agent`, `model`) | **Subset only** | Every entry must map to an action that exists; the table in §4 records the mapping |
| Auto-grow via `contenteditable` + mention pills | **Auto-growing `<textarea>`** | The issue names a textarea; a textarea cannot hold inline pills, and pills exist to carry file/agent mentions that §3 has no source for |
| Composer status text | **Kept** | BrainRoot's understandable-progress rule; the pin reports progress in the timeline instead. Recorded divergence, not parity |

## 4. Slash catalog — what is shipped and why

| `/trigger` | Title | Real action | Pin origin |
|---|---|---|---|
| `/agent` | Switch agent | cycles Plan ↔ Build through the existing `agent_host_set_agent` call | pinned `agent.cycle`, `slash: "agent"` |
| `/plan` | Plan mode | sets Plan explicitly | BrainRoot addition — the pin cycles a many-agent list; two modes make an explicit form worth it |
| `/build` | Build mode | sets Build explicitly | BrainRoot addition, same reason |
| `/model` | Choose model | moves focus to the composer's model control | pinned `model.choose`, `slash: "model"`; the pinned action opens a dialog, which is U5's surface |

Everything else the pin offers is absent because its action does not exist yet. `/undo`, `/redo`, `/new`, and `/fork` were considered: the first two have no checkpoint command wired to the composer, `/new` is owned by `App.svelte` (**outside this stage's allowlist**), and `/fork` needs the message identity that #138 would add.

## 5. Accessibility

The pin's popover is a bare `div` of `<button>`s reached with `ArrowUp`/`ArrowDown` while the editor keeps focus. That is a combobox in behavior and nothing in the markup, so this stage states the pattern instead of copying the markup:

- the editor is `role="combobox"` with `aria-expanded`, `aria-controls`, `aria-autocomplete="list"`, and `aria-activedescendant` only when a row is active;
- the list is `role="listbox"` with `role="option"` rows carrying `aria-selected` — rows are **not** focusable buttons and the editor never loses focus;
- the empty state is plain text, not an empty listbox;
- `aria-disabled` instead of the `disabled` attribute (the repository gate forbids `disabled`, because it drops focus);
- no `tabindex` and no `outline: none`, both gate-enforced;
- Escape returns focus to the editor with the caret where the user left it.

## 6. Performance and cleanup

- The popover is markup in the composer, not a component tree or a portal: opening it creates no listener, timer, observer, process, or WebView. Closing removes it. Nothing survives the close, and the `Listen`-free design is why this stage adds no teardown handler to audit.
- The auto-grow runs on the textarea's own `input`/`scrollHeight` read inside one frame; it allocates nothing.
- No dependency added (`New dependency allowed: no`).
- Bundle budgets (TARGET, `docs/22` §3): CSS ≤ 35 KB gzip, JS ≤ 60 KB gzip. MEASURED values are in the issue evidence comment, not claimed here.

## 7. Deferred to the release gate

Written now, run at the versioned gate (`B20-U4-T01`…`T05` from the issue), plus `pnpm run test:frontend` including `src/composer.test.mjs`, and `sh scripts/check-full-linux.sh`. Nothing in §7 is claimed as passing.

## 8. Scope notes and not claimed

**Two paths outside the issue's allowlist were touched, both following the U2 precedent.** `package.json` gains `src/composer.test.mjs` in `test:frontend`, because an unregistered test never runs — exactly why U2 registered `src/sessionTabs.test.mjs`. And `docs/22`'s status line was corrected: it still said U2–U7 were open while U2 was closed and U3 had stopped.

No attachment, shell, tool-toggle, variant, or real `@` listing exists, and no screenshot in this stage shows one. The model control remains the existing native `<select>` inside the composer's control row, restyled to the pinned trigger geometry; U5 replaces it with the searchable v2 selector. `ModelPicker.svelte`'s undefined `--radius` is overridden from the composer's own stylesheet rather than edited, because that file belongs to U5 — the override lives in `Composer.svelte`, inside this stage's allowlist.
