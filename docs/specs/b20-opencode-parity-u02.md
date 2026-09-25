# B20-U2 — Titlebar tab strip parity: behavior matrix and reconciliation

- **Status:** specification step of [#131](https://github.com/AlexandreZanata/brain-root-idea/issues/131). Markup rework follows in this same stage.
- **Pin:** `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769`, read-only.
- **Sources read:** `packages/app/src/components/titlebar.tsx` (27,680 B), `titlebar-tab-strip.tsx` (13,551 B), `titlebar-tab-nav.tsx` (14,501 B), `titlebar-tab-popover.tsx` (3,304 B), `titlebar-tab-gesture.ts` (567 B), `titlebar-tab-order.ts` (518 B).
- **Base:** `src/lib/SessionTabs.svelte` (122 lines), `src/lib/AppHeader.svelte` (131 lines).

## 1. Behavior matrix (pin is the authority)

| # | Behavior | Pin evidence | BrainRoot today | U2 target |
|---|---|---|---|---|
| 1 | Tab item geometry | `h-7` (28 px), `rounded-[6px]`, `px-1.5` (6 px), `gap-1.5` (6 px), label `text-[13px] font-medium` (`titlebar-tab-nav.tsx:188,231`) | `padding: var(--space-1) var(--space-2)`, `border-radius: var(--radius)` (**undefined**, see §4) | 28 px height, `--radius-control` (6 px, already the pin's value), `--font-size-small` (13 px), 6 px padding/gap |
| 2 | Active vs inactive text | Inactive `text-v2-text-text-faint`, active and editing `text-v2-text-text-base` (`:231`) | `.tab` `--text-muted`, `.tab.active` `--text` | Active `--text`; inactive uses the `[AA]`-safe role because the pin's faint token measures below 4.5 (§3) |
| 3 | Active surface | `group-data-[active='true']` raises the item surface; the strip itself keeps the titlebar background | `.tab.active` gets `--border` + `--surface` | Keep the existing surface treatment; do not invent a new one in this stage |
| 4 | Close control | `hover-reveal`: hidden at rest, revealed on hover, **always visible when active or editing** (`:291-293`) | Always visible when `tabs.length > 1` | Mirror the reveal rule; keep last-tab protection (ours, not the pin's) |
| 5 | Close must not drag or navigate | `onPointerDown` → `preventDefault()` + `stopPropagation()` (`:294-297`); `isTabCloseTarget` guards the drag (`titlebar-tab-gesture.ts`) | n/a (no drag yet) | Same guard; a pointer press on close never starts a drag |
| 6 | Navigation trigger | Mouse navigation happens on **pointer down**; the click handler ignores `detail > 0` so it only serves keyboard activation (`:225-230`) | `onclick` only | Pointer-down navigation with a keyboard path, so pressing then dragging still navigates as the pin does |
| 7 | Drag eligibility | `canStartTabDrag(pointerType)` → **touch cannot start a drag** (`titlebar-tab-gesture.ts`) | n/a | Pointer drag, excluded for `pointerType === "touch"`, so touch scrolling still works |
| 8 | Reorder scope | `mergeVisibleTabOrder(all, current, next)` reorders **only the visible subset** and maps it back into the full order (`titlebar-tab-order.ts`) | n/a | Same: the drag reorders what is on screen; hidden tabs keep their relative positions |
| 9 | Keyboard tab traversal | Strip-level keybinds `ctrl+shift+tab` / `ctrl+tab` for previous/next (`titlebar-tab-strip.tsx:236,244`) | none | Implement as the pin does, wrapping at both ends |
| 10 | Wrap-around | `adjacentTabKey` uses modulo, so both ends wrap (`titlebar-tab-order.ts`) | n/a | Same |
| 11 | Overflow handling | The strip is `overflow-x-auto no-scrollbar` with pointer-events-none **gradient fades** on both edges, `w-6`, `linear-gradient(… var(--v2-background-bg-deep) …)` (`titlebar-tab-strip.tsx:288-395`); overflow is detected as `scrollWidth > clientWidth` (`:259`) | `overflow-x: auto` only, no fades | Horizontal scroll plus edge fades from `--v2-background-bg-deep`, shown only while overflowing |
| 12 | Reaching a hidden tab | The keybinds in #9 — **there is no overflow menu in the pin** (§2.1) | n/a | Keybinds, as the pin does |
| 13 | Hover preview card | `titlebar-tab-popover.tsx`: opens after `OPEN_DELAY = 2_000`, closes at `CLOSE_DELAY = 0`, `SKIP_WINDOW = 500` skips the delay when hopping between tabs, `pointer-events: none`, non-interactive, shows project / title / path / server | none | A non-interactive hover preview with the same delays. Our data is thinner, so it shows what exists and omits absent rows instead of inventing them |
| 14 | Per-tab context menu | `MenuV2.Context` with exactly two items: **Rename** and **Close tab** (`titlebar-tab-nav.tsx:336-339`) | none | Out of scope for U2 — see §2.3 |
| 15 | Rename | Double-click the title → `contenteditable`; Enter saves, Escape reverts, blur saves; blocked while dragging, editing, or pending (`canOpenTabRename`) | none | Out of scope for U2 — see §2.3 |
| 16 | Update loader | A titlebar-level install/restart pill driven by `props.update = { installing(), version(), install() }` (`titlebar.tsx:49,124,611-636`) | no updater exists | Not built — see §2.2 |
| 17 | Per-tab activity indicator | **Not present in the pin** — no spinner, indicator, loading, streaming, or busy reference in `titlebar-tab-nav.tsx` | `●` next to a streaming tab | Keep as a BrainRoot feature. It is **not** claimed as parity |
| 18 | Reduced motion | The pin relies on Kobalte transitions and `tw-animate-css`; scroll-timeline animations are Chrome-only | `prefers-reduced-motion` disables all transitions globally | Any transition added here must stay inside that rule; the fades are static gradients, not animation |

## 2. Corrections to the issue (recorded before any code)

Three items in the issue's Outcome describe things the pin does not do. Implementing them would have produced invented parity, so the issue is corrected instead of quietly reinterpreted.

### 2.1 "Overflow popover when tabs do not fit" — the pin has no overflow menu

The pin handles overflow with horizontal scrolling, edge gradient fades, and the `ctrl+tab` / `ctrl+shift+tab` keybinds. The only "popover" in the tab area is the hover **preview card**, which is non-interactive and cannot list anything selectable. A dropdown of hidden sessions would be a BrainRoot invention presented as parity.

**Resolution:** implement the pin's fades plus keybinds. If a genuine overflow menu is wanted later, it is a new, separately justified feature.

### 2.2 "Update loader" — the capability does not exist

The pin's loader is a titlebar pill gated on an updater object. BrainRoot has no updater, no update feed, and no installer path, so there is nothing truthful to render. The plan already forbids faking absent capabilities (the `@` picker, file tabs, and the terminal follow the same rule).

**Resolution:** not built in U2. Recorded as absent until an updater exists.

### 2.3 Context menu and rename — real pin behavior, deferred by scope

Rename and the two-item context menu are genuine pin behaviors, but the issue's Forbidden scope rules out touching the session data model that rename would write to, and a context menu is a new dialog primitive that belongs with U6.

**Resolution:** out of scope here, recorded so the omission is a decision rather than an oversight.

### 2.4 "Activity indicator" — ours, not the pin's

Kept, because it is useful product behavior, but described as a BrainRoot feature. The parity claims for U2 do not include it.

## 3. Accessibility reconciliation

`scripts/check-accessibility.sh` rejects custom `tabindex`, static-element interaction, removed focus outlines, `user-select: none`, and the `disabled` attribute anywhere in `src/`.

| Where the pin conflicts | Pin | Chosen equivalent |
|---|---|---|
| Inactive tab label color | `text-v2-text-text-faint` = `#808080`, which U1 measured at **3.93:1** dark and **3.78:1** light on `bg-layer-01` — below WCAG AA 4.5 | Inactive labels use `--text-muted` (the U1 `[AA]` role decision). This is a visible, deliberate difference from the pin, not a defect |
| Hover preview trigger | `tabIndex={-1}` on the trigger (`titlebar-tab-popover.tsx:60`) | No tabindex at all. The preview is non-interactive and `pointer-events: none`, so it is neither focusable nor reachable — which needs no tabindex to express |
| Tab semantics | `data-slot="titlebar-tab-item"` on a `div` containing a `MenuV2.Context.Trigger` and a close button — no `role="tab"` anywhere | The current BrainRoot markup puts `role="tab"` on a wrapper containing buttons, which is invalid ARIA. U2 replaces it with real controls and drops the tablist/tab roles rather than keeping a broken pattern. The strip becomes a labelled group of real buttons |
| Pointer-only drag | Pointer reorder with no keyboard path | A keyboard reorder equivalent is required: the issue's decision stands. The pin's `ctrl+tab` traversal does not satisfy it, because traversal changes which tab is active while reorder changes order |
| Close revealed on hover only | `hover-reveal` with `opacity-100` when active/editing | The control keeps a real hit area and a visible focus ring whenever it is focused, so a keyboard user is never chasing a hidden control. `opacity` changes, never `display` or `visibility` |
| Strip scrolling | `no-scrollbar` hides the scrollbar | Keyboard traversal plus focus-visible scrolling means the scrollbar is not the only affordance |

Any new `svelte-ignore` needs its justification written here before it is added.

## 4. Defect found while specifying: `--radius` is undefined

`src/lib/SessionTabs.svelte:68,110` and `src/lib/ModelPicker.svelte:65` declare `border-radius: var(--radius)`.

- `--radius` is **defined nowhere in the repository**. A repository-wide search across `*.css`, `*.html`, `*.svelte`, and `*.ts` finds no `--radius:` declaration, and `git log -S"--radius:"` shows it was never added to `theme.css`.
- An undefined custom property makes the declaration invalid at computed-value time, so `border-radius` falls back to its initial value: **0**. The tabs and the model picker render with square corners today.
- This is a silent class of failure: the CSS parser accepts it, the build succeeds, and nothing warns.

**Resolution:** U2 fixes the two declarations in `SessionTabs.svelte` to `--radius-control` (6 px, which is also the pin's own `rounded-[6px]`). `ModelPicker.svelte` is **not** in U2's allowed files — it belongs to U5 — so it stays broken and is recorded here and in the batch journal.

**Follow-up recorded, not built:** a static check that fails when a `var(--token)` used in `src/` is not defined in `theme.css`, with an allowlist for properties set from JavaScript (`--agent-width` is set inline in `App.svelte:651` with a `var(--agent-width, 23rem)` fallback, and would be a false positive). This is the only mechanism that would have caught `--radius`.

## 5. Out of scope for U2

- Rename and the per-tab context menu (§2.3).
- The update loader (§2.2).
- Persisting tab order — it stays in memory for the session.
- Any restyling of surfaces outside this strip; U3–U7 own the rest.
- No new dependency, no drag-and-drop or animation library.

## 6. Verification plan

**Fast non-test micro-gate (run in this stage):**

1. `pnpm run check` — 0 errors, 0 warnings.
2. `sh scripts/check-docs.sh && sh scripts/check-security.sh && sh scripts/check-accessibility.sh` — clean, with no new `svelte-ignore`.
3. `git diff --name-only <start>...HEAD` — only the allowlisted paths.
4. `pnpm build` — CSS gzip ≤ 35 KB and JS gzip ≤ 60 KB, values recorded as MEASURED.

**Deferred to the versioned release gate (ADR 0014, written but not run):**

- Order helpers: `adjacentTabKey` wraps at both ends; `mergeVisibleTabOrder` reorders only visible keys and preserves hidden ones; large moves; single-tab and empty cases.
- Drag lifecycle: pointer capture, one coalesced update, commit on release, restore on cancel, blur, and lost capture; no drag from the close control; no drag for touch.
- Overflow: fades appear only while `scrollWidth > clientWidth`, and disappear when the strip fits.
- Keyboard: traversal wraps; the reorder equivalent moves one slot and stops at the ends; focus follows the moved tab; focus is never lost.
- Hover preview: 2 s delay, 0 s close, 500 ms skip window, non-interactive, and nothing announced to assistive technology.
- Canvas remount: switching, reordering, and overflow interaction must not remount the Canvas panel (a B19 guarantee).
- Cleanup: no timer, listener, or observer survives unmount or tab close; the preview delay timer in particular must be cleared.
