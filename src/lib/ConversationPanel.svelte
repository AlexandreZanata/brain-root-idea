<script lang="ts">
  import Badge from "./Badge.svelte";
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import SuggestionItem from "./SuggestionItem.svelte";
  import Turn from "./Turn.svelte";
  import WelcomeCard from "./WelcomeCard.svelte";
  import type { ConversationTurn } from "../conversation";

  let {
    setupMessage = null,
    turns = [],
    statusLabel = "",
    canSend = false,
    isCancellable = false,
    prompt = $bindable(""),
    suggestions = [],
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
    onsubmit: () => void;
    oncancel: () => void;
  } = $props();

  let textarea: HTMLTextAreaElement | undefined;

  function chooseSuggestion(text: string) {
    prompt = text;
    textarea?.focus();
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

  <div class="br-panel__body history" aria-label="Conversation">
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

    {#each turns as turn (turn.id)}
      <Turn {turn} />
    {/each}
  </div>

  <form
    class="composer"
    onsubmit={(event) => {
      event.preventDefault();
      onsubmit();
    }}
  >
    <label for="prompt" class="br-visually-hidden">What do you want to build?</label>
    <textarea
      id="prompt"
      name="prompt"
      rows="3"
      class="br-field"
      placeholder="Describe what you want to build…"
      bind:value={prompt}
      bind:this={textarea}
    ></textarea>
    <div class="composer-bar">
      <span class="br-chip" title="Configured model for MVP-0">glm-5.3-flash</span>
      <span class="composer-state">{statusLabel}</span>
      <div class="actions">
        <Button variant="primary" type="submit" inactive={!canSend}>
          <span>Send</span>
          <Icon name="send" size="sm" />
        </Button>
        <Button variant="secondary" inactive={!isCancellable} onclick={oncancel}>Cancel</Button>
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
    gap: 0.9rem;
    padding: 1rem;
  }

  .suggestions {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .setup {
    margin: 0;
    padding: 0.6rem 1rem;
    border-bottom: 1px solid var(--border);
    color: var(--text);
    font-size: 0.78rem;
  }

  .composer {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 1rem 1rem;
    border-top: 1px solid var(--border);
  }

  .composer-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .composer-state {
    flex: 1;
    text-align: right;
    color: var(--text-muted);
    font-size: 0.72rem;
  }

  .actions {
    display: flex;
    gap: 0.4rem;
  }

  @media (max-width: 1080px) {
    .agent {
      min-height: 20rem;
    }
  }

  @media (max-width: 640px) {
    .composer {
      padding: 0.65rem 0.75rem 0.85rem;
    }

    .composer-bar {
      flex-wrap: wrap;
      row-gap: 0.4rem;
    }

    .composer-state {
      flex-basis: 100%;
      text-align: left;
    }
  }
</style>
