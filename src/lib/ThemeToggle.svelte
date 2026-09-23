<script lang="ts">
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";

  let { theme, ontoggle }: { theme: "dark" | "light"; ontoggle: () => void } = $props();
</script>

<Button
  variant="ghost"
  onclick={ontoggle}
  title={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
>
  <Icon name={theme === "dark" ? "sun" : "moon"} size="sm" />
  <span>{theme === "dark" ? "Light" : "Dark"}</span>
</Button>
