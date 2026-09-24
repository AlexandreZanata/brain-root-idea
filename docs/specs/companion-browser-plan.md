# Companion Browser: integrated side browser and data portability

**Status:** adopted as the initial Companion Browser plan; not authorization to implement or a claim of measured performance

**Platform order:** Linux first; Windows after the Linux contract is proven; macOS after Windows

**Scope:** desktop BrainRoot, not a mobile operating-system app

**Reviewed:** 2026-09-23

**Recorded decision (2026-09-23, maintainer):** this document is the initial Companion Browser plan and will be used as such. The Companion Browser is created for Linux only at first; Windows and macOS are not part of the initial creation and require separate future batches with their own probes and evidence.

## Outcome and boundaries

A user can keep the conversation beside a large Companion Canvas, open the result of their project in a real localhost preview, resize or swap the two sides, and switch the Canvas between a desktop-width and phone-width view. Later, the same Canvas hosts a user-operated browser with a private profile and an explicit, reversible path for importing supported data from other browsers. This must feel like one product, not a miniature traditional IDE or a Chromium clone.

This plan extends the existing `05-companion-canvas.md`, `08-browser-architecture.md`, and ADR 0006. It **does not move** remote browsing, profile migration, the Deck, or swipe navigation into MVP-0 or the MVP-1 acceptance slice. It does not weaken Safe Mode or authorize access to installed browser profiles. The local preview comes first; full human browsing follows only when isolation and resource lifecycle are demonstrated.

Success means useful web work beside chat with no extra content WebView at idle, one HOT heavy content view by default, no hidden browser-agent data sharing, and honest measured resource evidence. “Extremely fast/light” is a goal, not a factual claim until measured on the Linux reference machine and separately on each later platform. The current MVP-0 process-tree memory target already fails; see `10-performance-budget.md` and the B05 soak report. A browser feature may not hide that baseline cost.

## Product interaction contract

1. **Conversation + Canvas:** default Focus layout is approximately 30/70; Watch, Chat, and Full Canvas presets remain available. One divider resizes the regions and its position persists per project. A Swap action puts Canvas on the other side without changing the selected page; keyboard, screen-reader, and reduced-motion alternatives are required.
2. **One visible content slot:** `Preview` displays the user's local app. `Browser` displays user-directed remote pages only in the later Human Browser phase. A compact destination switcher changes the slot; inactive heavy destinations become COLD placeholders rather than accumulating live WebViews. No permanent file tree, developer console, or tab strip is added to the default UI.
3. **Viewport presets:** desktop, tablet, common phone widths, and custom size. The page is centered in the available Canvas area; a phone frame is optional decoration and never steals layout space when narrow. Changing width should resize the existing content view where supported, not recreate it solely to change CSS breakpoints. Label this **responsive viewport preview**, not “running the actual iPhone/Android browser”: OS engine, touch, DPR, UA, fonts, and device APIs can differ. Real-device verification remains a later testing capability.
4. **Spatial navigation:** in the later Deck/Phone phase, horizontal drag/swipe on the *Canvas chrome* switches destination cards; vertical scroll stays in the page. Gestures beginning inside a page must not steal its own scrolling, selection, drag-and-drop, or touch interactions. Buttons and keyboard commands provide the same operation. “Swap sides” and “switch card” are distinct actions.
5. **Visible lifecycle:** show loading, ready, stale snapshot, crashed, and retry states. If the browser is COLD, tell the user a page may reload and in-memory state may be lost. Preserve only safe reconstruction metadata; never claim arbitrary forms, media, or authenticated sessions can be restored perfectly.
6. **Optional code inspection:** `Show changes` may reveal Code View on demand. The default adjacent surface is the running result, not source code. A future split of two *content* views requires a separate measured decision because it conflicts with the one-HOT default.

## Architecture contract

