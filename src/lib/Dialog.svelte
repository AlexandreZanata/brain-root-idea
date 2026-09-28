<script lang="ts">
  /**
   * B20-U6 — the single dialog primitive, on the pinned `dialog-v2` shape
   * (@ 34aa427): one centered container on a scrim, an optional header with a
   * close button, a body, and an optional footer row.
   *
   * The pin uses Kobalte's dialog for its modal focus containment. This uses the
   * platform's own `<dialog>` + `showModal()` instead: same containment, same
   * Escape handling, no dependency, and no focus-index overrides at all — the
   * repository's accessibility gate forbids them outright. Escape fires the
   * dialog's `cancel` event; a press on the scrim is a click whose target is the
   * dialog element itself; both route through `requestClose`, which the host
   * answers by unmounting this component. Focus returns to the invoker through
   * `dialogs.ts`'s chain, not here.
   */
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let {
    title,
    description = "",
    size = "normal",
    variant = "default",
    fit = false,
    label,
    onclose,
    footer,
    children
  }: {
    title?: string;
    description?: string;
    /** The pin's sizes: `normal` 480×368, `large` 640×480, `x-large` viewport-capped. */
    size?: "normal" | "large" | "x-large" | "palette";
    /** `settings` swaps the container to `bg-base` like the pin's settings variant. */
    variant?: "default" | "settings";
    /** The pin's `fit`: height driven by content instead of the size's fixed height. */
    fit?: boolean;
    /** Accessible name, used when there is no visible title. */
    label: string;
    onclose: () => void;
    footer?: Snippet;
    children: Snippet;
  } = $props();

  let dialog = $state<HTMLDialogElement | undefined>(undefined);

  onMount(() => {
    // The pin's `onOpenAutoFocus` focuses the first `[autofocus]` element, so
    // the palette's search field gets focus instead of the dialog shell. The
    // marker is a data attribute: the platform's `autofocus` would also run on
    // page load, and Svelte flags it as an accessibility warning.
    dialog?.showModal();
    const target = dialog?.querySelector<HTMLElement>("[data-autofocus]");
    target?.focus();
  });

  function requestClose() {
    onclose();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      requestClose();
    }
  }

  function onClick(event: MouseEvent) {
    if (event.target === dialog) {
      requestClose();
    }
  }
</script>

<dialog
  bind:this={dialog}
  class="br-dialog"
  class:br-dialog--settings={variant === "settings"}
  class:br-dialog--fit={fit}
  data-size={size}
  aria-label={title ? undefined : label}
  aria-labelledby={title ? "br-dialog-title" : undefined}
  aria-describedby={description ? "br-dialog-description" : undefined}
  onkeydown={onKeydown}
  onclick={onClick}
  oncancel={(event) => {
    event.preventDefault();
    requestClose();
  }}
>
  <div class="br-dialog__container">
    {#if title || description}
      <div class="br-dialog__header">
        <div class="br-dialog__title-group">
          {#if title}
            <h2 class="br-dialog__title" id="br-dialog-title">{title}</h2>
          {/if}
          {#if description}
            <p class="br-dialog__description" id="br-dialog-description">{description}</p>
          {/if}
        </div>
        <button type="button" class="br-dialog__close" aria-label="Close" onclick={requestClose}>
          <Icon name="close" size="sm" />
        </button>
      </div>
    {/if}
    <div class="br-dialog__body">
      {@render children()}
    </div>
    {#if footer}
      <div class="br-dialog__footer">
        {@render footer()}
      </div>
    {/if}
  </div>
</dialog>

<style>
  /* The dialog element is the scrim; the container is the pinned box. Sizes
     come from the pin's dialog-v2.css. `palette` is the command palette's
     override: top-anchored so content height can grow without the layer
     jumping. */
  .br-dialog {
    position: fixed;
    inset: 0;
    width: 100%;
    height: 100%;
    max-width: none;
    max-height: none;
    margin: 0;
    padding: 0;
    border: 0;
    background: transparent;
    overflow: hidden;
  }

  .br-dialog::backdrop {
    background: var(--v2-overlay-simple-overlay-scrim);
  }

  .br-dialog__container {
    display: flex;
    flex-direction: column;
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 480px;
    height: 368px;
    max-width: calc(100vw - 24px);
    max-height: calc(100vh - 24px);
    background: var(--v2-background-bg-layer-01);
    box-shadow: var(--v2-elevation-overlay);
    border-radius: 6px;
    overflow: hidden;
  }

  .br-dialog[data-size="large"] .br-dialog__container {
    width: 640px;
    height: 480px;
  }

  .br-dialog[data-size="x-large"] .br-dialog__container {
    width: min(calc(100vw - 32px), 980px);
    height: min(calc(100vh - 92px), 600px);
    max-width: none;
    max-height: none;
  }

  .br-dialog[data-size="palette"] .br-dialog__container {
    top: max(48px, calc((100vh - 480px) / 2));
    transform: translateX(-50%);
    width: min(calc(100vw - 24px), 640px);
    height: auto;
    min-height: 280px;
    max-height: min(calc(100vh - 96px), 480px);
    background: var(--v2-background-bg-base);
    box-shadow: var(--v2-elevation-floating);
    border-radius: 12px;
  }

  .br-dialog--fit .br-dialog__container {
    height: auto;
    max-height: calc(100vh - 48px);
  }

  .br-dialog--settings .br-dialog__container {
    background: var(--v2-background-bg-base);
  }

  .br-dialog__header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    flex-shrink: 0;
    padding: 16px;
  }

  .br-dialog__title-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .br-dialog__title {
    margin: 0;
    font-size: 15px;
    font-weight: 500;
    line-height: 20px;
    letter-spacing: -0.13px;
    color: var(--text);
  }

  .br-dialog__description {
    margin: 0;
    font-size: 13px;
    font-weight: 400;
    line-height: 20px;
    color: var(--text-muted);
  }

  .br-dialog__close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    margin-top: -2px;
    margin-inline-start: auto;
    flex-shrink: 0;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
  }

  .br-dialog__close:hover {
    background: var(--v2-overlay-simple-overlay-hover);
  }

  .br-dialog__body {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .br-dialog__footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex-shrink: 0;
    padding: 16px;
  }

  .br-dialog__close:focus-visible {
    outline: var(--focus-width) solid var(--focus);
    outline-offset: var(--focus-offset);
  }
</style>
