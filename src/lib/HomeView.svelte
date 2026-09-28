<script lang="ts">
  /**
   * B20-U6 — the home/projects view. The pin's home lists projects and starts
   * sessions; no workspace or project listing exists here (the gap U4 recorded
   * for the `@` picker), so projects is an honest empty state and the real
   * content is the sessions this app actually has.
   *
   * Planned destinations use the existing `ActionCard` `MVP-1` badge
   * convention — declared, not faked. Nothing in this view opens a file,
   * folder, or terminal, because none of those capabilities exist yet.
   */
  import ActionCard from "./ActionCard.svelte";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import EmptyState from "./EmptyState.svelte";
  import WelcomeCard from "./WelcomeCard.svelte";

  let {
    sessions,
    onnew,
    onselect,
    onclose
  }: {
    sessions: { id: number; title: string; turnCount: number }[];
    onnew: () => void;
    onselect: (id: number) => void;
    onclose: () => void;
  } = $props();
</script>

<Dialog title="Home" label="Home" size="large" fit {onclose}>
  <div class="home">
    <WelcomeCard />

    <section class="home-section" aria-labelledby="home-sessions">
      <h3 class="home-section-title" id="home-sessions">Sessions</h3>
      <div class="home-list">
        {#each sessions as session (session.id)}
          <Button variant="row" onclick={() => onselect(session.id)}>
            <span class="home-row-text">
              <span class="home-row-title">{session.title}</span>
              <span class="home-row-subtitle">
                {session.turnCount === 1 ? "1 turn" : `${session.turnCount} turns`}
              </span>
            </span>
          </Button>
        {/each}
        <Button variant="row" onclick={onnew}>
          <span class="home-row-text">
            <span class="home-row-title">New session</span>
            <span class="home-row-subtitle">Start a blank conversation</span>
          </span>
        </Button>
      </div>
    </section>

    <section class="home-section" aria-labelledby="home-projects">
      <h3 class="home-section-title" id="home-projects">Projects</h3>
      <EmptyState
        icon="repo"
        title="No project is open"
        description="BrainRoot works inside one conversation for now. Project and workspace listings arrive in MVP-1."
      />
    </section>

    <section class="home-section" aria-labelledby="home-planned">
      <h3 class="home-section-title" id="home-planned">Planned</h3>
      <div class="home-list">
        <ActionCard icon="files" title="Files" subtitle="Browse and edit the workspace" />
        <ActionCard icon="terminal" title="Terminal" subtitle="Run commands next to the build" />
        <ActionCard icon="repo" title="Changes" subtitle="Review what the agent changed" />
      </div>
    </section>
  </div>
</Dialog>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: 36px;
    padding: 24px 32px 32px;
  }

  .home-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .home-section-title {
    margin: 0;
    padding-bottom: 8px;
    font-size: 15px;
    font-weight: 600;
    line-height: 1;
    color: var(--text);
  }

  .home-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .home-row-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .home-row-title {
    font-size: 13px;
    font-weight: 500;
    line-height: 16px;
    color: var(--text);
  }

  .home-row-subtitle {
    font-size: 13px;
    font-weight: 400;
    line-height: 16px;
    color: var(--text-muted);
  }
</style>