```text
Svelte shell (chat, layout, intent; no privileged browsing data)
  └─ typed commands/events → Rust core
       ├─ Canvas/Browser Manager: role, owner, lifecycle, geometry, navigation
       ├─ Import service: explicit user-selected source → validate → preview → commit/undo
       └─ Resource Governor: one HOT content view, idle cleanup, measured ownership
            ├─ Preview WebView: approved localhost origins, no shell IPC
            ├─ Human WebView: remote pages, separate persistent profile, no shell IPC
            └─ Agent Browser: separate on-demand automation profile/process
```

The privileged shell, local preview, Human Browser, and Agent Browser stay four distinct trust roles. The Human Browser does not become the agent's browser; a user-mediated share must identify the exact page/data allowed, and cookies/authenticated storage are never silently shared. A profile path belongs to BrainRoot and is not an installed Chrome/Firefox profile. Browser controls in Svelte send validated intent to Rust; Rust owns URL policy, WebView creation/destruction, data directories, and permissions. Do not embed arbitrary pages as an iframe in the privileged shell.

Linux uses the available WebKitGTK runtime through Tauri/WRY; Windows uses WebView2; macOS uses WKWebView. Reusing OS engines avoids shipping a second Chromium binary, but it does not imply identical site compatibility, process memory, profile formats, media support, or focus behavior. A platform probe must determine whether child WebViews can be positioned, resized, isolated, destroyed, and recreated reliably on the exact supported Linux display stacks. If not, stop and review ADR 0006 rather than quietly introducing bundled Chromium or another always-on process.

The navigation allowlist is role-specific: Preview only approved project localhost origins; Human Browser normal `http`/`https` after user direction; `file:`, `javascript:`, `data:`, application-internal schemes, external protocol handlers, popups, downloads, camera/microphone, and cross-origin credential handoff need explicit policies and tests. The browser never receives privileged Tauri IPC. Threat-model prompt injection from pages before any content can reach an agent.

## Data import contract

“Import data from Chrome/Firefox/other popular browsers” is a capability matrix, not a promise that every browser profile can be copied. Import is **off by default**, starts from a user-picked export file or a separately approved source, reads only the selected category, and never runs at launch. Show source, categories, item counts, duplicates, unsupported items, destination, storage impact, and rollback before commit. Keep original browsers untouched. Reject malformed, oversized, symlink-escaping, and script-bearing input. Store migration diagnostics without URLs or secrets by default.

| Category | First supported path | Later investigation / boundary |
|---|---|---|
| Bookmarks and folders | User-exported Netscape-style HTML from Chrome, Firefox, Edge, etc.; sanitize URLs and titles; deterministic merge/duplicate preview; export back to HTML | Direct profile readers only after OS/browser/version matrix, lock-safe snapshot strategy, explicit consent, and regression fixtures |
| Homepage, search engine, appearance | Offer a small portable mapping as explicit choices, not raw settings-file copying | Browser-specific preference adapters only when meaning and supported destination exist |
| History and open tabs | Not in first import slice | Opt-in import only with retention/delete policy, date/source filtering, source-format evidence, and privacy tests |
| Passwords, passkeys, payment/autofill, cookies, login sessions | **Excluded from initial import** | Separate security ADR and threat model; never copy live browser databases or use plaintext CSV as an implicit default; OS secure-store and recovery design required |
| Extensions and browser-specific settings | Not imported | No compatibility promise across Chromium, Gecko, and WebKit engines; evaluate individual features only when user value is proven |

