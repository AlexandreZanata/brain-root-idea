<script lang="ts">
  let {
    value = $bindable(368),
    min = 280,
    max = 560,
    step = 8,
    label = "Resize chat panel"
  }: {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
  } = $props();
</script>

<input
  type="range"
  class="br-resizer"
  aria-label={label}
  aria-orientation="vertical"
  {min}
  {max}
  {step}
  bind:value
/>
