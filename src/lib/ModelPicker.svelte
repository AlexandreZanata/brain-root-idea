<script lang="ts">
  /**
   * B20-U5: the model selector, rebuilt on the pinned `ModelSelectorPopoverV2`
   * shape — a 284 px layer with a 28 px search row, a hairline divider, sticky
   * provider labels, name-sorted groups, and the pin's search matcher.
   *
   * It replaces the native select element this file used to render, so the undefined
   * `--radius` that U2 and U4 both flagged is gone with the element rather than
   * patched around. What the pin has and this does not is recorded in
   * `docs/specs/b20-opencode-parity-u05.md`: provider icons (a 282 kB sprite),
   * a hardcoded popularity ranking, latest/free tags, and the manage-models row.
   */
  import Icon from "./Icon.svelte";
  import { modelKey, type AgentModelSelection, type CatalogModel } from "../agentHost";
  import {
    EMPTY_CATALOG_LABEL,
    NO_MATCH_LABEL,
    PICKER_EMPTY_CHIP,
    PICKER_STALE_CHIP,
    countLabel,
    findOption,
    groupModels,
    initialActive,
    moveActive,
    optionKeys,
    pickerKeyIntent,
    type ModelOption
  } from "../modelPicker";

  let {
    models = [],
    selected = null,
    stale = false,
    inactive = false,
    onselect
  }: {
    models?: CatalogModel[];
    selected?: AgentModelSelection | null;
    stale?: boolean;
    inactive?: boolean;
    onselect: (providerId: string, modelId: string) => void;
  } = $props();

  const LIST_ID = "model-picker-list";
  const DETAIL_ID = "model-picker-detail";

  let open = $state(false);
  let search = $state("");
  let active = $state<string | null>(null);
  let control = $state<HTMLDivElement | undefined>(undefined);
  let trigger = $state<HTMLButtonElement | undefined>(undefined);
  let searchField = $state<HTMLInputElement | undefined>(undefined);

  const currentKey = $derived(selected ? modelKey(selected) : null);
  const groups = $derived(groupModels(models, search));
  const keys = $derived(optionKeys(groups));
  const total = $derived(models.length);
  const matched = $derived(keys.length);
  const filtering = $derived(search.trim().length > 0);
  const activeOption = $derived(findOption(groups, active));
  const currentModel = $derived(
    models.find((entry) => modelKey(entry) === currentKey) ?? null
  );
  /** Stable DOM ids per option, so `aria-activedescendant` never has to be derived from the key text. */
  const rowIds = $derived.by(() => {
    const ids = new Map<string, string>();
    let index = 0;
    for (const group of groups) {
      for (const option of group.options) {
        ids.set(option.key, `model-option-${index}`);
        index += 1;
      }
    }
    return ids;
  });

  const triggerText = $derived(
    total === 0
      ? PICKER_EMPTY_CHIP
      : (currentModel?.model_name ?? selected?.model_id ?? "Choose a model")
  );
  const triggerLabel = $derived(
    stale ? "Model (stale catalog)" : total === 0 ? "Model (none available)" : "Model"
  );
  const triggerTitle = $derived(
    stale
      ? "Catalog offline — prices may be outdated"
      : total === 0
        ? EMPTY_CATALOG_LABEL
        : triggerText
  );
  const emptyLabel = $derived(total === 0 ? EMPTY_CATALOG_LABEL : NO_MATCH_LABEL);

  $effect(() => {
    if (!open) {
      return;
    }
    // The pinned view defers focus and the scroll-into-view by a macrotask and
    // a frame, so the rows exist before either runs.
    const frame = requestAnimationFrame(() => {
      searchField?.focus();
      scrollTo(active);
    });
    return () => cancelAnimationFrame(frame);
  });

  $effect(() => {
    if (!open) {
      return;
    }
    // `focusout` only fires when focus really moves, so a press on a
    // non-focusable surface — the conversation history, the Canvas, empty
    // chrome — would leave the layer floating. The pin dismisses on
    // `pointerDownOutside`; this is that, installed on open and removed on
    // close so nothing outlives the popover.
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (target instanceof Node && control?.contains(target)) {
        return;
      }
      closePicker(false);
    };
    window.addEventListener("pointerdown", onPointerDown, true);
    return () => window.removeEventListener("pointerdown", onPointerDown, true);
  });

  function scrollTo(key: string | null) {
    if (!key) {
      return;
    }
    document.getElementById(rowIds.get(key) ?? "")?.scrollIntoView({ block: "nearest" });
  }

  /** Also the `/model` command's entry point, so focus lands in the search field. */
  export function openPicker() {
    if (inactive) {
      return;
    }
    search = "";
    active = initialActive(optionKeys(groupModels(models, "")), currentKey);
    open = true;
  }

  function closePicker(restoreFocus: boolean) {
    open = false;
    search = "";
    active = null;
    if (restoreFocus) {
      trigger?.focus();
    }
  }

  function toggle() {
    if (open) {
      closePicker(true);
      return;
    }
    openPicker();
  }

  function choose(option: ModelOption) {
    closePicker(true);
    onselect(option.providerId, option.modelId);
  }

  function selectActive() {
    const option = findOption(groups, active);
    if (option) {
      choose(option);
    }
  }

  function move(delta: 1 | -1) {
    const next = moveActive(keys, active, delta);
    active = next;
    scrollTo(next);
  }

  function setSearch(value: string) {
    search = value;
    active = initialActive(optionKeys(groupModels(models, value)), null);
  }

  function onSearchKeydown(event: KeyboardEvent) {
    const intent = pickerKeyIntent(event);
    switch (intent.kind) {
      case "none":
        return;
      case "close":
        event.preventDefault();
        closePicker(true);
        return;
      case "select":
        event.preventDefault();
        selectActive();
        return;
      case "move":
        event.preventDefault();
        move(intent.delta);
        return;
      case "edge": {
        event.preventDefault();
        const next =
          intent.edge === "first" ? (keys[0] ?? null) : (keys[keys.length - 1] ?? null);
        active = next;
        scrollTo(next);
        return;
      }
    }
  }

  function onControlFocusOut(event: FocusEvent) {
    const next = event.relatedTarget;
    if (next instanceof Node && control?.contains(next)) {
      return;
    }
    if (open) {
      closePicker(false);
    }
  }
