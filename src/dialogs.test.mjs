import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  DIALOG_CHAIN_CLOSED,
  PALETTE_COMMON_IDS,
  PALETTE_ENTRY_LIMIT,
  SETTINGS_READONLY_NOTE,
  appCommands,
  closeDialog,
  groups,
  matchesEntry,
  moveActive,
  openDialog,
  paletteEntries,
  paletteKeyIntent,
  preferredEntries,
  restoreFocus,
  sessionEntries,
  settingsTabs,
  uniqueEntries
} from "./dialogs.ts";

// B20-U6. The chain, the matcher, the orders, the active-row arithmetic, the
// key intent, and the settings rows are imported and exercised directly; the
// no-fabrication rules (every command real, every backend-less row read-only,
// no permission or question dock) are asserted against the component sources.
//
// Deferred to the versioned release gate (ADR 0014): case IDs B20-U6-T01…T06
// live on issue #135.

const here = dirname(fileURLToPath(import.meta.url));
const read = (relative) => readFileSync(join(here, relative), "utf8");
const dialogSource = read(join("lib", "Dialog.svelte"));
const paletteSource = read(join("lib", "CommandPalette.svelte"));
const settingsSource = read(join("lib", "SettingsDialog.svelte"));
const errorSource = read(join("lib", "ErrorScreen.svelte"));
const homeSource = read(join("lib", "HomeView.svelte"));
const modelSource = read("dialogs.ts");
const appSource = read("App.svelte");
const tabsSource = read(join("lib", "SessionTabs.svelte"));

const entry = (id, over = {}) => ({
  id,
  title: id,
  category: "Commands",
  ...over
});

const fakeTarget = () => {
  let calls = 0;
  return {
    focus: () => {
      calls += 1;
    },
    get calls() {
      return calls;
    }
  };
};

// ---------------------------------------------------------------------------
// Dialog chain (B20-U6-T01: focus returns to the invoker)
// ---------------------------------------------------------------------------

test("opening records the invoker; closing restores exactly it, once", () => {
  const invoker = fakeTarget();
  const opened = openDialog(DIALOG_CHAIN_CLOSED, "palette", invoker);
  assert.equal(opened.open, "palette");
  assert.equal(opened.rootInvoker, invoker);

  const closed = closeDialog(opened);
  assert.equal(closed.chain.open, null);
  assert.equal(closed.restore, invoker);
  assert.equal(restoreFocus(closed.restore), true);
  assert.equal(invoker.calls, 1);

  // A second close finds nothing to restore: focus is not stolen twice.
  const again = closeDialog(closed.chain);
  assert.equal(again.restore, null);
  assert.equal(restoreFocus(again.restore), false);
  assert.equal(invoker.calls, 1);
});

test("a handoff keeps the root invoker, so the last close reaches the first control", () => {
  const invoker = fakeTarget();
  let chain = openDialog(DIALOG_CHAIN_CLOSED, "palette", invoker);
  chain = openDialog(chain, "settings", null);
  assert.equal(chain.open, "settings");
  assert.equal(chain.rootInvoker, invoker);

  chain = openDialog(chain, "home", fakeTarget());
  assert.equal(chain.rootInvoker, invoker);

  const closed = closeDialog(chain);
  assert.equal(closed.restore, invoker);
  assert.equal(restoreFocus(closed.restore), true);
  assert.equal(invoker.calls, 1);
});

test("an opening with no invoker still closes cleanly", () => {
  const chain = openDialog(DIALOG_CHAIN_CLOSED, "error", null);
  const closed = closeDialog(chain);
  assert.equal(closed.restore, null);
  assert.equal(restoreFocus(closed.restore), false);
});

// ---------------------------------------------------------------------------
// Command palette (B20-U6-T02: only commands that exist)
// ---------------------------------------------------------------------------

test("the matcher is the pin's lowercase substring test over title, description, category", () => {
  const item = entry("theme.toggle", {
    title: "Toggle theme",
    description: "Switch between dark and light",
    category: "Commands"
  });
  assert.equal(matchesEntry(item, "toggle"), true);
  assert.equal(matchesEntry(item, "DARK"), true);
  assert.equal(matchesEntry(item, "commands"), true);
  assert.equal(matchesEntry(item, ""), true);
  assert.equal(matchesEntry(item, "missing"), false);
});

test("first id wins in the dedupe, and groups keep insertion order", () => {
  const items = [
    entry("a"),
    entry("b", { category: "Sessions" }),
    entry("a", { title: "duplicate" }),
    entry("c")
  ];
  assert.deepEqual(
    uniqueEntries(items).map((item) => item.id),
    ["a", "b", "c"]
  );
  assert.deepEqual(
    groups(uniqueEntries(items)).map((group) => [group.category, group.entries.length]),
    [
      ["Commands", 2],
      ["Sessions", 1]
    ]
  );
});

