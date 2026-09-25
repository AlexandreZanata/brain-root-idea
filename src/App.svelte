<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import {
    CONVERSATION_CONTRACT_VERSION,
    appendChunk,
    beginTurn,
    cancelTurn,
    clampPanelWidth,
    credentialSetupMessage,
    isConversationEnvelope,
    settleTurn,
    type ConversationEnvelope,
    type ConversationState,
    type ConversationTurn,
    type CredentialStatus
  } from "./conversation";
  import { requestHealth } from "./health";
  import {
    AGENT_EVENT_NAME,
    chooseSendPath,
    hostLabel,
    isAgentEventEnvelope,
    requestHostCancelSend,
    requestHostCatalog,
    requestHostSelectModel,
    requestHostSend,
    requestHostSetAgent,
    requestHostStart,
    requestHostStatus,
    requestHostStop,
    type AgentEventEnvelope,
    type AgentHostStatus,
    type AgentMode,
    type AgentModelSelection,
    type CatalogModel,
    type CostProfile,
    pickProfileModel,
    type HostPhase,
    type SendPath
  } from "./agentHost";
  import { acceptsEvent, taskStatus } from "./presentation";
  import { createStreamBuffer } from "./streamBuffer";
  import AppHeader from "./lib/AppHeader.svelte";
  import Button from "./lib/Button.svelte";
  import CanvasPanel from "./lib/CanvasPanel.svelte";
  import ConversationPanel from "./lib/ConversationPanel.svelte";
  import PanelResizer from "./lib/PanelResizer.svelte";
  import SessionTabs from "./lib/SessionTabs.svelte";
  import WorkspaceRail from "./lib/WorkspaceRail.svelte";

  type HealthState = "checking" | "ready" | "failed";
  type Theme = "dark" | "light";

  type SessionTabState = {
    id: number;
    title: string;
    prompt: string;
    turns: ConversationTurn[];
  };

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
  // Frontend tabs share the single legacy conversation backend for now;
  // per-tab backend sessions arrive with the Agent Host session API (S03+).
  let tabs = $state<SessionTabState[]>([
    { id: 1, title: "Session 1", prompt: "", turns: [] }
  ]);
  let activeTabId = $state(1);
  let nextTabId = 2;
  let activeTab = $derived(
    tabs.find((tab) => tab.id === activeTabId) ?? {
      id: activeTabId,
      title: "Session",
      prompt: "",
      turns: []
    }
  );
  let nextTurnId = 1;
  let activeTurnId: number | null = null;
  let streamTabId: number | null = $state(null);
  let streamingTabId = $derived(activeTurnId === null ? null : streamTabId);
  // Which backend owns the in-flight turn. Decided per submit; reset on terminal.
  let sendPath = $state<SendPath>("legacy");
  let agentSession = $state<string | null>(null);
  let hostPhase = $state<HostPhase>("checking");
  let hostStatus = $state<AgentHostStatus | null>(null);
  let hostDetail = $state("");
  let hostModels = $state<CatalogModel[]>([]);
  let staleModels = $state(false);
  let selectedModel = $state<AgentModelSelection | null>(null);
  // Rust defaults to build; the toggle is the source of truth after mount.
  let agentMode = $state<AgentMode>("build");
  // No profile until the user opts in; manual picks return to custom.
  let profile = $state<CostProfile | null>(null);
  let hostStatusLabel = $derived(
    hostPhase === "failed" && hostDetail ? hostDetail : hostLabel(hostPhase, hostStatus)
  );
  const streamBuffer = createStreamBuffer(
    (text) => {
      if (activeTurnId !== null) {
        const target = tabs.find((tab) => tab.id === streamTabId);
        if (target) {
          target.turns = appendChunk(target.turns, activeTurnId, text);
        }
      }
    },
    {
      schedule: (callback) => window.requestAnimationFrame(callback),
      cancel: (handle) => window.cancelAnimationFrame(handle)
    }
  );
  let agentWidth = $state(368);
  let clampedAgentWidth = $derived(clampPanelWidth(agentWidth, 280, 560));
  let canvasOnLeft = $state(false);
  let sideNote = $derived(
    canvasOnLeft ? "Canvas on the left, chat on the right." : "Canvas on the right, chat on the left."
  );

  function toggleSides() {
    canvasOnLeft = !canvasOnLeft;
  }

  $effect(() => {
    if (agentWidth !== clampedAgentWidth) {
      agentWidth = clampedAgentWidth;
    }
  });

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
      activeTab.prompt.trim().length > 0
  );
  let task = $derived(taskStatus(conversationState, setupMessage));

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
    let unlistenAgent: UnlistenFn | undefined;

    void (async () => {
      try {
        unlisten = await listen<unknown>("conversation_event", ({ payload }) => {
          onConversationEvent(payload);
        });
        unlistenAgent = await listen<unknown>(AGENT_EVENT_NAME, ({ payload }) => {
          onAgentEvent(payload);
        });
        if (disposed) {
          unlisten();
          unlistenAgent();
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

      await refreshHost();
    })();

    return () => {
      disposed = true;
      unlisten?.();
      unlistenAgent?.();
      streamBuffer.dispose();
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

  async function refreshHost() {
    try {
      hostStatus = await requestHostStatus();
      hostPhase = hostStatus.running ? "running" : "stopped";
    } catch {
      hostPhase = "stopped";
      hostDetail = "The sidecar status could not be read.";
      return;
    }
    await loadModels();
  }

  async function loadModels() {
    if (!hostStatus?.running) {
      hostModels = [];
      staleModels = false;
      return;
    }
    try {
      const catalog = await requestHostCatalog(false);
      hostModels = catalog.models;
      staleModels = catalog.stale;
      selectedModel = catalog.selected;
    } catch {
      if (hostModels.length > 0) {
        staleModels = true;
      }
    }
  }

  async function selectModel(providerId: string, modelId: string) {
    profile = null;
    try {
      selectedModel = await requestHostSelectModel(providerId, modelId);
    } catch (error) {
      hostPhase = "failed";
      hostDetail = error instanceof Error ? error.message : "The model could not be selected.";
    }
  }

  async function selectProfile(next: CostProfile) {
    const pick = pickProfileModel(hostModels, next);
    if (!pick) {
      return;
    }
    profile = next;
    await selectAgent(pick.agent);
    try {
      selectedModel = await requestHostSelectModel(pick.providerId, pick.modelId);
    } catch (error) {
      profile = null;
      hostPhase = "failed";
      hostDetail = error instanceof Error ? error.message : "The model could not be selected.";
    }
  }
  async function selectAgent(mode: AgentMode) {    if (mode === agentMode) {
      return;
    }
    profile = null;
    const previous = agentMode;
    agentMode = mode;
    try {
      await requestHostSetAgent(mode);
    } catch (error) {
      agentMode = previous;
      hostPhase = "failed";
      hostDetail = error instanceof Error ? error.message : "The agent mode could not be set.";
    }
  }

  async function startHost() {
    if (hostPhase === "starting") {
      return;
    }
    hostPhase = "starting";
    hostDetail = "";
    try {
      hostStatus = await requestHostStart();
      hostPhase = "running";
      await loadModels();
    } catch (error) {
      hostPhase = "failed";
      hostDetail = error instanceof Error ? error.message : "The sidecar could not be started.";
    }
  }

  async function stopHost() {
    try {
      hostStatus = await requestHostStop();
    } catch {
      // The stopped view is correct whether or not the call landed.
    }
    hostPhase = "stopped";
  }

  function selectTab(id: number) {
    activeTabId = id;
  }

  function newTab() {
    const id = nextTabId;
    nextTabId += 1;
    tabs = [...tabs, { id, title: `Session ${id}`, prompt: "", turns: [] }];
    activeTabId = id;
  }

  async function closeTab(id: number) {
    if (tabs.length <= 1) {
      return;
    }
    if (id === streamTabId && activeTurnId !== null) {
      await onCancel();
    }
    const remaining = tabs.filter((tab) => tab.id !== id);
    tabs = remaining;
    if (activeTabId === id) {
      const fallback = remaining[remaining.length - 1];
      if (fallback) {
        activeTabId = fallback.id;
      }
    }
  }

  async function submitPrompt() {
    const tab = activeTab;
    const message = tab.prompt.trim();
    if (!canSend || message.length === 0) {
      return;
    }

    const turnId = nextTurnId;
    nextTurnId += 1;
    streamBuffer.flush();
    activeTurnId = turnId;
    streamTabId = tab.id;
    tab.turns = beginTurn(tab.turns, turnId, message);
    if (tab.title.startsWith("Session ") && tab.turns.length === 1) {
      tab.title = message.slice(0, 28);
    }
    tab.prompt = "";
    conversationState = "sending";
    sendPath = chooseSendPath(hostPhase === "running");

    if (sendPath === "agent") {
      try {
        const accepted = await requestHostSend(message);
        agentSession = accepted.session;
      } catch (error) {
        agentSession = null;
        sendPath = "legacy";
        failActive(errorMessage(error), errorCode(error));
      }
      return;
    }

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
    if (sendPath !== "legacy") {
      return;
    }
    if (!isConversationEnvelope(payload)) {
      failActive("The core returned an unexpected conversation event.");
      return;
    }
    applyConversationEvent(payload);
  }

  function onAgentEvent(payload: unknown) {
    if (!isAgentEventEnvelope(payload)) {
      failActive("The core returned an unexpected agent event.");
      return;
    }
    applyAgentEvent(payload);
  }

  function applyAgentEvent(envelope: AgentEventEnvelope) {
    const turnId = activeTurnId;
    const tab = tabs.find((candidate) => candidate.id === streamTabId);
    if (turnId === null || !tab) {
      return;
    }
    if (sendPath !== "agent" || envelope.event.session !== agentSession) {
      return;
    }
    if (!acceptsEvent(conversationState, envelope.event.type)) {
      return;
    }

    switch (envelope.event.type) {
      case "started":
        conversationState = "streaming";
        break;
      case "text_chunk":
        conversationState = "streaming";
        streamBuffer.push(envelope.event.text);
        break;
      case "completed":
        streamBuffer.flush();
        tab.turns = settleTurn(tab.turns, turnId, "succeeded");
        conversationState = "succeeded";
        activeTurnId = null;
        agentSession = null;
        sendPath = "legacy";
        break;
      case "failed":
        agentSession = null;
        sendPath = "legacy";
        failActive(envelope.event.error.message, envelope.event.error.code);
        break;
      case "cancelled":
        streamBuffer.flush();
        tab.turns = cancelTurn(tab.turns, turnId);
        conversationState = "ready";
        activeTurnId = null;
        agentSession = null;
        sendPath = "legacy";
        break;
    }
  }

  function applyConversationEvent(envelope: ConversationEnvelope) {
    const turnId = activeTurnId;
    const tab = tabs.find((candidate) => candidate.id === streamTabId);
    if (turnId === null || !tab) {
      return;
    }
    if (!acceptsEvent(conversationState, envelope.event.type)) {
      return;
    }

    switch (envelope.event.type) {
      case "started":
        conversationState = "streaming";
        break;
      case "text_chunk":
        conversationState = "streaming";
        streamBuffer.push(envelope.event.text);
        break;
      case "completed":
        streamBuffer.flush();
        tab.turns = settleTurn(tab.turns, turnId, "succeeded");
        conversationState = "succeeded";
        activeTurnId = null;
        break;
      case "failed":
        failActive(envelope.event.error.message, envelope.event.error.code);
        break;
      case "cancelled":
        streamBuffer.flush();
        tab.turns = cancelTurn(tab.turns, turnId);
        conversationState = "ready";
        activeTurnId = null;
        break;
    }
  }

  async function onCancel() {
    if (!isCancellable) {
      return;
    }
    streamBuffer.flush();
    conversationState = "cancelling";
    if (sendPath === "agent") {
      // The terminal event arrives from the worker; every worker exit emits
      // exactly one, so "cancelling" always resolves.
      try {
        await requestHostCancelSend();
      } catch (error) {
        if (conversationState === "cancelling") {
          agentSession = null;
          sendPath = "legacy";
          failActive(errorMessage(error), errorCode(error));
        }
      }
      return;
    }
    try {
      await invoke("conversation_cancel");
    } catch (error) {
      if (conversationState === "cancelling") {
        failActive(errorMessage(error), errorCode(error));
      }
    }
  }

  function failActive(message: string, code = "") {
    streamBuffer.flush();
    if (activeTurnId !== null) {
      const tab = tabs.find((candidate) => candidate.id === streamTabId);
      if (tab) {
        tab.turns = settleTurn(tab.turns, activeTurnId, "failed", message, code);
      }
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
  <div class="workspace-bar">
    <SessionTabs
      {tabs}
      activeId={activeTabId}
      streamingId={streamingTabId}
      onselect={selectTab}
      onclose={closeTab}
      onnew={newTab}
    />
    <div class="host-badge">
      <span class="host-dot" class:on={hostPhase === "running"} role="presentation"></span>
      <span role="status">{hostStatusLabel}</span>
      {#if hostPhase === "running"}
        <Button variant="secondary" onclick={stopHost}>Stop</Button>
      {:else if hostPhase === "stopped" || hostPhase === "failed"}
        <Button variant="secondary" onclick={startHost}>Start</Button>
      {/if}
    </div>
    <Button variant="secondary" onclick={toggleSides}>Swap sides</Button>
    <p class="workspace-side-note" role="status">{sideNote}</p>
  </div>
  <div class="workspace" class:canvas-left={canvasOnLeft} style="--agent-width: {clampedAgentWidth}px">
    <WorkspaceRail sections={railSections} />
    <ConversationPanel
      {setupMessage}
      turns={activeTab.turns}
      statusLabel={task.label}
      {canSend}
      {isCancellable}
      bind:prompt={activeTab.prompt}
      {suggestions}
      models={hostModels}
      selectedModel={selectedModel}
      staleModels={staleModels}
      modelDisabled={isBusy}
      agentMode={agentMode}
      profile={profile}
      onselectmodel={selectModel}
      onselectagent={selectAgent}
      onselectprofile={selectProfile}
      onsubmit={submitPrompt}
      oncancel={onCancel}
    />
    <PanelResizer bind:value={agentWidth} min={280} max={560} step={8} />
    <CanvasPanel />
  </div>
</main>

<style>
  .app {
    display: grid;
    grid-template-rows: auto auto minmax(0, 1fr);
    height: 100vh;
  }

  .workspace-bar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    min-width: 0;
  }

  .workspace-bar > :global(.tabs) {
    flex: 1;
  }

  .host-badge {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-supporting);
    color: var(--text-muted);
    white-space: nowrap;
  }

  .host-dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .host-dot.on {
    background: var(--accent);
  }

  .workspace-side-note {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--text-supporting);
  }

  .workspace {
    display: grid;
    grid-template-columns: 4.6rem var(--agent-width, 23rem) auto minmax(0, 1fr);
    gap: var(--space-4);
    padding: var(--space-4);
    min-height: 0;
  }

  /* Swap sides without remounting Canvas: DOM order stays
     rail → chat → resizer → Canvas, so component and WebView
     instances survive the toggle. Only visual order changes. */
  .workspace > :global(.br-rail) {
    order: 1;
  }

  .workspace > :global(.agent) {
    order: 2;
  }

  .workspace > :global(.br-resizer) {
    order: 3;
  }

  .workspace > :global(.canvas) {
    order: 4;
  }

  .workspace.canvas-left {
    grid-template-columns: 4.6rem minmax(0, 1fr) auto var(--agent-width, 23rem);
  }

  .workspace.canvas-left > :global(.agent) {
    order: 4;
  }

  .workspace.canvas-left > :global(.canvas) {
    order: 2;
  }

  @media (max-width: 1080px) {
    .workspace {
      grid-template-columns: 1fr;
      overflow-y: auto;
    }
  }

  @media (max-width: 640px) {
    .workspace {
      padding: var(--space-2);
      gap: var(--space-2);
    }
  }
</style>
