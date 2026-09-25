import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  COMPOSER_COMMANDS,
  CONTEXT_EMPTY_LABEL,
  CONTEXT_ITEMS,
  COMMAND_EMPTY_LABEL,
  POPOVER_CLOSED,
  activeIdFor,
  applyCommand,
  commandById,
  commandMenuPopover,
  commandSuggestions,
  composerKeyDown,
  detectTrigger,
  filterCommands,
  moveActive,
  openCommandMenu,
  openContext,
  popoverEmptyLabel,
  popoverForValue,
  popoverOpen,
  popoverSuggestions,
  replaceTrigger
} from "./composer.ts";

// B20-U4. The trigger regexes, the filter, the wrap arithmetic, and the key
// reducer are imported from `src/composer.ts` and exercised directly; the
// component's markup and the no-fabrication rules are asserted against the
// component source. The pinned geometry (60→180 px editor, 28 px submit) is
// read from the markup because it is layout, not behavior.
//
// Deferred to the versioned release gate (ADR 0014): case IDs B20-U4-T01…T05
// live on issue #133.

const here = dirname(fileURLToPath(import.meta.url));
const composerSource = readFileSync(join(here, "lib", "Composer.svelte"), "utf8");
const editorSource = readFileSync(join(here, "lib", "TextArea.svelte"), "utf8");

const key = (over) => ({
  key: "",
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  isComposing: false,
  ...over
});

test("the palette opens only when the whole draft is one slash token", () => {
  assert.deepEqual(detectTrigger("/", 1), { kind: "command", query: "" });
  assert.deepEqual(detectTrigger("/mod", 4), { kind: "command", query: "mod" });
  assert.equal(detectTrigger("look at /mod", 12), null);
  assert.equal(detectTrigger("hello", 5), null);
});

test("the context trigger is the last @ after a start or a space", () => {
  assert.deepEqual(detectTrigger("@", 1), { kind: "context", query: "" });
  assert.deepEqual(detectTrigger("look at @src", 12), { kind: "context", query: "src" });
  assert.deepEqual(detectTrigger("@a @b", 5), { kind: "context", query: "b" });
  assert.equal(detectTrigger("mail@host", 9), null);
  assert.equal(detectTrigger("@src ", 5), null);
  assert.equal(detectTrigger("@src", 0), null);
});

test("a command filter matches the trigger and the title prefix", () => {
  assert.equal(filterCommands("").length, COMPOSER_COMMANDS.length);
  assert.deepEqual(
    filterCommands("mo").map((command) => command.id),
    ["model"]
  );
  assert.deepEqual(
    filterCommands("PLAN").map((command) => command.id),
    ["plan"]
  );
  assert.deepEqual(filterCommands("nothing-matches").length, 0);
  assert.deepEqual(
    commandSuggestions("").map((item) => item.label),
    ["/agent", "/plan", "/build", "/model"]
  );
});

test("every catalog entry maps to a real composer action", () => {
  const actions = COMPOSER_COMMANDS.map((command) => command.action).sort();
  assert.deepEqual(actions, ["agent.build", "agent.cycle", "agent.plan", "model.focus"]);
  for (const command of COMPOSER_COMMANDS) {
    assert.equal(commandById(command.id)?.trigger, command.trigger);
    assert.ok(command.title.length > 0);
    assert.ok(command.description.length > 0);
  }
  assert.equal(commandById("not-a-command"), null);
});

test("the active row survives a filter and otherwise starts at the first result", () => {
  assert.equal(activeIdFor(["a", "b"], "b"), "b");
  assert.equal(activeIdFor(["a", "b"], "c"), "a");
  assert.equal(activeIdFor(["a", "b"], null), "a");
  assert.equal(activeIdFor([], null), null);
});

test("arrow navigation wraps at both ends and starts from the right edge", () => {
  assert.equal(moveActive(["a", "b", "c"], null, 1), "a");
  assert.equal(moveActive(["a", "b", "c"], null, -1), "c");
  assert.equal(moveActive(["a", "b", "c"], "c", 1), "a");
  assert.equal(moveActive(["a", "b", "c"], "a", -1), "c");
  assert.equal(moveActive(["a", "b", "c"], "b", 1), "c");
  assert.equal(moveActive(["only"], "only", 1), "only");
  assert.equal(moveActive([], "gone", 1), null);
});

