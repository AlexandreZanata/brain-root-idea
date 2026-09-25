<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import {
    PREVIEW_OPEN_DELAY_MS,
    adjacentTabId,
    canShowTabPreview,
    canStartTabDrag,
    dropIndexAt,
    mergeVisibleOrder,
    moveWithinOrder,
    previewSkipsDelay
  } from "../tabOrder";

  export type SessionTab = {
    id: number;
    title: string;
  };

  let {
    tabs = [],
    activeId = 0,
    streamingId = null,
    onselect,
    onclose,
    onnew,
    onreorder = () => {}
  }: {
    tabs?: SessionTab[];
    activeId?: number;
    streamingId?: number | null;
    onselect: (id: number) => void;
    onclose: (id: number) => void;
    onnew: () => void;
    onreorder?: (orderedIds: number[]) => void;
  } = $props();

  // B20-U2. The strip is a group of real buttons instead of a `tablist` whose
  // `tab` wrappers contained buttons, which was invalid ARIA and blocked every
  // extension. Reorder follows the pinned OpenCode semantics: a drag moves the
  // visible tabs only and hidden ones keep their slots; the keyboard moves one
  // slot in the full order and stops at the ends.

  let strip = $state<HTMLDivElement | null>(null);
  let fadeLeft = $state(false);
  let fadeRight = $state(false);

  let dragId = $state<number | null>(null);
  let dragPointerId: number | null = null;
  let dragElement: HTMLElement | null = null;
  let dragStartOrder: number[] = [];
  let workingOrder = $state<number[] | null>(null);

  let previewId = $state<number | null>(null);
  let previewAnchor = $state<{ left: number; top: number; width: number } | null>(null);
  let previewTimer: ReturnType<typeof setTimeout> | null = null;
  let lastPreviewClosedAt = 0;

  const order = $derived(tabs.map((tab) => tab.id));
  const renderOrder = $derived(workingOrder ?? order);
  const renderTabs = $derived(
    renderOrder
      .map((id) => tabs.find((tab) => tab.id === id))
      .filter((tab): tab is SessionTab => tab !== undefined)
  );

  function tabNodes(): HTMLElement[] {
    const element = strip;
    if (!element) return [];
    return Array.from(element.querySelectorAll<HTMLElement>("[data-tab-id]"));
  }

  // A tab counts as visible when its box overlaps the strip's box, which is what
  // decides whether a drag may move it (the pin's `mergeVisibleTabOrder`).
  function visibleTabs(): { id: number; left: number; right: number }[] {
    const element = strip;
    if (!element) return [];
    const bounds = element.getBoundingClientRect();
    const visible: { id: number; left: number; right: number }[] = [];
    for (const node of tabNodes()) {
      const rect = node.getBoundingClientRect();
      if (rect.right > bounds.left + 1 && rect.left < bounds.right - 1) {
        visible.push({ id: Number(node.dataset.tabId), left: rect.left, right: rect.right });
      }
    }
    return visible;
  }

  // The fades are the overflow signal: each side only shows while content is
  // actually hidden beyond that edge, which is stricter than the pin's
  // "scrollWidth > clientWidth" report and needs no second flag.
  function measure() {
    const element = strip;
    if (!element) return;
    fadeLeft = element.scrollLeft > 1;
    fadeRight = element.scrollLeft + element.clientWidth < element.scrollWidth - 1;
  }

  $effect(() => {
    const element = strip;
    if (!element) return;
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  });

  // Re-measure whenever the rendered tabs change, so the fades track content.
  $effect(() => {
    renderTabs;
    measure();
  });

  function stopPreviewTimer() {
    if (previewTimer !== null) {
      clearTimeout(previewTimer);
      previewTimer = null;
    }
  }

  function hidePreview() {
    stopPreviewTimer();
    if (previewId !== null) {
      previewId = null;
      lastPreviewClosedAt = Date.now();
    }
    previewAnchor = null;
  }

  function showPreview(id: number) {
    const node = tabNodes().find((candidate) => Number(candidate.dataset.tabId) === id);
    if (!node) return;
    const rect = node.getBoundingClientRect();
    previewAnchor = { left: rect.left, top: rect.bottom + 6, width: rect.width };
    previewId = id;
  }

  function schedulePreview(id: number) {
    if (!canShowTabPreview(dragId !== null || previewId === id)) return;
    stopPreviewTimer();
    if (previewSkipsDelay(Date.now(), lastPreviewClosedAt)) {
      showPreview(id);
      return;
    }
    previewTimer = setTimeout(() => {
      previewTimer = null;
      showPreview(id);
    }, PREVIEW_OPEN_DELAY_MS);
  }

  onDestroy(stopPreviewTimer);

  function onTabPointerDown(event: PointerEvent, id: number) {
    if (event.button !== 0) return;
    hidePreview();
    // The pin navigates on pointer down and leaves the click handler to keyboard
    // activation only, so a press that becomes a drag still switches session.
    onselect(id);
    if (!canStartTabDrag(event.pointerType)) return;
    const node = event.currentTarget as HTMLElement;
    dragElement = node;
    dragPointerId = event.pointerId;
    dragStartOrder = order.slice();
    workingOrder = null;
    dragId = id;
    // Capture keeps the pointermove stream alive when the cursor leaves the tab.
    // It is an enhancement, not a precondition: if the environment refuses it,
    // the drag still tracks while the pointer stays over the tab instead of
    // failing in the middle of a gesture.
    try {
      node.setPointerCapture(event.pointerId);
    } catch {
      dragPointerId = event.pointerId;
    }
  }

  function onTabPointerMove(event: PointerEvent) {
    if (dragId === null || event.pointerId !== dragPointerId) return;
    const visible = visibleTabs();
    if (visible.length === 0) return;
    const visibleIds = visible.map((entry) => entry.id);
    const moved = moveWithinOrder(visibleIds, dragId, dropIndexAt(event.clientX, visible));
    workingOrder = mergeVisibleOrder(dragStartOrder, visibleIds, moved);
  }

  function endDrag() {
    if (dragElement && dragPointerId !== null && dragElement.hasPointerCapture?.(dragPointerId)) {
      try {
        dragElement.releasePointerCapture(dragPointerId);
      } catch {
        // Already released, or the pointer is gone: nothing left to undo.
      }
    }
    dragElement = null;
    dragPointerId = null;
    dragStartOrder = [];
    dragId = null;
    workingOrder = null;
  }

  // Focus follows the tab a reorder moved, rather than relying on the DOM node
  // surviving the move: a keyboard user must land on the tab they just moved.
  async function reorderAndFocus(next: number[], id: number) {
    onreorder(next);
    await tick();
    tabNodes()
      .find((node) => Number(node.dataset.tabId) === id)
      ?.focus();
  }

  async function commitDrag() {
    const committed = workingOrder;
    const movedId = dragId;
    endDrag();
    if (movedId === null || committed === null) return;
    await reorderAndFocus(committed, movedId);
  }

  async function onTabKeyDown(event: KeyboardEvent, id: number) {
    if (event.key === "Escape" && dragId !== null) {
      event.preventDefault();
      endDrag();
      return;
    }
    if (!event.altKey || (event.key !== "ArrowLeft" && event.key !== "ArrowRight")) return;
    event.preventDefault();
    const index = order.indexOf(id);
    if (index === -1) return;
    const target = Math.min(order.length - 1, Math.max(0, index + (event.key === "ArrowLeft" ? -1 : 1)));
    if (target === index) return;
    endDrag();
    await reorderAndFocus(moveWithinOrder(order, id, target), id);
  }

  // Ctrl+Tab / Ctrl+Shift+Tab traverse sessions from anywhere in the window, as
  // the pin does. Registered with its teardown so nothing survives unmount.
  $effect(() => {
    function onWindowKeyDown(event: KeyboardEvent) {
      if (!event.ctrlKey || event.key !== "Tab" || order.length === 0) return;
      event.preventDefault();
      const next = adjacentTabId(order, activeId, event.shiftKey ? -1 : 1);
      if (next !== undefined) onselect(next);
    }
    window.addEventListener("keydown", onWindowKeyDown);
    return () => window.removeEventListener("keydown", onWindowKeyDown);
  });
