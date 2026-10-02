import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  PREVIEW_OPEN_DELAY_MS,
  PREVIEW_SKIP_WINDOW_MS,
  adjacentTabId,
  canShowTabPreview,
  canStartTabDrag,
  dropIndexAt,
  mergeVisibleOrder,
  moveWithinOrder,
  previewSkipsDelay
} from "./tabOrder.ts";

// B20-U2. The arithmetic is imported from `src/tabOrder.ts` and exercised
// directly; the component's markup, cleanup, and token use are asserted
// statically, because there is no DOM runtime in the test suite. Written here,
// executed at the versioned release gate per ADR 0014.

const componentPath = join(dirname(fileURLToPath(import.meta.url)), "lib", "SessionTabs.svelte");
const component = readFileSync(componentPath, "utf-8");

test("adjacentTabId wraps at both ends and refuses an unknown current tab", () => {
  const order = [1, 2, 3];

  assert.equal(adjacentTabId(order, 1, -1), 3, "previous from the first tab wraps to the last");
  assert.equal(adjacentTabId(order, 3, 1), 1, "next from the last tab wraps to the first");
  assert.equal(adjacentTabId(order, 2, 1), 3);
  assert.equal(adjacentTabId(order, 2, -1), 1);
  assert.equal(adjacentTabId(order, 9, 1), undefined, "an id outside the order has no neighbour");
  assert.equal(adjacentTabId([], 1, 1), undefined, "an empty order has no neighbour");
  assert.equal(adjacentTabId(order, undefined, 1), undefined);
  assert.equal(adjacentTabId([1], 1, 1), 1, "a single tab is its own neighbour");
});

test("moveWithinOrder shifts one slot, clamps, and never mutates its input", () => {
  const order = [1, 2, 3, 4];
  const snapshot = [...order];

  assert.deepEqual(moveWithinOrder(order, 1, 2), [2, 3, 1, 4]);
  assert.deepEqual(moveWithinOrder(order, 4, 0), [4, 1, 2, 3]);
  assert.deepEqual(moveWithinOrder(order, 1, -5), [1, 2, 3, 4], "a target before the start clamps to 0");
  assert.deepEqual(moveWithinOrder(order, 2, 99), [1, 3, 4, 2], "a target past the end clamps to the last slot");
  assert.deepEqual(moveWithinOrder(order, 3, 2), order, "a move onto the current slot is a no-op");
  assert.deepEqual(moveWithinOrder(order, 42, 1), order, "an unknown id changes nothing");
  assert.deepEqual(order, snapshot, "the input array must not be mutated");

  const same = moveWithinOrder(order, 3, 2);
  assert.notEqual(same, order, "the caller always receives a fresh array");
});

test("mergeVisibleOrder reorders only the visible subset and pins the hidden tabs", () => {
  // Tabs 2 and 4 are scrolled out of view and must keep their slots.
  const all = [1, 2, 3, 4, 5];
  const visibleBefore = [1, 3, 5];

  assert.deepEqual(mergeVisibleOrder(all, visibleBefore, [5, 3, 1]), [5, 2, 3, 4, 1]);
  assert.deepEqual(mergeVisibleOrder(all, visibleBefore, [3, 1, 5]), [3, 2, 1, 4, 5]);
  assert.deepEqual(
    mergeVisibleOrder(all, visibleBefore, visibleBefore),
    all,
    "an unchanged visible order leaves everything in place"
  );

  // A replacement list of the wrong length is refused rather than merged, so
  // two tabs can never land in the same slot.
  assert.deepEqual(mergeVisibleOrder(all, visibleBefore, [5]), all);
  assert.deepEqual(mergeVisibleOrder(all, visibleBefore, [5, 3, 1, 9]), all);
  assert.deepEqual(mergeVisibleOrder(all, [], []), all, "nothing visible means nothing moves");
});

test("dropIndexAt resolves the slot under the pointer and clamps past both ends", () => {
  const rects = [
    { left: 0, right: 100 },
    { left: 100, right: 200 },
    { left: 200, right: 300 }
  ];

  assert.equal(dropIndexAt(10, rects), 0);
  assert.equal(dropIndexAt(60, rects), 1, "past the first midpoint");
  assert.equal(dropIndexAt(160, rects), 2);
  assert.equal(dropIndexAt(-50, rects), 0, "dragging before the strip clamps to the first slot");
  assert.equal(dropIndexAt(9999, rects), 2, "dragging past the strip clamps to the last slot");
  assert.equal(dropIndexAt(50, []), -1, "no rects means no slot");
});

test("a touch gesture scrolls the strip instead of starting a drag", () => {
  assert.equal(canStartTabDrag("mouse"), true);
  assert.equal(canStartTabDrag("pen"), true);
  assert.equal(canStartTabDrag("touch"), false);
});

