<script lang="ts">
  /**
   * B20-U6 — the error screen, on the pinned `pages/error.tsx` shape (@ 34aa427):
   * a centered text block with a title, a product-language description, the
   * technical detail behind a disclosure, and an action row.
   *
   * The pin's actions are Restart, Export logs, Report, and Check for updates.
   * None of those capabilities exist here, so the only action offered is one
   * that does something — start the sidecar — plus Close. The disclosure is the
   * platform's own `details` element, so it opens and closes by keyboard with
   * no scripting and no invented ARIA.
   */
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";

  let {
    title,
    description,
    technical,
    canStart = false,
    onstart,
    onclose
  }: {
    title: string;
    description: string;
    /** The raw code and message, kept behind the disclosure and out of the way. */
    technical: string;
    canStart: boolean;
    onstart: () => void;
    onclose: () => void;
  } = $props();
</script>

<Dialog label={title} size="normal" fit {onclose}>
  <div class="error">
    <div class="error-copy">
      <h2 class="error-title">{title}</h2>
      <p class="error-description">{description}</p>
    </div>
    {#if technical}
      <details class="error-details">
        <summary>Technical details</summary>
        <pre class="error-technical">{technical}</pre>
      </details>
    {/if}
    <div class="error-actions">
      {#if canStart}
        <Button variant="primary" onclick={onstart}>Start the sidecar</Button>
      {/if}
      <Button variant="ghost" onclick={onclose}>Close</Button>
    </div>
  </div>
</Dialog>

<style>
  .error {
    display: flex;
    flex-direction: column;
    gap: 36px;
    padding: 32px 40px 40px;
  }

  .error-copy {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .error-title {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    line-height: 20px;
    color: var(--text);
  }

  .error-description {
    margin: 0;
    font-size: 13px;
    font-weight: 400;
    line-height: 20px;
    color: var(--text-muted);
  }

  .error-details summary {
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    line-height: 20px;
    color: var(--text-muted);
  }

  .error-technical {
    max-height: 24rem;
    margin: 8px 0 0;
    padding: 8px;
    border-radius: 6px;
    background: var(--v2-background-bg-layer-01);
    overflow: auto;
    font-family: ui-monospace, "JetBrains Mono", monospace;
    font-size: 12px;
    line-height: 18px;
    color: var(--text-muted);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .error-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    max-width: 16rem;
  }

  .error-details summary:focus-visible {
    outline: var(--focus-width) solid var(--focus);
    outline-offset: var(--focus-offset);
  }
</style>
