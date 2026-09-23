<script lang="ts">
  import Badge from "./Badge.svelte";
  import Button from "./Button.svelte";
  import Icon, { type IconName } from "./Icon.svelte";

  let {
    icon,
    title,
    subtitle,
    badge = "MVP-1"
  }: { icon: IconName; title: string; subtitle: string; badge?: string } = $props();
</script>

<Button variant="row" inactive title="Planned for MVP-1">
  <span class="br-card__icon"><Icon name={icon} size="md" /></span>
  <span class="br-card__text">
    <span class="br-card__title">{title}</span>
    <span class="br-card__subtitle">{subtitle}</span>
  </span>
  <Badge>{badge}</Badge>
</Button>
