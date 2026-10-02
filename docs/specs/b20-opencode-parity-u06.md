# B20-U6 — Dialog and screen parity

- **Status:** Implemented on `batch/b20-opencode-parity` as `IMPLEMENTED_UNVERIFIED`; runtime acceptance deferred to the versioned release gate (ADR 0014).
- **Issue:** [#135](https://github.com/AlexandreZanata/brain-root-idea/issues/135)
- **Pin:** `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769`, read-only.

## 1. Which authority governs which behavior

| Pin surface | Size | Role here |
|---|---|---|
| `packages/ui/src/v2/components/dialog-v2.tsx` + `dialog-v2.css` | 3,717 B + 4,144 B | **The authority for the shell.** `Dialog` is Kobalte's dialog: modal, dismissible, focus-trapped by the library, `onOpenAutoFocus` focuses the first `[autofocus]` element. Its CSS fixes the sizes and slots. |
| `components/command-palette.ts` + `dialog-command-palette-v2.tsx` + `.css` | 9,221 B + 12,767 B + 4,689 B | **The authority for the palette.** `matchesEntry`, `groups`, `uniqueCommandPaletteEntries`, `preferredCommandEntries` (COMMON order with a 5-entry fallback), `move` (modulo wrap + `scrollIntoView`), and the four-key input handler are ported. |
| `components/settings-v2/dialog-settings-v2.tsx` + `settings-v2.css` + `general.tsx` | 4,753 B + 14,896 B + 20,378 B | **The authority for settings shape.** Vertical tab list with `SectionTitle` groups and a nav footer; panels use `settings-v2-section` / `settings-v2-list` / `settings-v2-row` geometry. |
| `pages/error.tsx` + `error-description.ts` | 13,256 B + 310 B | **The authority for the error screen shape:** title + description + a technical-details disclosure + an action row. |
| `components/dialog-usage-exceeded.tsx` | 1,318 B | **Shape only.** Title + description + two actions. Its quota source of truth does not exist here, so no quota number, percentage, or limit is invented. |
| `context/command.tsx` | 14,046 B | The palette keybind source: `DEFAULT_PALETTE_KEYBIND = "mod+k,mod+shift+p"` — both chords open the palette. |

Out of scope, with reasons: `settings-keybinds` (rebinding needs a settings backend — the table here is read-only), `dialog-manage-models` and provider auth flows (no manage screen, no credential entry surface), `pages/layout` home internals (no project/workspace listing exists; U4 recorded the same gap for the `@` picker).

## 2. Geometry, measured from the pinned CSS

**Shell (`dialog-v2.css`):** container `480×368` (`large` `640×480`; `x-large` `min(calc(100vw - 32px), 980px) × min(calc(100vh - 92px), 600px)`; `fit` sets `height: auto`), `border-radius: 6px`, `background: --v2-background-bg-layer-01`, `box-shadow: --v2-elevation-overlay`, scrim `--v2-overlay-simple-overlay-scrim`. Header `padding: 16px`; title 15px/20px; description 13px/20px `--v2-text-text-muted`; close button `20×20`, `border-radius: 4px`, `margin-top: -2px`, `margin-inline-start: auto`, hover `--v2-overlay-simple-overlay-hover`. Footer `padding: 16px; gap: 8px; justify-content: flex-end`.

**Palette (`dialog-command-palette-v2.css`):** container `width: min(calc(100vw - 24px), 640px)`, `height: auto; min-height: 280px; max-height: min(calc(100vh - 96px), 480px)`, `border-radius: 12px`, `background: --v2-background-bg-base`, `box-shadow: --v2-elevation-floating`, `margin-top: max(48px, calc((100vh - 480px) / 2))` — the top edge is pinned so height can grow and shrink without the layer jumping. Search row `padding: 6px` around a 36px input at 6px radius on `color-mix(--v2-background-bg-layer-02 60%, transparent)`, leading icon `padding-left: 12px`, `gap: 8px`. Results `gap: 16px` between groups, `padding: 6px 6px 8px`; group title `margin: 6px 0; padding: 0 12px`, 13px/16px `--v2-text-text-muted`; rows `height: 36px; padding: 0 12px; border-radius: 6px; scroll-margin: 6px 0`, active row `--v2-overlay-simple-overlay-hover`; title 13px/16px weight 530; description and meta 13px/16px muted. Empty state `min-height: 120px`, centered.

**Settings (`settings-v2.css`):** body `padding: 0`; the vertical tab list sits on `--v2-background-bg-layer-01`; tab header `padding: 40px 40px 32px` with a base-to-transparent gradient mask; tab body `gap: 36px; padding: 0 40px 40px`; section `gap: 16px`; section title 15px/1 `padding-bottom: 8px`; list `border-radius: 8px; background: --v2-background-bg-layer-01; padding-inline: 20px` with `inset 0 0 0 0.5px --v2-border-border-muted`; rows `padding-block: 20px; border-bottom: 0.5px --v2-border-border-base`, row copy `gap: 8px`, row title 13px/1 weight 530, row description 13px/20px `--v2-text-text-muted`; nav footer two 11px faint lines.

**Error (`pages/error.tsx`):** centered column `gap: 8px` for the text block and `gap: 36px` between blocks; title 15px-class heading; description weak text; details disclosure `max-height: 24rem`, monospace 12px; action row `gap: 12px`, `flex-wrap`, capped at 16rem.

## 3. Behavior, ported exactly

- **Open:** the palette opens on `mod+k` and `mod+shift+p` (the pin's own `DEFAULT_PALETTE_KEYBIND`); the shell focuses the first `[data-autofocus]` element like the pin's `onOpenAutoFocus` does. The marker is a data attribute, not the platform `autofocus`: that one also fires at page load and Svelte raises `a11y_autofocus` for it (recorded in §5).
- **Dismiss:** Escape (`cancel` on a native modal dialog), a press on the scrim (the pin's Kobalte dialog is dismissible; the scrim press is `event.target === dialog`), and the close button. Selection or a command run closes the layer first.
- **Palette filter:** `matchesEntry` ported character for character — lowercase `includes` over `[title, description, category]`, any match wins.
- **Grouping and dedupe:** `groups` by category in insertion order; `uniqueCommandPaletteEntries` dedupes by id; empty query shows `preferredCommandEntries` (pin's `COMMON_COMMAND_IDS` order intersected with what exists, else the first `ENTRY_LIMIT = 5`) plus recent sessions in place of the pin's recent files; a non-empty query shows matching commands plus sessions.
- **Keys on the palette input:** `ArrowDown`/`ArrowUp` move the active row with modulo wrap and scroll it into view on the next frame; `Enter` runs the active entry (still `preventDefault` with zero rows, exactly like the pin — Enter never submits underneath); `Escape` closes. `Tab` is not intercepted. **Filtering resets the active row to 0** (the pin's `createEffect`).
- **Settings:** the pin's two nav sections (`Desktop` → General, Shortcuts; `Server` → Servers, Providers, Models) map to five tabs of existing data; every row that has no backend is marked read-only in its own copy and writes nothing.

## 4. What BrainRoot adds, and why

- **The root-invoker focus chain.** The pin's `dialog.show()` stacks; here a command can hand off from the palette to another surface. `dialogs.ts` keeps the *root* invoker (the control pressed before any dialog opened) and restores focus to it when the last dialog closes, so a palette → settings handoff still returns focus to the button that started the chain. Tested with a fake focus target.
- **Native `<dialog>` + `showModal()`** supplies modal focus containment, inert background, and the `cancel` event with **zero** `tabindex` — the repository's accessibility gate forbids positive/negative `tabindex`, which rules out the roving-tabindex patterns both the pin's tabs and its Kobalte trap rely on.
- **Tabs as buttons.** Settings navigation uses real buttons with `aria-current`, following U2's precedent (`role="group"` + buttons) instead of `role="tab"` without roving focus, which would be invalid ARIA.
- **Listbox rows stay unfocusable**, mirroring U5: the search input is `role="combobox"` with `aria-activedescendant`, rows are `role="option"` with `aria-selected`.
- **Host tab** maps the pin's `servers.tsx` (server connections) onto BrainRoot's one real connection: the local sidecar, read-only status/address/version.
- **Home sessions** are real tabs with turn counts; the projects block is an honest empty state; planned entries use the existing `ActionCard` `MVP-1` badge convention.

## 5. Deliberate divergences, recorded

| Pin behavior | Decision | Reason |
|---|---|---|
| Kobalte `Dialog` + `TabsV2` + `MenuV2` | Native `<dialog>` and plain buttons | No new dependency (the batch forbids one), and the gate forbids the tabindex order those primitives use to manage focus |
| Hidden scrollbars (`scrollbar-width: none`) in settings and dialog content | Scrollbars visible | Hidden scrollables silently clip content from the people least likely to discover scrolling |
| `font-weight` 440/530/640 (variable-font axis) | 400/500/600 | U1 ships no web font; system stacks only carry integer weights |
| `font-variation-settings: "slnt" 0` | Not ported | Same reason |
| Settings rows that need a backend (language, sounds, editor, notifications, updates, display, keybind rebinding) | Not ported | No settings backend exists; a control that saves nothing is a fake control. The issue's own stop condition treats a settings backend as a separate issue |
| `DialogUsageExceeded` quota numbers, "don't show again" | Shape only: title + description + actions | No quota feature exists; no limit, percentage, or countdown is invented |
| Error actions Restart / Export logs / Report / Check for updates | Only actions that exist: `Start the sidecar`, `Close` | No updater, no log export, no report channel — a button that does nothing is worse than its absence |
| Provider icons in palette rows, session avatars | Not ported | Same sprite-and-asset reasoning as U5; rows are text + meta |
| `useDialog` stacking, `dialog.show()` chaining | Single-slot registry with the root-invoker chain (§4) | One modal at a time is simpler and avoids stacked scrims |
| `onOpenAutoFocus` on the first `[autofocus]` element | First `[data-autofocus]` element, focused after `showModal()` | The platform `autofocus` also runs at page load, and Svelte flags it (`a11y_autofocus`); same open behavior, no warning |
| Home = projects + new-session routes | Home is one dialog: sessions, projects empty state, planned entries | No workspace/project listing exists (recorded in U4's spec) |

## 6. States, written before they were implemented

| State | Trigger | Rendered |
|---|---|---|
| Settings read-only rows | No backend | Each such row's description ends with a plain read-only note; the row offers no control that pretends to save |
| Empty catalog / no selection | `models.length === 0` / no `selected` | Models tab says so in product language |
| Stale catalog | `staleModels` | The Models tab row shows the same stale wording the composer chip uses |
| Error screen | Host `failed` phase | Existing failure copy as title and description; technical detail (code + message) behind a disclosure; `Start the sidecar` only when a start would do something |
| Home with zero extra sessions | One tab | Sessions section lists it; no empty-state noise for a single session |
| Palette no match | Query filters everything | `No commands match.` in the pinned 120px centered state |

## 7. Accessibility and cleanup

- One modal at a time, opened from a real button or the `mod+k` / `mod+shift+p` chords; Escape closes; the scrim press closes; the close button closes. Focus returns to the root invoker.
- No `tabindex`, no `disabled` attribute, no `outline: none`, no `user-select: none` — the gate rejects all four and is not weakened. Disabling is expressed as `aria-disabled`, following `Button.svelte`.
- The dialog is `aria-labelledby` its title; the palette search has a bound label and `role="combobox"` + `aria-activedescendant`; settings sections are labelled groups.
- **Cleanup by construction:** the only listener is the app-level keydown for the palette chords (`svelte:window`, removed with the component). No dialog installs a listener, timer, or observer; the palette's `scrollIntoView` uses a single `requestAnimationFrame` that is never retained past its frame.

## 8. Performance

No new measured budget in this stage's acceptance beyond the batch bundle ceilings (CSS ≤35 KB gzip, JS ≤60 KB gzip). MEASURED bundle numbers are recorded in the issue evidence with their environment. The palette renders at most `ENTRY_LIMIT`-bounded groups plus sessions; settings panels render five small tab bodies and none of them fetches.

## 9. Deferred to the release gate

`B20-U6-T01`…`T06` from the issue (dialog focus/Escape/invoker return; palette lists only real commands and runs them; screenshot and DOM carry no secret-shaped value; settings render without a backend and never write one; no permission or question dock is reachable; no dialog survives close), plus `pnpm run test:frontend` (including the new `src/dialogs.test.mjs`) and `sh scripts/check-full-linux.sh`. Nothing in this section is claimed as passing.

## 10. Not claimed

No permission dock, no question dock, no keybind rebinding, no language/sounds/editor/notifications/updates settings, no quota dialog content, no updater, no log export, no report channel, no project or workspace listing, and no screenshot in this stage shows one. The docks stay out because a UI that appears to grant or deny agent authority without an enforced boundary (ADR 0015/0016) would be a false capability claim.
