<script lang="ts">
  import Button from "./Button.svelte";
  import PreviewPanel from "./PreviewPanel.svelte";

  const tabs = ["Preview", "Components", "Logs", "AI Notes"];
</script>

<section class="br-panel canvas" aria-labelledby="canvas-title">
  <div class="br-panel__head br-panel__head--center">
    <div>
      <h2 id="canvas-title" class="br-panel__title">Preview / Canvas</h2>
      <p class="br-panel__subtitle">The result lives here — dominant by design.</p>
    </div>
    <div class="br-tabs" aria-label="Canvas views">
      {#each tabs as tab, index (tab)}
        <Button
          variant="tab"
          inactive={index !== 0}
          current={index === 0}
          title={index === 0 ? tab : `${tab} — planned for MVP-1`}
        >
          {tab}
        </Button>
      {/each}
    </div>
  </div>

  <div class="br-panel__body canvas-body">
    <PreviewPanel />
  </div>
</section>

<style>
  .canvas {
    height: 100%;
  }

  .canvas-body {
    display: grid;
    grid-template-rows: minmax(0, 1fr);
    gap: 1rem;
    padding: 1.25rem;
  }

  @media (max-width: 1080px) {
    .canvas {
      min-height: 24rem;
    }
  }
</style>