test("the rewrite replaces the leading slash and the last @", () => {
  assert.equal(replaceTrigger("/mod", "/", "/model "), "/model ");
  assert.equal(replaceTrigger("@src", "@", "@/path/x "), "@/path/x ");
  assert.equal(replaceTrigger("look @a @b", "@", "@/b "), "look @a @/b ");
  assert.equal(replaceTrigger("plain", "/", "/agent "), "/agent ");
});

test("selecting a command rewrites the draft; the menu variant prepends", () => {
  assert.equal(applyCommand("/mo", false, "/model"), "/model ");
  assert.equal(applyCommand("", true, "/agent"), "/agent ");
  assert.equal(applyCommand("  rewrite the header  ", true, "/agent"), "/agent rewrite the header");
});

test("a closed popover never consumes a key", () => {
  for (const candidate of [
    key({ key: "Enter" }),
    key({ key: "Escape" }),
    key({ key: "ArrowDown" }),
    key({ key: "Tab" }),
    key({ key: "g", ctrlKey: true })
  ]) {
    const outcome = composerKeyDown(POPOVER_CLOSED, [], candidate);
    assert.equal(outcome.handled, false);
    assert.equal(outcome.popover.kind, "closed");
  }
});

test("Escape closes the popover and asks for the editor back, without cancelling", () => {
  const open = { kind: "command", menu: false, query: "mo", activeId: "model" };
  const outcome = composerKeyDown(open, ["model"], key({ key: "Escape" }));
  assert.equal(outcome.handled, true);
  assert.equal(outcome.popover.kind, "closed");
  assert.equal(outcome.refocusEditor, true);
  assert.equal(outcome.selectedId, undefined);
});

test("ctrl+g closes the popover exactly like Escape", () => {
  const open = { kind: "context", query: "", activeId: null };
  const outcome = composerKeyDown(open, [], key({ key: "G", ctrlKey: true }));
  assert.equal(outcome.handled, true);
  assert.equal(outcome.popover.kind, "closed");
  assert.equal(outcome.refocusEditor, true);
});

test("Enter with an active row selects it and never submits", () => {
  const open = { kind: "command", menu: false, query: "", activeId: "plan" };
  const outcome = composerKeyDown(open, ["agent", "plan"], key({ key: "Enter" }));
  assert.equal(outcome.handled, true);
  assert.equal(outcome.selectedId, "plan");
  assert.equal(outcome.popover.kind, "closed");
});

test("Enter without an active row is consumed but changes nothing", () => {
  const open = { kind: "context", query: "", activeId: null };
  const outcome = composerKeyDown(open, [], key({ key: "Enter" }));
  assert.equal(outcome.handled, true);
  assert.equal(outcome.selectedId, undefined);
  assert.equal(outcome.popover.kind, "context");
});

test("Enter while composing is left to the input method", () => {
  const open = { kind: "command", menu: false, query: "", activeId: "agent" };
  const outcome = composerKeyDown(open, ["agent"], key({ key: "Enter", isComposing: true }));
  assert.equal(outcome.handled, false);
});

test("Tab selects the active row, exactly like Enter", () => {
  const open = { kind: "command", menu: false, query: "", activeId: "build" };
  const outcome = composerKeyDown(open, ["build"], key({ key: "Tab" }));
  assert.equal(outcome.handled, true);
  assert.equal(outcome.selectedId, "build");
});

test("arrow keys and their readline twins move the active row", () => {
  const open = { kind: "command", menu: false, query: "", activeId: "agent" };
  const ids = ["agent", "plan", "build"];
  assert.equal(composerKeyDown(open, ids, key({ key: "ArrowDown" })).popover.activeId, "plan");
  assert.equal(composerKeyDown(open, ids, key({ key: "ArrowUp" })).popover.activeId, "build");
  assert.equal(
    composerKeyDown(open, ids, key({ key: "n", ctrlKey: true })).popover.activeId,
    "plan"
  );
  assert.equal(
    composerKeyDown(open, ids, key({ key: "p", ctrlKey: true })).popover.activeId,
    "build"
  );
  // No rows to move through: the key belongs to the editor.
  assert.equal(composerKeyDown(open, [], key({ key: "ArrowDown" })).handled, false);
});

