<script lang="ts">
  import { onMount } from "svelte";
  import { requestHealth } from "./health";

  type HealthState = "checking" | "ready" | "failed";

  let healthState: HealthState = $state("checking");
  let detail = $state("Waiting for the core health result.");

  onMount(async () => {
    try {
      await requestHealth();
      healthState = "ready";
      detail = "The core health contract responded normally.";
    } catch (error) {
      healthState = "failed";
      detail = error instanceof Error ? error.message : "The core health request failed.";
    }
  });

  function onPromptSubmit(event: SubmitEvent) {
    event.preventDefault();
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
      <form class="prompt" onsubmit={onPromptSubmit}>
        <label for="prompt">What do you want to build?</label>
        <textarea id="prompt" name="prompt" rows="4"></textarea>
        <button class="send" type="submit">Send</button>
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
  }

  .prompt {
    display: flex;
    flex-direction: column;
    flex: 1;
    gap: 0.5rem;
  }

  .prompt label {
    font-size: 0.875rem;
    color: #cdd9e5;
  }

  textarea {
    flex: 1;
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

  .send {
    align-self: flex-start;
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
