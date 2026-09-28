import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  MIRROR_MAX_TEXT_CHARS,
  MIRROR_MAX_TURNS,
  PANEL_IDS,
  TERMINAL_MIRROR_LABEL,
  TERMINAL_MIRROR_NOTE,
  boundMirrorTurns,
  closeAll,
  closePanel,
  emptyRegistry,
  fileTabAfterDoubleClick,
  foldMirrorEvent,
  hotPanel,
  nextTabListScrollLeft,
  openPanel,
  openPanels,
  shouldCloseFileTab,
  tabStripWheelDelta
} from "./panels.ts";

// B20-U7. The registry's one-HOT single-slot model, the mirror ledger's fold
// and bound, and the file-tab arithmetic are imported and exercised directly;
// the lifecycle, honesty, and focus rules the release gate observes live on as
// source freezes against the components the registry drives.
//
// Deferred to the versioned release gate (ADR 0014): case IDs B20-U7-T01…T06
// live on issue #136 — T01 the 20-cycle open/close soak (MEASURED), T02 one
// HOT heavy view whose switch destroys the previous, T03 no survivor after
// close, T04 the mirror labelled a mirror and never spawning, T05 honest empty
// states with no fabricated path, T06 keyboard and focus parity.

const here = dirname(fileURLToPath(import.meta.url));
const read = (relative) => readFileSync(join(here, relative), "utf8");
const panelsSource = read("panels.ts");
const canvasSource = read(join("lib", "CanvasPanel.svelte"));
const sideSource = read(join("lib", "SidePanel.svelte"));
const filesSource = read(join("lib", "FileTabs.svelte"));
const reviewSource = read(join("lib", "ReviewPanel.svelte"));
const terminalSource = read(join("lib", "TerminalMirror.svelte"));
const componentSources = [
  canvasSource,
  sideSource,
  filesSource,
  reviewSource,
  terminalSource
];

// Comments are stripped before token checks: prose may name a forbidden thing
// to say it does not exist ("this is not a pty").
const code = (source) =>
  source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "");

// Built by concatenation so this file itself carries no focus-override token.
const FOCUS_OVERRIDE_TOKENS = [
  "tab" + "index",
  "disabl" + "ed=",
  "outline: n" + "one",
  "user-select: " + "n" + "one"
];

// ---------------------------------------------------------------------------
// Registry: one HOT heavy view, one open panel, deterministic release
// ---------------------------------------------------------------------------

test("the registry prices the five presets: two heavy views, three light", () => {
  assert.deepEqual([...PANEL_IDS], [
    "preview",
    "browser",
    "files",
    "review",
    "terminal"
  ]);
  const registry = emptyRegistry();
  for (const id of PANEL_IDS) {
    assert.equal(registry[id].id, id);
    assert.equal(registry[id].phase, "closed");
  }
  assert.equal(registry.preview.kind, "heavy");
  assert.equal(registry.browser.kind, "heavy");
  assert.equal(registry.files.kind, "light");
  assert.equal(registry.review.kind, "light");
  assert.equal(registry.terminal.kind, "light");
});

test("every open releases the previous panel first; the board never holds two", () => {
  const order = [
    "preview",
    "files",
    "browser",
    "review",
    "terminal",
    "files",
    "browser"
  ];
  let registry = emptyRegistry();
  let opened = null;
  for (const id of order) {
    const result = openPanel(registry, id);
    assert.deepEqual(result.release, opened === null ? [] : [opened]);
    registry = result.registry;
    opened = id;
    assert.deepEqual(openPanels(registry), [id]);
    const heavy = registry[id].kind === "heavy";
    assert.equal(registry[id].phase, heavy ? "hot" : "cold");
    assert.equal(hotPanel(registry), heavy ? id : null);
  }
});

test("a second HOT heavy view is unreachable by construction (T02)", () => {
  const preview = openPanel(emptyRegistry(), "preview").registry;
  assert.equal(hotPanel(preview), "preview");

  const switched = openPanel(preview, "browser");
  assert.deepEqual(switched.release, ["preview"]);
  assert.equal(hotPanel(switched.registry), "browser");
  assert.equal(switched.registry.preview.phase, "closed");

  // A light panel opens cold and empties the HOT slot entirely.
  const light = openPanel(preview, "review");
  assert.deepEqual(light.release, ["preview"]);
  assert.equal(hotPanel(light.registry), null);
  assert.equal(light.registry.review.phase, "cold");

  // Re-opening what is open releases nothing.
  const again = openPanel(preview, "preview");
  assert.deepEqual(again.release, []);
  assert.equal(hotPanel(again.registry), "preview");
});

