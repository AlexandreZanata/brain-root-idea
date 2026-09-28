/**
 * B20-U7 — the Canvas panel registry and lifecycle, pure.
 *
 * Every Canvas preset is a resource record: a kind (heavy WebView or light
 * DOM), a lifecycle phase (closed → cold → hot → closed), and a cleanup path
 * that runs on close. The one-HOT rule of ADR 0006 is enforced in the
 * transition function itself — `openPanel` returns the ids that MUST tear down
 * so a second heavy view can never go HOT — instead of being observed after
 * the fact. A test can freeze the invariant without a DOM; `CanvasPanel.svelte`
 * only renders what the registry says and unmounts what it says to release.
 *
 * Vocabulary follows `docs/09-resource-governor.md`: COLD = open, owned,
 * idle-cheap (DOM only); HOT = the single heavy view; CLOSED = released.
 */

/** The five Canvas presets U7 unifies. `preview`/`browser` are the B18 views. */
export type PanelId = "preview" | "browser" | "files" | "review" | "terminal";

/** Heavy views own a WebView; light views are DOM and never go HOT. */
export type PanelKind = "heavy" | "light";

/** Lifecycle phase. Only heavy views can be `hot`, and only one at a time. */
export type PanelPhase = "closed" | "cold" | "hot";

export type PanelRecord = {
  id: PanelId;
  kind: PanelKind;
  phase: PanelPhase;
};

export type PanelRegistry = Record<PanelId, PanelRecord>;

export const PANEL_IDS: readonly PanelId[] = [
  "preview",
  "browser",
  "files",
  "review",
  "terminal"
];

const PANEL_KINDS: Record<PanelId, PanelKind> = {
  preview: "heavy",
  browser: "heavy",
  files: "light",
  review: "light",
  terminal: "light"
};

export function emptyRegistry(): PanelRegistry {
  const registry = {} as PanelRegistry;
  for (const id of PANEL_IDS) {
    registry[id] = { id, kind: PANEL_KINDS[id], phase: "closed" };
  }
  return registry;
}

/** The one HOT heavy view, or null when the board is quiet. */
export function hotPanel(registry: PanelRegistry): PanelId | null {
  for (const id of PANEL_IDS) {
    if (registry[id].phase === "hot") {
      return id;
    }
  }
  return null;
}

/** Every id that currently holds resources (cold or hot). */
export function openPanels(registry: PanelRegistry): PanelId[] {
  return PANEL_IDS.filter((id) => registry[id].phase !== "closed");
}

export type OpenResult = {
  registry: PanelRegistry;
  /** Ids whose resources must be released BEFORE the new view is shown. */
  release: PanelId[];
};

/**
 * Open one panel. The Canvas body has a single slot — switching to a panel
 * DESTROYS whatever was showing rather than parking it (the ADR 0006 rule is
 * enforced, not relaxed: `canvasTransition`'s own copy says the previous view
 * "was closed when you switched tabs"). So every other open panel lands in
 * `release` and must run its teardown before the new one mounts. Heavy views
 * go HOT, light views stay COLD, and a second HOT view is unreachable by
 * construction.
 */
export function openPanel(registry: PanelRegistry, id: PanelId): OpenResult {
  const kind = PANEL_KINDS[id];
  const release: PanelId[] = [];
  const next = clone(registry);

  for (const other of PANEL_IDS) {
    if (other !== id && next[other].phase !== "closed") {
      release.push(other);
      next[other] = { ...next[other], phase: "closed" };
    }
  }

  next[id] = { ...next[id], phase: kind === "heavy" ? "hot" : "cold" };
  return { registry: next, release };
}

/** Close one panel; its cleanup path runs because the view unmounts. */
export function closePanel(registry: PanelRegistry, id: PanelId): PanelRegistry {
  const next = clone(registry);
  next[id] = { ...next[id], phase: "closed" };
  return next;
}

/**
 * Application exit / window close walks the registry leaf-first (docs/09):
 * everything is released and the board returns to empty.
 */
export function closeAll(_registry: PanelRegistry): PanelRegistry {
  return emptyRegistry();
}

function clone(registry: PanelRegistry): PanelRegistry {
  const next = {} as PanelRegistry;
  for (const id of PANEL_IDS) {
    next[id] = { ...registry[id] };
  }
  return next;
}

// ---------------------------------------------------------------------------
// File tabs (pin `file-tab-scroll.ts` + `SortableTabV2` chrome)
// ---------------------------------------------------------------------------

export type FileTab = {
  id: string;
  /** Display name only; the full path never fabricates a workspace. */
  name: string;
  /** The pin's temporary tab: italic label, pinned by double-click. */
  temporary: boolean;
};

/** The pin's `nextTabListScrollLeft`: only chase growth, or the opener jump. */
export function nextTabListScrollLeft(input: {
  prevScrollWidth: number;
  scrollWidth: number;
  clientWidth: number;
  prevContextOpen: boolean;
  contextOpen: boolean;
}): number | undefined {
  if (input.scrollWidth <= input.prevScrollWidth) return undefined;
  if (!input.prevContextOpen && input.contextOpen) return 0;
  if (input.scrollWidth <= input.clientWidth) return undefined;
  return input.scrollWidth - input.clientWidth;
}

