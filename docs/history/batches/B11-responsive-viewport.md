# B11 — Responsive viewport presets (CB-A A5)

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B11-Responsive-Viewport). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Ready (awaiting maintainer approval to merge)
- Objective: The preview can be viewed at Desktop (1280 × 800), Tablet (834 × 1112), Phone (390 × 844), and Custom sizes — re-bounding the existing view in place, showing the actual size honestly when the Canvas area limits a preset, and labelling the feature as a responsive viewport preview rather than device emulation.
- Branch: `batch/b11-responsive-viewport`
- Draft/final PR: [#78](https://github.com/AlexandreZanata/brain-root-idea/pull/78)
- Baseline commit: `ee68390b1011f8fc332850e880d771c21b9a9ff3`
- Target/resulting version: `0.0.7` (preview slice; no artifact)
- Started/completed: 2026-09-23 / 2026-09-23
- Supported test environment: frozen B01 reference environment (Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 pinned, Node.js v26.3.1, pnpm 11.13.0)
- Repository history mirror: this file (created at batch close)

## Non-goals

- Backend, supervisor, view-hosting, or origin-policy changes.
- Device emulation (user agent, touch, DPR), remote browsing, Human Browser, import, Deck, phone, Windows, macOS.
- Persistence of the chosen preset.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B11-S01 | [#77](https://github.com/AlexandreZanata/brain-root-idea/issues/77) | Responsive viewport presets: the pure model (`VIEWPORT_PRESETS`, `viewportSize`, `customViewportSize`, `presetLabel`) with desktop/tablet/phone/custom sizes clamped to the available Canvas area, and the Preview tab controls that re-bound the existing view in place, show the actual size when limited, and carry the non-emulation label; `docs/05` records the behavior | `20640a4` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/77#issuecomment-5799230689) | Frontend-only; the geometry path from B10 is unchanged and no view is recreated | Closed |
| B11-S02 | [#79](https://github.com/AlexandreZanata/brain-root-idea/issues/79) | Finalization: version `0.0.7` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`; changelog `0.0.7` section; this history record created; PR #78 marked Ready and the full Linux gate run on the latest head; merge/tag deferred to maintainer approval | pending | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/79) | The authenticated repository owner cannot approve its own PR; no tag is created before the merge commit exists on `main` | In review |

Update one row whenever a microstep closes or is reopened. Never paste secrets or unbounded raw logs.

## Decisions and changed assumptions

- Decision (B11-S01): presets are Desktop 1280 × 800, Tablet 834 × 1112, Phone 390 × 844, plus Custom clamped to 240–1920 × 240–1200; when the Canvas area is smaller than the nominal size, the preview uses the available area and the UI states the actual size, so the label never overpromises; the feature is a responsive viewport preview, not device emulation.
- Decision (B11-S01): preset changes flow through the existing slot geometry sync (`ResizeObserver` + `preview_set_bounds`), so the owned server and the loaded page are never restarted for a viewport change.
- Known limitation (B11-S01): visual confirmation of the preset behavior needs the maintainer's rendered check; focus/keyboard routing, zoom/scale, DPR, and soak remain unmeasured; the preset is not persisted.

## Failures and recovery

- None that changed scope. The first `svelte-check` pass flagged the stage element as possibly null inside the `ResizeObserver` callback; the element is captured after the null check and no gate was weakened.

## Final gates

- Full CI: `check-full-linux` on the finalization head — run URL recorded in the [issue evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/79) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B11-Responsive-Viewport)
- Review: requested; awaiting maintainer approval (documented single-maintainer administrator path)
- Security/privacy: frontend-only change; `check-security.sh` green with the dependency list unchanged; no destination, permission, or backend change
- Performance: no new resource; preset changes re-bound the existing view; the harness still proves the view and owned server stop with no leftovers
- Cleanup: the debug harness ran with the presets in place and left no process, listener, or fixture server
- Artifact/checksum: none for this batch
- Known limitations: visual confirmation of the preset behavior needs the rendered check; focus/keyboard routing, zoom/scale, DPR, and soak remain unmeasured; the preset is not persisted

## Result and next batch

B11 completes the CB-A A5 slice: the preview can be viewed at desktop, tablet, phone, and custom sizes, the existing view is re-bounded in place, and the UI never disguises a limited size as a nominal one. Next: the roadmap's next capability step in its own batch, keeping the unmeasured acceptance checks (focus/keyboard routing, zoom/scale, DPR, soak) explicit. Rollback: revert the batch commits; the preview keeps the B10 geometry sync and remains fully usable.
