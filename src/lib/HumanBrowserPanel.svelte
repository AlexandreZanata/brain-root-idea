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
  import { DEFAULT_PREVIEW_PRESET, normalizeCanvasRect, viewportSize } from "../preview";
  import Button from "./Button.svelte";

  let address = $state("");
  let status = $state<HumanStatus | null>(null);
  let error = $state("");
  let confirmingClear = $state(false);
  let stage = $state<HTMLDivElement | null>(null);
  let slot = $state<HTMLDivElement | null>(null);
  let slotWidth = $state(0);
  let slotHeight = $state(0);
  let unlisten: UnlistenFn | undefined;
  let frame = 0;

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
    };
  });

  onDestroy(() => {
    void humanHide().catch(() => undefined);
  });

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
        void syncBounds();
      }
    });
  }

  function currentBounds(): [number, number, number, number] | null {
    if (!slot) {
      return null;
    }
    const rect = slot.getBoundingClientRect();
    return normalizeCanvasRect(
      { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
      { width: window.innerWidth, height: window.innerHeight }
    );
  }

  async function syncBounds() {
    const bounds = currentBounds();
    if (!bounds) {
      return;
    }
    try {
      status = await humanSetBounds(bounds);
    } catch {
      // A stale rectangle is corrected on the next resize.
    }
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

  <div class="browser-stage" bind:this={stage}>
    <div
      class="browser-slot"
      bind:this={slot}
      style="width: {slotWidth}px; height: {slotHeight}px"
    >
      {#if !visible}
        <p class="browser-placeholder">
          Enter a web address to open it here. This browser is separate from your
          installed browsers and blocked from opening downloads or popups.
          Switching Canvas tabs closes this page; in-memory state may be lost.
        </p>
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

  .browser-placeholder {
    margin: 0;
    padding: 1rem;
    color: var(--text-subtle);
    font-size: 0.82rem;
    line-height: 1.5;
  }

  @media (max-width: 640px) {
    .browser-bar {
      grid-template-columns: 1fr;
    }
  }
</style>
