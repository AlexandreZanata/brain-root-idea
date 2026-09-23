<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    variant = "secondary",
    type = "button",
    inactive = false,
    title = undefined,
    current = false,
    onclick = undefined,
    children
  }: {
    variant?: "primary" | "secondary" | "ghost" | "row" | "rail" | "tab";
    type?: "button" | "submit";
    inactive?: boolean;
    title?: string;
    current?: boolean;
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();
</script>

<button
  {type}
  class="br-btn br-btn--{variant}"
  aria-disabled={inactive || undefined}
  aria-current={current ? "true" : undefined}
  {title}
  {onclick}
>
  {@render children()}
</button>
