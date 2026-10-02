// B20-U2 tab-strip order logic, ported from the pinned OpenCode
// `titlebar-tab-order.ts` and `titlebar-tab-gesture.ts`
// (anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769).
//
// Pure functions only: no DOM and no framework, so `src/sessionTabs.test.mjs`
// can exercise them directly. The component owns the measuring, the timers,
// and the pointer capture; this module owns the arithmetic.

export type TabKey = number;

/** Hover-preview delays, matching the pin's `titlebar-tab-popover.tsx`. */
export const PREVIEW_OPEN_DELAY_MS = 2000;
export const PREVIEW_SKIP_WINDOW_MS = 500;

/** Invalid drop indexes clamp to the ends rather than throwing. */
export const TAB_ORDER_OFFSETS = [-1, 1] as const;

export type TabOffset = (typeof TAB_ORDER_OFFSETS)[number];

/**
 * Neighbour of `current` in `order`, wrapping at both ends — the pin's
 * `adjacentTabKey`. Returns `undefined` when there is nothing to move to,
 * which happens for an empty order or an id that is not in it.
 */
export function adjacentTabId(
  order: TabKey[],
  current: TabKey | undefined,
  offset: TabOffset
): TabKey | undefined {
  if (current === undefined || order.length === 0) return undefined;
  const index = order.indexOf(current);
  if (index === -1) return undefined;
  return order[(index + offset + order.length) % order.length];
}

/**
 * Move `id` to `targetIndex`, shifting the rest. Out-of-range targets clamp,
 * and a move onto the current slot returns an equal copy so callers can
 * compare cheaply. Returns a new array; the input is never mutated.
 */
export function moveWithinOrder(order: TabKey[], id: TabKey, targetIndex: number): TabKey[] {
  const from = order.indexOf(id);
  if (from === -1) return order.slice();
  const to = Math.min(order.length - 1, Math.max(0, targetIndex));
  if (to === from) return order.slice();
  const next = order.slice();
  next.splice(from, 1);
  next.splice(to, 0, id);
  return next;
}

/**
 * Reorder only the tabs that were visible, keeping every hidden tab in its
 * original slot — the pin's `mergeVisibleTabOrder`.
 *
 * `all` is the full order, `visibleBefore` the ids that were on screen when
 * the drag started (in screen order), and `visibleAfter` those same ids in
 * their new screen order. Anything not in `visibleBefore` keeps its place.
 */
export function mergeVisibleOrder(
  all: TabKey[],
  visibleBefore: TabKey[],
  visibleAfter: TabKey[]
): TabKey[] {
  // A caller handing back a different number of visible ids than it took would
  // move a tab into a slot another tab still occupies, producing an order with
  // a duplicate in it. Refuse the merge instead; the app enforces the same rule
  // when it adopts the result.
  if (visibleAfter.length !== visibleBefore.length) return all.slice();
  const visible = new Set(visibleBefore);
  let cursor = 0;
  return all.map((id) => {
    if (!visible.has(id)) return id;
    const replacement = visibleAfter[cursor];
    cursor += 1;
    return replacement ?? id;
  });
}

/**
 * Index of the drop slot for a pointer at `pointerX`, using the midpoint of
 * each visible tab: the slot is the number of tabs whose centre is left of
 * the pointer. Clamped to the available slots so a pointer dragged past
 * either end still resolves.
 */
export function dropIndexAt(
  pointerX: number,
  rects: readonly { left: number; right: number }[]
): number {
  if (rects.length === 0) return -1;
  let index = 0;
  for (const rect of rects) {
    if (pointerX > (rect.left + rect.right) / 2) index += 1;
  }
  return Math.min(rects.length - 1, Math.max(0, index));
}

/**
 * The pin refuses to start a tab drag from touch, so a touch gesture scrolls
 * the strip instead. Every other pointer type may drag.
 */
export function canStartTabDrag(pointerType: string): boolean {
  return pointerType !== "touch";
}

/**
 * Whether a preview may appear: never while dragging, while a tab is pressed,
 * or while a rename or menu is open.
 */
export function canShowTabPreview(blocked: boolean): boolean {
  return !blocked;
}

/** Whether a hover should skip the open delay, reusing a recent close. */
export function previewSkipsDelay(now: number, lastClosedAt: number): boolean {
  if (lastClosedAt === 0) return false;
  return now - lastClosedAt < PREVIEW_SKIP_WINDOW_MS;
}
