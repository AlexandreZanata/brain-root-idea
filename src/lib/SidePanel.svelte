<script lang="ts">
  /**
   * B20-U7 — the side-panel shell, on the pinned `session-side-panel.tsx`
   * geometry (@ 34aa427): `aside role="region"` on `bg-v2-background-bg-base`,
   * `border-radius: 10px`, `--v2-elevation-raised`, overflow clipped, with a
   * sticky header slot and a content slot. The pin packs file tabs and the
   * review tab into one panel; here the Canvas preset strip picks the content,
   * so this is the shared chrome both render inside.
   */
  import type { Snippet } from "svelte";

  let {
    label,
    title,
    meta,
    children
  }: {
    /** Region name for assistive technology. */
    label: string;
    title?: string;
    /** Header trailing slot (counts, toggles). */
    meta?: Snippet;
    children: Snippet;
  } = $props();
</script>

<aside class="side-panel" role="region" aria-label={label}>
  {#if title || meta}
    <div class="side-panel__head">
      {#if title}
        <h3 class="side-panel__title">{title}</h3>
      {/if}
      {#if meta}
        <div class="side-panel__meta">{@render meta()}</div>
      {/if}
    </div>
  {/if}
  <div class="side-panel__body">
    {@render children()}
  </div>
</aside>

<style>
  .side-panel {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    height: 100%;
    border-radius: 10px;
    background: var(--v2-background-bg-base);
    box-shadow: var(--v2-elevation-raised);
    overflow: hidden;
  }

  .side-panel__head {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
    padding: 12px 16px;
    border-bottom: 1px solid var(--v2-border-border-weak-base);
  }

  .side-panel__title {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    line-height: 20px;
    letter-spacing: -0.04px;
    color: var(--text);
  }

  .side-panel__meta {
    margin-inline-start: auto;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .side-panel__body {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
</style>