test("preferred entries follow common order, or fall back to the first limit", () => {
  const all = ["zeta", "session.next", "alpha", "session.new", "omega", "session.previous"].map(
    (id) => entry(id)
  );
  assert.deepEqual(
    preferredEntries(all).map((item) => item.id),
    ["session.new", "session.previous", "session.next"]
  );
  // No common id exists: the first PALETTE_ENTRY_LIMIT in list order.
  const uncommon = ["one", "two", "three", "four", "five", "six", "seven"].map((id) => entry(id));
  assert.deepEqual(
    preferredEntries(uncommon).map((item) => item.id),
    ["one", "two", "three", "four", "five"].slice(0, PALETTE_ENTRY_LIMIT)
  );
  assert.deepEqual(preferredEntries([]), []);
});

test("the active row advances with modulo wrap at both ends", () => {
  assert.equal(moveActive(3, 2, 1), 0);
  assert.equal(moveActive(3, 0, -1), 2);
  assert.equal(moveActive(3, 1, 1), 2);
  assert.equal(moveActive(0, 0, 1), 0);
});

test("the key intent is the pin's four keys and nothing else", () => {
  assert.deepEqual(paletteKeyIntent({ key: "ArrowDown" }), { kind: "move", delta: 1 });
  assert.deepEqual(paletteKeyIntent({ key: "ArrowUp" }), { kind: "move", delta: -1 });
  assert.deepEqual(paletteKeyIntent({ key: "Enter" }), { kind: "select" });
  assert.deepEqual(paletteKeyIntent({ key: "Escape" }), { kind: "close" });
  // Tab and friends are left alone, and there is no modified-key guard.
  assert.deepEqual(paletteKeyIntent({ key: "Tab" }), { kind: "none" });
  assert.deepEqual(paletteKeyIntent({ key: "a" }), { kind: "none" });
  assert.deepEqual(paletteKeyIntent({ key: "Enter", ctrlKey: true }), { kind: "select" });
});

test("every palette command id is a handler App.svelte really wires", () => {
  for (const running of [true, false]) {
    for (const command of appCommands(running)) {
      assert.ok(
        appSource.includes(`"${command.id}"`),
        `command ${command.id} has no handler in App.svelte`
      );
    }
  }
  // Host commands are the honest pair: exactly one exists at a time.
  assert.deepEqual(
    appCommands(true).map((item) => item.id).filter((id) => id.startsWith("host.")),
    ["host.stop"]
  );
  assert.deepEqual(
    appCommands(false).map((item) => item.id).filter((id) => id.startsWith("host.")),
    ["host.start"]
  );
});

test("displayed keybinds are ones this app really has", () => {
  const withKeybind = appCommands(true).filter((item) => item.keybind).map((item) => item.keybind);
  assert.deepEqual(withKeybind, ["Ctrl+Shift+Tab", "Ctrl+Tab"]);
  // The traversal the labels promise is wired in SessionTabs; the palette
  // chord is wired in App.
  assert.ok(tabsSource.includes('event.ctrlKey || event.key !== "Tab"'));
  assert.ok(appSource.includes('key === "k"'));
});

test("session entries are real tabs, capped at the entry limit", () => {
  const tabs = Array.from({ length: 8 }, (_, index) => ({
    id: index + 1,
    title: `Session ${index + 1}`,
    turnCount: index
  }));
  const entries = sessionEntries(tabs);
  assert.equal(entries.length, PALETTE_ENTRY_LIMIT);
  assert.deepEqual(
    entries.map((item) => item.id),
    ["session:1", "session:2", "session:3", "session:4", "session:5"]
  );
  assert.equal(entries[0].description, "0 turns");
  assert.equal(entries[1].description, "1 turn");
});

test("an empty query shows preferred commands plus sessions; a query filters both", () => {
  const commands = appCommands(false);
  const sessions = sessionEntries([
    { id: 1, title: "Theme work", turnCount: 2 },
    { id: 2, title: "Release", turnCount: 1 }
  ]);

  const initial = paletteEntries(commands, sessions, "");
  assert.deepEqual(
    initial.map((item) => item.id),
    ["session.new", "session.previous", "session.next", "home.open", "settings.open", "session:1", "session:2"]
  );

  const filtered = paletteEntries(commands, sessions, "theme");
  assert.deepEqual(
    filtered.map((item) => item.id),
    ["theme.toggle", "session:1"]
  );
  assert.deepEqual(paletteEntries(commands, sessions, "nothing-here"), []);
});

// ---------------------------------------------------------------------------
// Settings (B20-U6-T04: rendered without a backend, writes nothing)
// ---------------------------------------------------------------------------

const DATA = {
  theme: "dark",
  credentialStatus: "configured",
  credentialMessage: "The key is in the system credential store.",
  hostRunning: true,
  hostPort: 4096,
  hostVersion: "0.0.16",
  modelCount: 460,
  selectedModel: "GLM-4.6",
  modelsStale: true
};