test("any other key keeps the popover open and unhandled", () => {
  const open = { kind: "command", menu: false, query: "", activeId: "agent" };
  const outcome = composerKeyDown(open, ["agent"], key({ key: "a" }));
  assert.equal(outcome.handled, false);
  assert.equal(outcome.popover, open);
});

test("the draft decides the popover, and a menu keeps its own query", () => {
  assert.equal(popoverForValue(POPOVER_CLOSED, "hello", 5).kind, "closed");
  assert.equal(popoverForValue(POPOVER_CLOSED, "/", 1).kind, "command");
  assert.equal(popoverForValue(POPOVER_CLOSED, "@/x", 3).kind, "context");
  const menu = commandMenuPopover("plan");
  assert.equal(popoverForValue(menu, "hello", 5), menu);
  const inline = popoverForValue(POPOVER_CLOSED, "/mod", 4);
  assert.equal(inline.activeId, "model");
  assert.equal(popoverOpen(inline), true);
});

test("the + menu appends the trigger on a blank draft and searches otherwise", () => {
  const blank = openCommandMenu("");
  assert.equal(blank.draft, "/");
  assert.equal(blank.popover.kind, "command");
  assert.equal(blank.popover.menu, false);
  assert.equal(blank.popover.activeId, "agent");
  const typed = openCommandMenu("rewrite the header");
  assert.equal(typed.draft, "rewrite the header");
  assert.equal(typed.popover.menu, true);
  assert.equal(typed.popover.query, "");
});

test("the @ picker is a shell with an honest empty state, never a fabricated path", () => {
  const context = openContext("");
  assert.equal(context.draft, "@");
  assert.equal(context.popover.kind, "context");
  assert.deepEqual(CONTEXT_ITEMS, []);
  assert.deepEqual(popoverSuggestions(context.popover), []);
  assert.equal(popoverEmptyLabel(context.popover), CONTEXT_EMPTY_LABEL);
  assert.equal(CONTEXT_EMPTY_LABEL.includes("No workspace"), true);
  assert.equal(COMMAND_EMPTY_LABEL, "No commands match.");
  assert.equal(
    popoverEmptyLabel({ kind: "command", menu: false, query: "zz", activeId: null }),
    COMMAND_EMPTY_LABEL
  );
  assert.equal(popoverSuggestions(commandMenuPopover("zz")).length, 0);
});

test("the composer keeps the repository's accessibility rules", () => {
  // The gate's own pattern, lookbehind included: `aria-disabled` is the
  // allowed spelling and must not be flagged.
  for (const source of [composerSource, editorSource]) {
    assert.equal(/(?<!aria-)\bdisabled(?:=|\s|>)/.test(source), false);
    assert.equal(source.includes("user-select"), false);
    assert.equal(/outline\s*:\s*none/.test(source), false);
  }
  assert.equal(composerSource.includes(":focus-visible"), true);
  assert.equal(composerSource.includes("prefers-reduced-motion"), true);
  assert.equal(composerSource.includes('role="combobox"'), true);
  assert.equal(composerSource.includes('role="listbox"'), true);
  assert.equal(composerSource.includes('role="option"'), true);
  assert.equal(composerSource.includes('id="prompt"'), true);
  assert.equal(editorSource.includes("aria-activedescendant"), true);
  assert.equal(editorSource.includes("aria-multiline"), true);
  assert.equal(/label\s+for=\{id\}/.test(editorSource), true);
});

test("the composer keeps the pinned geometry and only the pinned controls", () => {
  // The pinned editor range and type scale.
  assert.equal(/min-height:\s*60px/.test(editorSource), true);
  assert.equal(/max-height:\s*180px/.test(editorSource), true);
  assert.equal(/font-size:\s*13px/.test(editorSource), true);
  assert.equal(/line-height:\s*20px/.test(editorSource), true);
  // The pinned submit button, its icon states, and the raised surface.
  assert.equal(composerSource.includes("--v2-elevation-raised"), true);
  assert.equal(composerSource.includes("--v2-elevation-button-contrast"), true);
  assert.equal(/name=\{stopping \? "stop" : "arrow-up"\}/.test(composerSource), true);
  // Capabilities BrainRoot does not have must not appear as controls.
  for (const absent of ["Attach", "Shell", "Variant", "attachment"]) {
    assert.equal(composerSource.includes(absent), false, `${absent} must not be built`);
  }
});
