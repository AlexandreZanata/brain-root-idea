<script lang="ts">
  /**
   * B20-U7 — the file tabs, on the pinned `file-tabs.tsx` +
   * `session-sortable-tab-v2.tsx` chrome (@ 34aa427): a tab carries the file
   * name (italic while the tab is temporary), a close button revealed on hover
   * and pinned on the active tab, middle-click closes, double-click pins a
   * temporary tab. The strip scrolls horizontally on vertical wheel, exactly
   * the pin's `file-tab-scroll.ts` rule.
   *
   * There is no workspace listing (the gap #133 recorded), so the tab list is
   * empty today and the content region shows the honest empty state. When a
   * workspace exists this component needs no change — it already renders what
   * it is given, and nothing here fabricates a path or a filename.
   */
  import type { Snippet } from "svelte";
  import EmptyState from "./EmptyState.svelte";
  import Icon from "./Icon.svelte";
  import {
    nextTabListScrollLeft,
    shouldCloseFileTab,
    tabStripWheelDelta,
    type FileTab
  } from "../panels";

  let {
    tabs,
    onclosetab,
    onpintab,
    children
  }: {
    tabs: FileTab[];
    onclosetab: (id: string) => void;
    onpintab: (id: string) => void;
    children?: Snippet;
  } = $props();

  let strip = $state<HTMLDivElement | undefined>(undefined);
  let prevScrollWidth = 0;

  // Vertical wheel scrolls the strip by 50 px per notch (the pin's rule). The
  // listener lives on the strip only and is removed the moment the panel goes
  // away (B20-U7-T03).
  $effect(() => {
    const el = strip;
    if (!el) {
      return;
    }
    const onWheel = (event: WheelEvent) => {
      const delta = tabStripWheelDelta({ deltaY: event.deltaY, deltaX: event.deltaX });
      if (delta === null) {
        return;
      }
      event.preventDefault();
      el.scrollLeft += delta;
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  });

  // The pin chases the newest tab with a MutationObserver plus a frame; tab
  // changes arrive as props here, so the chase runs from reactivity instead —
  // same result, nothing watching the DOM. Runs the pin's `nextTabListScrollLeft`.
  $effect(() => {
    const el = strip;
    const count = tabs.length;
    if (!el || count === 0) {
      return;
    }
    const left = nextTabListScrollLeft({
      prevScrollWidth,
      scrollWidth: el.scrollWidth,
      clientWidth: el.clientWidth,
      prevContextOpen: false,
      contextOpen: false
    });
    prevScrollWidth = el.scrollWidth;
    if (left !== undefined) {
      el.scrollTo({ left, behavior: "smooth" });
    }
  });

  function onAuxClick(event: MouseEvent, id: string) {
    if (shouldCloseFileTab({ button: event.button, targetIsCloseButton: false })) {
      onclosetab(id);
    }
  }
</script>

{#if tabs.length === 0}
  <EmptyState
    icon="files"
    title="No files open"
    description="File tabs open the files a project has. BrainRoot has no workspace listing yet; tabs arrive with projects in MVP-1."
  />
{:else}
  <div class="strip" bind:this={strip} role="group" aria-label="Open files">
    {#each tabs as tab (tab.id)}
      <div class="tab-wrap">
        <button
          type="button"
          class="tab"
          class:tab--temporary={tab.temporary}
          aria-label={tab.name}
          onauxclick={(event) => onAuxClick(event, tab.id)}
          ondblclick={() => onpintab(tab.id)}
        >
          <Icon name="files" size="sm" />
          <span class="tab__name">{tab.name}</span>
        </button>
        <button
          type="button"
          class="tab__close"
          aria-label={`Close ${tab.name}`}
          onclick={() => onclosetab(tab.id)}
        >
          <Icon name="close" size="sm" />
        </button>
      </div>
    {/each}
  </div>
  {#if children}
    <div class="content">
      {@render children()}
    </div>
  {/if}
{/if}

<style>
  .strip {
    display: flex;
    align-items: stretch;
    flex-shrink: 0;
    height: 48px;
    overflow-x: auto;
    scrollbar-width: none;
    border-bottom: 1px solid var(--v2-border-border-weak-base);
  }

  .strip::-webkit-scrollbar {
    display: none;
  }

  .tab-wrap {
    position: relative;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    max-width: 280px;
    border-inline-end: 1px solid var(--v2-border-border-weak-base);
    background: var(--v2-background-bg-base);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    padding: 14px 12px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    font: inherit;
    font-size: 14px;
    font-weight: 500;
    line-height: 20px;
    cursor: pointer;
  }

  .tab__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tab--temporary .tab__name {
    font-style: italic;
  }

  .tab-wrap:hover .tab {
    color: var(--text);
  }

  .tab__close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    margin-inline-end: 8px;
    padding: 0;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
  }

  .tab-wrap:hover .tab__close,
  .tab:focus-visible + .tab__close {
    opacity: 1;
  }

  .tab__close:hover {
    background: var(--v2-overlay-simple-overlay-hover);
    color: var(--text);
  }

  .tab:focus-visible,
  .tab__close:focus-visible {
    outline: var(--focus-width) solid var(--focus);
    outline-offset: var(--focus-offset);
  }

  .content {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 16px;
  }

  @media (prefers-reduced-motion: reduce) {
    .tab__close {
      transition: none;
    }
  }
</style>
