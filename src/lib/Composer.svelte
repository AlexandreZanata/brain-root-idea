<script lang="ts">
  /**
   * B20-U4: the composer, rebuilt on the pinned OpenCode v2 shape.
   *
   * Geometry comes from `packages/session-ui/src/v2/components/prompt-input/index.tsx`
   * at the pin: a rounded `bg-base` form with the raised elevation, a 60→180 px
   * editor with 16/16/8 px padding at 13 px/20 px type, and a 44 px control row
   * whose submit is a 28 px icon button. Behavior comes from that package's
   * `machine.ts`, ported into `src/composer.ts` so it can be tested without a DOM.
   *
   * Two deliberate divergences are recorded in `docs/specs/b20-opencode-parity-u04.md`:
   * the model control is still the existing native select until U5 replaces it,
   * and the cost-profile switch is BrainRoot's own control with no pinned
   * counterpart.
   */
  import Icon from "./Icon.svelte";
  import ModelPicker from "./ModelPicker.svelte";
  import TextArea from "./TextArea.svelte";
  import { tick } from "svelte";
  import { composerKeyAction } from "../presentation";
  import {
    POPOVER_CLOSED,
    applyCommand,
    commandById,
    commandMenuPopover,
    composerKeyDown,
    openCommandMenu,
    openContext,
    popoverEmptyLabel,
    popoverForValue,
    popoverOpen,
    popoverSuggestions,
    type ComposerCommand,
    type ComposerKey,
    type ComposerPopover
  } from "../composer";
  import type { AgentMode, AgentModelSelection, CatalogModel, CostProfile } from "../agentHost";

  let {
    prompt = $bindable(""),
    canSend = false,
    isCancellable = false,
    statusLabel = "",
    models = [],
    selectedModel = null,
    staleModels = false,
    modelInactive = false,
    agentMode = "build",
    profile = null,
    onselectmodel,
    onselectagent,
    onselectprofile,
    onsubmit,
    oncancel
  }: {
    prompt?: string;
    canSend?: boolean;
    isCancellable?: boolean;
    statusLabel?: string;
    models?: CatalogModel[];
    selectedModel?: AgentModelSelection | null;
    staleModels?: boolean;
    modelInactive?: boolean;
    agentMode?: AgentMode;
    profile?: CostProfile | null;
    onselectmodel: (providerId: string, modelId: string) => void;
    onselectagent: (mode: AgentMode) => void;
    onselectprofile: (profile: CostProfile) => void;
    onsubmit: () => void;
    oncancel: () => void;
  } = $props();

  const costProfiles: { id: CostProfile; label: string; title: string }[] = [
    { id: "fast", label: "Fast", title: "Plan + cheapest model" },
    { id: "balanced", label: "Balanced", title: "Build + cheapest roomy model" },
    { id: "max", label: "Max", title: "Build + frontier-priced model" }
  ];

  type MenuKind = "add" | "cost" | "agent";

  const LIST_ID = "composer-suggestions";

  let editor = $state<{ focus: () => void; focusEnd: () => void } | undefined>(undefined);
  let editorWrap = $state<HTMLDivElement | undefined>(undefined);
  let popoverEl = $state<HTMLDivElement | undefined>(undefined);
  let modelWrap = $state<HTMLDivElement | undefined>(undefined);
  let searchField = $state<HTMLInputElement | undefined>(undefined);
  let popover = $state<ComposerPopover>(POPOVER_CLOSED);
  let openMenu = $state<MenuKind | null>(null);

  /** The panel asks for focus back after it scrolls the history to the latest turn. */
  export function focus() {
    editor?.focus();
  }

  const stopping = $derived(isCancellable);
  const suggestions = $derived(popoverSuggestions(popover));
  const suggestionIds = $derived(suggestions.map((item) => item.id));
  const activeId = $derived(popover.kind === "closed" ? null : popover.activeId);
  const listVisible = $derived(popoverOpen(popover) && !(popover.kind === "command" && popover.menu));
  const menuVisible = $derived(popover.kind === "command" && popover.menu);
  const costProfile = $derived(costProfiles.find((item) => item.id === profile) ?? null);
  const agentLabel = $derived(agentMode === "plan" ? "Plan" : "Build");
  const agentHint = $derived(
    agentMode === "plan" ? "Plan: explore without changing files" : "Build: let the agent change files"
  );

  function optionId(id: string): string {
    return `composer-suggestion-${id}`;
  }

  function keyOf(event: KeyboardEvent): ComposerKey {
    return {
      key: event.key,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      shiftKey: event.shiftKey,
      isComposing: event.isComposing
    };
  }

  $effect(() => {
    // The pinned menu autofocuses its search field on the next frame.
    if (!menuVisible) {
      return;
    }
    requestAnimationFrame(() => searchField?.focus());
  });

  function submit() {
    popover = POPOVER_CLOSED;
    openMenu = null;
    onsubmit();
  }

  function onFormSubmit(event: SubmitEvent) {
    event.preventDefault();
    submit();
  }

  function toggleMenu(kind: MenuKind) {
    openMenu = openMenu === kind ? null : kind;
  }

  function onMenuFocusOut(event: FocusEvent) {
    const wrap = event.currentTarget as HTMLElement | null;
    const next = event.relatedTarget;
    if (next instanceof Node && wrap?.contains(next)) {
      return;
    }
    openMenu = null;
  }

  function onMenuKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") {
      return;
    }
    event.preventDefault();
    const wrap = event.currentTarget as HTMLElement;
    openMenu = null;
    // The trigger is the first button in the disclosure, so it is still the
    // natural place to return focus before the menu items are removed.
    wrap.querySelector("button")?.focus();
  }

  function addCommands() {
    const next = openCommandMenu(prompt);
    prompt = next.draft;
    popover = next.popover;
    openMenu = null;
    // The menu variant focuses its own search field through the effect above;
    // the inline variant keeps the editor focused with the caret after the `/`.
    if (!(next.popover.kind === "command" && next.popover.menu)) {
      void focusEditorEnd();
    }
  }

  function addContext() {
    const next = openContext(prompt);
    prompt = next.draft;
    popover = next.popover;
    openMenu = null;
    void focusEditorEnd();
  }

  async function focusEditorEnd() {
    await tick();
    editor?.focusEnd();
  }

  function focusModel() {
    const control = modelWrap?.querySelector("select");
    if (!control) {
      return;
    }
    control.focus();
    try {
      control.showPicker?.();
    } catch {
      // Focus already moved, which is the part that always works.
    }
  }

  function runCommand(command: ComposerCommand) {
    switch (command.action) {
      case "agent.cycle":
        onselectagent(agentMode === "plan" ? "build" : "plan");
        editor?.focus();
        break;
      case "agent.plan":
        onselectagent("plan");
        editor?.focus();
        break;
      case "agent.build":
        onselectagent("build");
        editor?.focus();
        break;
      case "model.focus":
        focusModel();
        break;
    }
  }

  function choose(id: string, menu: boolean) {
    const command = commandById(id);
    popover = POPOVER_CLOSED;
    if (!command) {
      return;
    }
    prompt = applyCommand(prompt, menu, `/${command.trigger}`);
    runCommand(command);
  }

  function onEditorKeydown(event: KeyboardEvent) {
    const outcome = composerKeyDown(popover, suggestionIds, keyOf(event));
    if (outcome.handled) {
      event.preventDefault();
      if (outcome.selectedId) {
        choose(outcome.selectedId, menuVisible);
        return;
      }
      popover = outcome.popover;
      if (outcome.refocusEditor) {
        editor?.focus();
      }
      return;
    }
    const action = composerKeyAction(event, event.isComposing);
    if (action === "submit") {
      event.preventDefault();
      submit();
    } else if (action === "cancel" && isCancellable) {
      event.preventDefault();
      oncancel();
      editor?.focus();
    }
  }

  function onEditorInput(event: Event) {
    const field = event.currentTarget as HTMLTextAreaElement | null;
    if (!field) {
      return;
    }
    popover = popoverForValue(popover, field.value, field.selectionStart ?? field.value.length);
  }

  function onEditorFocusOut(event: FocusEvent) {
    const next = event.relatedTarget;
    if (next instanceof Node && (editorWrap?.contains(next) || popoverEl?.contains(next))) {
      return;
    }
    popover = POPOVER_CLOSED;
  }

  function onSearchInput(event: Event) {
    const value = (event.currentTarget as HTMLInputElement | null)?.value ?? "";
    popover = commandMenuPopover(value);
  }

  function onSearchKeydown(event: KeyboardEvent) {
    const outcome = composerKeyDown(popover, suggestionIds, keyOf(event));
    if (!outcome.handled) {
      return;
    }
    event.preventDefault();
    if (outcome.selectedId) {
      choose(outcome.selectedId, true);
      return;
    }
    popover = outcome.popover;
    if (outcome.popover.kind === "closed") {
      editor?.focus();
    }
  }
