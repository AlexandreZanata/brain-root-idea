<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import {
    CONVERSATION_CONTRACT_VERSION,
    appendChunk,
    beginTurn,
    cancelTurn,
    credentialSetupMessage,
    isConversationEnvelope,
    settleTurn,
    type ConversationEnvelope,
    type ConversationState,
    type ConversationTurn,
    type CredentialStatus
  } from "./conversation";
  import { requestHealth } from "./health";

  type HealthState = "checking" | "ready" | "failed";

  let healthState = $state<HealthState>("checking");
  let detail = $state("Waiting for the core health result.");
  let conversationState = $state<ConversationState>("empty");
  let credentialStatus = $state<CredentialStatus | null>(null);
  let credentialDetail = $state("");
  let listenerReady = $state(false);
  let prompt = $state("");
  let turns: ConversationTurn[] = $state([]);
  let nextTurnId = 1;
  let activeTurnId: number | null = null;

  let setupMessage = $derived(
    credentialStatus === null
      ? credentialDetail
      : credentialSetupMessage(credentialStatus)
  );
  let isBusy = $derived(
    conversationState === "sending" ||
      conversationState === "streaming" ||
      conversationState === "cancelling"
  );
  let isCancellable = $derived(
    conversationState === "sending" || conversationState === "streaming"
  );
  let canSend = $derived(
    healthState === "ready" &&
      listenerReady &&
      credentialStatus === "configured" &&
      !isBusy &&
      prompt.trim().length > 0
  );

  onMount(() => {
    let disposed = false;
    let unlisten: UnlistenFn | undefined;

    void (async () => {
      try {
        unlisten = await listen<unknown>("conversation_event", ({ payload }) => {
          onConversationEvent(payload);
        });
        if (disposed) {
          unlisten();
          return;
        }
        listenerReady = true;
      } catch {
        detail = "The conversation channel could not be opened.";
      }

      try {
        await requestHealth();
        if (!disposed) {
          healthState = "ready";
          detail = "The core health contract responded normally.";
          conversationState = "ready";
        }
      } catch (error) {
        if (!disposed) {
          healthState = "failed";
          detail = error instanceof Error ? error.message : "The core health request failed.";
        }
      }

      try {
        const status = await invoke<CredentialStatus>("provider_status");
        if (!disposed) {
          credentialStatus = status;
        }
      } catch {
        if (!disposed) {
          credentialDetail = "The credential status could not be read. Restart BrainRoot and try again.";
        }
      }
    })();

    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  async function onPromptSubmit(event: SubmitEvent) {
    event.preventDefault();
    const message = prompt.trim();
    if (!canSend || message.length === 0) {
      return;
    }

    const turnId = nextTurnId;
    nextTurnId += 1;
    activeTurnId = turnId;
    turns = beginTurn(turns, turnId, message);
    prompt = "";
    conversationState = "sending";

    try {
      await invoke("conversation_send", {
        request: {
          contractVersion: CONVERSATION_CONTRACT_VERSION,
          message
        }
      });
    } catch (error) {
      failActive(errorMessage(error), errorCode(error));
    }
  }

  function onConversationEvent(payload: unknown) {
    if (!isConversationEnvelope(payload)) {
      failActive("The core returned an unexpected conversation event.");
      return;
    }
    applyConversationEvent(payload);
  }

  function applyConversationEvent(envelope: ConversationEnvelope) {
    const turnId = activeTurnId;
    if (turnId === null) {
      return;
    }

    switch (envelope.event.type) {
      case "started":
        conversationState = "streaming";
        break;
      case "text_chunk":
        conversationState = "streaming";
        turns = appendChunk(turns, turnId, envelope.event.text);
        break;
      case "completed":
        turns = settleTurn(turns, turnId, "succeeded");
        conversationState = "succeeded";
        activeTurnId = null;
        break;
      case "failed":
        failActive(envelope.event.error.message, envelope.event.error.code);
        break;
      case "cancelled":
        turns = cancelTurn(turns, turnId);
        conversationState = "ready";
        activeTurnId = null;
        break;
    }
  }

  async function onCancel() {
    if (!isCancellable) {
      return;
    }
    conversationState = "cancelling";
    try {
      await invoke("conversation_cancel");
    } catch (error) {
      if (conversationState === "cancelling") {
        failActive(errorMessage(error), errorCode(error));
      }
    }
  }

  function failActive(message: string, code = "") {
    if (activeTurnId !== null) {
      turns = settleTurn(turns, activeTurnId, "failed", message, code);
    }
    activeTurnId = null;
    conversationState = "failed";
  }

  function errorMessage(error: unknown): string {
    if (typeof error === "object" && error !== null) {
      const message = (error as Record<string, unknown>).message;
      if (typeof message === "string" && message.length > 0) {
        return message;
      }
    }
    return error instanceof Error ? error.message : "The request could not be started.";
  }

  function errorCode(error: unknown): string {
    if (typeof error === "object" && error !== null) {
      const code = (error as Record<string, unknown>).code;
      if (typeof code === "string") {
        return code;
      }
    }
    return "";
  }
</script>

<main class="app">
  <header class="chrome">
    <h1>BrainRoot</h1>
    <div class="core">
      <p class="core-status" aria-live="polite">
        {#if healthState === "checking"}
          Checking the core…
        {:else if healthState === "ready"}
          Ready
        {:else}
          Not ready
        {/if}
      </p>
      <p class="core-detail">{detail}</p>
    </div>
  </header>

  <div class="workspace">
    <section class="agent" aria-labelledby="agent-title">
      <h2 id="agent-title">Build</h2>
      <p class="conversation-status" aria-live="polite">
        {#if conversationState === "sending"}
          Starting…
        {:else if conversationState === "streaming"}
          Building…
        {:else if conversationState === "cancelling"}
          Cancelling…
        {:else if conversationState === "succeeded"}
          Done
        {:else if conversationState === "failed"}
          Needs attention
        {:else if setupMessage}
          Needs setup
        {:else}
          Ready for a request
        {/if}
      </p>

      {#if setupMessage}
        <p class="setup" role="status">{setupMessage}</p>
      {/if}

      <div class="conversation-history" aria-live="polite" aria-label="Conversation">
        {#each turns as turn (turn.id)}
          <article class="turn">
            <p class="message-label">You</p>
            <p class="message user-message">{turn.prompt}</p>
            {#if turn.response.length > 0}
              <p class="message-label">BrainRoot</p>
              <p class="message assistant-message">{turn.response}</p>
            {/if}
            {#if turn.status === "cancelled"}
              <p class="turn-cancelled">Cancelled.</p>
            {:else if turn.error.length > 0}
              <p class="turn-error" role="alert">{turn.error}</p>
              {#if turn.errorCode.length > 0}
                <details class="technical">
                  <summary>Technical details</summary>
                  <code>{turn.errorCode}</code>
                </details>
              {/if}
            {/if}
          </article>
        {/each}
      </div>

      <form class="prompt" onsubmit={onPromptSubmit}>
        <label for="prompt">What do you want to build?</label>
        <textarea id="prompt" name="prompt" rows="4" bind:value={prompt}></textarea>
        <div class="actions">
          <button class="send" type="submit" disabled={!canSend}>Send</button>
          <button
            class="cancel"
            type="button"
            disabled={!isCancellable}
            onclick={onCancel}
          >
            Cancel
          </button>
        </div>
      </form>
    </section>

    <section class="canvas" aria-labelledby="canvas-title">
      <h2 id="canvas-title">Companion Canvas</h2>
      <p>Preview comes in MVP-1.</p>
    </section>
  </div>
</main>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
  }

  :global(body) {
    background: #0f1115;
    color: #e8eef5;
    font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  }

  .app {
    display: grid;
    grid-template-rows: auto 1fr;
    gap: 1rem;
    min-height: 100vh;
    padding: 1.25rem;
    box-sizing: border-box;
  }

  .chrome {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.25rem 1rem;
  }

  h1 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  h2 {
    margin: 0;
    font-size: 0.9375rem;
    font-weight: 600;
  }

  .core {
    text-align: right;
  }

  .core-status {
    margin: 0;
    font-size: 0.875rem;
  }

  .core-detail {
    margin: 0;
    font-size: 0.8125rem;
    color: #9aa7b4;
  }

  .workspace {
    display: grid;
    grid-template-columns: minmax(18rem, 30%) minmax(0, 1fr);
    gap: 1rem;
    min-height: 0;
  }

  .agent,
  .canvas {
    border: 1px solid #2a323c;
    border-radius: 0.75rem;
    padding: 1rem;
    background: #171c22;
  }

  .agent {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-height: 0;
  }

  .conversation-status {
    margin: 0;
    color: #9aa7b4;
    font-size: 0.8125rem;
  }

  .conversation-history {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 0.75rem;
    min-height: 8rem;
    overflow: auto;
  }

  .turn {
    display: grid;
    gap: 0.25rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid #2a323c;
  }

  .message-label,
  .message,
  .turn-error,
  .turn-cancelled {
    margin: 0;
  }

  .message-label {
    color: #9aa7b4;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .message,
  .turn-error,
  .turn-cancelled {
    overflow-wrap: anywhere;
    font-size: 0.875rem;
    line-height: 1.45;
    white-space: pre-wrap;
  }

  .user-message {
    color: #cdd9e5;
  }

  .turn-error {
    color: #ffb4ab;
  }

  .turn-cancelled {
    color: #9aa7b4;
  }

  .setup {
    margin: 0;
    color: #cdd9e5;
    font-size: 0.875rem;
  }

  .technical {
    margin: 0;
    color: #9aa7b4;
    font-size: 0.8125rem;
  }

  .technical summary {
    cursor: pointer;
  }

  .technical code {
    color: #cdd9e5;
  }

  .prompt {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .prompt label {
    font-size: 0.875rem;
    color: #cdd9e5;
  }

  textarea {
    min-height: 6rem;
    resize: vertical;
    padding: 0.625rem 0.75rem;
    border: 1px solid #2a323c;
    border-radius: 0.5rem;
    background: #0f1115;
    color: #e8eef5;
    font: inherit;
    font-size: 0.9375rem;
    line-height: 1.4;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .send {
    padding: 0.5rem 1rem;
    border: 1px solid transparent;
    border-radius: 0.5rem;
    background: #2f6feb;
    color: #ffffff;
    font: inherit;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 120ms ease;
  }

  .send:hover {
    background: #1f5fd8;
  }

  .send:disabled {
    background: #39424d;
    color: #9aa7b4;
    cursor: not-allowed;
  }

  .cancel {
    padding: 0.5rem 1rem;
    border: 1px solid #2a323c;
    border-radius: 0.5rem;
    background: transparent;
    color: #e8eef5;
    font: inherit;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
  }

  .cancel:hover:not(:disabled) {
    background: #232a33;
  }

  .cancel:disabled {
    color: #9aa7b4;
    cursor: not-allowed;
  }

  :focus-visible {
    outline: 2px solid #8ab4ff;
    outline-offset: 2px;
  }

  .canvas {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-height: 60vh;
  }

  .canvas p {
    margin: 0;
    font-size: 0.9375rem;
    color: #9aa7b4;
  }

  @media (max-width: 840px) {
    .workspace {
      grid-template-columns: minmax(0, 1fr);
    }

    .canvas {
      min-height: 40vh;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .send {
      transition: none;
    }
  }
</style>
