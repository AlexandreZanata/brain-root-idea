<script lang="ts">
  import { onMount } from "svelte";

  let {
    value = $bindable(368),
    min = 280,
    max = 560,
    step = 8,
    label = "Resize chat panel"
  }: {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
  } = $props();

  let input = $state<HTMLInputElement | null>(null);
  let dragging = $state(false);
  let focused = $state(false);
  let frame = 0;
  let pending: number | null = null;
  let startX = 0;
  let startValue = value;

  function clamp(next: number): number {
    if (!Number.isFinite(next)) {
      return min;
    }
    return Math.min(max, Math.max(min, next));
  }

  function applyPending() {
    if (pending === null) {
      return;
    }
    value = clamp(pending);
    pending = null;
  }

  function schedule(next: number) {
    pending = clamp(next);
    if (frame !== 0) {
      return;
    }
    frame = requestAnimationFrame(() => {
      frame = 0;
      applyPending();
    });
  }

  function cancelFrame() {
    if (frame !== 0) {
      cancelAnimationFrame(frame);
      frame = 0;
    }
    pending = null;
  }

  function endDrag(commit: boolean) {
    if (!dragging) {
      return;
    }
    dragging = false;
    if (commit) {
      if (frame !== 0) {
        cancelAnimationFrame(frame);
        frame = 0;
      }
      applyPending();
      return;
    }
    cancelFrame();
    value = clamp(startValue);
  }

  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) {
      return;
    }
    event.preventDefault();
    input?.focus({ preventScroll: true });
    startX = event.clientX;
    startValue = value;
    dragging = true;
    input?.setPointerCapture(event.pointerId);
  }

  function onPointerMove(event: PointerEvent) {
    if (!dragging) {
      return;
    }
    schedule(startValue + (event.clientX - startX));
  }

  function onPointerUp(event: PointerEvent) {
    if (input?.hasPointerCapture(event.pointerId)) {
      input.releasePointerCapture(event.pointerId);
    }
    endDrag(true);
  }

  function onPointerCancel(event: PointerEvent) {
    if (input?.hasPointerCapture(event.pointerId)) {
      input.releasePointerCapture(event.pointerId);
    }
    endDrag(false);
  }

  function onKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape" && dragging) {
      event.preventDefault();
      endDrag(false);
    }
  }

  function onWindowBlur() {
    endDrag(false);
  }

  onMount(() => {
    window.addEventListener("blur", onWindowBlur);
    return () => {
      window.removeEventListener("blur", onWindowBlur);
      cancelFrame();
    };
  });
</script>

<div class="br-resizer" class:dragging class:focused>
  <span class="br-resizer__edge" aria-hidden="true"></span>
  <input
    type="range"
    class="br-resizer__input"
    aria-label={label}
    aria-orientation="vertical"
    {min}
    {max}
    {step}
    bind:this={input}
    bind:value
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerCancel}
    onlostpointercapture={() => endDrag(false)}
    onfocus={() => (focused = true)}
    onblur={() => (focused = false)}
    onkeydown={onKeyDown}
  />
</div>
