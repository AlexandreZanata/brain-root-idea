<script lang="ts">
  import Button from "./Button.svelte";
  import HumanBrowserPanel from "./HumanBrowserPanel.svelte";
  import PreviewPanel from "./PreviewPanel.svelte";
  import { canvasTransition, type CanvasTransition } from "../presentation";

  type CanvasTab = "preview" | "browser";

  const tabs: { id: CanvasTab | null; label: string }[] = [
    { id: "preview", label: "Preview" },
    { id: "browser", label: "Browser" },
    { id: null, label: "Components" },
    { id: null, label: "Logs" },
    { id: null, label: "AI Notes" }
  ];

  let active = $state<CanvasTab>("browser");
  let transition = $state<CanvasTransition | null>(null);

  // B18-S05 header swipe: dedicated chrome zone only. The page body and the
  // native WebView never see these handlers. Thresholds are fixed so the S07
  // release probe can verify them deterministically.
  const SWIPE_MIN_DISTANCE_PX = 48;
  const SWIPE_FLICK_DISTANCE_PX = 24;
  const SWIPE_FLICK_MAX_MS = 300;
  const SWIPE_HORIZONTAL_RATIO = 2;

  let swipeZone = $state<HTMLDivElement | null>(null);
  let tracking = false;
  let startX = 0;
  let startY = 0;
  let startT = 0;

  function swipeTarget(dx: number): CanvasTab | null {
    if (dx < 0) {
      return active === "preview" ? "browser" : null;
    }
    return active === "browser" ? "preview" : null;
  }

  function swipeStartsOnControl(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) {
      return false;
    }
    return target.closest("button, a, input, textarea, select") !== null;
  }

  function capturePointer(element: HTMLDivElement | null, pointerId: number) {
    if (!element || typeof element.setPointerCapture !== "function") {
      return;
    }
    try {
      element.setPointerCapture(pointerId);
    } catch {
      // A released pointer simply ends tracking on the next event.
    }
  }

  function releasePointer(element: HTMLDivElement | null, pointerId: number) {
    if (!element || typeof element.releasePointerCapture !== "function") {
      return;
    }
    try {
      if (element.hasPointerCapture?.(pointerId)) {
        element.releasePointerCapture(pointerId);
      }
    } catch {
      // Nothing to release; tracking already ends below.
    }
  }

  function onSwipeDown(event: PointerEvent) {
    if (event.isPrimary === false) {
      return;
    }
    if (event.pointerType === "mouse" && event.button !== 0) {
      return;
    }
    if (swipeStartsOnControl(event.target)) {
      return;
    }
    tracking = true;
    startX = event.clientX;
    startY = event.clientY;
    startT = performance.now();
    capturePointer(swipeZone, event.pointerId);
  }

  function onSwipeUp(event: PointerEvent) {
    if (!tracking) {
      return;
    }
    tracking = false;
    releasePointer(swipeZone, event.pointerId);
    const dx = event.clientX - startX;
    const dy = event.clientY - startY;
    const dt = performance.now() - startT;
    const adx = Math.abs(dx);
    const ady = Math.abs(dy);
    if (adx <= SWIPE_HORIZONTAL_RATIO * ady) {
      return;
    }
    const flick = dt <= SWIPE_FLICK_MAX_MS && adx >= SWIPE_FLICK_DISTANCE_PX;
    if (adx < SWIPE_MIN_DISTANCE_PX && !flick) {
      return;
    }
    const next = swipeTarget(dx);
    if (next !== null) {
      selectTab(next);
    }
  }

  function onSwipeCancel(event: PointerEvent) {
    if (!tracking) {
      return;
    }
    tracking = false;
    releasePointer(swipeZone, event.pointerId);
  }

  function selectTab(tab: CanvasTab) {
    if (tab === active) {
      return;
    }
    transition = canvasTransition(tab);
    active = tab;
  }
</script>

<section class="br-panel canvas" aria-labelledby="canvas-title">
  <!-- svelte-ignore a11y_no_static_element_interactions: the header swipe is a
       pointer-only shortcut and the zone itself stays a plain div with no added
       role, keeping the natural focus order; keyboard and assistive-technology
       parity is provided by the Canvas tab buttons inside it, which select the
       same destinations. -->
  <div
    class="br-panel__head br-panel__head--center canvas-swipe"
    bind:this={swipeZone}
    onpointerdown={onSwipeDown}
    onpointerup={onSwipeUp}
    onpointercancel={onSwipeCancel}
    onlostpointercapture={onSwipeCancel}
  >
    <div>
      <h2 id="canvas-title" class="br-panel__title">Preview / Canvas</h2>
      <p class="br-panel__subtitle">The result lives here — dominant by design.</p>
      {#if transition}
        <p class="canvas-transition" role="status">
          {transition.label} — {transition.note}
        </p>
      {/if}
    </div>
    <div class="br-tabs" aria-label="Canvas views">
      {#each tabs as tab (tab.label)}
        <Button
          variant="tab"
          inactive={tab.id === null || active !== tab.id}
          current={tab.id === active}
          title={tab.id === null ? `${tab.label} — planned for MVP-1` : tab.label}
          onclick={() => {
            if (tab.id) {
              selectTab(tab.id);
            }
          }}
        >
          {tab.label}
        </Button>
      {/each}
    </div>
  </div>

  <div class="br-panel__body canvas-body">
    {#if active === "preview"}
      <PreviewPanel />
    {:else if active === "browser"}
      <HumanBrowserPanel />
    {/if}
  </div>
</section>

<style>
  .canvas {
    height: 100%;
  }

  .canvas-transition {
    margin: 0.35rem 0 0;
    color: var(--text-subtle);
    font-size: 0.72rem;
  }

  .canvas-body {
    display: grid;
    grid-template-rows: minmax(0, 1fr);
    gap: 1rem;
    padding: 1.25rem;
  }

  @media (max-width: 1080px) {
    .canvas {
      min-height: 24rem;
    }
  }
</style>
