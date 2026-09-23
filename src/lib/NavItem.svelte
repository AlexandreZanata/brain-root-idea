<script lang="ts">
  import Button from "./Button.svelte";
  import Icon, { type IconName } from "./Icon.svelte";

  let { label, icon, active = false }: { label: string; icon: IconName; active?: boolean } =
    $props();
</script>

<Button
  variant="rail"
  inactive={!active}
  current={active}
  title={active ? label : `${label} — planned for MVP-1`}
>
  <Icon name={icon} size="md" />
  <span>{label}</span>
</Button>
