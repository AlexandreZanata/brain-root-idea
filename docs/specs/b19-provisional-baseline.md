# B19 provisional baseline (B18 tip — freeze to the merge SHA post-merge)

- **Status:** provisional rebaseline for [B19-S00](https://github.com/AlexandreZanata/brain-root-idea/issues/119); **not** the frozen B19 baseline.
- **Baseline tree:** B18 batch tip `f4919f4` on `batch/b18-fluid-canvas` (B18 PR #108 OPEN, CI green, review pending at the time of writing).
- **Follow-up (required):** once B18 merges, a tracked follow-up freezes this note to the merge SHA — re-verifying every number below and superseding any that diverge. Until then, the B18 record (`docs/history/batches/B18-fluid-canvas.md`) wins on any conflict.
- **Update 2026-09-25:** B18 merged into `main` at `1dbf7de`, so the baseline above is no longer provisional in that respect; this note was **not** re-frozen line by line. The `0.0.15` gate in `docs/history/batches/B19-foundation.md` supersedes the size/performance absolutes below and records the same-numbers caveat, because the pivot also landed on the same branch.
- **Plan:** [B19–B20 frontend system plan](b19-frontend-system-and-usability-plan.md) (verbatim from `origin/codex/b19-frontend-system-plan` `f83dca3`); its visual/interaction values remain `TARGET`, never measured fact.

## Component list (B18 tip)

Roles from source inspection (B18-S00..S06) and file naming; [B19-S01](b19-frontend-system-and-usability-plan.md) verifies each component's action hierarchy, size/alignment/state defects, and target tokens. Line counts are Hindley-style size hints, not quality claims.

| File | Lines | Role at B18 tip |
|---|---|---|
| `src/App.svelte` | 394 | Shell grid (header, swap bar, workspace); swap-side state; conversation + stream-buffer wiring |
| `src/lib/theme.css` | 620 | Semantic tokens (palette, radii, type) and shared rules: panels, buttons, tabs, fields, rail, resizer edge, swap bar, swipe zone, responsive stacking |
| `src/lib/PreviewPanel.svelte` | 515 | Dev-server form, viewport presets (phone default), centered phone slot, latest-wins bounds sync with Retry |
| `src/lib/HumanBrowserPanel.svelte` | 505 | Address bar, Back/Forward/Reload peers, confirmed Clear-data, phone-slot native view, latest-wins bounds sync with Retry, empty-destination menu |
| `src/lib/ConversationPanel.svelte` | 236 | Build surface: setup status, turn history, suggestions, composer with Send/Cancel peers |
| `src/lib/CanvasPanel.svelte` | 204 | Preview/Browser destination tabs (browser-first), header/chrome swipe, honest transition note, one-HOT cold recreate |
| `src/lib/PanelResizer.svelte` | 151 | Edge-only divider: 24 px hit area, pointer capture, one-rAF coalescing, semantic native range preserved |
| `src/lib/AppHeader.svelte` | 131 | Brand identity, core health status/detail, theme control host |
| `src/lib/Turn.svelte` | 104 | Per-turn status chip and collapsible long answers (naming role; verified at S01) |
| `src/lib/Icon.svelte` | 84 | Named icon set (`IconName`; naming role; verified at S01) |
| `src/lib/TextArea.svelte` | 37 | Labeled composer field (`id="prompt"`; naming role; verified at S01) |
| `src/lib/Button.svelte` | 32 | Button primitive, six variants, focus retention via `aria-disabled` |
| `src/lib/ActionCard.svelte` | 21 | Card primitive for actions (naming role; verified at S01) |
| `src/lib/WorkspaceRail.svelte` | 21 | Workspace section navigation (Build/Agents/Browser/Files/Terminal/Settings) |
| `src/lib/NavItem.svelte` | 17 | Rail item wrapper (naming role; verified at S01) |
| `src/lib/ThemeToggle.svelte` | 15 | Theme switch control (naming role; verified at S01) |
| `src/lib/EmptyState.svelte` | 15 | Empty-state primitive (naming role; verified at S01) |
| `src/lib/WelcomeCard.svelte` | 14 | First-run welcome card (naming role; verified at S01) |
| `src/lib/SuggestionItem.svelte` | 11 | Suggestion row control (naming role; verified at S01) |
| `src/lib/Badge.svelte` | 7 | Status badge primitive (naming role; verified at S01) |

## Measured size/performance baseline (reused from B18-S07 evidence)

Same host/session pair B17 `5f2835c` vs B18 `211af91`, release builds, governor performance, AC online; full procedure in [the B18 baseline report](performance-reports/b18-fluid-canvas-baseline.md). B19 work must compare same-session against the **merged B18** head and report deltas honestly.

- Startup to readiness (n=10): B17 median 0.856 s → B18 median 1.007 s, p95 1.028 s (TARGET ≤ 1.0 s p50 misses by 7 ms on B18).
- Settled idle CPU (5 windows): B17 median 0.069 % → B18 median 1.345 % (windows 1–2 settling; `< 1 %` budget fails on B18's median).
- Process-tree PSS median: 249.3 MB → 252.1 MB (150 MB target fails since B01).
- Release binary 8,085,800 B (+4,224); frontend JS 87,614 B gzip 30,205 (+5,546); CSS 16,667 B gzip 3,559 (+2,032).
- Deck one-HOT probe: `decision: go`, switches 0–4 ms, children 3 → 5/6 → 6 → 4 after cleanup.
- Release soak: 1200 fake conversations, RSS 34,180 → 35,280 KiB (bounded), threads 2 → 2.

## Accessibility gaps (static green, interactive open)

- Static gates pass on the B18 tip: `check-accessibility.sh` (focus-visible styling, live regions, no `tabindex`/`outline:none`/`user-select:none`/native-`disabled`); `svelte-check` 0 errors, 0 warnings.
- Open for B19-S01/S08 verification (all `UNKNOWN`, no GUI automation in B18's scope): full keyboard-only pass; screen-reader announcement quality (swap side note, swipe parity, transition notes); focus order against visual order after Swap (B18-U07); measured contrast ratios (never measured, both themes); pointer target sizes against the 40/36 px proposal; 320/640/1024/1440/1920 px widths; 200 %/400 % zoom; reduced-motion path behavior; high-contrast and large-text review.
- Sanitized current screenshots: **not captured** — no screenshot tooling exists in this task's environment (`grim`/`gnome-screenshot` absent) and capturing the Tauri window headlessly is out of scope; deferred to B19-S01 with maintainer assistance. No placeholder or mock screenshot is substituted.

## Approved adjustments

None. No maintainer adjustment to the plan's TARGETS is recorded; B19-S01 proposes adjustments, if any, as its own tracked decision.