Browser-native export HTML is a portability baseline documented by [Chrome Help](https://support.google.com/chrome/answer/96816?hl=en-u) and [Firefox Help](https://support.mozilla.org/en-US/kb/export-firefox-bookmarks-to-backup-or-transfer-bookmarks). It covers bookmarks, **not** general preferences, history, credentials, extensions, or sessions. Firefox documents that browser-to-browser migration capabilities vary, including password limitations by OS; therefore do not claim broad parity from one successful fixture ([Mozilla Support](https://support.mozilla.org/en-US/kb/switching-chrome-firefox)). WebView2 itself has app-specific user-data folders and profiles rather than inheriting Edge's browser identity ([Microsoft Learn](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/user-data-folder)).

Import transaction: select source → parse in a bounded, unprivileged-as-feasible path → normalize to versioned BrainRoot data types → show dry-run preview → checkpoint existing BrainRoot data → commit atomically → offer Undo and export. Idempotent re-import must not duplicate entries silently. A corrupt or canceled import leaves the destination unchanged; delete staging files. Test with synthetic fixtures only in CI, never real personal profiles.

## Delivery sequence for economical agents

These are **future batch candidates**, not executable issues. Assign the actual `Bnn` IDs, branch, PR, version, and exact allowlisted files only after the current batch has merged and the relevant architecture spike is accepted. Each candidate becomes a separate GitHub issue with the complete template from `18-mvp-execution-plan.md`; one focused commit and local micro-gate per issue. Keep each batch at ten or fewer issues; full CI and human/high-capability review run on the final batch head only. Record issue evidence and decisions in repository history and Wiki before merge.

The CB-A/CB-B candidates were implemented through B08–B16 under the earlier cadence. For any remaining candidate implemented from B17 onward, ADR 0014 replaces per-issue automated test execution with fast non-test checks; write fixture/tests in the microstep, execute them at the versioned release gate, and record `IMPLEMENTED_UNVERIFIED` until then.

### CB-A — adjacent localhost preview (MVP-1 Phase 4)

**Prerequisites:** owned dev-server/process lifecycle, approved localhost origin policy, Linux WebView probe. **One batch may be split further if scope exceeds the issue limit.**

1. **A1, geometry contract:** type the Canvas slot/preset/side/divider state; unit-test bounds, persistence, and migration from missing/invalid state. No WebView yet.
2. **A2, accessible layout:** render chat beside empty Canvas with divider, Swap, presets, keyboard and focus behavior; visual regression across narrow/large windows and reduced motion. Do not add remote navigation.
3. **A3, Linux WebView spike:** a disposable prototype measures child-view create/destroy, resize, focus escape, X11/Wayland behavior, localhost navigation, crash recovery, profile isolation, and 100 open/close cycles. Record raw measurements, environment, leaks, and a go/no-go decision; no production promise before this issue closes.
4. **A4, isolated Preview vertical slice:** one on-demand child WebView with no privileged IPC; allow only owned localhost origins; render ready/failure/retry; prove forbidden navigation and clean close. Fake dev server fixtures in CI.
5. **A5, responsive presets and adjacent result:** resize a live preview for desktop/phone widths, preserve chat, verify actual viewport dimensions with a fixture, test keyboard/scroll/focus, and document the difference from device emulation.
6. **A6, batch quality:** E2E critical journey, security/cleanup checks, release-profile before/after process-tree metrics, docs/history/Wiki, latest-head CI and review. Do not merge a regression without a documented exception.

### CB-B — Human Browser and Deck (post-MVP Phase 8)

**Prerequisites:** CB-A gate, isolated profile feasibility, explicit Human Browser policy. Each issue must state whether it changes the trust boundary and needs ADR review.

1. **B1, role and policy contract:** typed browser role, navigation decisions, profile owner, active/inactive state, and negative policy tests.
2. **B2, profile isolation probe:** verify persistent Human Browser storage is separate from Preview, shell, installed browsers, and Agent Browser across close/reopen and crash. Measure disk/memory and fail closed where isolation cannot be demonstrated.
3. **B3, minimal human navigation:** address entry, back/forward/reload, page title/URL, external-open, clear error state; one remote HOT view at a time. No hidden credential/agent bridge.
4. **B4, Deck switching:** Preview ↔ Browser uses snapshot/placeholder and destroy/recreate policy; prove old view and listeners are gone and state-loss message is honest. Benchmark switching and resource return.
5. **B5, permission controls:** downloads, popups, camera/mic, clipboard, unsafe schemes, site data deletion, profile reset, and site compatibility/failure UX; include denial tests.
6. **B6, batch quality:** synthetic remote fixture, threat-model review, accessibility, soak, platform evidence, history/Wiki, latest-head CI/review.

### CB-C — portable import (post-MVP, after CB-B)

1. **C1, schema and threat model:** normalized bookmark/folder representation, source provenance, URL policy, limits, rollback transaction, and fixture corpus; document excluded private categories.
2. **C2, HTML parser with no UI:** Chrome/Firefox/Edge export fixtures, nested folders, encodings, bad/malicious input, duplicate/idempotence tests, and bounded CPU/memory. Dependency review before any parser library.
3. **C3, dry-run and consent UI:** source picker, counts, warnings, explicit confirm/cancel and accessible status; no import on mere file selection.
4. **C4, atomic commit/Undo/export:** preserve originals, detect repeats, recover interrupted import, export supported bookmarks; test backup/restore and staging cleanup.
5. **C5, batch quality:** privacy/security audit, large synthetic fixture measurement, history/Wiki, latest-head CI/review. History/preferences require new separately scoped batches and evidence.

### CB-D — spatial phone presentation (post-MVP Phase 9)

Add non-intercepting swipe/drag on Canvas chrome, card controls, keyboard parity, phone/square/landscape/desktop/full presets, screen-reader announcements, and reduced-motion transitions. Test conflicting page gestures and verify no second HOT view or idle event loop. This is a desktop presentation mode; Android/iOS apps and actual mobile-engine parity remain out of scope.

### Platform expansion

After each Linux batch is stable, run a **separate Windows WebView2** and then **macOS WKWebView** contract/probe batch before promising feature parity. Keep import file format portable, but re-verify profile directories, permissions, media/download behavior, focus, responsive metrics, process-tree accounting, and cleanup on each OS. Never extrapolate Linux memory numbers.

## Rigid gates and stop rules

- Every implementation issue must include starting commit, permitted files, exact commands and expected assertions, negative tests, ownership/cleanup, rollback, and stop conditions. The candidate list above is not sufficient for an economical agent to begin coding.
- From B17, local micro-gates use only fast non-test scope, diff, secret, and applicable format/schema/docs checks. Write deterministic synthetic fixtures and tests with no live account, but defer their execution to the versioned release gate. No unexecuted isolation, viewport, cancellation, or cleanup test may be reported as passing. One issue/commit with `Refs #N`; manually close the issue as `IMPLEMENTED_UNVERIFIED` only after PR/Wiki evidence agrees.
- Final batch gate: `check-full-linux` on latest head, review, no open regression, documented WebView/process-tree measurements, permission denial tests, 100-cycle leak probe for lifecycle changes, keyboard/focus and responsive E2E, docs/ADR/history/Wiki sync. Browser-compatibility findings are reported rather than concealed.
- Stop immediately if the proposed approach needs privileged IPC in remote content, direct reads of a live browser profile, credential/cookie import, an extra resident browser engine/process at idle, a new dependency without review, or a second HOT view without a measured ADR. Escalate to a maintainer and revise the issue/ADR before code.
- At batch close, versions follow `19-release-and-versioning.md`; no preassigned version for these proposed batches and no version bump per microstep.

## Decisions still required before implementation

1. Which Linux distributions/display stacks and WebKitGTK versions constitute the browser support floor? The child-view spike must include the actual reference environment.
2. Should the first Human Browser support arbitrary websites or only a curated destination set? This affects navigation policy and QA scope.
3. What are the exact retained categories and deletion policy for user browsing history, if it is ever imported?
4. What explicit user-share flow lets the agent inspect a Human Browser page, and what information is excluded?
5. Does the measured one-HOT lifecycle meet perceived switching speed, or is a platform-specific WARM mode worth its memory/security cost?
