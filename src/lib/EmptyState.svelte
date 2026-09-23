<script lang="ts">
  import Icon, { type IconName } from "./Icon.svelte";

  let {
    icon = "monitor",
    title,
    description
  }: { icon?: IconName; title: string; description: string } = $props();
</script>

<div class="br-empty">
  <span class="br-empty__icon"><Icon name={icon} size="lg" /></span>
  <h3 class="br-empty__title">{title}</h3>
  <p class="br-empty__description">{description}</p>
</div>
