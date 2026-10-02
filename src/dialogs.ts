/**
 * B20-U6 — the dialog system's pure model: the registry and its root-invoker
 * focus chain, the command palette's entries, matcher, grouping, preferred
 * ordering, active-row arithmetic, and key intent, and the settings rows built
 * from data that already exists.
 *
 * The palette pieces are ported from the pin (`components/command-palette.ts`
 * and `components/dialog-command-palette-v2.tsx` @ 34aa427): `matchesEntry`,
 * `uniqueCommandPaletteEntries`, `groups`, `preferredCommandEntries`'s ordering
 * with its `ENTRY_LIMIT` fallback, and the modulo `move`. They live in a plain
 * `.ts` module so `src/dialogs.test.mjs` can freeze them without a DOM.
 *
 * Two rules hold everywhere in this file. Nothing claims a capability that does
 * not exist: every entry maps to a handler `App.svelte` really wires, and every
 * settings row without a backend is marked read-only instead of pretending to
 * save. And nothing here touches the DOM — focus restoration takes a
 * `FocusTarget` so the chain is testable with a fake.
 */

/** The four surfaces U6 adds. One is open at a time. */
export type DialogKind = "palette" | "settings" | "error" | "home";

/** The smallest thing focus can be returned to; a real element satisfies it. */
export type FocusTarget = { focus: () => void };

/**
 * The registry is one slot plus the invoker that started the chain. A handoff
 * (palette → settings) keeps the original invoker, so the *last* close in the
 * chain returns focus to the control the user pressed first.
 */
export type DialogChain = {
  open: DialogKind | null;
  rootInvoker: FocusTarget | null;
};

export const DIALOG_CHAIN_CLOSED: DialogChain = { open: null, rootInvoker: null };

export function openDialog(
  chain: DialogChain,
  kind: DialogKind,
  invoker: FocusTarget | null
): DialogChain {
  return {
    open: kind,
    rootInvoker: chain.open === null ? invoker : chain.rootInvoker
  };
}

/** Returns the chain to install and the target to restore, if any. */
export function closeDialog(chain: DialogChain): {
  chain: DialogChain;
  restore: FocusTarget | null;
} {
  return { chain: DIALOG_CHAIN_CLOSED, restore: chain.rootInvoker };
}

export function restoreFocus(target: FocusTarget | null): boolean {
  if (!target) {
    return false;
  }
  target.focus();
  return true;
}

// ---------------------------------------------------------------------------
// Command palette
// ---------------------------------------------------------------------------

export type PaletteEntry = {
  id: string;
  title: string;
  description?: string;
  category: string;
  /** Display text for a keybind this app really has; absent when there is none. */
  keybind?: string;
};

/** The pin's `ENTRY_LIMIT`. */
export const PALETTE_ENTRY_LIMIT = 5;

/**
 * The pin's `COMMON_COMMAND_IDS` names `workspace.new`, `terminal.toggle`, and
 * `review.toggle`, which have no handler here. The mechanism is ported exactly;
 * the list is BrainRoot's own real commands.
 */
export const PALETTE_COMMON_IDS = [
  "session.new",
  "session.previous",
  "session.next",
  "home.open",
  "settings.open"
] as const;

/**
 * The pin's `matchesEntry`, character for character: a lowercase substring test
 * over title, description, and category; any match wins.
 */
export function matchesEntry(entry: PaletteEntry, query: string): boolean {
  const value = query.toLowerCase();
  return [entry.title, entry.description, entry.category].some((text) =>
    text?.toLowerCase().includes(value)
  );
}

/** The pin's `uniqueCommandPaletteEntries`: first id wins. */
export function uniqueEntries(items: PaletteEntry[]): PaletteEntry[] {
  const seen = new Set<string>();
  return items.filter((item) => {
    if (seen.has(item.id)) {
      return false;
    }
    seen.add(item.id);
    return true;
  });
}

/** The pin's `groups`: category groups in insertion order, entries in order. */
export function groups(
  entries: PaletteEntry[]
): { category: string; entries: PaletteEntry[] }[] {
  const map = new Map<string, PaletteEntry[]>();
  for (const entry of entries) {
    map.set(entry.category, [...(map.get(entry.category) ?? []), entry]);
  }
  return Array.from(map.entries()).map(([category, items]) => ({
    category,
    entries: items
  }));
}

