<script lang="ts" module>
  export type IconName =
    | "chat"
    | "agents"
    | "browser"
    | "files"
    | "terminal"
    | "settings"
    | "spark"
    | "chevron"
    | "send"
    | "sun"
    | "moon"
    | "monitor"
    | "code"
    | "repo"
    | "grid"
    | "plus"
    | "arrow-up"
    | "stop"
    | "chevron-down";
</script>

<script lang="ts">
  let { name, size = "md" }: { name: IconName; size?: "sm" | "md" | "lg" } = $props();
</script>

<span class="icon icon--{size}" aria-hidden="true">
  {#if name === "chat"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M4 5h16v11H8l-4 3V5z" stroke-linejoin="round" /></svg>
  {:else if name === "agents"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="5" y="7" width="14" height="11" rx="0" /><path d="M12 3v4M9 12h.01M15 12h.01" stroke-linecap="round" /></svg>
  {:else if name === "browser"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><circle cx="12" cy="12" r="8" /><path d="M4 12h16M12 4c2.5 2.4 2.5 13.6 0 16M12 4c-2.5 2.4-2.5 13.6 0 16" /></svg>
  {:else if name === "files"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M4 7a2 2 0 0 1 2-2h3l2 2h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V7z" stroke-linejoin="round" /></svg>
  {:else if name === "terminal"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="4" y="5" width="16" height="14" rx="0" /><path d="m8 10 2.5 2.5L8 15M13 15h3" stroke-linecap="round" stroke-linejoin="round" /></svg>
  {:else if name === "settings"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><circle cx="12" cy="12" r="3" /><path d="M12 4v2M12 18v2M4 12h2M18 12h2M6.3 6.3l1.4 1.4M16.3 16.3l1.4 1.4M6.3 17.7l1.4-1.4M16.3 7.7l1.4-1.4" stroke-linecap="round" /></svg>
  {:else if name === "spark"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M12 4v4M12 16v4M4 12h4M16 12h4M6.5 6.5l2.8 2.8M14.7 14.7l2.8 2.8M6.5 17.5l2.8-2.8M14.7 9.3l2.8-2.8" stroke-linecap="round" /></svg>
  {:else if name === "chevron"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="m9 6 6 6-6 6" stroke-linecap="round" stroke-linejoin="round" /></svg>
  {:else if name === "send"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M5 12h13M13 6l6 6-6 6" stroke-linecap="round" stroke-linejoin="round" /></svg>
  {:else if name === "sun"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><circle cx="12" cy="12" r="4" /><path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M5.6 18.4l1.4-1.4M17 7l1.4-1.4" stroke-linecap="round" /></svg>
  {:else if name === "moon"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z" stroke-linejoin="round" /></svg>
  {:else if name === "monitor"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4"><rect x="3" y="5" width="18" height="12" rx="0" /><path d="M9 20h6M12 17v3" stroke-linecap="round" /></svg>
  {:else if name === "code"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="m9 8-4 4 4 4M15 8l4 4-4 4" stroke-linecap="round" stroke-linejoin="round" /></svg>
  {:else if name === "repo"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M4 12a8 8 0 0 1 8-8h4M20 12a8 8 0 0 1-8 8h-4M14 2l2 2-2 2M10 18l-2 2 2 2" stroke-linecap="round" stroke-linejoin="round" /></svg>
  {:else if name === "plus"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M12 5v14M5 12h14" stroke-linecap="round" /></svg>
  {:else if name === "arrow-up"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 19V6M6 12l6-6 6 6" stroke-linecap="round" stroke-linejoin="round" /></svg>
  {:else if name === "stop"}
    <svg viewBox="0 0 24 24" fill="currentColor" stroke="none"><rect x="7" y="7" width="10" height="10" rx="1.5" /></svg>
  {:else if name === "chevron-down"}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="m6 9 6 6 6-6" stroke-linecap="round" stroke-linejoin="round" /></svg>
  {:else}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="4" y="4" width="7" height="7" rx="0" /><rect x="13" y="4" width="7" height="7" rx="0" /><rect x="4" y="13" width="7" height="7" rx="0" /><rect x="13" y="13" width="7" height="7" rx="0" /></svg>
  {/if}
</span>

<style>
  .icon {
    display: inline-flex;
    flex-shrink: 0;
  }

  .icon svg {
    width: 100%;
    height: 100%;
    display: block;
  }

  .icon--sm {
    width: 0.9rem;
    height: 0.9rem;
  }

  .icon--md {
    width: 1.15rem;
    height: 1.15rem;
  }

  .icon--lg {
    width: 1.6rem;
    height: 1.6rem;
  }
</style>
