<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import {
    customViewportSize,
    DEFAULT_PREVIEW_PRESET,
    hidePreview,
    isPreviewStatus,
    normalizeCanvasRect,
    parseCommandLine,
    presetLabel,
    PREVIEW_STATUS_EVENT,
    previewReasonMessage,
    previewStatus,
    setPreviewBounds,
    showPreview,
    startPreview,
    stopPreview,
    viewportSize,
    type PreviewPhase,
    type ViewportPresetId
  } from "../preview";
  import Button from "./Button.svelte";

  const LOOPBACK_HOST = "127.0.0.1";
  const PREVIEW_SCHEME = "http";
  const presetIds: ViewportPresetId[] = ["desktop", "tablet", "phone", "custom"];

  let phase = $state<PreviewPhase>("idle");
  let port = $state<number | null>(null);
  let reason = $state<string | null>(null);
  let command = $state("");
  let folder = $state("");
  let errorDetail = $state("");
  let slot = $state<HTMLDivElement | null>(null);
  let stage = $state<HTMLDivElement | null>(null);
  let preset = $state<ViewportPresetId>(DEFAULT_PREVIEW_PRESET);
  let customWidth = $state(390);
  let customHeight = $state(844);
  let available = $state({ width: 0, height: 0 });
  let unlisten: UnlistenFn | undefined;
  let frame = 0;

  let custom = $derived(customViewportSize({ width: customWidth, height: customHeight }));
  let viewport = $derived(viewportSize(preset, available, custom));
  let actualSize = $derived(
    viewport.exact
      ? `${viewport.width} × ${viewport.height}`
      : `${viewport.width} × ${viewport.height} — limited by the Canvas area`
  );

  let busy = $derived(phase === "starting" || phase === "stopping");
  let ready = $derived(phase === "ready");
  let canStart = $derived(command.trim().length > 0 && folder.trim().length > 0 && !busy);
  let canStop = $derived(phase === "starting" || phase === "ready");
  let address = $derived(port === null ? "" : `${PREVIEW_SCHEME}://${LOOPBACK_HOST}:${port}`);
  let statusLine = $derived.by(() => {
    switch (phase) {
      case "starting":
        return "Starting your app…";
      case "ready":
        return "Preview ready";
      case "stopping":
        return "Stopping the preview…";
      case "stopped":
        return "Preview stopped.";
      default:
        return "";
    }
  });
  let failure = $derived(
    phase === "failed"
      ? `${previewReasonMessage(reason)} ${errorDetail}`.trim()
      : ""
  );

  $effect(() => {
    if (!slot) {
      return;
    }
    const observer = new ResizeObserver(scheduleBounds);
    observer.observe(slot);
    scheduleBounds();
    return () => observer.disconnect();
  });

  $effect(() => {
    if (!stage) {
      return;
    }
    const element = stage;
    const observer = new ResizeObserver(() => {
      const rect = element.getBoundingClientRect();
      available = { width: rect.width, height: rect.height };
    });
    observer.observe(element);
    const rect = element.getBoundingClientRect();
    available = { width: rect.width, height: rect.height };
    return () => observer.disconnect();
  });

  onMount(() => {
    let disposed = false;
    void (async () => {
      try {
        const stopListening = await listen<unknown>(
          PREVIEW_STATUS_EVENT,
          ({ payload }) => {
            if (isPreviewStatus(payload)) {
              applyStatus(payload.phase, payload.port, payload.reason);
            }
          }
        );
        if (disposed) {
          stopListening();
          return;
        }
        unlisten = stopListening;
      } catch {
        errorDetail = "Live preview updates are unavailable; use the status read below.";
      }
      try {
        const status = await previewStatus();
        if (!disposed) {
          applyStatus(status.phase, status.port, status.reason);
        }
      } catch {
        // The form still works; failures surface when starting.
      }
    })();

    window.addEventListener("resize", scheduleBounds);

    return () => {
      disposed = true;
      window.removeEventListener("resize", scheduleBounds);
      unlisten?.();
      void hidePreview().catch(() => undefined);
    };
  });

  function applyStatus(
    nextPhase: PreviewPhase,
    nextPort: number | null,
    nextReason: string | null
  ) {
    phase = nextPhase;
    reason = nextReason;
    if (nextPort !== null) {
      port = nextPort;
    }
    if (nextPhase === "ready") {
      void ensureViewVisible();
    }
    if (nextPhase === "idle" || nextPhase === "stopped" || nextPhase === "failed") {
      if (nextPhase !== "failed") {
        port = null;
      }
      void hidePreview().catch(() => undefined);
    }
  }

  function scheduleBounds() {
    if (frame !== 0) {
      return;
    }
    frame = requestAnimationFrame(() => {
      frame = 0;
      if (phase === "ready") {
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
      await setPreviewBounds(bounds);
    } catch {
      // A stale rectangle is corrected on the next resize.
    }
  }

  async function ensureViewVisible() {
    if (port === null) {
      return;
    }
    const bounds = currentBounds();
    if (!bounds) {
      return;
    }
    try {
      await showPreview(port, bounds);
    } catch {
      errorDetail = "The preview area is not ready yet.";
    }
  }

  async function onStart() {
    if (!canStart) {
      return;
    }
    errorDetail = "";
    phase = "starting";
    const parsed = parseCommandLine(command);
    try {
      const startedPort = await startPreview({
        command: parsed.command,
        args: parsed.args,
        cwd: folder.trim()
      });
      port = startedPort;
      await ensureViewVisible();
    } catch (error) {
      phase = "failed";
      reason = typeof error === "object" && error !== null ? String((error as Record<string, unknown>).code ?? "") : "";
      errorDetail = "";
    }
  }

  async function onStop() {
    if (!canStop) {
      return;
    }
    phase = "stopping";
    try {
      await hidePreview();
    } catch {
      // Stopping the server below still releases the owned resources.
    }
    try {
      await stopPreview();
    } catch {
      phase = "failed";
      reason = "preview_stop_failed";
      errorDetail = "";
      return;
    }
    phase = "idle";
    port = null;
    reason = null;
  }

  function onRetry() {
    phase = "idle";
    reason = null;
    errorDetail = "";
  }
</script>

<div class="preview">
  <form
    class="preview-setup"
    onsubmit={(event) => {
      event.preventDefault();
      void onStart();
    }}
  >
    <p class="preview-lede">
      BrainRoot runs one command you choose, on one folder, and shows only the local
      address it starts. Nothing else is started, and it stops when you stop it or
      close BrainRoot. Switching Canvas tabs closes the preview view; the app reloads
      when you return, and the dev server keeps running until you stop it.
    </p>
    <div class="preview-fields">
      <label for="preview-command">Command</label>
      <input
        id="preview-command"
        name="preview-command"
        class="br-field preview-field"
        placeholder="npm run dev"
        autocomplete="off"
        bind:value={command}
      />
      <label for="preview-folder">Project folder</label>
      <input
        id="preview-folder"
        name="preview-folder"
        class="br-field preview-field"
        placeholder="/home/you/project"
        autocomplete="off"
        bind:value={folder}
      />
    </div>
    <div class="preview-actions">
      <Button variant="primary" type="submit" inactive={!canStart} onclick={onStart}>
        Start preview
      </Button>
      <Button variant="secondary" inactive={!canStop} onclick={() => void onStop()}>
        Stop
      </Button>
    </div>
  </form>

  {#if statusLine}
    <p class="preview-status" role="status">{statusLine}{address ? ` — ${address}` : ""}</p>
  {/if}

  {#if failure}
    <p class="preview-error" role="alert">{failure}</p>
    <div class="preview-actions">
      <Button variant="secondary" onclick={onRetry}>Try again</Button>
    </div>
  {/if}

  <div class="preview-presets" role="group" aria-label="Viewport size">
    {#each presetIds as id (id)}
      <Button
        variant="tab"
        current={preset === id}
        inactive={preset !== id}
        onclick={() => {
          preset = id;
        }}
      >
        {presetLabel(id)}
      </Button>
    {/each}
  </div>

  {#if preset === "custom"}
    <div class="preview-custom">
      <label for="preview-width">Width</label>
      <input
        id="preview-width"
        name="preview-width"
        class="br-field preview-field"
        type="number"
        min="240"
        max="1920"
        step="10"
        bind:value={customWidth}
      />
      <label for="preview-height">Height</label>
      <input
        id="preview-height"
        name="preview-height"
        class="br-field preview-field"
        type="number"
        min="240"
        max="1200"
        step="10"
        bind:value={customHeight}
      />
    </div>
  {/if}

  <p class="preview-size">
    {actualSize} · Responsive viewport preview — not device emulation.
  </p>

  <div class="preview-stage" bind:this={stage}>
    <div
      class="preview-slot"
      bind:this={slot}
      style="width: {viewport.width}px; height: {viewport.height}px;"
    >
      {#if !ready}
        <p class="preview-placeholder">
          Your app appears here once the preview is running.
        </p>
      {/if}
    </div>
  </div>
</div>

<style>
  .preview {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    height: 100%;
    min-height: 0;
  }

  .preview-setup {
    display: grid;
    gap: 0.5rem;
  }

  .preview-lede {
    margin: 0;
    color: var(--text-subtle);
    font-size: 0.8rem;
    line-height: 1.45;
  }

  .preview-fields {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: 0.4rem 0.6rem;
  }

  .preview-fields label,
  .preview-custom label {
    color: var(--text-subtle);
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .preview-field {
    min-height: 0;
    padding: 0.45rem 0.6rem;
  }

  .preview-actions {
    display: flex;
    gap: 0.5rem;
  }

  .preview-presets {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .preview-custom {
    display: grid;
    grid-template-columns: auto minmax(0, 6rem) auto minmax(0, 6rem);
    align-items: center;
    gap: 0.4rem 0.6rem;
  }

  .preview-status,
  .preview-size {
    margin: 0;
    color: var(--text-subtle);
    font-size: 0.8rem;
  }

  .preview-error {
    margin: 0;
    color: var(--text);
    font-size: 0.82rem;
  }

  .preview-stage {
    flex: 1;
    display: grid;
    place-items: start center;
    overflow: auto;
    min-height: 8rem;
  }

  .preview-slot {
    position: relative;
    max-width: 100%;
    border: 1px dashed var(--border);
    border-radius: var(--radius-control);
    background: var(--surface-raised);
  }

  .preview-placeholder {
    margin: 0;
    padding: 1rem;
    color: var(--text-subtle);
    font-size: 0.82rem;
  }

  @media (max-width: 640px) {
    .preview-fields,
    .preview-custom {
      grid-template-columns: 1fr;
    }
  }
</style>