/**
 * The pin's `preferredCommandEntries`: the common ids that exist, in common
 * order; when none exist, the first `PALETTE_ENTRY_LIMIT` in list order.
 */
export function preferredEntries(
  all: PaletteEntry[],
  commonIds: readonly string[] = PALETTE_COMMON_IDS
): PaletteEntry[] {
  const order = new Map<string, number>(commonIds.map((id, index) => [id, index]));
  const picked = all.filter((entry) => order.has(entry.id));
  const base = picked.length > 0 ? picked : all.slice(0, PALETTE_ENTRY_LIMIT);
  return picked.length > 0
    ? [...base].sort((a, b) => (order.get(a.id) ?? 0) - (order.get(b.id) ?? 0))
    : base;
}

/** The pin's `move`: an active row advances with modulo wrap at both ends. */
export function moveActive(count: number, active: number, delta: 1 | -1): number {
  if (count <= 0) {
    return 0;
  }
  return (active + delta + count) % count;
}

export type PaletteKeyIntent =
  | { kind: "none" }
  | { kind: "move"; delta: 1 | -1 }
  | { kind: "select" }
  | { kind: "close" };

export type PaletteKey = {
  key: string;
  altKey?: boolean;
  metaKey?: boolean;
  ctrlKey?: boolean;
  shiftKey?: boolean;
};

/**
 * The pin's palette input handler, exactly: arrows move, Enter selects, Escape
 * closes, and everything else — including Tab — is left alone. Unlike the model
 * picker's handler there is no modified-key guard and no composition guard in
 * the pin, so there is none here either.
 */
export function paletteKeyIntent(event: PaletteKey): PaletteKeyIntent {
  if (event.key === "ArrowDown") {
    return { kind: "move", delta: 1 };
  }
  if (event.key === "ArrowUp") {
    return { kind: "move", delta: -1 };
  }
  if (event.key === "Enter") {
    return { kind: "select" };
  }
  if (event.key === "Escape") {
    return { kind: "close" };
  }
  return { kind: "none" };
}

export const PALETTE_EMPTY_LABEL = "No commands match.";
export const PALETTE_SEARCH_LABEL = "Search commands";

/**
 * Every command is a handler `App.svelte` really wires. Host commands are the
 * honest pair: exactly one of start/stop exists at a time, because the other
 * would do nothing.
 */
export function appCommands(hostRunning: boolean): PaletteEntry[] {
  const commands: PaletteEntry[] = [
    {
      id: "session.new",
      title: "New session",
      description: "Start a blank conversation",
      category: "Commands"
    },
    {
      id: "session.previous",
      title: "Previous session",
      description: "Step back through open sessions",
      category: "Commands",
      keybind: "Ctrl+Shift+Tab"
    },
    {
      id: "session.next",
      title: "Next session",
      description: "Step through open sessions",
      category: "Commands",
      keybind: "Ctrl+Tab"
    },
    {
      id: "home.open",
      title: "Open home",
      description: "Sessions, projects, and planned destinations",
      category: "Commands"
    },
    {
      id: "settings.open",
      title: "Open settings",
      description: "Appearance, shortcuts, host, and models",
      category: "Commands"
    },
    {
      id: "layout.swap",
      title: "Swap sides",
      description: "Move the Canvas to the other side",
      category: "Commands"
    },
    {
      id: "theme.toggle",
      title: "Toggle theme",
      description: "Switch between dark and light",
      category: "Commands"
    }
  ];
  commands.push(
    hostRunning
      ? {
          id: "host.stop",
          title: "Stop sidecar",
          description: "Shut the local model loop down",
          category: "Commands"
        }
      : {
          id: "host.start",
          title: "Start sidecar",
          description: "Start the local model loop",
          category: "Commands"
        }
  );
  return commands;
}

/**
 * Sessions are the pin's palette "session" entries, built from real tabs. The
 * pin shows recent files alongside; no workspace listing exists here, so
 * sessions are the only secondary group and nothing is invented in its place.
 */
export function sessionEntries(
  tabs: { id: number; title: string; turnCount: number }[]
): PaletteEntry[] {
  return tabs.slice(0, PALETTE_ENTRY_LIMIT).map((tab) => ({
    id: `session:${tab.id}`,
    title: tab.title,
    description: tab.turnCount === 1 ? "1 turn" : `${tab.turnCount} turns`,
    category: "Sessions"
  }));
}

/**
 * The pin's `loadItems` shape: an empty query shows preferred commands plus
 * recent sessions; a query shows matching commands plus matching sessions.
 */
