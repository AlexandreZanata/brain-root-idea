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
  import AppHeader from "./lib/AppHeader.svelte";
  import CanvasPanel from "./lib/CanvasPanel.svelte";
  import ConversationPanel from "./lib/ConversationPanel.svelte";
  import WorkspaceRail from "./lib/WorkspaceRail.svelte";

  type HealthState = "checking" | "ready" | "failed";
  type Theme = "dark" | "light";

  const railSections = [
    { id: "build", label: "Build", active: true },
    { id: "agents", label: "Agents", active: false },
    { id: "browser", label: "Browser", active: false },
    { id: "files", label: "Files", active: false },
    { id: "terminal", label: "Terminal", active: false },
    { id: "settings", label: "Settings", active: false }
  ];

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

  let theme = $state<Theme>(initialTheme());
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

  $effect(() => {
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem("brainroot-theme", theme);
    } catch {
      // A missing storage backend must not break the shell.
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

  function initialTheme(): Theme {
    try {
      const stored = localStorage.getItem("brainroot-theme");
      if (stored === "light" || stored === "dark") {
        return stored;
      }
    } catch {
      // Fall through to the system preference.
    }
    return window.matchMedia?.("(prefers-color-scheme: light)").matches ? "light" : "dark";
  }

  function toggleTheme() {
    theme = theme === "dark" ? "light" : "dark";
  }

  async function submitPrompt() {
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
  <AppHeader {healthState} {detail} {theme} ontoggle={toggleTheme} />
  <div class="workspace">
    <WorkspaceRail sections={railSections} />
    <ConversationPanel
      {setupMessage}
      {turns}
      {statusLabel}
      {canSend}
      {isCancellable}
      bind:prompt={prompt}
      {suggestions}
      onsubmit={submitPrompt}
      oncancel={onCancel}
    />
    <CanvasPanel />
  </div>
</main>

<style>
  .app {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    height: 100vh;
  }

  .workspace {
    display: grid;
    grid-template-columns: 4.6rem minmax(19rem, 23rem) minmax(0, 1fr);
    gap: 0.9rem;
    padding: 0.9rem;
    min-height: 0;
  }

  @media (max-width: 1080px) {
    .workspace {
      grid-template-columns: 1fr;
      overflow-y: auto;
    }
  }
</style>