test("settings have the pin's five tabs in two sections", () => {
  const tabs = settingsTabs(DATA);
  assert.deepEqual(
    tabs.map((tab) => [tab.section, tab.id]),
    [
      ["Desktop", "general"],
      ["Desktop", "shortcuts"],
      ["Server", "servers"],
      ["Server", "providers"],
      ["Server", "models"]
    ]
  );
});

test("theme is the only row with a backend; every other row is read-only and says so", () => {
  const rows = settingsTabs(DATA).flatMap((tab) => tab.rows);
  const writable = rows.filter((row) => !row.readonly);
  assert.deepEqual(writable.map((row) => row.id), ["theme"]);
  for (const row of rows.filter((item) => item.readonly)) {
    // The credential row reads a real backend (the system credential store), so
    // "no settings backend" would be false there; its honest marker is the
    // status-only phrasing. Every other read-only row carries the standard note.
    const marker = row.id === "credential" ? /Status only/ : new RegExp(SETTINGS_READONLY_NOTE);
    assert.ok(marker.test(row.description), `read-only row ${row.id} fails to say it saves nothing`);
  }
});

test("settings values come from data that exists; nothing is invented", () => {
  const rows = settingsTabs(DATA).flatMap((tab) => tab.rows);
  const valueOf = (id) => rows.find((row) => row.id === id).value;
  assert.equal(valueOf("host-status"), "Running");
  assert.equal(valueOf("host-address"), "localhost:4096");
  assert.equal(valueOf("host-version"), "0.0.16");
  assert.equal(valueOf("credential"), "Configured");
  assert.equal(valueOf("models"), "GLM-4.6 · 460 available");

  // Unknown data degrades to honest placeholders, never to a guessed value.
  const cold = settingsTabs({
    ...DATA,
    hostRunning: false,
    hostPort: null,
    hostVersion: null,
    credentialStatus: "unavailable",
    modelCount: 0,
    selectedModel: null,
    modelsStale: false
  }).flatMap((tab) => tab.rows);
  const coldValueOf = (id) => cold.find((row) => row.id === id).value;
  assert.equal(coldValueOf("host-status"), "Stopped");
  assert.equal(coldValueOf("host-address"), "Not started");
  assert.equal(coldValueOf("host-version"), "Unknown");
  assert.equal(coldValueOf("credential"), "Unavailable");
  assert.equal(coldValueOf("models"), "No models are available");
  assert.ok(
    cold.find((row) => row.id === "models").description.includes("The catalog is stale.") === false
  );
});

// ---------------------------------------------------------------------------
// Source freezes: the rules a DOM-less unit test can still reach
// ---------------------------------------------------------------------------

test("the primitive is the platform dialog: containment without focus-index overrides", () => {
  assert.ok(dialogSource.includes("showModal()"));
  assert.ok(dialogSource.includes("oncancel"));
  // The gate forbids them repo-wide; the primitive must not smuggle any back.
  assert.equal(/tabindex/i.test(dialogSource + paletteSource + settingsSource + homeSource), false);
});

test("the palette keeps the search field as the focus owner (listbox pattern)", () => {
  assert.ok(paletteSource.includes('role="combobox"'));
  assert.ok(paletteSource.includes('role="listbox"'));
  assert.ok(paletteSource.includes("aria-activedescendant"));
  assert.ok(paletteSource.includes('role="option"'));
  // The pin's 36 px rows.
  assert.ok(paletteSource.includes("height: 36px"));
});

test("the error screen keeps technical detail behind a disclosure", () => {
  assert.ok(errorSource.includes("<details"));
  assert.ok(errorSource.includes("Technical details"));
  assert.ok(errorSource.includes("{technical}"));
});

test("no permission or question dock exists in these surfaces (T05)", () => {
  const surfaces = dialogSource + paletteSource + settingsSource + errorSource + homeSource + modelSource;
  assert.equal(/permission|question|grant|deny/i.test(surfaces), false);
});

test("no dialog leaves a listener, timer, or observer behind (T06)", () => {
  const surfaces = dialogSource + paletteSource + settingsSource + errorSource + homeSource + modelSource;
  assert.equal(/addEventListener|setInterval|setTimeout|Observer/.test(surfaces), false);
  // Closing is the host unmounting on this callback; the palette's own frame
  // scroll is one-shot and not retained.
  assert.ok(paletteSource.includes("onclose()"));
});

test("planned home entries use the existing MVP-1 badge, not a fake control", () => {
  assert.ok(homeSource.includes('subtitle="Browse and edit the workspace"'));
  // ActionCard is reused (extended, not forked) and carries the badge itself.
  const cardSource = read(join("lib", "ActionCard.svelte"));
  assert.ok(cardSource.includes('badge = "MVP-1"'));
  assert.ok(homeSource.includes("<ActionCard"));
});
