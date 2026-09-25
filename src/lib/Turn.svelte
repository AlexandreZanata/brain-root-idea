<script lang="ts">
  import { presentTurn, responsePreview, shouldCollapseTurn } from "../presentation";
  import type { ConversationTurn } from "../conversation";

  let {
    turn,
    isLatest = true,
    cost = null
  }: { turn: ConversationTurn; isLatest?: boolean; cost?: string | null } = $props();

  const presentation = $derived(presentTurn(turn));
  const collapseCandidate = $derived(shouldCollapseTurn(turn, isLatest));
  let expanded = $state(false);
  const showFull = $derived(!collapseCandidate || expanded);
  const responseId = $derived(`turn-${turn.id}-response`);
</script>

<article class="turn">
  <div class="turn-head">
    <p class="br-message-label">You</p>
    <span class="turn-status turn-status--{presentation.status}">{presentation.label}</span>
  </div>
  <p class="br-message br-message--user">{turn.prompt}</p>
  {#if turn.response.length > 0}
    <p class="br-message-label">BrainRoot</p>
    {#if showFull}
      <p class="br-message" id={responseId}>{turn.response}</p>
      {#if collapseCandidate}
        <button
          class="turn-toggle"
          type="button"
          aria-expanded="true"
          aria-controls={responseId}
          onclick={() => (expanded = false)}
        >
          Show less
        </button>
      {/if}
    {:else}
      <p class="br-message" id={responseId}>{responsePreview(turn.response)}</p>
      <button
        class="turn-toggle"
        type="button"
        aria-expanded="false"
        aria-controls={responseId}
        onclick={() => (expanded = true)}
      >
        Show full answer
      </button>
    {/if}
  {/if}
  {#if cost}
    <p class="turn-cost">{cost}</p>
  {/if}
  {#if turn.status === "cancelled"}
    <p class="br-note">Cancelled.</p>
  {:else if turn.error.length > 0}    <p class="br-error" role="alert">{turn.error}</p>
    {#if presentation.showTechnicalDetail}
      <details class="br-details">
        <summary>Technical details</summary>
        <code>{turn.errorCode}</code>
      </details>
    {/if}
  {/if}
</article>

<style>
  .turn {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding-bottom: 0.9rem;
    border-bottom: 1px solid var(--border);
  }

  .turn:last-child {
    border-bottom: 0;
    padding-bottom: 0;
  }

  .turn-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .turn-status {
    color: var(--text-subtle);
    font-size: 0.68rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .turn-status--failed {
    color: var(--text);
  }

  .turn-toggle {
    align-self: flex-start;
    padding: 0;
    border: 0;
    background: none;
    color: var(--text-muted);
    font: inherit;
    font-size: 0.72rem;
    text-decoration: underline;
    cursor: pointer;
  }

  .turn-cost {
    margin: 0;
    color: var(--text-subtle);
    font-size: 0.72rem;
  }
</style>