test("closing is exact: closePanel closes one, closeAll empties the board", () => {
  const open = openPanel(emptyRegistry(), "terminal").registry;
  assert.deepEqual(openPanels(open), ["terminal"]);

  const closed = closePanel(open, "terminal");
  assert.deepEqual(openPanels(closed), []);
  assert.equal(hotPanel(closed), null);

  for (const exited of [closeAll(open), closeAll(emptyRegistry())]) {
    for (const id of PANEL_IDS) {
      assert.equal(exited[id].phase, "closed");
    }
    assert.deepEqual(openPanels(exited), []);
    assert.equal(hotPanel(exited), null);
  }
});

// ---------------------------------------------------------------------------
// Mirror ledger: fold and bound, no process anywhere
// ---------------------------------------------------------------------------

test("the mirror folds text only into the running turn of its own session", () => {
  let turns = foldMirrorEvent([], "a", { type: "started" });
  turns = foldMirrorEvent(turns, "a", { type: "text_chunk", text: "hello " });
  const ignored = foldMirrorEvent(turns, "b", {
    type: "text_chunk",
    text: "not mine"
  });
  assert.equal(ignored, turns);
  turns = foldMirrorEvent(turns, "a", { type: "text_chunk", text: "world" });
  assert.equal(turns.length, 1);
  assert.equal(turns[0].session, "a");
  assert.equal(turns[0].text, "hello world");

  turns = foldMirrorEvent(turns, "a", { type: "completed" });
  assert.equal(turns[0].state, "completed");
  const late = foldMirrorEvent(turns, "a", {
    type: "text_chunk",
    text: "late"
  });
  assert.equal(late, turns);
});

test("interleaved sessions keep independent turns in one ledger", () => {
  let turns = foldMirrorEvent([], "a", { type: "started" });
  turns = foldMirrorEvent(turns, "b", { type: "started" });
  turns = foldMirrorEvent(turns, "a", { type: "text_chunk", text: "A" });
  turns = foldMirrorEvent(turns, "b", { type: "text_chunk", text: "B" });
  turns = foldMirrorEvent(turns, "a", { type: "text_chunk", text: "!" });
  assert.equal(turns.length, 2);
  assert.equal(turns[0].text, "A!");
  assert.equal(turns[1].text, "B");

  turns = foldMirrorEvent(turns, "a", { type: "cancelled" });
  assert.equal(turns[0].state, "cancelled");
  assert.equal(turns[1].state, "running");
});

test("a failed turn keeps its text and records the error", () => {
  let turns = foldMirrorEvent([], "a", { type: "started" });
  turns = foldMirrorEvent(turns, "a", { type: "text_chunk", text: "partial" });
  turns = foldMirrorEvent(turns, "a", {
    type: "failed",
    error: { message: "the turn broke" }
  });
  assert.equal(turns[0].state, "failed");
  assert.equal(turns[0].text, "partial");
  assert.equal(turns[0].error, "the turn broke");
});

test("malformed and unknown events change nothing", () => {
  const turns = foldMirrorEvent([], "a", { type: "started" });
  assert.equal(foldMirrorEvent(turns, "a", { type: "who_knows" }), turns);
  assert.equal(foldMirrorEvent(turns, "a", { type: "text_chunk" }), turns);
  assert.equal(
    foldMirrorEvent(turns, "a", { type: "completed" }) === turns,
    false
  );
  const failed = foldMirrorEvent(turns, "a", { type: "failed" });
  assert.equal(failed[0].state, "failed");
  assert.equal("error" in failed[0], false);
  // A text chunk with no running turn of that session is dropped whole.
  const closedTurns = foldMirrorEvent(turns, "a", { type: "completed" });
  assert.equal(
    foldMirrorEvent(closedTurns, "a", { type: "text_chunk", text: "x" }),
    closedTurns
  );
});

