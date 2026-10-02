/**
 * B20-U4 — the composer's pure model: triggers, filtering, selection, keys.
 *
 * Ported from the pinned OpenCode v2 reducer
 * (`packages/session-ui/src/v2/components/prompt-input/machine.ts` @ 34aa427),
 * with the trigger regexes copied verbatim so the palette opens and closes
 * exactly where the original does. It lives in a plain `.ts` module so
 * `src/composer.test.mjs` can freeze the behavior without rendering Svelte.
 */

/**
 * What a command actually does. Every variant maps to a call the composer can
 * already make; there is deliberately no `noop`, because an entry that does
 * nothing would be a fake control.
 */
export type ComposerAction = "agent.cycle" | "agent.plan" | "agent.build" | "model.focus";

export type ComposerCommand = {
  id: string;
  trigger: string;
  title: string;
  description: string;
  action: ComposerAction;
};

/**
 * The shipped catalog. The pin's own list (`share`, `unshare`, `new`, `undo`,
 * `redo`, `compact`, `fork`, `export`, `open`, `terminal`, `mcp`) is not ported
 * because none of those actions exists in BrainRoot yet; `agent` and `model`
 * are the two the pin defines at composer level, and `plan`/`build` are the
 * explicit form of the same agent switch that the pin only exposes as a cycle.
 */
export const COMPOSER_COMMANDS: readonly ComposerCommand[] = [
  {
    id: "agent",
    trigger: "agent",
    title: "Switch agent",
    description: "Cycle between Plan and Build",
    action: "agent.cycle"
  },
  {
    id: "plan",
    trigger: "plan",
    title: "Plan mode",
    description: "Explore without changing files",
    action: "agent.plan"
  },
  {
    id: "build",
    trigger: "build",
    title: "Build mode",
    description: "Let the agent change files",
    action: "agent.build"
  },
  {
    id: "model",
    trigger: "model",
    title: "Choose model",
    description: "Move focus to the model control",
    action: "model.focus"
  }
];

export type ComposerSuggestion = { id: string; label: string; description: string };

/**
 * The `@` picker's list. BrainRoot has no workspace, file, or MCP listing, so
 * this is empty on purpose: the popover shell, keyboard model, and empty state
 * are real, and no path is ever fabricated to fill the space.
 */
export const CONTEXT_ITEMS: readonly ComposerSuggestion[] = [];

export const CONTEXT_EMPTY_LABEL =
  "No workspace is open yet — file and directory mentions arrive with project listing.";

export const COMMAND_EMPTY_LABEL = "No commands match.";

export type ComposerTrigger = { kind: "command" | "context"; query: string };

/** The pin's context trigger, verbatim: the last `@` at a start or space. */
const CONTEXT_TRIGGER = /(?:^|\s)@([^\s@]*)$/;
/** The pin's command trigger, verbatim: the whole draft is one slash token. */
const COMMAND_TRIGGER = /^\/(\S*)$/;

export function detectTrigger(value: string, cursor: number): ComposerTrigger | null {
  const limit = Math.max(0, Math.min(cursor, value.length));
  const before = value.slice(0, limit);
  const context = before.match(CONTEXT_TRIGGER);
  if (context) {
    return { kind: "context", query: context[1] ?? "" };
  }
  const command = value.match(COMMAND_TRIGGER);
  return command ? { kind: "command", query: command[1] ?? "" } : null;
}

/**
 * Deliberately narrow: a command matches when its trigger or its title *starts*
 * with the query. Substring matching over descriptions made `/mo` return
 * everything whose blurb said "mode", which is noise, not a filter.
 */
export function filterCommands(query: string, commands = COMPOSER_COMMANDS): ComposerCommand[] {
  const trimmed = query.trim();
  if (trimmed.length === 0) {
    return [...commands];
  }
  return commands.filter(
    (command) =>
      command.trigger.toLowerCase().startsWith(trimmed.toLowerCase()) ||
      command.title.toLowerCase().startsWith(trimmed.toLowerCase())
  );
}

export function commandSuggestions(
  query: string,
  commands = COMPOSER_COMMANDS
): ComposerSuggestion[] {
  return filterCommands(query, commands).map((command) => ({
    id: command.id,
    label: `/${command.trigger}`,
    description: command.description
  }));
}

export function commandById(id: string, commands = COMPOSER_COMMANDS): ComposerCommand | null {
  return commands.find((command) => command.id === id) ?? null;
}

/**
 * `menu` marks the variant opened from the `+` menu: it keeps its own query in a
 * search field and does not close when the draft stops looking like a trigger.
 */
export type ComposerPopoverOpen =
  | { kind: "command"; menu: boolean; query: string; activeId: string | null }
  | { kind: "context"; query: string; activeId: string | null };

export type ComposerPopover = { kind: "closed" } | ComposerPopoverOpen;

export const POPOVER_CLOSED: ComposerPopover = { kind: "closed" };

export function popoverOpen(popover: ComposerPopover): popover is ComposerPopoverOpen {
  return popover.kind !== "closed";
}

export function popoverMenu(popover: ComposerPopover): boolean {
  return popover.kind === "command" && popover.menu;
}

export function popoverSuggestions(popover: ComposerPopover): ComposerSuggestion[] {
  if (popover.kind === "command") {
    return commandSuggestions(popover.query);
  }
  if (popover.kind === "context") {
    return [...CONTEXT_ITEMS];
  }
  return [];
}

