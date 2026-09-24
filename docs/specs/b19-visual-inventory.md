# B19 visual inventory and fixed spec (B18 tip — provisional until the merge freeze)

- **Status:** frozen inventory for [B19-S01](https://github.com/AlexandreZanata/brain-root-idea/issues/120); read-only inspection, **no UI code changed**.
- **Baseline tree:** B18 tip `f4919f4` on `batch/b18-fluid-canvas`. Every defect below cites exact source lines; every after-state is a plan `TARGET`, never measured fact. Re-verify against the merge SHA post-merge.
- **Root font size:** no override exists in source, so `1rem` computes to the 16 px default below.
- **Plan:** [B19–B20 frontend system plan](b19-frontend-system-and-usability-plan.md).

## S02 owner — tokens and shared CSS (`src/lib/theme.css`, 620 lines)

| Area | Before (source) | Defect vs contract | After (`TARGET`) |
|---|---|---|---|
| Spacing scale | Scattered gaps: rail `0.3rem` (`theme.css:76`), composer actions `0.4rem` (`ConversationPanel.svelte:211`), browser controls `0.5–0.6rem` (`HumanBrowserPanel.svelte:358-363`), workspace `0.9rem` (`App.svelte:345`), tabs `0.25rem` (`theme.css:264`) | No semantic 4/8/12/16/24/32 px scale; siblings carry ad-hoc values | Adopt `0.25/0.5/0.75/1/1.5/2rem` scale; parent owns sibling gaps |
| Radii | `--radius-surface: 0.6rem` (9.6 px), `--radius-control: 0.45rem` (7.2 px) (`theme.css:2-3`) | Close to but not exactly the proposed 10 px / 6 px | Panels 10 px, controls 6 px (`TARGET`) |
| Body type | `.br-message` 0.84rem (13.4 px), `.br-error/.br-note` 0.84rem, `.br-card__body` 0.8rem (12.8 px) (`theme.css:317-361`) | Below the 14–16 px body floor | Body 14–16 px (`TARGET`) |
| Smallest type | `.br-badge` 0.62rem ≈ 9.9 px (`theme.css:136`); `.br-btn--rail` 0.65rem (`theme.css:222`); `.br-message-label`/`.turn-status` 0.68rem (`theme.css:330`, `Turn.svelte:84`); `.br-chip`/`.br-card__subtitle` 0.7rem (`theme.css:150,436`); taglines/labels 0.72rem | Six usages under the 12 px supporting-text floor | Supporting text ≥ 12 px; no sub-12px badges (`TARGET`) |
| Button base | `.br-btn` padding-driven (`0.45rem 0.85rem`, no min-height) (`theme.css:161`) | No standard 40 px / compact 36 px heights; peer equality unenforceable | Standard 40 px, compact 36 px; equal height+width peers (`TARGET`) |

## S03 owner — primitives (`Button.svelte` 32 lines, `Icon.svelte` 84 lines)

- `Button.svelte:23-31` renders six variants with `aria-disabled` focus retention (keep); nothing constrains peer dimensions, long labels, or icon-only naming beyond `title`.
- `Icon.svelte:2-17` fixes 15 named icons (`chat`…`grid`) with `aria-hidden`; sizes sm 0.9 / md 1.15 / lg 1.6rem (`Icon.svelte:70-83`). Icon-only usages (`ThemeToggle`, rail) rely on the host button's text/`title` for naming — S03 names every icon-only control explicitly.
- After (`TARGET`): size/variant/state rules, long-label handling, equal-height/width grouping hooks consumed per-surface later; no icon package.

## S04 owner — composer (`TextArea.svelte` 37 lines, `ConversationPanel.svelte` 236 lines)

- Hierarchy: hidden label + field (`TextArea.svelte:27-37`, bound label verified), model chip, status text, Send (primary, icon+text) + Cancel (secondary, text-only) peers (`ConversationPanel.svelte:140-153`).
- Defects: peers are padding-driven with icon-vs-text content, so equal height/width is structural luck, not a rule; `.composer-bar` wraps at ≤640 px with status forced full-width (`ConversationPanel.svelte:221-235`) — reflow path exists, peer equality inside it unverified visually.
- After (`TARGET`): Send/Cancel equal peer height+width on one baseline, preserved keyboard/IME/cancel/focus/scroll, narrow stacking keeps peer equality per group.

## S05 owner — header (`AppHeader.svelte` 131 lines, `ThemeToggle.svelte` 15 lines)

- Hierarchy: brand mark 2.1rem + `h1` 1.05rem (16.8 px) + tagline 0.72rem (`AppHeader.svelte:65-93`); right side health dot + status 0.82rem + detail 0.72rem + ghost theme toggle (`AppHeader.svelte:95-115`, `ThemeToggle.svelte:8-15`).
- Defects: three competing text sizes plus a ghost button share one row with no shared baseline token; tagline/detail hide ≤640 px (`AppHeader.svelte:117-130`) while the toggle label stays — hierarchy under constraint unverified visually.
- After (`TARGET`): one baseline/hierarchy for identity, state, and theme control; technical labels accessible but non-dominant; real failures never hidden.

## S06 owner — rail (`WorkspaceRail.svelte` 21 lines, `NavItem.svelte` 17 lines)

- Hierarchy: six sections, only Build active; inactive items carry `title="… — planned for MVP-1"` (`NavItem.svelte:9-17`).
- Defects: rail buttons are padding-driven (`0.55rem 0.25rem`) with 0.65rem labels (`theme.css:216-234`) — no 44 px-scale target size, icon/label alignment unverified visually; five of six destinations are honest planned entries (keep, do not activate).
- After (`TARGET`): consistent target size, aligned icon/label, selected/hover/focus states, narrow reflow; no new destinations.

## S07 owner — edge and workspace (`PanelResizer.svelte` 151 lines, `App.svelte` 394 lines)

- B18 released contract (audit, do not reimplement): 24 px hit area, 2 px edge with hover/focus/drag contrast, native-range keyboard semantics, one-rAF coalescing, cancel/blur/teardown cleanup (`PanelResizer.svelte`, `theme.css` resizer block).
- Swap bar (new since the plan): `.workspace-bar` + `.workspace-side-note` 0.78rem (`theme.css:561-573`, `App.svelte` bar markup) — audit against the released edge/keyboard/focus rules; DOM order preserved by design (see B18-S04).
- After (`TARGET`): released behavior preserved; only tracked regressions get product-code fixes.

## B20 owners — inventoried now, refined later (no B19 change)

| Components | Current state (source) | Owner |
|---|---|---|
| `Turn.svelte` (104) | Status chip + collapsible long answers with `aria-expanded`/`aria-controls`; toggle is an underlined 0.72rem text button (`Turn.svelte:25-46,93-103`) | B20-S01 |
| `Badge.svelte` (7) | `0.62rem` uppercase badge (`theme.css:130-140`) — smallest type on the product surface | B20-S01 |
| `ActionCard.svelte` (21), `WelcomeCard.svelte` (14) | Row-card with icon/text/badge; accent block card with heading/body (`br-card*` rules `theme.css:376-438`) | B20-S02 |
| `SuggestionItem.svelte` (11), `EmptyState.svelte` (15) | Row button + chevron; centered icon/title/description block (`br-empty*` rules `theme.css:440-471`) | B20-S03 |
| `CanvasPanel.svelte` (204) | Browser-first tabs (content-width, 0.74rem), swipe zone `touch-action: pan-y`, transition note, one-HOT cold recreate | B20-S04 |
| `HumanBrowserPanel.svelte` (505) | Address/Go grid row, Back/Forward/Reload content-width peers, confirmed Clear-data, denial copy, bounds Retry row | B20-S05 |
| `PreviewPanel.svelte` (515) | Command/folder grid, Start/Stop content-width peers, preset tabs, custom size grid, size readout, bounds Retry row | B20-S06 |

Adjacent content-width peers (Send/Cancel, Back/Forward/Reload, Start/Stop, tab groups) are the concrete equal-height/width defects each owner packet resolves; none is “fixed” by this inventory.

## Frozen reference mapping

Each row: exact linked section → the single BrainRoot rule it justifies → do-not-copy note. References are inspection-only; they never override measured budgets or the security model.

| Reference section | BrainRoot rule | Do-not-copy |
|---|---|---|
| [GNOME HIG Buttons](https://developer.gnome.org/hig/patterns/controls/buttons.html) | Adjacent peer buttons share height and width (composer, browser nav, preview actions, tabs) | No GNOME branding, no GTK widgets |
| [GNOME HIG Typography](https://developer.gnome.org/hig/guidelines/typography.html) | Restrained scale: body 14–16 px, supporting ≥ 12 px, headings 16/20 px, OS scaling respected | No wholesale type scale paste; values stay BrainRoot tokens |
| [GNOME HIG Adaptive](https://developer.gnome.org/hig/guidelines/adaptive.html) | Validate 320/640/1024/1440/1920 px and 200 %/400 % zoom without truncating primary actions | No Adwaita theme |
| [Carbon Spacing](https://carbondesignsystem.com/elements/spacing/overview/) + [Button usage](https://carbondesignsystem.com/components/button/usage/) | Semantic 4/8/12/16/24/32 px scale; one high-emphasis action per group; parent owns gaps | No Carbon React/Sass, no visual skin |
| [W3C APG Window Splitter](https://www.w3.org/WAI/ARIA/apg/patterns/windowsplitter/) + [WCAG 2.2](https://www.w3.org/TR/WCAG22/) | Edge stays discoverable/focusable/operable without dragging; reflow, focus-visible, non-drag alternative, target size | Guidance only; validate on the Linux stack, claim no conformance |
| [Penpot design tokens](https://help.penpot.app/user-guide/design-systems/design-tokens/) + [variants](https://help.penpot.app/user-guide/design-systems/variants/) | One token/variant vocabulary for light/dark, size, emphasis, states (S02) | Inspect the model only; no MPL source in BrainRoot |
| [Excalidraw source `dc2c16d`](https://github.com/excalidraw/excalidraw/tree/dc2c16d9e2073078cb4b8f3dce942133b90eec3d) | Contextual Canvas controls with a dominant work surface (B20-S04 orientation) | No iconography, canvas runtime, or layout copying (MIT source, whiteboard domain) |

## Visual proof

Deferred: sanitized screenshots were already unavailable at S00 (no capture tooling; no mocks substituted). B19-S02+ captures per-surface before/after states at its release gate with maintainer assistance; through S01, equality of peer boxes and contrast stay structural findings, not visual claims.