</script>

<div class="picker" bind:this={control} onfocusout={onControlFocusOut}>
  <button
    type="button"
    class="trigger"
    bind:this={trigger}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-controls={open ? LIST_ID : undefined}
    aria-disabled={inactive || undefined}
    aria-label={triggerLabel}
    title={triggerTitle}
    onclick={toggle}
  >
    <span class="trigger-label">{triggerText}</span>
    <Icon name="chevron-down" size="sm" />
  </button>

  {#if stale}
    <span class="chip" role="status" title="Catalog offline — prices may be outdated"
      >{PICKER_STALE_CHIP}</span
    >
  {/if}

  {#if open}
    <!-- Pressing inside the layer must not blur the search field. -->
    <div
      class="popover"
      role="presentation"
      onmousedown={(event) => event.preventDefault()}
    >
      <div class="search-row">
        <Icon name="magnifier" size="sm" />
        <label for="model-picker-search" class="br-visually-hidden">Search models</label>
        <input
          id="model-picker-search"
          bind:this={searchField}
          role="combobox"
          aria-controls={LIST_ID}
          aria-expanded="true"
          aria-autocomplete="list"
          aria-activedescendant={active ? rowIds.get(active) : undefined}
          aria-label="Search models"
          placeholder="Search models…"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          value={search}
          oninput={(event) => setSearch((event.currentTarget as HTMLInputElement).value)}
          onkeydown={onSearchKeydown}
        />
        {#if filtering}
          <button
            type="button"
            class="clear"
            aria-label="Clear the search"
            onmousedown={(event) => event.preventDefault()}
            onclick={() => {
              setSearch("");
              searchField?.focus();
            }}
          >
            <Icon name="close" size="sm" />
          </button>
        {/if}
      </div>
      <div class="divider"></div>
      <span class="br-visually-hidden" aria-live="polite"
        >{countLabel(matched, total, filtering)}</span
      >
      <div class="scroll">
        {#if keys.length > 0}
          <div id={LIST_ID} class="list" role="listbox" aria-label="Models">
            {#each groups as group (group.providerId)}
              <div class="group" role="group" aria-label={group.providerName}>
                <p class="group-label" aria-hidden="true">{group.providerName}</p>
                {#each group.options as option (option.key)}
                  <!-- Focus stays in the search field, which points at the
                       active row; that is the listbox pattern without a
                       focusable row, and the keyboard path is the search
                       field's own key handling. -->
                  <!-- svelte-ignore a11y_interactive_supports_focus, a11y_click_events_have_key_events -->
                  <div
                    class="option"
                    id={rowIds.get(option.key)}
                    role="option"
                    aria-selected={option.key === currentKey}
                    aria-describedby={option.key === active ? DETAIL_ID : undefined}
                    data-option-key={option.key}
                    onpointermove={() => (active = option.key)}
                    onclick={() => choose(option)}
                  >
                    <span class="option-label">{option.label}</span>
                    <span class="option-meta">{option.meta}</span>
                  </div>
                {/each}
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty">{emptyLabel}</p>
        {/if}
      </div>
      {#if activeOption}
        <p class="detail" id={DETAIL_ID} role="tooltip">{activeOption.detail}</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    max-width: 100%;
  }

  .trigger {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    min-width: 0;
    max-width: 100%;
    height: 28px;
    padding: 0 4px 0 8px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    font-weight: 440;
    line-height: 20px;
    cursor: pointer;
    transition: background-color 150ms ease;
  }

  .trigger:hover {
    background: var(--surface-hover);
  }

  .trigger[aria-disabled="true"] {
    opacity: 0.5;
  }

  .trigger-label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip {
    flex-shrink: 0;
    color: var(--text-muted);
    font-size: 13px;
    line-height: 20px;
    white-space: nowrap;
  }

  .popover {
    position: absolute;
    right: 0;
    bottom: calc(100% + 6px);
    z-index: 40;
    display: flex;
    flex-direction: column;
    width: 284px;
    max-width: calc(100vw - 48px);
    padding: 0;
    border-radius: var(--radius-control);
    background: var(--v2-background-bg-layer-01);
    box-shadow: var(--v2-elevation-floating);
  }

  .search-row {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 10px 0 12px;
    color: var(--text-muted);
  }

  .search-row input {
    min-width: 0;
    flex: 1;
    height: 28px;
    border: 0;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    font-weight: 440;
    line-height: 20px;
  }

  .search-row input::placeholder {
    color: var(--text-subtle);
  }

  .clear {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
  }

  .clear:hover {
    background: var(--surface-hover);
  }

  .divider {
    height: 1px;
    background: var(--v2-border-border-muted);
  }

  .scroll {
    max-height: 220px;
    overflow-y: auto;
    padding: 2px 2px 0;
  }

  .group {
    display: flex;
    flex-direction: column;
  }

  .group-label {
    position: sticky;
    top: 0;
    z-index: 10;
    margin: 0;
    padding: 4px 12px;
    background: var(--v2-background-bg-layer-01);
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 600;
    line-height: 20px;
  }

  .option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    line-height: 20px;
    cursor: pointer;
  }

  .option[aria-selected="true"] {
    color: var(--accent);
  }

  .option:hover {
    background: var(--surface-hover);
  }

  .option-label {
    min-width: 0;
    flex: 1 1 auto;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .option-meta {
    flex-shrink: 0;
    color: var(--text-muted);
    font-size: 13px;
    white-space: nowrap;
  }

  .empty {
    display: flex;
    align-items: center;
    height: 48px;
    margin: 0;
    padding: 0 12px;
    color: var(--text-subtle);
    font-size: 13px;
    line-height: 20px;
  }

  .detail {
    margin: 0;
    padding: 6px 12px;
    border-top: 1px solid var(--v2-border-border-muted);
    color: var(--text-muted);
    font-size: 13px;
    line-height: 20px;
  }

  .trigger:focus-visible,
  .clear:focus-visible,
  .search-row input:focus-visible {
    outline: var(--focus-width) solid var(--focus);
    outline-offset: var(--focus-offset);
  }

  @media (prefers-reduced-motion: reduce) {
    .trigger {
      transition: none;
    }
  }
</style>
