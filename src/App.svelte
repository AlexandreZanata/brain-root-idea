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
</script>

<main>
  <h1>BrainRoot</h1>
  <p class="subtitle">Experimental Linux setup</p>
  <p class="state">
    {#if healthState === "checking"}
      Checking the core…
    {:else if healthState === "ready"}
      Ready
    {:else}
      Not ready
    {/if}
  </p>
  <p class="detail">{detail}</p>
</main>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
  }

  :global(body) {
    background: #101418;
    color: #e6edf3;
    font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  }

  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    min-height: 100vh;
    padding: 2rem;
    text-align: center;
  }

  h1 {
    margin: 0;
    font-size: 1.75rem;
    font-weight: 600;
  }

  .subtitle {
    margin: 0;
    font-size: 0.875rem;
    color: #7d8b99;
  }

  .state {
    margin: 0;
    font-size: 1rem;
    color: #9fb0c0;
  }

  .detail {
    margin: 0;
    max-width: 32rem;
    font-size: 0.875rem;
    color: #7d8b99;
  }
</style>
