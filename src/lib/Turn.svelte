<script lang="ts">
  import type { ConversationTurn } from "../conversation";

  let { turn }: { turn: ConversationTurn } = $props();
</script>

<article class="turn">
  <p class="br-message-label">You</p>
  <p class="br-message br-message--user">{turn.prompt}</p>
  {#if turn.response.length > 0}
    <p class="br-message-label">BrainRoot</p>
    <p class="br-message">{turn.response}</p>
  {/if}
  {#if turn.status === "cancelled"}
    <p class="br-note">Cancelled.</p>
  {:else if turn.error.length > 0}
    <p class="br-error" role="alert">{turn.error}</p>
    {#if turn.errorCode.length > 0}
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
</style>