test("the ledger is bounded: oldest turns go first, one huge turn keeps its tail", () => {
  const turns = Array.from({ length: MIRROR_MAX_TURNS + 5 }, (_, index) => ({
    session: "s",
    text: `t${index}`,
    state: "completed"
  }));
  const kept = boundMirrorTurns(turns);
  assert.equal(kept.length, MIRROR_MAX_TURNS);
  assert.equal(kept[0].text, `t5`);
  assert.equal(kept[kept.length - 1].text, `t${turns.length - 1}`);

  const huge = [
    {
      session: "s",
      text: `HEAD${"x".repeat(MIRROR_MAX_TEXT_CHARS)}TAIL`,
      state: "running"
    }
  ];
  const bounded = boundMirrorTurns(huge);
  assert.equal(bounded.length, 1);
  assert.equal(bounded[0].text.length, MIRROR_MAX_TEXT_CHARS);
  assert.ok(bounded[0].text.endsWith("TAIL"));
  assert.ok(!bounded[0].text.startsWith("HEAD"));

  // The fold applies the same bound, so streaming cannot outgrow the ledger.
  let streamed = foldMirrorEvent([], "s", { type: "started" });
  streamed = foldMirrorEvent(streamed, "s", {
    type: "text_chunk",
    text: `HEAD${"y".repeat(MIRROR_MAX_TEXT_CHARS)}TAIL`
  });
  assert.equal(streamed[0].text.length, MIRROR_MAX_TEXT_CHARS);
  assert.ok(streamed[0].text.endsWith("TAIL"));
});

test("the mirror's identity states what it is, everywhere it names itself", () => {
  assert.match(TERMINAL_MIRROR_LABEL, /mirror/i);
  assert.match(TERMINAL_MIRROR_NOTE, /mirror/i);
  assert.match(TERMINAL_MIRROR_NOTE, /no shell/i);
  const untouched = boundMirrorTurns([]);
  assert.deepEqual(untouched, []);
});

// ---------------------------------------------------------------------------
// File tabs: the pin's scroll, wheel, close, and pin arithmetic
// ---------------------------------------------------------------------------

test("the strip chases only growth, or jumps to the opener", () => {
  const base = {
    prevScrollWidth: 100,
    scrollWidth: 150,
    clientWidth: 80,
    prevContextOpen: false,
    contextOpen: false
  };
  assert.equal(nextTabListScrollLeft(base), 70);
  assert.equal(
    nextTabListScrollLeft({ ...base, scrollWidth: 100 }),
    undefined
  );
  assert.equal(
    nextTabListScrollLeft({ ...base, clientWidth: 200 }),
    undefined
  );
  assert.equal(
    nextTabListScrollLeft({ ...base, prevContextOpen: true, contextOpen: true }),
    70
  );
  assert.equal(nextTabListScrollLeft({ ...base, contextOpen: true }), 0);
});

test("vertical wheel scrolls the strip horizontally by 50, never diagonally", () => {
  assert.equal(tabStripWheelDelta({ deltaY: 120, deltaX: 0 }), 50);
  assert.equal(tabStripWheelDelta({ deltaY: -3, deltaX: 0 }), -50);
  assert.equal(tabStripWheelDelta({ deltaY: 0, deltaX: 30 }), null);
  assert.equal(tabStripWheelDelta({ deltaY: 30, deltaX: 40 }), null);
  assert.equal(tabStripWheelDelta({ deltaY: 0, deltaX: 0 }), null);
});

test("a tab closes on middle-click or its close button, and only then", () => {
  assert.equal(
    shouldCloseFileTab({ button: 1, targetIsCloseButton: false }),
    true
  );
  assert.equal(
    shouldCloseFileTab({ button: 0, targetIsCloseButton: true }),
    true
  );
  assert.equal(
    shouldCloseFileTab({ button: 0, targetIsCloseButton: false }),
    false
  );
});

test("double-click pins the temporary tab and touches nothing else", () => {
  const tabs = [
    { id: "a", name: "a.ts", temporary: true },
    { id: "b", name: "b.ts", temporary: false }
  ];
  const pinned = fileTabAfterDoubleClick(tabs, "a");
  assert.equal(pinned[0].temporary, false);
  assert.equal(pinned[1], tabs[1]);
  assert.deepEqual(fileTabAfterDoubleClick(tabs, "nope"), tabs);
});

