<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import {
    humanBack,
    humanClearData,
    humanErrorMessage,
    humanForward,
    humanHide,
    humanNavigate,
    humanReload,
    humanSetBounds,
    humanShow,
    humanStatus,
    isHumanStatus,
    normalizeAddress,
    HUMAN_STATUS_EVENT,
    type HumanStatus
  } from "../humanBrowser";
  import { DEFAULT_PREVIEW_PRESET, createBoundsSync, normalizeCanvasRect, viewportSize } from "../preview";
  import type { CanvasBounds } from "../preview";
  import Button from "./Button.svelte";

  let address = $state("");
  let status = $state<HumanStatus | null>(null);
  let error = $state("");
  let confirmingClear = $state(false);
  let stage = $state<HTMLDivElement | null>(null);
  let slot = $state<HTMLDivElement | null>(null);
  let addressInput = $state<HTMLInputElement | null>(null);
  let slotWidth = $state(0);
  let slotHeight = $state(0);
  let unlisten: UnlistenFn | undefined;
  let frame = 0;
  let boundsError = $state("");

  // B18-S06: one bounds send in flight plus one latest pending rectangle.
  // The echoed status is adopted only when no newer navigation state landed
  // meanwhile; failures surface a retry instead of failing silently.
  const boundsSync = createBoundsSync<HumanStatus>({
    send: (bounds) => humanSetBounds(bounds),
    apply: (result) => {
      boundsError = "";
      if (status?.visible === result.visible && status?.url === result.url) {
        status = result;
      }
    },
    onError: () => {
      boundsError = "The browser view could not be resized. Try again.";
    }
  });

  const plannedDestinations = ["Files", "Terminal", "Changes"];

  let visible = $derived(status?.visible ?? false);
  let canGo = $derived(address.trim().length > 0);
  let canBack = $derived(visible && (status?.can_go_back ?? false));
  let canForward = $derived(visible && (status?.can_go_forward ?? false));
  let pageLine = $derived(
    visible && status?.url
      ? `${status.title ?? "Untitled"} — ${status.url}`
      : ""
  );

  $effect(() => {
    if (!stage) {
      return;
    }
    const observer = new ResizeObserver(() => {
      updateSlotSize();
      scheduleBounds();
    });
    observer.observe(stage);
    updateSlotSize();
    scheduleBounds();
    return () => observer.disconnect();
  });

  onMount(() => {
    let disposed = false;
    void (async () => {
      try {
        const stopListening = await listen<unknown>(
          HUMAN_STATUS_EVENT,
          ({ payload }) => {
            if (isHumanStatus(payload)) {
              status = payload;
              if (payload.last_denial) {
                error = humanErrorMessage(payload.last_denial);
              } else if (payload.visible) {
                error = "";
              }
            }
          }
        );
        if (disposed) {
          stopListening();
          return;
        }
        unlisten = stopListening;
      } catch {
        error = "Live browser updates are unavailable; use the controls to refresh.";
      }
      try {
        const snapshot = await humanStatus();
        if (!disposed) {
          status = snapshot;
        }
      } catch {
        // The form still works; failures surface when navigating.
      }
    })();

    window.addEventListener("resize", scheduleBounds);

    return () => {
      disposed = true;
      window.removeEventListener("resize", scheduleBounds);
      unlisten?.();
      cancelBoundsFrame();
      boundsSync.dispose();
    };
  });

  onDestroy(() => {
    void humanHide().catch(() => undefined);
  });

  function focusAddress() {
    addressInput?.focus();
  }

  function updateSlotSize() {
    if (!stage) {
      return;
    }
    const rect = stage.getBoundingClientRect();
    const size = viewportSize(DEFAULT_PREVIEW_PRESET, {
      width: rect.width,
      height: rect.height
    });
    slotWidth = size.width;
    slotHeight = size.height;
  }

  function scheduleBounds() {
    if (frame !== 0) {
      return;
    }
    frame = requestAnimationFrame(() => {
      frame = 0;
      if (visible) {
        boundsSync.schedule(currentBounds());
      }
    });
  }

  function cancelBoundsFrame() {
    if (frame !== 0) {
      cancelAnimationFrame(frame);
      frame = 0;
    }
  }

  function retryBounds() {
    boundsError = "";
    cancelBoundsFrame();
    scheduleBounds();
  }

  function currentBounds(): CanvasBounds | null {
    if (!slot) {
      return null;
    }
    const rect = slot.getBoundingClientRect();
    return normalizeCanvasRect(
      { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
      { width: window.innerWidth, height: window.innerHeight }
    );
  }

  function errorCode(value: unknown): string {
    if (typeof value === "object" && value !== null) {
      const code = (value as Record<string, unknown>).code;
      if (typeof code === "string") {
        return code;
      }
    }
    return "";
  }

  async function onGo() {
    const parsed = normalizeAddress(address);
    if (parsed.length === 0) {
      return;
    }
    error = "";
    try {
      if (status?.visible) {
        status = await humanNavigate(parsed);
      } else {
        const bounds = currentBounds();
        if (!bounds) {
          error = "The browser area is not ready yet.";
          return;
        }
        status = await humanShow(parsed, bounds);
      }
      address = parsed;
    } catch (failure) {
      error = humanErrorMessage(errorCode(failure));
    }
  }

  async function onBack() {
    if (!canBack) {
      return;
    }
    try {
      status = await humanBack();
    } catch (failure) {
      error = humanErrorMessage(errorCode(failure));
    }
  }

  async function onForward() {
    if (!canForward) {
      return;
    }
    try {
      status = await humanForward();
    } catch (failure) {
      error = humanErrorMessage(errorCode(failure));
    }
  }

  async function onClearData() {
    try {
      status = await humanClearData();
      error = "";
    } catch (failure) {
      error = humanErrorMessage(errorCode(failure));
    }
    confirmingClear = false;
  }

  async function onReload() {
    if (!visible) {
      return;
    }
    try {
      status = await humanReload();
    } catch (failure) {
      error = humanErrorMessage(errorCode(failure));
    }
  }
</script>

<div class="browser">
  <form
    class="browser-bar"
    onsubmit={(event) => {
      event.preventDefault();
      void onGo();
    }}
  >
    <label for="browser-address">Address</label>
    <input
      id="browser-address"
      name="browser-address"
      class="br-field browser-field"
      placeholder="example.com"
      autocomplete="off"
      spellcheck="false"
      bind:this={addressInput}
      bind:value={address}
    />
    <Button variant="primary" type="submit" inactive={!canGo} onclick={onGo}>Go</Button>
  </form>

  <div class="browser-controls">
    {#if confirmingClear}
      <span class="browser-confirm">
        This removes cookies, storage, and cache for the BrainRoot browser.
      </span>
      <Button variant="secondary" onclick={() => void onClearData()}>
        Clear everything
      </Button>
      <Button variant="secondary" onclick={() => (confirmingClear = false)}>Cancel</Button>
    {:else}
      <Button variant="secondary" onclick={() => (confirmingClear = true)}>
        Clear browser data
      </Button>
    {/if}
  </div>

  <div class="browser-controls">
    <Button variant="secondary" inactive={!canBack} onclick={() => void onBack()}>
      Back
    </Button>
    <Button variant="secondary" inactive={!canForward} onclick={() => void onForward()}>
      Forward
    </Button>
    <Button variant="secondary" inactive={!visible} onclick={() => void onReload()}>
      Reload
    </Button>
  </div>

  {#if error}
    <p class="browser-error" role="alert">{error}</p>
  {:else if pageLine}
    <p class="browser-page" role="status">{pageLine}</p>
  {/if}

  {#if boundsError}
    <div class="browser-bounds">
      <p class="browser-error" role="alert">{boundsError}</p>
      <Button variant="secondary" onclick={retryBounds}>Retry</Button>
    </div>
  {/if}

  <div class="browser-stage" bind:this={stage}>
    <div
      class="browser-slot"
      bind:this={slot}
      style="width: {slotWidth}px; height: {slotHeight}px"
    >
      {#if !visible}
        <div class="browser-empty">
          <div class="browser-menu" role="group" aria-label="Canvas destinations">
            <button class="browser-menu__item" type="button" onclick={focusAddress}>
              <span class="browser-menu__label">Browser</span>
              <span class="browser-menu__hint">Enter an address to open it here</span>
            </button>
            {#each plannedDestinations as destination (destination)}
              <button
                class="browser-menu__item"
                type="button"
                aria-disabled="true"
                title={`${destination} — planned for a later phase`}
              >
                <span class="browser-menu__label">{destination}</span>
                <span class="browser-menu__hint">Planned</span>
              </button>
            {/each}
          </div>
          <p class="browser-note">
            Separate from your installed browsers; downloads and popups stay blocked.
            Switching Canvas tabs closes this page and in-memory state may be lost.
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .browser {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    height: 100%;
    min-height: 0;
  }

  .browser-bar {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 0.5rem;
  }

  .browser-bar label {
    color: var(--text-subtle);
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .browser-field {
    min-height: 0;
    padding: 0.45rem 0.6rem;
  }

  .browser-controls {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .browser-confirm {
    color: var(--text-subtle);
    font-size: 0.78rem;
  }

  .browser-page,
  .browser-error {
    margin: 0;
    font-size: 0.8rem;
  }

  .browser-page {
    color: var(--text-subtle);
  }

  .browser-bounds {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .browser-error {
    color: var(--text);
  }

  .browser-stage {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .browser-slot {
    position: relative;
    min-width: 1px;
    min-height: 1px;
    border: 1px dashed var(--border);
    border-radius: var(--radius-control);
    background: var(--surface-raised);
    overflow: hidden;
  }

  .browser-empty {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .browser-menu {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 0.15rem;
    padding: 1rem;
    overflow-y: auto;
  }

  .browser-menu__item {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    padding: 0.55rem 0.65rem;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .browser-menu__item:hover {
    background: var(--surface-hover);
  }

  .browser-menu__item[aria-disabled="true"] {
    cursor: default;
    color: var(--text-subtle);
  }

  .browser-menu__item[aria-disabled="true"]:hover {
    background: transparent;
  }

  .browser-menu__label {
    font-size: 0.86rem;
  }

  .browser-menu__hint {
    color: var(--text-subtle);
    font-size: 0.72rem;
  }

  .browser-note {
    margin: 0;
    padding: 0 1rem 0.9rem;
    color: var(--text-subtle);
    font-size: 0.72rem;
    line-height: 1.45;
  }

  @media (max-width: 640px) {
    .browser-bar {
      grid-template-columns: 1fr;
    }
  }
</style>
