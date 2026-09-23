<script lang="ts">
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";

  let { label, onclick }: { label: string; onclick: () => void } = $props();
</script>

<Button variant="row" {onclick}>
  <span>{label}</span>
  <Icon name="chevron" size="sm" />
</Button>
