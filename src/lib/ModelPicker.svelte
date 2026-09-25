<script lang="ts">
  import {
    modelDetail,
    modelKey,
    type AgentModelSelection,
    type CatalogModel
  } from "../agentHost";

  let {
    models = [],
    selected = null,
    stale = false,
    inactive = false,
    onselect
  }: {
    models?: CatalogModel[];
    selected?: AgentModelSelection | null;
    stale?: boolean;
    inactive?: boolean;
    onselect: (providerId: string, modelId: string) => void;
  } = $props();

  let currentKey = $derived(
    selected ? `${selected.provider_id}/${selected.model_id}` : ""
  );

  function handleChange(event: Event) {
    if (inactive) {
      return;
    }
    const value = (event.currentTarget as HTMLSelectElement | null)?.value ?? "";
    const found = models.find((entry) => modelKey(entry) === value);
    if (found) {
      onselect(found.provider_id, found.model_id);
    }
  }
</script>

{#if models.length === 0}
  <span class="br-chip" title="Start the sidecar to list models">no models</span>
{:else}
  <select
    class="model-picker"
    aria-label={stale ? "Model (stale catalog)" : "Model"}
    aria-disabled={inactive || undefined}
    value={currentKey}
    onchange={handleChange}
  >
    {#each models as entry (modelKey(entry))}
      <option value={modelKey(entry)} title={modelDetail(entry)}>
        {entry.model_name} · {entry.provider_name}
      </option>
    {/each}
  </select>
  {#if stale}
    <span class="br-chip" title="Catalog offline — prices may be outdated">stale</span>
  {/if}
{/if}

<style>
  .model-picker {
    max-width: 14rem;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--text-supporting);
  }

  .model-picker:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