</script>

<div
  class="tabs"
  role="group"
  aria-label="Sessions"
  onpointercancel={endDrag}
  onlostpointercapture={endDrag}
  onmouseleave={hidePreview}
>
  <div class="scroll-area">
    <div class="scroller" bind:this={strip} onscroll={measure}>
      {#each renderTabs as tab (tab.id)}
        {@const active = tab.id === activeId}
        {@const streaming = tab.id === streamingId}
        <div class="tab-slot" class:active>
          <button
            class="tab"
            class:dragging={dragId === tab.id}
            type="button"
            data-tab-id={tab.id}
            data-active={active ? "true" : undefined}
            aria-current={active ? "true" : undefined}
            title={tab.title}
            onpointerdown={(event) => onTabPointerDown(event, tab.id)}
            onpointermove={onTabPointerMove}
            onpointerup={() => void commitDrag()}
            onpointercancel={endDrag}
            onkeydown={(event) => void onTabKeyDown(event, tab.id)}
            onclick={(event) => {
              if (event.detail === 0) onselect(tab.id);
            }}
            onmouseenter={() => schedulePreview(tab.id)}
            onmouseleave={hidePreview}
            onfocus={hidePreview}
            onblur={hidePreview}
          >
            <span class="tab-title">{tab.title}</span>
            {#if streaming}
              <!-- BrainRoot feature: the pin has no per-tab activity indicator. -->
              <span class="tab-live" aria-hidden="true">●</span>
              <span class="br-visually-hidden">Streaming</span>
            {/if}
          </button>
          {#if tabs.length > 1}
            <button
              class="tab-close"
              type="button"
              aria-label={`Close ${tab.title}`}
              onpointerdown={(event) => {
                // A press on close must never reach the drag or the navigation.
                event.preventDefault();
                event.stopPropagation();
                hidePreview();
              }}
              onclick={() => onclose(tab.id)}
            >
              ×
            </button>
          {/if}
        </div>
      {/each}
    </div>
    {#if fadeLeft}
      <span class="fade fade-left" aria-hidden="true"></span>
    {/if}
    {#if fadeRight}
      <span class="fade fade-right" aria-hidden="true"></span>
    {/if}
  </div>
  <button class="tab-new" type="button" onclick={onnew} aria-label="New session" title="New session">
    +
  </button>
</div>

{#if previewId !== null && previewAnchor}
  {@const previewTab = tabs.find((tab) => tab.id === previewId)}
  {#if previewTab}
    <!-- Non-interactive hover preview, matching the pin's delays. Decorative:
         it takes no place in the focus order, never intercepts a pointer, and
         is not announced. -->
    <span
      class="tab-preview"
      aria-hidden="true"
      style="left: {previewAnchor.left}px; top: {previewAnchor.top}px; min-width: {previewAnchor.width}px"
    >
      {previewTab.title}
    </span>
  {/if}
{/if}

<style>
  .tabs {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    min-width: 0;
  }

  .scroll-area {
    position: relative;
    display: flex;
    flex: 1;
    min-width: 0;
  }

  .scroller {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .tab-slot {
    position: relative;
    display: flex;
    align-items: center;
    flex: 0 0 auto;
    max-width: 12rem;
    border-radius: var(--radius-control);
  }

  .tab-slot.active {
    background: var(--surface-raised);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    min-width: 0;
    height: 1.75rem;
    padding: 0 0.375rem;
    border: 0;
    border-radius: var(--radius-control);
    background: none;
    color: var(--text-muted);
    font: inherit;
    font-size: var(--text-supporting);
    font-weight: 500;
    cursor: pointer;
  }

  .tab.dragging {
    opacity: 0.7;
  }

  .tab:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .tab[data-active="true"] {
    color: var(--text);
  }

  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tab-live {
    color: var(--accent);
    font-size: 0.6em;
  }

  /* Close keeps its box and focus ring at all times so the hit area never
     disappears; only the opacity changes. Revealed on hover, pinned when
     active, always visible while focused. */
  .tab-close {
    flex: 0 0 auto;
    height: 1.75rem;
    padding: 0 0.25rem;
    border: 0;
    background: none;
    color: var(--text-muted);
    font: inherit;
    font-size: var(--text-supporting);
    line-height: 1;
    cursor: pointer;
    border-radius: var(--radius-control);
    opacity: 0;
    transition: opacity 120ms ease;
  }

  .tab-slot:hover .tab-close,
  .tab-slot.active .tab-close,
  .tab-close:focus-visible {
    opacity: 1;
  }

  .tab-close:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .tab-new {
    flex: 0 0 auto;
    height: 1.75rem;
    padding: 0 0.375rem;
    border: 0;
    background: none;
    color: var(--text-muted);
    font: inherit;
    font-size: var(--text-supporting);
    cursor: pointer;
    border-radius: var(--radius-control);
  }

  .tab-new:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  /* Edge fades use the surface the strip actually sits on rather than the pin's
     `bg-deep`, which assumes a different titlebar background. */
  .fade {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1.5rem;
    pointer-events: none;
  }

  .fade-left {
    left: 0;
    background: linear-gradient(to right, var(--bg), transparent);
  }

  .fade-right {
    right: 0;
    background: linear-gradient(to left, var(--bg), transparent);
  }

  .tab-preview {
    position: fixed;
    z-index: 30;
    display: block;
    max-width: 20rem;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    color: var(--text);
    font-size: var(--text-supporting);
    box-shadow: var(--v2-elevation-floating);
    pointer-events: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tab:focus-visible,
  .tab-close:focus-visible,
  .tab-new:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: 1px;
  }
</style>
