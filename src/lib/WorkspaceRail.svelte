<script lang="ts">
  import NavItem from "./NavItem.svelte";
  import type { IconName } from "./Icon.svelte";

  let { sections }: { sections: { id: string; label: string; active: boolean }[] } = $props();

  const icons: Record<string, IconName> = {
    build: "chat",
    agents: "agents",
    browser: "browser",
    files: "files",
    terminal: "terminal",
    settings: "settings"
  };
</script>

<nav class="br-rail" aria-label="Workspace sections">
  {#each sections as section (section.id)}
    <NavItem label={section.label} icon={icons[section.id] ?? "grid"} active={section.active} />
  {/each}
</nav>
