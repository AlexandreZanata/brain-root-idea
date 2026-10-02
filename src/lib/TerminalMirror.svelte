<script lang="ts">
  /**
   * B20-U7 — the terminal mirror, on the pinned `terminal-panel*.tsx` shape
   * (@ 34aa427): `aside role="region"` with an `h-10` header and a mono output
   * body. The pin hosts real PTYs; BrainRoot has no PTY and no Process
   * Manager, and building one is a separate batch with its own permission
   * design. So this panel mirrors what the turn already produced — the same
   * `agent_event` / `conversation_event` streams App.svelte consumes — and says
   * so in its name and its header. Nothing here spawns, simulates, or pretends
   * to be a shell.
   *
   * The mirror is a bounded ledger (`MIRROR_MAX_TURNS` / `MIRROR_MAX_TEXT_CHARS`)
   * and holds no listener after destroy: both unlistens run on the same
   * cleanup path that owns them (B20-U7-T03).
   */
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import EmptyState from "./EmptyState.svelte";
  import {
    AGENT_EVENT_NAME,
    isAgentEventEnvelope
  } from "../agentHost";
  import { isConversationEnvelope } from "../conversation";
  import {
    TERMINAL_MIRROR_LABEL,
    TERMINAL_MIRROR_NOTE,
    foldMirrorEvent,
    type MirrorTurn
  } from "../panels";

  let turns = $state<MirrorTurn[]>([]);

  onMount(() => {
    let disposed = false;
    const unlistens: UnlistenFn[] = [];

    void (async () => {
      try {
        unlistens.push(
          await listen<unknown>(AGENT_EVENT_NAME, ({ payload }) => {
            if (!isAgentEventEnvelope(payload)) {
              return;
            }
            turns = foldMirrorEvent(turns, payload.event.session, payload.event);
          })
        );
        unlistens.push(
          await listen<unknown>("conversation_event", ({ payload }) => {
            if (!isConversationEnvelope(payload)) {
              return;
            }
            // The legacy stream carries no session; the turn is the app's one.
            turns = foldMirrorEvent(turns, "conversation", payload.event);
          })
        );
      } catch {
        // Without the desktop shell there is no stream to mirror; the empty
        // state below says so honestly instead of inventing output.
      }
      if (disposed) {
        for (const unlisten of unlistens.splice(0)) {
          unlisten();
        }
      }
    })();

    return () => {
      disposed = true;
      for (const unlisten of unlistens.splice(0)) {
        unlisten();
      }
    };
  });

  function stateLabel(turn: MirrorTurn): string {
    switch (turn.state) {
      case "running":
        return "Running";
      case "completed":
        return "Completed";
      case "cancelled":
        return "Cancelled";
      case "failed":
        return "Failed";
    }
  }
</script>

<aside class="terminal" role="region" aria-label={TERMINAL_MIRROR_LABEL}>
  <div class="terminal__head">
    <h3 class="terminal__title">{TERMINAL_MIRROR_LABEL}</h3>
    <p class="terminal__note">{TERMINAL_MIRROR_NOTE}</p>
  </div>

  <div class="terminal__body">
    {#if turns.length === 0}
      <EmptyState
        icon="terminal"
        title="No turn output yet"
        description="Send a prompt and the answer streams here as plain text. This is a mirror of the turn output, not a shell."
      />
    {:else}
      {#each turns as turn, index (`${turn.session}-${index}`)}
        <article class="turn">
          <p class="turn__state" class:turn__state--failed={turn.state === "failed"}>
            {stateLabel(turn)}
          </p>
          {#if turn.text}
            <pre class="turn__text">{turn.text}</pre>
          {/if}
          {#if turn.error}
            <p class="turn__error" role="alert">{turn.error}</p>
          {/if}
        </article>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .terminal {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    height: 100%;
    border-radius: 10px;
    background: var(--v2-background-bg-base);
    box-shadow: var(--v2-elevation-raised);
    overflow: hidden;
  }

  .terminal__head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-shrink: 0;
    height: 40px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--v2-border-border-weak-base);
  }

  .terminal__title {
    margin: 0;
    font-size: 13px;
    font-weight: 500;
    line-height: 20px;
    color: var(--text);
  }

  .terminal__note {
    margin: 0;
    font-size: 12px;
    line-height: 20px;
    color: var(--text-subtle);
  }

  .terminal__body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 12px;
  }

  .turn {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .turn__state {
    margin: 0;
    font-family: ui-monospace, "JetBrains Mono", monospace;
    font-size: 11px;
    line-height: 16px;
    color: var(--text-subtle);
  }

  .turn__state--failed {
    color: var(--text-muted);
  }

  .turn__text {
    margin: 0;
    font-family: ui-monospace, "JetBrains Mono", monospace;
    font-size: 12px;
    line-height: 18px;
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .turn__error {
    margin: 0;
    font-family: ui-monospace, "JetBrains Mono", monospace;
    font-size: 12px;
    line-height: 18px;
    color: var(--text-muted);
  }
</style>