</script>

<form class="composer" onsubmit={onFormSubmit}>
  {#if popoverOpen(popover)}
    <!-- The shell only anchors and paints: pressing inside it must not blur the
         editor, so the pointer handler cancels the default and nothing more. -->
    <div
      class="popover"
      role="presentation"
      data-popover={popover.kind}
      bind:this={popoverEl}
      onmousedown={(event) => event.preventDefault()}
    >
      {#if menuVisible}
        <div class="popover-search">
          <label for="composer-command-search" class="br-visually-hidden">Commands</label>
          <input
            id="composer-command-search"
            aria-label="Commands"
            aria-autocomplete="list"
            aria-expanded="true"
            aria-controls={LIST_ID}
            aria-activedescendant={activeId ? optionId(activeId) : undefined}
            placeholder="/"
            value={popover.kind === "command" ? popover.query : ""}
            bind:this={searchField}
            oninput={onSearchInput}
            onkeydown={onSearchKeydown}
            onmousedown={(event) => event.stopPropagation()}
          />
        </div>
      {/if}
      <div id={LIST_ID} class="popover-body">
        {#if suggestions.length > 0}
          <div class="popover-list" role="listbox" aria-label="Suggestions">
            {#each suggestions as item (item.id)}
              <!-- Focus stays in the editor, which points at the active row; that
                   is the listbox pattern without a focusable row, and the keyboard
                   path is the editor's own key handling, so the pointer handler
                   needs no twin. -->
              <!-- svelte-ignore a11y_interactive_supports_focus, a11y_click_events_have_key_events -->
              <div
                class="popover-item"
                id={optionId(item.id)}
                role="option"
                aria-selected={item.id === activeId}
                data-suggestion-id={item.id}
                onpointermove={() => {
                  if (popover.kind !== "closed") {
                    popover = { ...popover, activeId: item.id };
                  }
                }}
                onclick={() => choose(item.id, false)}
              >
                <span class="popover-label">{item.label}</span>
                <span class="popover-description">{item.description}</span>
              </div>
            {/each}
          </div>
        {:else}
          <p class="popover-empty">{popoverEmptyLabel(popover)}</p>
        {/if}
      </div>
    </div>
  {/if}

  <div class="editor" bind:this={editorWrap} onfocusout={onEditorFocusOut}>
    <!-- The bound prompt label, <label for="prompt">, is rendered by TextArea. -->
    <TextArea
      id="prompt"
      name="prompt"
      label="What do you want to build?"
      placeholder={agentMode === "plan"
        ? "Describe what to explore… Type / for commands"
        : "Describe what you want to build… Type / for commands"}
      bind:value={prompt}
      bind:this={editor}
      role="combobox"
      expanded={listVisible}
      controls={listVisible ? LIST_ID : undefined}
      activeDescendant={listVisible && activeId ? optionId(activeId) : undefined}
      autocomplete={listVisible ? "list" : undefined}
      onkeydown={onEditorKeydown}
      oninput={onEditorInput}
    />
  </div>

  <div class="controls">
    <div class="menu" role="presentation" onfocusout={onMenuFocusOut} onkeydown={onMenuKeydown}>
      <button
        type="button"
        class="icon-btn"
        aria-haspopup="menu"
        aria-expanded={openMenu === "add"}
        aria-label="Add"
        onclick={() => toggleMenu("add")}
      >
        <Icon name="plus" size="sm" />
      </button>
      {#if openMenu === "add"}
        <div class="menu-list" role="menu" aria-label="Add">
          <button type="button" class="menu-item" role="menuitem" onclick={addCommands}>
            <span class="menu-item-label">Commands</span>
            <span class="menu-hint">/</span>
          </button>
          <button type="button" class="menu-item" role="menuitem" onclick={addContext}>
            <span class="menu-item-label">Context</span>
            <span class="menu-hint">@</span>
          </button>
        </div>
      {/if}
    </div>

    <div class="menu" role="presentation" onfocusout={onMenuFocusOut} onkeydown={onMenuKeydown}>
      <button
        type="button"
        class="select-btn"
        aria-haspopup="menu"
        aria-expanded={openMenu === "cost"}
        aria-label="Cost profile"
        title={costProfile?.title ?? "Choose a cost profile"}
        onclick={() => toggleMenu("cost")}
      >
        <span class="select-label">{costProfile?.label ?? "Cost"}</span>
        <Icon name="chevron-down" size="sm" />
      </button>
      {#if openMenu === "cost"}
        <div class="menu-list" role="menu" aria-label="Cost profile">
          {#each costProfiles as item (item.id)}
            <button
              type="button"
              class="menu-item"
              role="menuitemradio"
              aria-checked={profile === item.id}
              title={item.title}
              onclick={() => {
                openMenu = null;
                onselectprofile(item.id);
              }}
            >
              <span class="menu-item-label">{item.label}</span>
              {#if profile === item.id}<span class="menu-check" aria-hidden="true">✓</span>{/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="menu" role="presentation" onfocusout={onMenuFocusOut} onkeydown={onMenuKeydown}>
      <button
        type="button"
        class="select-btn"
        aria-haspopup="menu"
        aria-expanded={openMenu === "agent"}
        aria-label="Agent mode"
        title={agentHint}
        onclick={() => toggleMenu("agent")}
      >
        <span class="select-label">{agentLabel}</span>
        <Icon name="chevron-down" size="sm" />
      </button>
      {#if openMenu === "agent"}
        <div class="menu-list" role="menu" aria-label="Agent mode">
          <button
            type="button"
            class="menu-item"
            role="menuitemradio"
            aria-checked={agentMode === "plan"}
            title="Plan: explore without changing files"
            onclick={() => {
              openMenu = null;
              onselectagent("plan");
            }}
          >
            <span class="menu-item-label">Plan</span>
            {#if agentMode === "plan"}<span class="menu-check" aria-hidden="true">✓</span>{/if}
          </button>
          <button
            type="button"
            class="menu-item"
            role="menuitemradio"
            aria-checked={agentMode === "build"}
            title="Build: let the agent change files"
            onclick={() => {
              openMenu = null;
              onselectagent("build");
            }}
          >
            <span class="menu-item-label">Build</span>
            {#if agentMode === "build"}<span class="menu-check" aria-hidden="true">✓</span>{/if}
          </button>
        </div>
      {/if}
    </div>

    <div class="model-control" bind:this={modelWrap}>
      <ModelPicker
        {models}
        selected={selectedModel}
        stale={staleModels}
        inactive={modelInactive}
        onselect={onselectmodel}
      />
      <span class="model-chevron" aria-hidden="true"><Icon name="chevron-down" size="sm" /></span>
    </div>

    <span class="status">{statusLabel}</span>

    <button
      type="button"
      class="submit"
      aria-disabled={stopping ? undefined : canSend ? undefined : "true"}
      aria-label={stopping ? "Stop" : "Send"}
      title={stopping ? "Stop" : "Send"}
      onclick={() => {
        if (stopping) {
          oncancel();
          editor?.focus();
          return;
        }
        if (!canSend) {
          return;
        }
        submit();
      }}
    >
      <Icon name={stopping ? "stop" : "arrow-up"} size="sm" />
    </button>
  </div>
</form>

<style>
  .composer {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 100%;
    border-radius: var(--radius-surface);
    background: var(--v2-background-bg-base);
    box-shadow: var(--v2-elevation-raised);
  }

  .editor {
    min-height: 60px;
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
    min-height: 44px;
    padding: 0 8px;
  }

  .menu {
    position: relative;
    display: flex;
  }

  .icon-btn,
  .select-btn,
  .submit {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 0;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    font-weight: 440;
    line-height: 20px;
    cursor: pointer;
    transition: background-color 150ms ease, opacity 150ms ease;
  }

  .icon-btn {
    width: 28px;
    height: 28px;
  }

  .select-btn {
    gap: 2px;
    height: 28px;
    max-width: 96px;
    padding: 0 4px 0 8px;
  }

  .select-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-transform: capitalize;
  }

  .icon-btn:hover,
  .select-btn:hover {
    background: var(--surface-hover);
  }

  .menu-list {
    position: absolute;
    bottom: 34px;
    left: 0;
    z-index: 40;
    display: flex;
    flex-direction: column;
    min-width: 180px;
    padding: 8px;
    border-radius: var(--radius-surface);
    background: var(--v2-background-bg-base);
    box-shadow: var(--v2-elevation-raised);
  }

  .menu-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 32px;
    padding: 4px 8px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    font-weight: 440;
    line-height: 20px;
    text-align: start;
    cursor: pointer;
  }

  .menu-item:hover {
    background: var(--surface-hover);
  }

  .menu-check,
  .menu-hint {
    color: var(--text-muted);
  }

  .model-control {
    position: relative;
    display: flex;
    align-items: center;
    min-width: 0;
    max-width: 140px;
  }

  /* U5 replaces this native control with the searchable v2 selector. Until
     then the composer styles its own model control into the pinned trigger
     geometry rather than editing ModelPicker.svelte, which belongs to U5. */
  .model-control :global(select) {
    appearance: none;
    width: 100%;
    max-width: 140px;
    min-width: 0;
    height: 28px;
    padding: 0 20px 0 8px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    font-weight: 440;
    line-height: 20px;
    text-overflow: ellipsis;
  }

  /* ModelPicker's empty-catalog chip sits in the same slot as its select, so it
     takes the same trigger geometry instead of its default boxed chip. */
  .model-control :global(.br-chip) {
    display: inline-flex;
    align-items: center;
    height: 28px;
    max-width: 140px;
    padding: 0 20px 0 8px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 440;
    line-height: 20px;
    white-space: nowrap;
  }

  .model-chevron {
    position: absolute;
    right: 4px;
    display: inline-flex;
    color: var(--text);
    pointer-events: none;
  }

  .status {
    /* A zero basis keeps the submit button on the same line at the panel's
       narrowest width; the status grows into whatever is left. */
    flex: 1 1 0;
    min-width: 0;
    overflow: hidden;
    color: var(--text-muted);
    font-size: 13px;
    line-height: 20px;
    text-align: end;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .submit {
    width: 28px;
    height: 28px;
    padding: 6px;
    margin-left: auto;
    background-color: var(--v2-background-bg-contrast);
    background-image: linear-gradient(
      180deg,
      var(--v2-alpha-light-20) 0%,
      var(--v2-alpha-light-0) 100%
    );
    box-shadow: var(--v2-elevation-button-contrast);
    color: var(--v2-icon-icon-muted);
  }

  .submit[aria-disabled="true"] {
    opacity: 0.5;
  }

  .popover {
    position: absolute;
    inset-inline: 0;
    top: -8px;
    z-index: 40;
    display: flex;
    flex-direction: column;
    max-height: 320px;
    overflow: auto;
    padding: 8px;
    border-radius: var(--radius-surface);
    background: var(--v2-background-bg-base);
    box-shadow: var(--v2-elevation-raised);
    transform: translateY(-100%);
  }

  .popover-search {
    padding: 4px 8px;
  }

  .popover-search input {
    width: 100%;
    border: 0;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    line-height: 20px;
  }

  .popover-list {
    display: flex;
    flex-direction: column;
  }

  .popover-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-radius: var(--radius-control);
    color: var(--text);
    font-size: 13px;
    line-height: 20px;
    cursor: pointer;
  }

  .popover-item[aria-selected="true"] {
    background: var(--surface-hover);
  }

  .popover-label {
    flex-shrink: 0;
  }

  .popover-description,
  .popover-empty {
    margin: 0;
    min-width: 0;
    overflow: hidden;
    color: var(--text-muted);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .popover-empty {
    padding: 4px 8px;
    font-size: 13px;
    line-height: 20px;
    white-space: normal;
  }

  .icon-btn:focus-visible,
  .select-btn:focus-visible,
  .submit:focus-visible,
  .menu-item:focus-visible,
  .model-control :global(select:focus-visible),
  .popover-search input:focus-visible {
    outline: var(--focus-width) solid var(--focus);
    outline-offset: var(--focus-offset);
  }

  @media (prefers-reduced-motion: reduce) {
    .icon-btn,
    .select-btn,
    .submit {
      transition: none;
    }
  }
</style>
