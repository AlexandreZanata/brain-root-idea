# B06 — Workspace shell in the product visual language

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B06-Workspace-Shell). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Released
- Objective: The BrainRoot shell adopts the professional workspace look requested by the maintainer — dark chrome, left section rail, agent panel with welcome and suggestion actions, bottom composer with the configured-model chip, and a dominant Canvas with tabs — while every future surface is disabled and labeled MVP-1, so the look matches the product vision without claiming capabilities that do not exist.
- Branch: `batch/b06-workspace-shell` (deleted after merge)
- Draft/final PR: [#53](https://github.com/AlexandreZanata/brain-root-idea/pull/53)
- Merge commit: pending (filled by the post-merge finalization commit)
- Baseline commit: `ce945d673ff537f50a7b352152115a1cdc2deee2`
- Target/resulting version: `0.0.2` (annotated tag on the merge commit; release URL filled by the post-merge finalization commit)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned, Node.js v26.3.1, pnpm 11.13.0)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Browser, terminal, files, editor, preview, project opening, or agent selection.
- Fake UI: no non-functional control is presented as live.
- Dependencies, fonts, icon libraries, or image assets.
- Any capability, IPC, provider, contract, version, or dependency change.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B06-S01 | [#52](https://github.com/AlexandreZanata/brain-root-idea/issues/52) | Workspace shell rebuilt in the reference visual language: chrome with product identity/status, left section rail (Build active; Agents/Browser/Files/Terminal/Settings disabled MVP-1), agent panel with welcome card and functional suggestion chips, composer with model chip/status/send/cancel, Canvas panel with Preview active and Components/Logs/AI Notes disabled; all conversation behavior preserved | `f913518` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/52) | Future surfaces are honest disabled placeholders; the visual result awaited the maintainer's review; bundle grew to CSS 9.01 kB / JS 56.32 kB (gzip 2.07/20.29) | Closed |
| B06-S02 | [#54](https://github.com/AlexandreZanata/brain-root-idea/issues/54) | Monochrome design system: token themes (pure-black dark with white/near-black grays and no blue, white light), square controls, and 14 reusable components with `App.svelte` as composition; theme toggle persisted in `localStorage` with pre-paint boot; accessibility gate extended to the whole frontend | `7f884b9` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/54#issuecomment-5794216262) | The dark `--text-subtle` failed contrast (4.43–3.67:1) and was raised to `#8a8a8a` (5.04–6.08:1); visual review by the maintainer pending | Closed |
| B06-S03 | [#55](https://github.com/AlexandreZanata/brain-root-idea/issues/55) | Responsive small-viewports: horizontal icon-only rail below 1080/640px, wrapped header and composer, scrollable Canvas tabs, agent 20rem / Canvas 24rem minimums in stacked mode; orphaned `.rail`/`.agent`/`.canvas` selectors removed | `2ea240f` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/55#issuecomment-5794408775) | No status, prompt, send/cancel, or error is hidden at any width; the rendered result is the maintainer's resized-window confirmation | Closed |
| B06-S04 | [#56](https://github.com/AlexandreZanata/brain-root-idea/issues/56) | Reference palettes: dark follows Codex (near-black monochrome, zero blue — verified by grep); light follows the reference (white surfaces, dark text, blue primary/links/tint); tabs use link-style underlines; welcome spark and status dot follow the per-theme accent; every text pair ≥ 4.5:1 and UI/focus ≥ 3:1 | `0cd7fd9` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/56#issuecomment-5794633780) | The light `--text-subtle` was set to `#5b6b80` up front after pre-measuring `#64748b` at 4.20/4.47 | Closed |
| B06-S05 | [#57](https://github.com/AlexandreZanata/brain-root-idea/issues/57) | Resizable chat area + reusable TextArea: `clampPanelWidth` helper, new `TextArea` (label/rows/placeholder/`focus()`) and `PanelResizer` (native range 280–560px, step 8, vertical orientation, hidden when stacked); agent width in-memory only, stacked layout unchanged | `93a5840` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/57#issuecomment-5794885033) | Static a11y gate needs the literal `<label for="prompt">`; the rendered bound label comes from `TextArea` (`for={id}` → `for="prompt"`), documented with a one-line comment, gate unmodified | Closed |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B06-S01): the reference image's workspace language is applied with honest scope — only Build and Preview are live; every other surface is `aria-disabled` and titled "planned for MVP-1", and the model chip shows the real configured default instead of a fake picker.
- Decision (B06-S02): one token source (`src/lib/theme.css`) drives both palettes and all controls; components are thin wrappers over shared `br-*` classes and never hardcode colors; theme choice is UI-local state in `localStorage` (nonsecret, not project state); the accessibility gate scans the whole frontend.
- Decision (B06-S03): breakpoints `1080px` (stack, horizontal rail) and `640px` (icon-only rail, wrapped chrome/composer, scrollable tabs); the responsive contract lives in `theme.css` and component blocks, and zoom keeps working through the same queries.
- Decision (B06-S04): dark is Codex monochrome by maintainer order (no blue token, icon, focus, brand, status, or tab anywhere in dark); light keeps the reference blue for primary text, tab/link text, the welcome tint, and focus; tab styling is an underline token (`--accent`), not a filled pill.
- Decision (B06-S05): the resizer is a native range input (keyboard-operable without `tabindex`); the agent width is clamped to 280–560px and kept in memory only; the handle is hidden when the workspace stacks; `TextArea` owns the bound prompt label so the composer behavior stays identical.
- Decision (B06-S01): the accessibility invariants are unchanged (one polite live region, bound prompt label, `aria-disabled` instead of `disabled`, focus-visible, reduced motion, no `user-select`/`outline`/`tabindex`); the accessibility gate stays green without modification.

## Failures and recovery

- 2026-09-23, B06-S01: the accessibility gate initially failed on the visually hidden prompt label ordering (`<label class="…" for="prompt">` does not match the gate's bound-label pattern); the attributes were reordered to `<label for="prompt" class="…">` instead of weakening the gate.
- 2026-09-23, B06-S02: the first both-theme contrast pass failed on the dark `--text-subtle` (`#737373`, 4.43–3.67:1 over black/panel/raised/hover); raised the token to `#8a8a8a` (5.04–6.08:1) instead of loosening the check.
- 2026-09-23, B06-S05: the static accessibility gate requires the literal `<label for="prompt">` pattern while the reusable `TextArea` renders `for={id}` → `for="prompt"` at runtime; the mapping is documented with a one-line comment and the gate stayed unmodified instead of being weakened.

## Final gates

- Full CI: `check-full-linux` on the final head (run URL filled by the post-merge finalization commit)
- Review: single-maintainer administrator bypass documented; merged at B06-S06
- Security/privacy: `check-security` gate green; no secret or provider body in markup or evidence; no telemetry or unapproved destination
- Performance: bundle CSS 11.43 kB (gzip 2.75), JS 63.17 kB (gzip 23.42); no runtime resource added
- Cleanup: process smoke green on the rebuilt binary; no listener or child process left behind
- Artifact/checksum: none for this batch
- Known limitations: future surfaces are disabled placeholders; the rendered window still has no UI automation; the panel width is in-memory only

## Result and next batch

Batch B06 is released as `v0.0.2` (the tag, merge commit, CI run, and release URL are filled by the post-merge finalization commit): the shell now follows the requested product visual language on one monochrome token system, with responsive and resizable layout, while every future surface stays disabled and labeled MVP-1. This final record is completed in one documented post-merge commit on `main` through the administrator bypass because the merge commit, CI runs, tag, and release URL cannot exist before the merge. Next: the post-`0.0.2` batches (Windows minimum environment, then the MVP-1 Linux product loop), each with its own reference environment and records. Rollback: revert the batch commits before the tag; after tagging, the tag is never moved and a failed candidate gets a new prerelease identifier.
