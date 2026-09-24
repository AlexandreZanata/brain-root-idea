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
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
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
    font-size: var(--text-display);
    font-weight: 650;
    line-height: 1.2;
  }

  .tagline {
    margin: 0;
    font-size: var(--text-supporting);
    color: var(--text-muted);
  }

  .core {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .core-text {
    text-align: right;
  }

  .core-status {
    margin: 0;
    font-size: var(--text-body);
    font-weight: 600;
  }

  .core-detail {
    margin: 0;
    font-size: var(--text-supporting);
    color: var(--text-muted);
  }

  @media (max-width: 640px) {
    .chrome {
      padding: var(--space-2) var(--space-3);
    }

    h1 {
      font-size: var(--text-title);
    }

    .tagline,
    .core-detail {
      display: none;
    }
  }
</style>