/** The pin's wheel rule: vertical wheel scrolls the strip horizontally by 50. */
export function tabStripWheelDelta(event: {
  deltaY: number;
  deltaX: number;
}): number | null {
  if (Math.abs(event.deltaY) <= Math.abs(event.deltaX)) return null;
  return event.deltaY > 0 ? 50 : -50;
}

/** Close on middle-click anywhere on the tab, or on the close button. */
export function shouldCloseFileTab(event: {
  button: number;
  targetIsCloseButton: boolean;
}): boolean {
  return event.button === 1 || event.targetIsCloseButton;
}

/** Double-click pins (un-temporaries) the tab the pin marked temporary. */
export function fileTabAfterDoubleClick(tabs: FileTab[], id: string): FileTab[] {
  return tabs.map((tab) =>
    tab.id === id && tab.temporary ? { ...tab, temporary: false } : tab
  );
}

// ---------------------------------------------------------------------------
// Terminal mirror honesty (pin `terminal-panel*.tsx` minus the PTY)
// ---------------------------------------------------------------------------

/**
 * The mirror's visible identity. It must state it is a mirror wherever a real
 * terminal would put its title, so no surface can be read as an interactive
 * shell (B20-U7-T04).
 */
export const TERMINAL_MIRROR_LABEL = "Terminal — turn output mirror";
export const TERMINAL_MIRROR_NOTE =
  "This panel mirrors the turn output. There is no shell here yet.";

export type MirrorTurnState = "running" | "completed" | "cancelled" | "failed";

export type MirrorTurn = {
  session: string;
  text: string;
  state: MirrorTurnState;
  error?: string;
};

/** The mirror is a bounded ledger (docs/09: flush bounded state). */
export const MIRROR_MAX_TURNS = 20;
export const MIRROR_MAX_TEXT_CHARS = 200 * 1024;

/**
 * Fold one stream event into the mirrored turns. The session is its own
 * argument because the legacy `conversation_event` stream carries none — a
 * turn belongs to whatever source reported it. Text accumulates only for the
 * running turn; a turn that fails keeps its text and records the error
 * message. Nothing here spawns or simulates a process — it is a ledger of
 * what the turn already produced.
 */
export function foldMirrorEvent(
  turns: MirrorTurn[],
  session: string,
  event: { type: string; text?: string; error?: { message: string } }
): MirrorTurn[] {
  const folded = foldMirrorEventOnce(turns, session, event);
  // An event that changes nothing returns the ledger unchanged — identity
  // included, so a no-op stream cannot churn the render or the ledger.
  return folded === turns ? turns : boundMirrorTurns(folded);
}

function foldMirrorEventOnce(
  turns: MirrorTurn[],
  session: string,
  event: { type: string; text?: string; error?: { message: string } }
): MirrorTurn[] {
  switch (event.type) {
    case "started":
      return [...turns, { session, text: "", state: "running" }];
    case "text_chunk": {
      const chunk = event.text ?? "";
      const index = chunk === "" ? -1 : runningTurnIndex(turns, session);
      if (index === -1) {
        return turns;
      }
      const turn = turns[index];
      const next = [...turns];
      next[index] = { ...turn, text: turn.text + (event.text ?? "") };
      return next;
    }
    case "completed":
    case "cancelled":
      return markTurn(turns, session, event.type);
    case "failed":
      return markTurn(turns, session, "failed", event.error?.message);
    default:
      return turns;
  }
}

/**
 * Keep the ledger bounded: at most `MIRROR_MAX_TURNS` turns and
 * `MIRROR_MAX_TEXT_CHARS` characters, dropping the oldest first. One turn can
 * outgrow the whole budget alone (a long stream with no sibling turns), so a
 * lone turn keeps only its tail — the newest output, what a mirror shows.
 * The conversation view bounds its history the same way (`conversation.ts`).
 */
export function boundMirrorTurns(turns: MirrorTurn[]): MirrorTurn[] {
  let kept = turns.slice(-MIRROR_MAX_TURNS);
  let total = kept.reduce((sum, turn) => sum + turn.text.length, 0);
  while (kept.length > 1 && total > MIRROR_MAX_TEXT_CHARS) {
    const dropped = kept.shift();
    total -= dropped?.text.length ?? 0;
  }
  if (kept.length === 1 && total > MIRROR_MAX_TEXT_CHARS) {
    const only = kept[0];
    kept = [{ ...only, text: only.text.slice(-MIRROR_MAX_TEXT_CHARS) }];
  }
  return kept;
}

function markTurn(
  turns: MirrorTurn[],
  session: string,
  state: MirrorTurnState,
  error?: string
): MirrorTurn[] {
  const index = runningTurnIndex(turns, session);
  if (index === -1) {
    return turns;
  }
  const turn = turns[index];
  const next = [...turns];
  next[index] = { ...turn, state, ...(error !== undefined ? { error } : {}) };
  return next;
}

/** The running turn of one session, whichever slot it occupies. */
function runningTurnIndex(turns: MirrorTurn[], session: string): number {
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index];
    if (turn.session === session && turn.state === "running") {
      return index;
    }
  }
  return -1;
}
