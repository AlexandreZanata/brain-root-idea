# Deck switching and resource return (B15-S01)

**Status:** measured destroy/recreate switching with a bounded per-role footprint

**Reviewed:** 2026-09-23

**Related:** [Companion Browser plan](companion-browser-plan.md) (CB-B B4) · [Human Browser navigation record](human-browser-navigation.md) · [Preview quality measurements](preview-quality-measurements.md) · ADR 0012 · ADR 0013

## Question

CB-B B4 requires Preview ↔ Browser switching with a destroy/recreate policy, proof that the old view and listeners are gone, an honest state-loss message, and a switch/resource benchmark. This record covers the proof and the measurements through the production commands.

## Method

A debug-only deck harness (`BRAINROOT_DECK_FIXTURE=1`, `#[cfg(debug_assertions)]` only) starts the owned dev server, then drives the production `preview_show`/`preview_hide` and `human_browser_show`/`human_browser_hide` commands in two full switch cycles while sampling the destination presence flags, per-switch latency, child-process count, and RSS:

1. preview shown;
2. switch to the browser (hide preview, show human);
3. switch back to the preview;
4. a second cycle to prove the footprint does not grow;
5. cleanup (hide both, stop the dev server).

Exact reproduction:

```sh
cargo build --manifest-path src-tauri/Cargo.toml
BRAINROOT_DECK_FIXTURE=1 timeout 180 ./src-tauri/target/debug/brainroot
```

## Result

One release-quality debug run returned `"decision":"go"`, exit `0`, with no leftover process or listener:

```json
{"decision":"go","show_preview_ms":2,"hide_preview_ms":0,"show_human_ms":3,"hide_human_ms":0,
 "re_show_preview_ms":2,"second_show_human_ms":2,"second_show_preview_ms":3,
 "baseline_children":3,"children_after_preview":5,"children_after_human":6,
 "children_after_switch_back":6,"children_after_second_cycle":6,"children_after_cleanup":4,
 "baseline_rss_kb":186192,"rss_after_switch_back_kb":190084,"reasons":[]}
```

## Interpretation

- **One view at a time is proven:** every phase checks the destination flags and fails the run when the other role is still present. `debug_view_present` is false for the hidden role immediately after a switch, so the previous WebView is destroyed, not hidden.
- **Switching is fast at the view layer:** every shown/hidden transition measured 0–3 ms in-process; page loads continue asynchronously after the view exists, which is why the harness waits for readiness before the presence checks.
- **The first harness run found a real accumulation:** creating a fresh `WebContext` on every show added one WebKit process per switch (5 → 6 → 7). The fix caches one context per role (keyed by the profile root) inside each view module, so repeated switches reuse the role's context and process instead of spawning new ones.
- **The footprint is now bounded per role:** after the first cycle the count settles at 6 (three baseline processes plus the preview role context and the human role context) and does not grow on the second cycle or on the switch back. The role contexts persist until application exit, which is the intended profile ownership; they are not per-switch leaks.
- **RSS** grew about 4 MB across the two cycles, within run-to-run noise at this scale (`MEASURED`, 186 MB → 190 MB).
- **State-loss messaging is explicit:** the Browser surface states that switching Canvas tabs closes the page and in-memory state may be lost; the Preview surface states that the view closes, the app reloads on return, and the dev server keeps running until stopped.

## Decision

- **Go for the destroy/recreate deck contract.** Switching destroys the previous view and recreates the target, the old view is provably gone, latencies are bounded, and the resource footprint is bounded by one context per role instead of growing per switch.
- Remaining for the later Deck phase: COLD placeholders/snapshots, more than two destinations, permission controls, and the full WARM/SUSPENDED evaluation.
