<script lang="ts">
  import ThemeToggle from "./ThemeToggle.svelte";

  let {
    healthState,
    detail,
    theme,
    ontoggle
  }: {
    healthState: "checking" | "ready" | "failed";
    detail: string;
    theme: "dark" | "light";
    ontoggle: () => void;
  } = $props();
</script>

<header class="chrome">
  <div class="brand">
    <span class="brand-mark" aria-hidden="true">BR</span>
    <div class="brand-text">
      <h1>BrainRoot</h1>
      <p class="tagline">Experimental Linux model loop · MVP-0</p>
    </div>
  </div>
  <div class="core">
    <span
      class="br-status-dot"
      class:br-status-dot--ready={healthState === "ready"}
      aria-hidden="true"
    ></span>
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
    <ThemeToggle {theme} {ontoggle} />
  </div>
</header>

<style>
  .chrome {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.7rem 1.25rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
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
    border-radius: 0;
    background: var(--accent-bg);
    color: var(--accent-text);
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
    color: var(--text-muted);
  }

  .core {
    display: flex;
    align-items: center;
    gap: 0.55rem;
  }

  .core-text {
    text-align: right;
  }

  .core-status {
    margin: 0;
    font-size: 0.82rem;
    font-weight: 600;
  }

  .core-detail {
    margin: 0;
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  @media (max-width: 640px) {
    .chrome {
      padding: 0.55rem 0.8rem;
    }

    h1 {
      font-size: 0.95rem;
    }

    .tagline,
    .core-detail {
      display: none;
    }
  }
</style>