export function popoverEmptyLabel(popover: ComposerPopover): string {
  return popover.kind === "context" ? CONTEXT_EMPTY_LABEL : COMMAND_EMPTY_LABEL;
}

/** The pin's result rule: keep the active row if it survived the filter. */
export function activeIdFor(ids: string[], previous: string | null): string | null {
  if (previous && ids.includes(previous)) {
    return previous;
  }
  return ids[0] ?? null;
}

/** The pin's wrap rule: nothing active starts at the first going down, the last going up. */
export function moveActive(
  ids: string[],
  activeId: string | null,
  direction: 1 | -1
): string | null {
  if (ids.length === 0) {
    return null;
  }
  const current = activeId ? ids.indexOf(activeId) : -1;
  const index =
    current < 0
      ? direction === 1
        ? 0
        : ids.length - 1
      : (current + direction + ids.length) % ids.length;
  return ids[index] ?? null;
}

/** The pin's rewrite: the first `/` for a command, the last `@` for a mention. */
export function replaceTrigger(value: string, trigger: "@" | "/", replacement: string): string {
  const index = trigger === "/" ? value.indexOf(trigger) : value.lastIndexOf(trigger);
  return index < 0 ? replacement : value.slice(0, index) + replacement;
}

export function applyCommand(value: string, menu: boolean, label: string): string {
  if (menu) {
    const trimmed = value.trim();
    return trimmed.length > 0 ? `${label} ${trimmed}` : `${label} `;
  }
  return replaceTrigger(value, "/", `${label} `);
}

/**
 * The popover the draft implies after an edit. A menu keeps its own query; every
 * other state closes as soon as the draft stops matching its trigger — which is
 * why `look at /mod` never opens the palette.
 */
export function popoverForValue(
  current: ComposerPopover,
  value: string,
  cursor: number
): ComposerPopover {
  if (popoverMenu(current)) {
    return current;
  }
  const trigger = detectTrigger(value, cursor);
  if (!trigger) {
    return POPOVER_CLOSED;
  }
  const ids = trigger.kind === "command" ? commandSuggestions(trigger.query).map((item) => item.id) : [];
  return { kind: trigger.kind, menu: false, query: trigger.query, activeId: ids[0] ?? null };
}

export function commandMenuPopover(query: string): ComposerPopover {
  const ids = commandSuggestions(query).map((item) => item.id);
  return { kind: "command", menu: true, query, activeId: ids[0] ?? null };
}

/**
 * The `+` menu's *Commands* item. On an empty draft the pin appends the trigger
 * and opens the inline palette; otherwise it opens the same list as a menu with
 * its own search field, because there is no trigger left to type into.
 */
export function openCommandMenu(value: string): { popover: ComposerPopover; draft: string } {
  if (value.trim().length > 0) {
    return { popover: commandMenuPopover(""), draft: value };
  }
  const popover = popoverForValue(POPOVER_CLOSED, "/", 1);
  return { popover, draft: `${value}/` };
}

export function openContext(value: string): { popover: ComposerPopover; draft: string } {
  return { popover: { kind: "context", query: "", activeId: null }, draft: `${value}@` };
}

export type ComposerKey = {
  key: string;
  ctrlKey?: boolean;
  metaKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
  isComposing?: boolean;
};

export type ComposerKeyOutcome = {
  handled: boolean;
  popover: ComposerPopover;
  /** A row was chosen: the caller runs the command and rewrites the draft. */
  selectedId?: string;
  /** The popover closed: the caller returns focus to the editor. */
  refocusEditor?: boolean;
};

/**
 * The pin's `keyDown`, ported branch by branch. Anything it does not consume
 * falls through to the composer's own Enter/Escape handling, which is how
 * "Enter submits and Escape cancels" keeps working while the popover is shut.
 */
export function composerKeyDown(
  popover: ComposerPopover,
  ids: string[],
  event: ComposerKey
): ComposerKeyOutcome {
  if (event.ctrlKey && !event.metaKey && !event.altKey && event.key.toLowerCase() === "g") {
    if (!popoverOpen(popover)) {
      return { handled: false, popover };
    }
    return { handled: true, popover: POPOVER_CLOSED, refocusEditor: true };
  }
  if (!popoverOpen(popover)) {
    return { handled: false, popover };
  }
  if (event.key === "Escape") {
    return { handled: true, popover: POPOVER_CLOSED, refocusEditor: true };
  }
  const enter = event.key === "Enter" && !event.isComposing;
  if (event.key === "Tab" || enter) {
    const activeId = popover.activeId;
    if (!activeId) {
      // The pin consumes the key and does nothing, so Enter never submits while
      // a popover is open without an active row.
      return { handled: true, popover };
    }
    return { handled: true, popover: POPOVER_CLOSED, selectedId: activeId };
  }
  const direction: 1 | -1 | 0 =
    event.key === "ArrowDown" || (event.ctrlKey && event.key === "n")
      ? 1
      : event.key === "ArrowUp" || (event.ctrlKey && event.key === "p")
        ? -1
        : 0;
  if (direction === 0 || ids.length === 0) {
    return { handled: false, popover };
  }
  return { handled: true, popover: { ...popover, activeId: moveActive(ids, popover.activeId, direction) } };
}
