<script lang="ts">
  /**
   * B20-U6 — the command palette, on the pinned `dialog-command-palette-v2`
   * shape (@ 34aa427): one search row, category groups, 36 px rows, the active
   * row on `--v2-overlay-simple-overlay-hover`, and the pin's four keys.
   *
   * Rows stay unfocusable like U5's listbox: the search field is the focus
   * owner and points at the active row through `aria-activedescendant`, which
   * is the listbox pattern without a single focus-index override. The only commands listed
   * are the handlers `App.svelte` really wires — a palette entry that opens
   * nothing is a fake control.
   */
  import { tick } from "svelte";
  import Dialog from "./Dialog.svelte";
  import Icon from "./Icon.svelte";
  import {
    PALETTE_EMPTY_LABEL,
    PALETTE_SEARCH_LABEL,
    groups,
    moveActive,
    paletteEntries,
    paletteKeyIntent,
    type PaletteEntry
  } from "../dialogs";

  let {
    commands,
    sessions,
    onrun,
    onclose
  }: {
    commands: PaletteEntry[];
    sessions: PaletteEntry[];
    onrun: (id: string) => void;
    onclose: () => void;
  } = $props();

  let query = $state("");
  let active = $state(0);

  const entries = $derived(paletteEntries(commands, sessions, query));
  const grouped = $derived(groups(entries));
  const activeEntry = $derived(entries[active] ?? null);

  /** Stable DOM ids per entry so `aria-activedescendant` never derives from text. */
  const rowIds = $derived.by(() => {
    const ids = new Map<string, string>();
    entries.forEach((entry, index) => ids.set(entry.id, `palette-option-${index}`));
    return ids;
  });

  /** The pin resets the active row to the first result whenever the query changes. */
  function setQuery(value: string) {
    query = value;
    active = 0;
  }

  function scrollActive() {
    const id = activeEntry ? rowIds.get(activeEntry.id) : undefined;
    if (!id) {
      return;
    }
    // The pin scrolls on the next frame; nothing is retained past it.
    requestAnimationFrame(() => {
      document.getElementById(id)?.scrollIntoView({ block: "nearest" });
    });
  }

  async function move(delta: 1 | -1) {
    if (entries.length === 0) {
      return;
    }
    active = moveActive(entries.length, active, delta);
    await tick();
    scrollActive();
  }

  /** The host owns the close policy: some commands hand off to another surface. */
  function run(entry: PaletteEntry | null) {
    if (!entry) {
      return;
    }
    onrun(entry.id);
  }

  function onSearchKeydown(event: KeyboardEvent) {
    const intent = paletteKeyIntent(event);
    switch (intent.kind) {
      case "none":
        return;
      case "move":
        event.preventDefault();
        move(intent.delta);
        return;
      case "select":
        // The pin still prevents default with zero rows: Enter never submits
        // whatever sits behind the palette.
        event.preventDefault();
        run(activeEntry);
        return;
      case "close":
        event.preventDefault();
        onclose();
        return;
    }
  }
</script>

<Dialog label="Commands" size="palette" {onclose}>
  <div class="palette">
    <div class="search-row">
      <Icon name="magnifier" size="sm" />
      <label for="palette-search" class="br-visually-hidden">{PALETTE_SEARCH_LABEL}</label>
      <input
        id="palette-search"
        role="combobox"
        aria-controls="palette-list"
        aria-expanded="true"
        aria-autocomplete="list"
        aria-activedescendant={activeEntry ? rowIds.get(activeEntry.id) : undefined}
        aria-label={PALETTE_SEARCH_LABEL}
        placeholder="Search commands…"
        autocomplete="off"
        spellcheck="false"
        data-autofocus
        value={query}
        oninput={(event) => setQuery((event.currentTarget as HTMLInputElement).value)}
        onkeydown={onSearchKeydown}
      />
    </div>
    <div class="scroll">
      {#if entries.length > 0}
        <div id="palette-list" class="results" role="listbox" aria-label="Commands">
          {#each grouped as group (group.category)}
            <div class="group">
              <p class="group-title" aria-hidden="true">{group.category}</p>
              {#each group.entries as entry (entry.id)}
                <!-- svelte-ignore a11y_interactive_supports_focus, a11y_click_events_have_key_events -->
                <div
                  class="row"
                  class:row--active={entry.id === activeEntry?.id}
                  id={rowIds.get(entry.id)}
                  role="option"
                  aria-selected={entry.id === activeEntry?.id}
                  onpointermove={() => {
                    const index = entries.findIndex((candidate) => candidate.id === entry.id);
                    if (index >= 0) {
                      active = index;
                    }
                  }}
                  onclick={() => run(entry)}
                >
                  <span class="row-main">
                    <span class="row-title">{entry.title}</span>
                    {#if entry.description}
                      <span class="row-description">{entry.description}</span>
                    {/if}
                  </span>
                  {#if entry.keybind}
                    <span class="row-meta">{entry.keybind}</span>
                  {/if}
                </div>
              {/each}
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty">{PALETTE_EMPTY_LABEL}</p>
      {/if}
    </div>
  </div>
</Dialog>

<style>
  .palette {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
  }

  .search-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
    height: 36px;
    margin: 6px;
    padding: 0 12px;
    border-radius: 6px;
    background: color-mix(in srgb, var(--v2-background-bg-layer-02) 60%, transparent);
    color: var(--text-muted);
    transition:
      background-color 120ms ease-in-out,
      box-shadow 120ms ease-in-out;
  }

  .search-row:focus-within {
    background: var(--v2-background-bg-layer-02);
  }

  .search-row input {
    min-width: 0;
    flex: 1;
    border: 0;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    font-weight: 500;
    line-height: 16px;
  }

  .search-row input::placeholder {
    color: var(--text-subtle);
  }

  .scroll {
    min-height: 0;
    flex: 1;
    overflow-y: auto;
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 6px 6px 8px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .group-title {
    margin: 6px 0;
    padding: 0 12px;
    font-size: 13px;
    font-weight: 400;
    line-height: 16px;
    color: var(--text-muted);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    height: 36px;
    flex-shrink: 0;
    padding: 0 12px;
    border-radius: 6px;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    scroll-margin: 6px 0;
  }

  .row--active {
    background: var(--v2-overlay-simple-overlay-hover);
  }

  .row-main {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .row-title {
    overflow: hidden;
    font-size: 13px;
    font-weight: 500;
    line-height: 16px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-description,
  .row-meta {
    overflow: hidden;
    font-size: 13px;
    font-weight: 400;
    line-height: 16px;
    color: var(--text-muted);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-meta {
    flex-shrink: 0;
  }

  .empty {
    display: grid;
    min-height: 120px;
    place-items: center;
    margin: 0;
    font-size: 13px;
    font-weight: 400;
    line-height: 16px;
    color: var(--text-muted);
  }

  .search-row input:focus-visible {
    outline: var(--focus-width) solid var(--focus);
    outline-offset: var(--focus-offset);
  }

  @media (prefers-reduced-motion: reduce) {
    .search-row {
      transition: none;
    }
  }

  @media (max-width: 640px) {
    .row-main {
      flex-direction: column;
      align-items: flex-start;
      gap: 1px;
    }
  }
</style>
