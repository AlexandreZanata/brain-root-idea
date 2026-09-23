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

  const railSections = [
    { id: "build", label: "Build", active: true },
    { id: "agents", label: "Agents", active: false },
    { id: "browser", label: "Browser", active: false },
    { id: "files", label: "Files", active: false },
    { id: "terminal", label: "Terminal", active: false },
    { id: "settings", label: "Settings", active: false }
  ];

  const canvasTabs = ["Preview", "Components", "Logs", "AI Notes"];

  const suggestions = [
    {
      label: "Explain what this MVP-0 experiment does",
      prompt: "Explain what this MVP-0 experiment does, in three sentences."
    },
    {
      label: "Write a short tagline for BrainRoot",
      prompt: "Write a short, honest tagline for BrainRoot."
    },
    {
      label: "Summarize the last release in three bullets",
      prompt: "Summarize the last BrainRoot release in three bullets."
    },
    {
      label: "Draft a friendly reply to a bug report",
      prompt: "Draft a friendly first reply to a user who reported a bug."
    }
  ];

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
  let textarea: HTMLTextAreaElement | undefined;

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
  let statusLabel = $derived.by(() => {
    switch (conversationState) {
      case "sending":
        return "Starting…";
      case "streaming":
        return "Building…";
      case "cancelling":
        return "Cancelling…";
      case "succeeded":
        return "Done";
      case "failed":
        return "Needs attention";
      default:
        return setupMessage ? "Needs setup" : "Ready for a request";
    }
  });

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

  function useSuggestion(text: string) {
    prompt = text;
    textarea?.focus();
  }

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
    <div class="brand">
      <span class="brand-mark" aria-hidden="true">BR</span>
      <div class="brand-text">
        <h1>BrainRoot</h1>
        <p class="tagline">Experimental Linux model loop · MVP-0</p>
      </div>
    </div>
    <div class="core">
      <span class="status-dot" class:status-ready={healthState === "ready"} aria-hidden="true"></span>
      <div class="core-text">
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
    </div>
  </header>

  <div class="workspace">
    <nav class="rail" aria-label="Workspace sections">
      {#each railSections as section (section.id)}
        <button
          type="button"
          class="rail-item"
          class:rail-active={section.active}
          aria-current={section.active ? "true" : undefined}
          aria-disabled={!section.active}
          title={section.active ? section.label : `${section.label} — planned for MVP-1`}
        >
          <span class="rail-icon" aria-hidden="true">
            {#if section.id === "build"}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M4 5h16v11H8l-4 3V5z" stroke-linejoin="round"/></svg>
            {:else if section.id === "agents"}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="5" y="7" width="14" height="11" rx="2"/><path d="M12 3v4M9 12h.01M15 12h.01" stroke-linecap="round"/></svg>
            {:else if section.id === "browser"}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><circle cx="12" cy="12" r="8"/><path d="M4 12h16M12 4c2.5 2.4 2.5 13.6 0 16M12 4c-2.5 2.4-2.5 13.6 0 16"/></svg>
            {:else if section.id === "files"}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M4 7a2 2 0 0 1 2-2h3l2 2h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V7z" stroke-linejoin="round"/></svg>
            {:else if section.id === "terminal"}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="4" y="5" width="16" height="14" rx="2"/><path d="m8 10 2.5 2.5L8 15M13 15h3" stroke-linecap="round" stroke-linejoin="round"/></svg>
            {:else}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><circle cx="12" cy="12" r="3"/><path d="M12 4v2M12 18v2M4 12h2M18 12h2M6.3 6.3l1.4 1.4M16.3 16.3l1.4 1.4M6.3 17.7l1.4-1.4M16.3 7.7l1.4-1.4" stroke-linecap="round"/></svg>
            {/if}
          </span>
          <span class="rail-label">{section.label}</span>
        </button>
      {/each}
    </nav>

    <section class="agent" aria-labelledby="agent-title">
      <div class="panel-head">
        <div>
          <h2 id="agent-title">Build</h2>
          <p class="panel-subtitle">Describe a change and watch the model answer.</p>
        </div>
        <span class="panel-badge">MVP-0</span>
      </div>

      {#if setupMessage}
        <p class="setup" role="status">{setupMessage}</p>
      {/if}

      <div class="conversation-history" aria-label="Conversation">
        {#if turns.length === 0}
          <article class="welcome">
            <h3>
              <span class="welcome-spark" aria-hidden="true">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M12 4v4M12 16v4M4 12h4M16 12h4M6.5 6.5l2.8 2.8M14.7 14.7l2.8 2.8M6.5 17.5l2.8-2.8M14.7 9.3l2.8-2.8" stroke-linecap="round"/></svg>
              </span>
              Welcome to BrainRoot
            </h3>
            <p>
              The MVP-0 Linux model loop: ask one prompt, watch the streamed answer, and cancel
              anytime. Files, browser, terminal, and preview arrive in MVP-1.
            </p>
          </article>

          <div class="suggestions">
            {#each suggestions as suggestion (suggestion.label)}
              <button type="button" class="suggestion" onclick={() => useSuggestion(suggestion.prompt)}>
                <span>{suggestion.label}</span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="m9 6 6 6-6 6" stroke-linecap="round" stroke-linejoin="round"/></svg>
              </button>
            {/each}
          </div>
        {/if}

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

      <form class="composer" onsubmit={onPromptSubmit}>
        <label for="prompt" class="visually-hidden">What do you want to build?</label>
        <textarea
          id="prompt"
          name="prompt"
          rows="3"
          placeholder="Describe what you want to build…"
          bind:value={prompt}
          bind:this={textarea}
        ></textarea>
        <div class="composer-bar">
          <span class="model-chip" title="Configured model for MVP-0">glm-5.3-flash</span>
          <span class="composer-state">{statusLabel}</span>
          <div class="actions">
            <button class="send" type="submit" aria-disabled={!canSend}>
              <span>Send</span>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M5 12h13M13 6l6 6-6 6" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </button>
            <button class="cancel" type="button" aria-disabled={!isCancellable} onclick={onCancel}>
              Cancel
            </button>
          </div>
        </div>
      </form>
    </section>

    <section class="canvas" aria-labelledby="canvas-title">
      <div class="canvas-head">
        <div>
          <h2 id="canvas-title">Preview / Canvas</h2>
          <p class="panel-subtitle">The result lives here — dominant by design.</p>
        </div>
        <div class="canvas-tabs" aria-label="Canvas views">
          {#each canvasTabs as tab, index (tab)}
            <button
              type="button"
              class="canvas-tab"
              class:canvas-tab-active={index === 0}
              aria-current={index === 0 ? "true" : undefined}
              aria-disabled={index !== 0}
              title={index === 0 ? tab : `${tab} — planned for MVP-1`}
            >
              {tab}
            </button>
          {/each}
        </div>
      </div>

      <div class="canvas-body">
        <div class="canvas-empty">
          <span class="canvas-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4"><rect x="3" y="5" width="18" height="12" rx="2"/><path d="M9 20h6M12 17v3" stroke-linecap="round"/></svg>
          </span>
          <h3>Preview comes in MVP-1.</h3>
          <p>Local previews and the full Companion Canvas arrive with the next milestone. The agent loop works today.</p>
        </div>

        <div class="canvas-cards">
          <button type="button" class="canvas-card" aria-disabled="true" title="Planned for MVP-1">
            <span class="card-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="m9 8-4 4 4 4M15 8l4 4-4 4" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </span>
            <span class="card-text">
              <span class="card-title">Generate a UI</span>
              <span class="card-subtitle">Create a modern UI from a prompt</span>
            </span>
            <span class="card-badge">MVP-1</span>
          </button>
          <button type="button" class="canvas-card" aria-disabled="true" title="Planned for MVP-1">
            <span class="card-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M4 12a8 8 0 0 1 8-8h4M20 12a8 8 0 0 1-8 8h-4M14 2l2 2-2 2M10 18l-2 2 2 2" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </span>
            <span class="card-text">
              <span class="card-title">Open a project</span>
              <span class="card-subtitle">Connect an existing folder</span>
            </span>
            <span class="card-badge">MVP-1</span>
          </button>
          <button type="button" class="canvas-card" aria-disabled="true" title="Planned for MVP-1">
            <span class="card-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><rect x="13" y="13" width="7" height="7" rx="1.5"/></svg>
            </span>
            <span class="card-text">
              <span class="card-title">Use a template</span>
              <span class="card-subtitle">Start from a ready template</span>
            </span>
            <span class="card-badge">MVP-1</span>
          </button>
        </div>
      </div>
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
    background: #0d1014;
    color: #e8eef5;
    font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  }

  .app {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    height: 100vh;
    box-sizing: border-box;
  }

  .chrome {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.7rem 1.25rem;
    border-bottom: 1px solid #232a33;
    background: #12161b;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    min-width: 0;
  }

  .brand-mark {
    display: grid;
    place-items: center;
    width: 2.1rem;
    height: 2.1rem;
    border-radius: 0.55rem;
    background: linear-gradient(140deg, #2f6feb, #1b3d8f);
    color: #ffffff;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.03em;
  }

  .brand-text {
    min-width: 0;
  }

  h1 {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 650;
    line-height: 1.2;
  }

  .tagline {
    margin: 0;
    font-size: 0.72rem;
    color: #9aa7b4;
  }

  .core {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    text-align: right;
  }

  .status-dot {
    width: 0.55rem;
    height: 0.55rem;
    border-radius: 50%;
    background: #7d8b99;
  }

  .status-ready {
    background: #3fb950;
  }

  .core-status {
    margin: 0;
    font-size: 0.82rem;
    font-weight: 600;
  }

  .core-detail {
    margin: 0;
    font-size: 0.72rem;
    color: #9aa7b4;
  }

  .workspace {
    display: grid;
    grid-template-columns: 4.6rem minmax(19rem, 23rem) minmax(0, 1fr);
    gap: 0.9rem;
    padding: 0.9rem;
    min-height: 0;
  }

  .rail {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding: 0.45rem;
    border: 1px solid #232a33;
    border-radius: 0.75rem;
    background: #12161b;
    min-height: 0;
    overflow-y: auto;
  }

  .rail-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.3rem;
    padding: 0.55rem 0.25rem;
    border: 1px solid transparent;
    border-radius: 0.6rem;
    background: transparent;
    color: #9aa7b4;
    font: inherit;
    font-size: 0.65rem;
    cursor: pointer;
  }

  .rail-item:hover {
    background: #171c22;
    color: #e8eef5;
  }

  .rail-active {
    background: #1b2430;
    border-color: #2f6feb;
    color: #e8eef5;
  }

  .rail-item[aria-disabled="true"] {
    color: #7d8b99;
    cursor: not-allowed;
  }

  .rail-item[aria-disabled="true"]:hover {
    background: transparent;
    color: #7d8b99;
  }

  .rail-icon svg {
    width: 1.15rem;
    height: 1.15rem;
  }

  .rail-label {
    font-weight: 600;
  }

  .agent,
  .canvas {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border: 1px solid #232a33;
    border-radius: 0.75rem;
    background: #141920;
  }

  .panel-head,
  .canvas-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.85rem 1rem;
    border-bottom: 1px solid #232a33;
  }

  .canvas-head {
    align-items: center;
    flex-wrap: wrap;
  }

  h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 650;
  }

  .panel-subtitle {
    margin: 0.15rem 0 0;
    font-size: 0.75rem;
    color: #9aa7b4;
  }

  .panel-badge,
  .card-badge {
    flex-shrink: 0;
    padding: 0.15rem 0.45rem;
    border: 1px solid #2a323c;
    border-radius: 999px;
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: #9aa7b4;
  }

  .setup {
    margin: 0;
    padding: 0.6rem 1rem;
    border-bottom: 1px solid #232a33;
    color: #cdd9e5;
    font-size: 0.78rem;
  }

  .conversation-history {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .welcome {
    padding: 0.9rem;
    border: 1px solid #232a33;
    border-radius: 0.7rem;
    background: #171c22;
  }

  .welcome h3 {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin: 0 0 0.35rem;
    font-size: 0.9rem;
  }

  .welcome-spark svg {
    width: 1rem;
    height: 1rem;
    color: #8ab4ff;
  }

  .welcome p {
    margin: 0;
    font-size: 0.8rem;
    line-height: 1.5;
    color: #9aa7b4;
  }

  .suggestions {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .suggestion {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem;
    border: 1px solid #232a33;
    border-radius: 0.6rem;
    background: #171c22;
    color: #e8eef5;
    font: inherit;
    font-size: 0.78rem;
    text-align: left;
    cursor: pointer;
  }

  .suggestion:hover {
    border-color: #2f6feb;
    background: #1b2430;
  }

  .suggestion svg {
    width: 0.9rem;
    height: 0.9rem;
    color: #9aa7b4;
    flex-shrink: 0;
  }

  .turn {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding-bottom: 0.9rem;
    border-bottom: 1px solid #232a33;
  }

  .turn:last-child {
    border-bottom: 0;
    padding-bottom: 0;
  }

  .message-label,
  .message,
  .turn-error,
  .turn-cancelled {
    margin: 0;
  }

  .message-label {
    color: #9aa7b4;
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .message,
  .turn-error,
  .turn-cancelled {
    overflow-wrap: anywhere;
    font-size: 0.84rem;
    line-height: 1.5;
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

  .technical {
    margin: 0;
    color: #9aa7b4;
    font-size: 0.75rem;
  }

  .technical summary {
    cursor: pointer;
  }

  .technical code {
    color: #cdd9e5;
  }

  .composer {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 1rem 1rem;
    border-top: 1px solid #232a33;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
    border: 0;
  }

  textarea {
    width: 100%;
    box-sizing: border-box;
    min-height: 4.2rem;
    resize: vertical;
    padding: 0.65rem 0.75rem;
    border: 1px solid #232a33;
    border-radius: 0.65rem;
    background: #0f1318;
    color: #e8eef5;
    font: inherit;
    font-size: 0.84rem;
    line-height: 1.45;
  }

  textarea::placeholder {
    color: #7d8b99;
  }

  .composer-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .model-chip {
    padding: 0.25rem 0.55rem;
    border: 1px solid #2a323c;
    border-radius: 999px;
    background: #1b2430;
    color: #cdd9e5;
    font-size: 0.7rem;
    font-weight: 600;
  }

  .composer-state {
    flex: 1;
    text-align: right;
    color: #9aa7b4;
    font-size: 0.72rem;
  }

  .actions {
    display: flex;
    gap: 0.4rem;
  }

  .send {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.45rem 0.85rem;
    border: 1px solid transparent;
    border-radius: 0.55rem;
    background: #2f6feb;
    color: #ffffff;
    font: inherit;
    font-size: 0.8rem;
    font-weight: 650;
    cursor: pointer;
    transition: background-color 120ms ease;
  }

  .send svg {
    width: 0.9rem;
    height: 0.9rem;
  }

  .send:hover {
    background: #1f5fd8;
  }

  .send[aria-disabled="true"] {
    background: #39424d;
    color: #cdd9e5;
    cursor: not-allowed;
  }

  .cancel {
    padding: 0.45rem 0.85rem;
    border: 1px solid #2a323c;
    border-radius: 0.55rem;
    background: transparent;
    color: #e8eef5;
    font: inherit;
    font-size: 0.8rem;
    font-weight: 650;
    cursor: pointer;
  }

  .cancel:hover:not([aria-disabled="true"]) {
    background: #232a33;
  }

  .cancel[aria-disabled="true"] {
    color: #9aa7b4;
    cursor: not-allowed;
  }

  .canvas-tabs {
    display: flex;
    gap: 0.25rem;
    padding: 0.2rem;
    border: 1px solid #232a33;
    border-radius: 0.6rem;
    background: #0f1318;
  }

  .canvas-tab {
    padding: 0.35rem 0.7rem;
    border: 0;
    border-radius: 0.45rem;
    background: transparent;
    color: #9aa7b4;
    font: inherit;
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .canvas-tab-active {
    background: #1b2430;
    color: #e8eef5;
  }

  .canvas-tab[aria-disabled="true"] {
    color: #7d8b99;
    cursor: not-allowed;
  }

  .canvas-body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-rows: 1fr auto;
    gap: 1rem;
    padding: 1.25rem;
    overflow-y: auto;
  }

  .canvas-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    text-align: center;
  }

  .canvas-icon {
    display: grid;
    place-items: center;
    width: 3.2rem;
    height: 3.2rem;
    border: 1px solid #2a323c;
    border-radius: 0.8rem;
    background: #171c22;
  }

  .canvas-icon svg {
    width: 1.6rem;
    height: 1.6rem;
    color: #8ab4ff;
  }

  .canvas-empty h3 {
    margin: 0.35rem 0 0;
    font-size: 1rem;
  }

  .canvas-empty p {
    margin: 0;
    max-width: 26rem;
    color: #9aa7b4;
    font-size: 0.82rem;
    line-height: 1.5;
  }

  .canvas-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
    gap: 0.6rem;
  }

  .canvas-card {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.7rem 0.8rem;
    border: 1px solid #232a33;
    border-radius: 0.65rem;
    background: #171c22;
    color: #e8eef5;
    font: inherit;
    text-align: left;
    cursor: not-allowed;
  }

  .card-icon svg {
    width: 1.25rem;
    height: 1.25rem;
    color: #8ab4ff;
  }

  .card-text {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .card-title {
    font-size: 0.8rem;
    font-weight: 650;
  }

  .card-subtitle {
    font-size: 0.7rem;
    color: #9aa7b4;
  }

  .card-badge {
    padding: 0.1rem 0.35rem;
    font-size: 0.58rem;
  }

  :focus-visible {
    outline: 2px solid #8ab4ff;
    outline-offset: 2px;
  }

  @media (max-width: 1080px) {
    .workspace {
      grid-template-columns: 1fr;
      overflow-y: auto;
    }

    .rail {
      flex-direction: row;
      overflow-x: auto;
    }

    .agent,
    .canvas {
      min-height: 26rem;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .send {
      transition: none;
    }
  }
</style>
