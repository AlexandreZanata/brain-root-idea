<script lang="ts">
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
    onnew
  }: {
    tabs?: SessionTab[];
    activeId?: number;
    streamingId?: number | null;
    onselect: (id: number) => void;
    onclose: (id: number) => void;
    onnew: () => void;
  } = $props();
</script>

<div class="tabs" role="tablist" aria-label="Sessions">
  {#each tabs as tab (tab.id)}
    {@const active = tab.id === activeId}
    {@const streaming = tab.id === streamingId}
    <div class="tab" class:active role="tab" aria-selected={active}>
      <button
        class="tab-name"
        aria-current={active ? "true" : undefined}
        onclick={() => onselect(tab.id)}
        title={tab.title}
      >
        <span class="tab-title">{tab.title}</span>
        {#if streaming}
          <span class="tab-live" aria-label="streaming">●</span>
        {/if}
      </button>
      {#if tabs.length > 1}
        <button
          class="tab-close"
          aria-label={`Close ${tab.title}`}
          onclick={() => onclose(tab.id)}
        >×</button>
      {/if}
    </div>
  {/each}
  <button class="tab-new" onclick={onnew} aria-label="New session" title="New session">+</button>
</div>

<style>
  .tabs {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    min-width: 0;
    overflow-x: auto;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    max-width: 12rem;
    padding: var(--space-1) var(--space-2);
    border: 1px solid transparent;
    border-radius: var(--radius);
    color: var(--text-muted);
  }

  .tab.active {
    border-color: var(--border);
    background: var(--surface);
    color: var(--text);
  }

  .tab-name {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    min-width: 0;
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    cursor: pointer;
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

  .tab-close,
  .tab-new {
    padding: 0 var(--space-1);
    border: 0;
    background: none;
    color: var(--text-muted);
    font: inherit;
    cursor: pointer;
    border-radius: var(--radius);
  }

  .tab-close:hover,
  .tab-new:hover,
  .tab-name:focus-visible,
  .tab-close:focus-visible,
  .tab-new:focus-visible {
    color: var(--text);
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
