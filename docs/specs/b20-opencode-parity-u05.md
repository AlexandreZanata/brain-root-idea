# B20-U5 — Model selector parity

- **Status:** Implemented on `batch/b20-opencode-parity` as `IMPLEMENTED_UNVERIFIED`; runtime acceptance deferred to the versioned release gate (ADR 0014).
- **Issue:** [#134](https://github.com/AlexandreZanata/brain-root-idea/issues/134)
- **Pin:** `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769`, read-only.

## 1. Which authority governs which behavior

The pin ships three model surfaces. Naming them separately is the batch rule; blending them is how parity quietly becomes invention.

| Pin surface | Size | Role here |
|---|---|---|
| `ModelSelectorPopoverV2` + `ModelSelectorPopoverV2View` (`components/dialog-select-model.tsx`) | 21,202 B (whole file) | **The authority.** This is what the pinned composer's model button renders (`prompt-input-v2.tsx` passes `ModelSelectorPopoverV2` as `modelControl`), so it is the shape U4 left a placeholder for. |
| `DialogSelectModel` (same file, line 524) + `dialog-select-model-unpaid-v2.tsx` | 9,634 B | **Out of scope.** The full-screen dialog and the unpaid wall belong to U6's dialog system. `/model` opens the dialog in the pin; here it still moves focus to the control (U4). |
| `dialog-manage-models.tsx` | 11,108 B | **Out of scope.** The popover's last row is a `Manage models` action (`manageKey = "action:manage"`) that opens it. There is no manage screen yet, so no row is rendered — an inert "Manage" row would be a fake control. |

Supporting sources read: `dialog-select-model-search.ts` (609 B — the matcher, ported verbatim), `model-tooltip.tsx` (4,736 B — the tooltip's field list), `utils/search-keydown.ts` (the document-level type-to-filter helper), `pages/session/session-model-helpers.ts` (1,796 B — session/prompt model sync, not this surface).

## 2. Geometry, measured from the v2 view

| Piece | Pin |
|---|---|
| Popover | `w-[284px]`, `rounded-md`, `bg-v2-background-bg-layer-01`, `shadow-[var(--v2-elevation-floating)]`, `p-0`; `MenuV2` `placement="top-start"` `gutter={6}` |
| Search row | `flex h-7 items-center gap-2 rounded-sm pl-3 pr-2.5`, magnifier icon `shrink-0` in `--v2-icon-icon-muted`, bare input `h-7 min-w-0 flex-1 text-[13px] font-[440] leading-5`, `autocomplete/autocorrect/autocapitalize/spellcheck` off |
| Clear | only when the search is non-empty: `size-5 rounded-sm`, `hover:bg-v2-overlay-simple-overlay-hover`, `onPointerDown` prevented |
| Divider | `h-px bg-v2-border-border-muted` |
| Scroll | `max-h-[220px]` |
| Group label | sticky (`top-0 z-10`), `bg-v2-background-bg-layer-01 px-3`, the provider name, truncated |
| Row | menu radio row; the content is `<span class="truncate">{name}</span>` plus tag slots |
| Empty | `flex h-12 items-center px-3 text-[13px] text-v2-text-text-faint` |

## 3. Behavior, ported exactly

- **Order of work on open:** set `active` to the current model if it survives the filter, else the first option; then, on the next macrotask *and* the next frame, focus the search and `scrollIntoView({ block: "nearest" })` the active row. Closing resets both `search` and `active`.
- **Filter:** `matchesModelSearch(query, [name, id, provider.name])`, ported character for character — normalize to lowercase, replace runs of non-letter/non-number with a space, trim, collapse; then a compact form with the spaces removed; a value matches when **either** form contains the query's corresponding form. That is what makes `claude3.5` and `claude 3.5` both match.
- **Grouping:** filter → sort the flat list by model name → group by provider id → sort the groups by provider name. (`sortModelGroups` ranks a hardcoded `popularProviders` list first; see §5.)
- **Keys on the search field:** `Tab` is **not** intercepted (the browser keeps it); `Escape` closes and restores focus to the trigger; `ArrowDown`/`ArrowUp` move the active row by ±1 with modulo wrap; `Enter` (not composing) selects the active row; `Alt`/`Meta` modified keys are ignored. Moving the active row scrolls it into view on the next microtask.
- **Filtering resets the active row** to the first surviving result.
- **Selection** closes the popover and applies the model, and focus returns to the trigger.
- **Pointer:** pressing inside the popover never blurs the search field.

## 4. What BrainRoot adds, and why

- **Home / End** jump to the first and last option. The pin has no such keys; this issue's accessibility contract names them and they are standard listbox behavior. Recorded as an addition, not as parity.
- **A polite result-count live region** ("12 models", "3 of 460 models match"). The issue requires the count to be announced instead of the items, so a 460-row catalog does not flood assistive technology.
- **The tooltip's content** is the fields the catalog actually carries — name, provider, context, input price, output price — with the existing `? ctx` / `?/M` markers for unknowns. The pin's tooltip also shows input modalities, a reasoning verdict, and latest/free tags; `CatalogModel` has none of those fields and none are invented.

## 5. Deliberate divergences, recorded

| Pin behavior | Decision | Reason |
|---|---|---|
| Provider icons from `packages/ui/src/components/provider-icons/sprite.svg` | **Not ported** | The sprite alone is **282,211 B**, larger than the entire CSS budget; the provider name is already text |
| `popularProviders` ranking | **Alphabetical by provider name** | The popularity list is a pinned app constant for a provider set BrainRoot does not have; inventing a ranking would be fabricating product opinion |
| `Free` and `Latest` tags (`isFree`, `item.latest`) | **Not ported** | The catalog carries no such fields |
| `Manage models` row | **Not ported** | No manage screen exists (U6); the row would do nothing |
| `handleDocumentSearchKeydown`: a document-level capture listener that redirects typing into the search field | **Not ported** | The search field is autofocused, so typing already lands there; a document listener is one more thing that can outlive the popover |
| Tooltip portaled to the right (`placement="right-start"`, `gutter 6`, `openDelay 0`) | **Rendered inside the popover, below the list**, tied to the active row by `aria-describedby` | A portaled tooltip needs a portal primitive (U6) and a hover-only one is clipped by the 220 px scroll container and invisible to keyboard users |
| `w-[284px]` fixed | The composer's control slot caps the trigger; the popover is `min(284px, …)` | The agent panel can be 280 px wide, so a hard 284 px would overflow the viewport |
| `MenuV2` `placement="top-start"` | Right-anchored: the layer's right edge meets the trigger's, `bottom: calc(100% + 6px)` keeps the pin's 6 px gutter | The chat panel can sit at either viewport edge ("Swap sides"), so a left-anchored 284 px layer would run off-screen in one of the two orientations |
| Rows are `MenuV2.RadioItem` inside a menu | `role="listbox"` + `role="option"` rows with `aria-selected` | The repository's accessibility contract asks for the listbox pattern with `aria-activedescendant`, and the gate forbids a custom `tabindex`, so the rows are not focusable by design |

## 6. States, written before they were implemented

| State | Trigger | Rendered |
|---|---|---|
| Empty catalog | `models.length === 0` | The existing muted "no models" label as the trigger (unchanged closed state), and the pin's empty row inside the popover |
| No match | search filters everything out | `rounded` empty row: "No models match." plus the count region reading "0 of N models match" |
| Stale | the app's existing `stale` flag | The existing `stale` chip, now `role="status"` so its appearance is announced, and the trigger's `aria-label` says the catalog is stale |
| Inactive | `inactive` (a turn is running) | `aria-disabled` on the trigger and the open is refused, preserving the native control's guard from U4 |
| Error | The picker never fetches. A failed catalog refresh leaves either the previous list with `stale` set or an empty list | No error surface of its own; documented rather than invented |

## 7. Accessibility and cleanup

- The trigger is a real `<button>` with `aria-haspopup="listbox"`, `aria-expanded`, and `aria-controls` only while open.
- The search field carries `role="combobox"`, `aria-expanded`, `aria-controls`, `aria-autocomplete="list"`, and `aria-activedescendant` while a row is active; focus never leaves it.
- Groups are `role="group"` with `aria-label` set to the provider name; the visible sticky header is `aria-hidden` so the name is announced once.
- Rows are `role="option"` with `aria-selected` marking the current model, and the active row also carries `aria-describedby` to the tooltip element.
- No `tabindex`, no `disabled` attribute, no `outline: none`, and no removal of focus outlines.
- **No listener, timer, or observer is created.** The popover opens from a click, closes on `focusout` of the control, on `Escape`, and on selection; the sticky labels are CSS. There is nothing to release, and no `rAF` is retained past a frame.

## 8. Performance

The issue makes one MEASURED claim: filtering the full catalog stays within one frame of input. The measurement method and its result are recorded in the issue evidence, not here, and the environment is named with it. The pin does not virtualize and neither does this stage: 460 rows are 460 DOM nodes, which is what the pinned view does.

## 9. Deferred to the release gate

`B20-U5-T01`…`T06` from the issue, plus `pnpm run test:frontend` (including the new `src/modelPicker.test.mjs`) and `sh scripts/check-full-linux.sh`. Nothing in this section is claimed as passing.

## 10. Not claimed

No provider icon, star, free tag, latest tag, manage-models row, or unpaid wall exists, and no screenshot in this stage shows one. The selector is the composer's control, not a dialog; the dialog surface stays with U6.