export function paletteEntries(
  commands: PaletteEntry[],
  sessions: PaletteEntry[],
  query: string
): PaletteEntry[] {
  const trimmed = query.trim();
  if (!trimmed) {
    return uniqueEntries([...preferredEntries(commands), ...sessions]);
  }
  return uniqueEntries([
    ...commands.filter((entry) => matchesEntry(entry, trimmed)),
    ...sessions.filter((entry) => matchesEntry(entry, trimmed))
  ]);
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

export type SettingsRow = {
  id: string;
  title: string;
  description: string;
  /** `true` when no backend exists for this row and it must offer no control. */
  readonly: boolean;
  /** Display value for read-only rows. */
  value?: string;
};

export type SettingsTab = {
  id: string;
  section: "Desktop" | "Server";
  label: string;
  rows: SettingsRow[];
};

export const SETTINGS_READONLY_NOTE = "Read-only: no settings backend yet.";

export type SettingsData = {
  theme: "dark" | "light";
  credentialStatus: "configured" | "not_configured" | "unavailable";
  credentialMessage: string;
  hostRunning: boolean;
  hostPort: number | null;
  hostVersion: string | null;
  modelCount: number;
  selectedModel: string | null;
  modelsStale: boolean;
};

/**
 * The pin's five settings tabs, filled with data this app already has. Theme is
 * the one row with a real backend (it persists to this device); everything else
 * is read-only and says so, because a control that saves nothing is a fake
 * control.
 */
export function settingsTabs(data: SettingsData): SettingsTab[] {
  const modelValue =
    data.modelCount === 0
      ? "No models are available"
      : `${data.selectedModel ?? "No model selected"} · ${data.modelCount} available`;
  return [
    {
      id: "general",
      section: "Desktop",
      label: "General",
      rows: [
        {
          id: "theme",
          title: "Theme",
          description: `Dark or light. Saved on this device; currently ${data.theme}.`,
          readonly: false
        },
        {
          id: "canvas-side",
          title: "Canvas side",
          description: `Swap sides from the workspace bar. ${SETTINGS_READONLY_NOTE}`,
          readonly: true,
          value: "Session state"
        }
      ]
    },
    {
      id: "shortcuts",
      section: "Desktop",
      label: "Shortcuts",
      rows: [
        {
          id: "shortcut-commands",
          title: "Ctrl+K",
          description: `Open commands. Rebinding ${SETTINGS_READONLY_NOTE}`,
          readonly: true
        },
        {
          id: "shortcut-sessions",
          title: "Ctrl+Tab",
          description: `Next session; Ctrl+Shift+Tab for the previous one. Rebinding ${SETTINGS_READONLY_NOTE}`,
          readonly: true
        }
      ]
    },
    {
      id: "servers",
      section: "Server",
      label: "Servers",
      rows: [
        {
          id: "host-status",
          title: "Sidecar",
          description: `The local model loop. ${SETTINGS_READONLY_NOTE}`,
          readonly: true,
          value: data.hostRunning ? "Running" : "Stopped"
        },
        {
          id: "host-address",
          title: "Address",
          description: `Where the sidecar listens. ${SETTINGS_READONLY_NOTE}`,
          readonly: true,
          value: data.hostPort === null ? "Not started" : `localhost:${data.hostPort}`
        },
        {
          id: "host-version",
          title: "Version",
          description: `Reported by the sidecar. ${SETTINGS_READONLY_NOTE}`,
          readonly: true,
          value: data.hostVersion ?? "Unknown"
        }
      ]
    },
    {
      id: "providers",
      section: "Server",
      label: "Providers",
      rows: [
        {
          id: "credential",
          title: "Credential",
          description: `${data.credentialMessage} Status only — the key never leaves the system credential store.`,
          readonly: true,
          value:
            data.credentialStatus === "configured"
              ? "Configured"
              : data.credentialStatus === "not_configured"
                ? "Not configured"
                : "Unavailable"
        }
      ]
    },
    {
      id: "models",
      section: "Server",
      label: "Models",
      rows: [
        {
          id: "models",
          title: "Catalog",
          description:
            `Choose a model from the composer's model control. ${SETTINGS_READONLY_NOTE}` +
            (data.modelsStale ? " The catalog is stale." : ""),
          readonly: true,
          value: modelValue
        }
      ]
    }
  ];
}