// ---------------------------------------------------------------------------
// Source freezes for the deferred release-gate cases (T02–T06)
// ---------------------------------------------------------------------------

test("the Canvas body is one registry-driven slot, never two parked views (T02)", () => {
  assert.ok(
    canvasSource.includes(
      "const shown = $derived(openPanels(registry)[0] ?? null);"
    )
  );
  assert.ok(canvasSource.includes("registry = openPanel(registry, tab).registry;"));
  // Exactly one body slot: one opening branch and four alternatives.
  assert.equal(canvasSource.split("{#if shown === ").length - 1, 1);
  assert.equal(canvasSource.split("{:else if shown === ").length - 1, 4);
  for (const id of PANEL_IDS) {
    assert.ok(canvasSource.includes(`shown === "${id}"`));
    assert.ok(canvasSource.includes(`{ id: "${id}", label:`));
  }
  // The planned presets stay planned badges, not fake controls.
  assert.equal(canvasSource.split("id: null").length - 1, 3);
  for (const label of ["Components", "Logs", "AI Notes"]) {
    assert.ok(canvasSource.includes(`label: "${label}"`));
  }
  // The B18-S05 swipe thresholds are untouched.
  for (const line of [
    "SWIPE_MIN_DISTANCE_PX = 48",
    "SWIPE_FLICK_DISTANCE_PX = 24",
    "SWIPE_FLICK_MAX_MS = 300",
    "SWIPE_HORIZONTAL_RATIO = 2"
  ]) {
    assert.ok(canvasSource.includes(line), line);
  }
});

test("no panel leaves a timer, interval, observer, or listener behind (T03)", () => {
  for (const source of componentSources) {
    for (const token of ["setInterval", "setTimeout", "MutationObserver", "new Worker"]) {
      assert.ok(!code(source).includes(token), token);
    }
  }
  // The wheel listener is removed by the same effect that owns it.
  assert.ok(filesSource.includes('addEventListener("wheel", onWheel'));
  assert.ok(filesSource.includes('removeEventListener("wheel", onWheel)'));
  // Both stream listeners are unlistened on both paths (immediate and raced).
  assert.ok(terminalSource.includes("disposed"));
  assert.ok(terminalSource.split("unlistens.splice(0)").length - 1 >= 2);
});

test("the terminal panel is labelled a mirror and can never spawn (T04)", () => {
  const panelsCode = code(panelsSource);
  const terminalCode = code(terminalSource);
  assert.ok(terminalSource.includes("aria-label={TERMINAL_MIRROR_LABEL}"));
  assert.ok(terminalSource.includes("{TERMINAL_MIRROR_LABEL}"));
  assert.ok(terminalSource.includes("{TERMINAL_MIRROR_NOTE}"));
  for (const source of [panelsCode, terminalCode]) {
    for (const token of [
      "invoke(",
      "spawn",
      /\bpty\b/i,
      "child_process",
      "@tauri-apps/api/shell",
      "Command.create"
    ]) {
      const hit =
        typeof token === "string" ? source.includes(token) : token.test(source);
      assert.ok(!hit, String(token));
    }
  }
});

test("the empty states are honest: no fabricated path, filename, or control (T05)", () => {
  assert.ok(filesSource.includes("{#if tabs.length === 0}"));
  assert.ok(filesSource.includes('"No files open"'));
  assert.ok(reviewSource.includes("diffs.length === 0"));
  assert.ok(reviewSource.includes('"Nothing to review yet"'));
  // The diff-style selector only renders when there is something to restyle.
  assert.ok(reviewSource.includes("diffs.length > 0"));
  assert.ok(terminalSource.includes('"No turn output yet"'));
  for (const source of componentSources) {
    assert.ok(!/\/home\/|\/Users\//.test(source));
  }
});

test("interactive controls are real buttons with visible focus (T06)", () => {
  for (const source of componentSources) {
    for (const token of FOCUS_OVERRIDE_TOKENS) {
      assert.ok(!source.includes(token), token);
    }
  }
  assert.ok(filesSource.includes('type="button"'));
  assert.ok(filesSource.includes("Close ${tab.name}"));
  assert.ok(filesSource.includes(":focus-visible"));
  assert.ok(canvasSource.includes('variant="tab"'));
});
