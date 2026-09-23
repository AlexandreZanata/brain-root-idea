<script lang="ts">
  import ActionCard from "./ActionCard.svelte";
  import Button from "./Button.svelte";
  import EmptyState from "./EmptyState.svelte";
  import type { IconName } from "./Icon.svelte";

  const tabs = ["Preview", "Components", "Logs", "AI Notes"];

  const actions: { icon: IconName; title: string; subtitle: string }[] = [
    { icon: "code", title: "Generate a UI", subtitle: "Create a modern UI from a prompt" },
    { icon: "repo", title: "Open a project", subtitle: "Connect an existing folder" },
    { icon: "grid", title: "Use a template", subtitle: "Start from a ready template" }
  ];
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
    <EmptyState
      title="Preview comes in MVP-1."
      description="Local previews and the full Companion Canvas arrive with the next milestone. The agent loop works today."
    />
    <div class="canvas-cards">
      {#each actions as action (action.title)}
        <ActionCard icon={action.icon} title={action.title} subtitle={action.subtitle} />
      {/each}
    </div>
  </div>
</section>

<style>
  .canvas {
    height: 100%;
  }

  .canvas-body {
    display: grid;
    grid-template-rows: 1fr auto;
    gap: 1rem;
    padding: 1.25rem;
  }

  .canvas-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
    gap: 0.6rem;
  }

  @media (max-width: 1080px) {
    .canvas {
      min-height: 24rem;
    }
  }
</style>
