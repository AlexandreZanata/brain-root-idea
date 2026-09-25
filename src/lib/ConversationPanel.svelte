<script lang="ts">
  import Badge from "./Badge.svelte";
  import Button from "./Button.svelte";
  import Composer from "./Composer.svelte";
  import SuggestionItem from "./SuggestionItem.svelte";
  import Turn from "./Turn.svelte";
  import WelcomeCard from "./WelcomeCard.svelte";
  import { isPinnedToBottom } from "../presentation";
  import type {
    AgentMode,
    AgentModelSelection,
    CatalogModel,
    CostProfile
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
    modelInactive = false,
    agentMode = "build",
    profile = null,
    costs = {},
    costScope = "",
    onselectmodel,
    onselectagent,
    onselectprofile,
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
    modelInactive?: boolean;
    agentMode?: AgentMode;
    profile?: CostProfile | null;
    costs?: Record<string, string>;
    costScope?: string | number;
    onselectmodel: (providerId: string, modelId: string) => void;
    onselectagent: (mode: AgentMode) => void;
    onselectprofile: (profile: CostProfile) => void;
    onsubmit: () => void;
    oncancel: () => void;
  } = $props();

  /**
   * B20-U4 moved the composer into `Composer.svelte`, including its Enter,
   * Shift+Enter and Escape handling. This panel keeps the conversation history,
   * the scroll pin, and the focus helpers that call back into the composer.
   */
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
      <Turn {turn} isLatest={index === turns.length - 1} cost={costs[`${costScope}:${turn.id}`] ?? null} />
    {/each}
  </div>

  {#if !pinned && turns.length > 0}
    <div class="jump-row">
      <Button variant="ghost" onclick={jumpToLatest}>Jump to latest</Button>
    </div>
  {/if}

  <div class="composer-host">
    <Composer
      bind:this={composer}
      bind:prompt
      {canSend}
      {isCancellable}
      {statusLabel}
      {models}
      {selectedModel}
      {staleModels}
      {modelInactive}
      {agentMode}
      {profile}
      {onselectmodel}
      {onselectagent}
      {onselectprofile}
      onsubmit={() => {
        pinned = true;
        onsubmit();
      }}
      {oncancel}
    />
  </div>
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

  .composer-host {
    padding: var(--space-3) var(--space-4) var(--space-4);
    border-top: 1px solid var(--border);
  }

  @media (max-width: 1080px) {
    .agent {
      min-height: 20rem;
    }
  }

  @media (max-width: 640px) {
    .composer-host {
      padding: var(--space-2) var(--space-2);
    }
  }
</style>
