<script lang="ts">
  const productName = "BrainRoot";
</script>

<main>
  <h1>{productName}</h1>
  <p class="state">Experimental Linux setup</p>
  <p class="detail">
    The agent surface and Companion Canvas arrive in later batches.
  </p>
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
