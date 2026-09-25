<script lang="ts">
  import Badge from "./Badge.svelte";
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import ModelPicker from "./ModelPicker.svelte";
  import SuggestionItem from "./SuggestionItem.svelte";
  import TextArea from "./TextArea.svelte";
  import Turn from "./Turn.svelte";
  import WelcomeCard from "./WelcomeCard.svelte";
  import { composerKeyAction, isPinnedToBottom } from "../presentation";
  import type {
    AgentMode,
    AgentModelSelection,
    CatalogModel
  } from "../agentHost";
  import type { ConversationTurn } from "../conversation";

  let {
    setupMessage = null,
    turns = [],
    statusLabel = "",
    canSend = false,
    isCancellable = false,
    prompt = $bindable(""),
    suggestions = [],
    models = [],
    selectedModel = null,
    staleModels = false,
    modelDisabled = false,
    agentMode = "build",
    onselectmodel,
    onselectagent,
    onsubmit,
    oncancel
  }: {
    setupMessage?: string | null;
    turns?: ConversationTurn[];
    statusLabel?: string;
    canSend?: boolean;
    isCancellable?: boolean;
    prompt?: string;
    suggestions?: { label: string; prompt: string }[];
    models?: CatalogModel[];
    selectedModel?: AgentModelSelection | null;
    staleModels?: boolean;
    modelDisabled?: boolean;
    agentMode?: AgentMode;
    onselectmodel: (providerId: string, modelId: string) => void;
    onselectagent: (mode: AgentMode) => void;
    onsubmit: () => void;
    oncancel: () => void;
  } = $props();

  let composer: { focus: () => void } | undefined;
  let history: HTMLDivElement | undefined;
  let pinned = $state(true);

  $effect(() => {
    void turns;
    if (history && pinned) {
      history.scrollTop = history.scrollHeight;
    }
  });

  function onHistoryScroll() {
    if (!history) {
      return;
    }
    pinned = isPinnedToBottom(history.scrollTop, history.clientHeight, history.scrollHeight);
  }

  function jumpToLatest() {
    if (history) {
      history.scrollTop = history.scrollHeight;
    }
    pinned = true;
    composer?.focus();
  }

  function onComposerKeydown(event: KeyboardEvent) {
    const action = composerKeyAction(event, event.isComposing);
    if (action === "submit") {
      event.preventDefault();
      pinned = true;
      onsubmit();
    } else if (action === "cancel" && isCancellable) {
      event.preventDefault();
      oncancel();
      composer?.focus();
    }
  }

  function chooseSuggestion(text: string) {
    prompt = text;
    composer?.focus();
  }
</script>

<section class="br-panel agent" aria-labelledby="agent-title">
  <div class="br-panel__head">
    <div>
      <h2 id="agent-title" class="br-panel__title">Build</h2>
      <p class="br-panel__subtitle">Describe a change and watch the model answer.</p>
    </div>
    <Badge>MVP-0</Badge>
  </div>

  {#if setupMessage}
    <p class="setup" role="status">{setupMessage}</p>
  {/if}

  <div
    class="br-panel__body history"
    aria-label="Conversation"
    bind:this={history}
    onscroll={onHistoryScroll}
  >
    {#if turns.length === 0}
      <WelcomeCard />
      <div class="suggestions">
        {#each suggestions as suggestion (suggestion.label)}
          <SuggestionItem
            label={suggestion.label}
            onclick={() => chooseSuggestion(suggestion.prompt)}
          />
        {/each}
      </div>
    {/if}

    {#each turns as turn, index (turn.id)}
      <Turn {turn} isLatest={index === turns.length - 1} />
    {/each}
  </div>

  {#if !pinned && turns.length > 0}
    <div class="jump-row">
      <Button variant="ghost" onclick={jumpToLatest}>Jump to latest</Button>
    </div>
  {/if}

  <form
    class="composer"
    onsubmit={(event) => {
      event.preventDefault();
      pinned = true;
      onsubmit();
    }}
  >
    <!-- composer label <label for="prompt"> is rendered by TextArea with id="prompt" -->
    <TextArea
      id="prompt"
      name="prompt"
      label="What do you want to build?"
      rows={3}
      placeholder="Describe what you want to build…"
      bind:value={prompt}
      bind:this={composer}
      onkeydown={onComposerKeydown}
    />
    <div class="composer-bar">
      <div class="agent-toggle" role="group" aria-label="Agent mode">
        <Button
          variant="secondary"
          title="Plan: read-only exploration, cheaper"
          current={agentMode === "plan"}
          onclick={() => onselectagent("plan")}
        >Plan</Button>
        <Button
          variant="secondary"
          title="Build: edits files"
          current={agentMode === "build"}
          onclick={() => onselectagent("build")}
        >Build</Button>
      </div>
      <ModelPicker
        {models}
        selected={selectedModel}
        stale={staleModels}
        disabled={modelDisabled}
        onselect={onselectmodel}
      />
      <span class="composer-state">{statusLabel}</span>
      <div class="br-btn-group">
        <Button variant="primary" type="submit" inactive={!canSend}>
          <span>Send</span>
          <Icon name="send" size="sm" />
        </Button>
        <Button variant="secondary" inactive={!isCancellable} onclick={() => {
          oncancel();
          composer?.focus();
        }}>Cancel</Button>
      </div>
    </div>
  </form>
</section>

<style>
  .agent {
    height: 100%;
  }

  .history {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
  }

  .suggestions {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .jump-row {
    display: flex;
    justify-content: center;
    padding: 0 var(--space-4) var(--space-2);
  }

  .setup {
    margin: 0;
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border);
    color: var(--text);
    font-size: var(--text-body);
  }

  .composer {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4) var(--space-4);
    border-top: 1px solid var(--border);
  }

  .composer-bar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .agent-toggle {
    display: flex;
    gap: var(--space-1);
  }

  .composer-state {
    flex: 1;
    text-align: right;
    color: var(--text-muted);
    font-size: var(--text-supporting);
  }

  @media (max-width: 1080px) {
    .agent {
      min-height: 20rem;
    }
  }

  @media (max-width: 640px) {
    .composer {
      padding: var(--space-3) var(--space-3) var(--space-3);
    }

    .composer-bar {
      flex-wrap: wrap;
      row-gap: var(--space-2);
    }

    .composer-state {
      flex-basis: 100%;
      text-align: left;
    }
  }
</style>
