<script lang="ts">
  import Button from "./Button.svelte";
  import HumanBrowserPanel from "./HumanBrowserPanel.svelte";
  import PreviewPanel from "./PreviewPanel.svelte";

  type CanvasTab = "preview" | "browser";

  const tabs: { id: CanvasTab | null; label: string }[] = [
    { id: "preview", label: "Preview" },
    { id: "browser", label: "Browser" },
    { id: null, label: "Components" },
    { id: null, label: "Logs" },
    { id: null, label: "AI Notes" }
  ];

  let active = $state<CanvasTab>("preview");
</script>

<section class="br-panel canvas" aria-labelledby="canvas-title">
  <div class="br-panel__head br-panel__head--center">
    <div>
      <h2 id="canvas-title" class="br-panel__title">Preview / Canvas</h2>
      <p class="br-panel__subtitle">The result lives here — dominant by design.</p>
    </div>
    <div class="br-tabs" aria-label="Canvas views">
      {#each tabs as tab (tab.label)}
        <Button
          variant="tab"
          inactive={tab.id === null || active !== tab.id}
          current={tab.id === active}
          title={tab.id === null ? `${tab.label} — planned for MVP-1` : tab.label}
          onclick={() => {
            if (tab.id) {
              active = tab.id;
            }
          }}
        >
          {tab.label}
        </Button>
      {/each}
    </div>
  </div>

  <div class="br-panel__body canvas-body">
    {#if active === "preview"}
      <PreviewPanel />
    {:else if active === "browser"}
      <HumanBrowserPanel />
    {/if}
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