test("the hover preview waits, then skips its delay only inside the window", () => {
  assert.equal(canShowTabPreview(false), true);
  assert.equal(canShowTabPreview(true), false, "a drag or an open preview blocks it");
  assert.equal(previewSkipsDelay(1000, 0), false, "nothing was closed yet");
  assert.equal(
    previewSkipsDelay(1000 + PREVIEW_SKIP_WINDOW_MS - 1, 1000),
    true,
    "just inside the skip window"
  );
  assert.equal(
    previewSkipsDelay(1000 + PREVIEW_SKIP_WINDOW_MS, 1000),
    false,
    "the window is exclusive at its edge"
  );
  assert.equal(PREVIEW_OPEN_DELAY_MS, 2000, "the pin's open delay");
  assert.equal(PREVIEW_SKIP_WINDOW_MS, 500, "the pin's skip window");
});

test("the strip uses valid ARIA instead of a tab wrapper containing buttons", () => {
  assert.ok(component.includes('role="group"'), "the strip is a labelled group");
  assert.ok(component.includes('aria-label="Sessions"'));
  assert.ok(
    !component.includes('role="tab"') && !component.includes('role="tablist"'),
    "the invalid tablist/tab pattern must not come back"
  );
  assert.ok(
    component.includes('aria-current={active ? "true" : undefined}'),
    "the active tab is marked with aria-current on the control itself"
  );
  assert.ok(component.includes('aria-label={`Close ${tab.title}`}'));
  assert.ok(component.includes('aria-label="New session"'));
});

test("the component keeps the repository's interaction rules", () => {
  for (const [pattern, why] of [
    ["tabindex", "custom tabindex must never override the natural order"],
    ["user-select: none", "text selection must not be blocked"],
    ["outline: none", "focus outlines must not be removed"],
    ['role="tablist"', "the invalid tablist pattern"]
  ]) {
    assert.ok(!component.toLowerCase().includes(pattern.toLowerCase()), why);
  }
  assert.ok(
    !/(?<!aria-)\bdisabled(?:=|\s|>)/.test(component),
    "controls must not use the disabled attribute"
  );
  // A pointer drag is only acceptable because a keyboard equivalent exists.
  assert.ok(component.includes("Alt") || component.includes("altKey"), "a keyboard reorder path");
  assert.ok(component.includes("altKey"), "the reorder modifier is read from the event");
});

test("a reorder keeps focus on the tab it moved", () => {
  // Both the keyboard and the pointer path go through one helper, so neither can
  // silently drop focus onto the document body.
  assert.ok(component.includes("reorderAndFocus"), "one path owns reorder-and-focus");
  assert.ok(component.includes("await tick()"), "the DOM is flushed before focusing");
  assert.ok(component.includes("?.focus()"), "the moved tab is focused explicitly");
  assert.ok(
    component.includes("onkeydown={(event) => void onTabKeyDown(event, tab.id)}"),
    "the keyboard path uses it"
  );
  assert.ok(
    component.includes("onpointerup={() => void commitDrag()}"),
    "the pointer path uses it too"
  );
});

test("every listener and timer the component opens has a teardown", () => {
  assert.ok(
    component.includes("return () => window.removeEventListener(\"keydown\", onWindowKeyDown)"),
    "the window keydown listener is removed on unmount"
  );
  assert.ok(
    component.includes("return () => observer.disconnect()"),
    "the ResizeObserver is disconnected on unmount"
  );
  assert.ok(component.includes("onDestroy(stopPreviewTimer)"), "the preview timer dies with the component");
  assert.ok(
    component.includes("if (previewTimer !== null)") && component.includes("clearTimeout(previewTimer)"),
    "the pending preview timer is cleared rather than left to fire"
  );
  assert.ok(
    component.includes("setPointerCapture") && component.includes("releasePointerCapture"),
    "pointer capture is taken and released"
  );
});

test("the strip consumes ported tokens and defines no raw colour", () => {
  for (const token of [
    "--radius-control",
    "--text-supporting",
    "--text-muted",
    "--surface-hover",
    "--surface-raised",
    "--bg",
    "--focus",
    "--v2-elevation-floating"
  ]) {
    assert.ok(component.includes(`var(${token})`), `${token} should be consumed as a token`);
  }
  const style = component.slice(component.indexOf("<style>"));
  assert.ok(
    !/#[0-9a-f]{3,8}\b/i.test(style),
    "the component style must not define raw hex colours"
  );
  assert.ok(
    !/\b\d+px\s+solid\b/.test(style) || !style.includes("border: 1px solid #"),
    "no raw colour in borders"
  );
  // The pinned geometry: 28px tabs, 6px inline padding and gap, 13px labels.
  assert.ok(style.includes("height: 1.75rem"), "tabs are 28px tall as in the pin");
  assert.ok(style.includes("padding: 0 0.375rem"), "6px inline padding as in the pin");
  assert.ok(style.includes("gap: 0.375rem"), "6px gap as in the pin");
});

test("the hover preview is decorative and reachable by neither pointer nor keyboard", () => {
  const preview = component.slice(component.indexOf('class="tab-preview"'));
  assert.ok(component.includes('class="tab-preview"'));
  assert.ok(component.includes('aria-hidden="true"'), "the preview is hidden from assistive technology");
  assert.ok(component.includes("pointer-events: none"), "the preview never intercepts a click");
  assert.ok(
    component.includes("position: fixed"),
    "the preview escapes the scroller's clipping rather than being cut off"
  );
  assert.ok(preview.length > 0);
});
