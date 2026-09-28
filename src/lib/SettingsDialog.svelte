<script lang="ts">
  /**
   * B20-U6 — settings, on the pinned `dialog-settings-v2` shape (@ 34aa427):
   * an `x-large` settings-variant dialog with a vertical tab list grouped into
   * `Desktop` and `Server` sections, a nav footer carrying the app name and
   * version, and panels built from `settings-v2-section` / `-list` / `-row`
   * geometry.
   *
   * The pin's `TabsV2` is a roving-focus primitive and the gate forbids focus
   * index overrides outright, so navigation is real buttons with `aria-current` — the
   * same correction U2 made. Theme is the one row with a backend (it already
   * persists to this device); every other row is read-only and says so, because
   * a control that saves nothing is a fake control.
   */
  import Dialog from "./Dialog.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";
  import { SETTINGS_READONLY_NOTE, settingsTabs, type SettingsData } from "../dialogs";

  let {
    data,
    ontoggletheme,
    onclose
  }: {
    data: SettingsData;
    ontoggletheme: () => void;
    onclose: () => void;
  } = $props();

  const tabs = $derived(settingsTabs(data));
  let current = $state("general");
  const currentTab = $derived(tabs.find((tab) => tab.id === current) ?? tabs[0]);
  const sections = $derived(
    [...new Set(tabs.map((tab) => tab.section))].map((section) => ({
      section,
      tabs: tabs.filter((tab) => tab.section === section)
    }))
  );
</script>

<Dialog title="Settings" label="Settings" size="x-large" variant="settings" {onclose}>
  <div class="settings">
    <div class="nav" role="group" aria-label="Settings sections">
      {#each sections as block (block.section)}
        <div class="nav-section">
          <p class="nav-section-title">{block.section}</p>
          <div class="nav-items">
            {#each block.tabs as tab (tab.id)}
              <button
                type="button"
                class="nav-item"
                aria-current={tab.id === current ? "true" : undefined}
                onclick={() => (current = tab.id)}
              >
                {tab.label}
              </button>
            {/each}
          </div>
        </div>
      {/each}
      <div class="nav-footer">
        <span>BrainRoot</span>
        <span>MVP-0</span>
      </div>
    </div>

    <div class="panel">
      <div class="panel-header">
        <h3 class="panel-title">{currentTab.label}</h3>
      </div>
      <div class="panel-body">
        <div class="list">
          {#each currentTab.rows as row (row.id)}
            <div class="row">
              <div class="row-copy">
                <span class="row-title">{row.title}</span>
                <span class="row-description">{row.description}</span>
              </div>
              <div class="row-control">
                {#if row.id === "theme"}
                  <ThemeToggle theme={data.theme} ontoggle={ontoggletheme} />
                {:else if row.value !== undefined}
                  <span class="row-value">{row.value}</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
        {#if currentTab.rows.every((row) => row.readonly)}
          <p class="readonly-note">{SETTINGS_READONLY_NOTE}</p>
        {/if}
      </div>
    </div>
  </div>
</Dialog>

<style>
  .settings {
    display: flex;
    min-height: 0;
    height: 100%;
  }

  .nav {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    width: 240px;
    min-width: 200px;
    flex-shrink: 0;
    padding: 16px 8px;
    background: var(--v2-background-bg-layer-01);
    overflow-y: auto;
  }

  .nav-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 0 4px 12px;
  }

  .nav-section-title {
    margin: 0;
    font-size: 13px;
    font-weight: 500;
    line-height: 16px;
    color: var(--text-muted);
  }

  .nav-items {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    width: 100%;
    padding: 6px 8px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    font-weight: 500;
    line-height: 16px;
    text-align: left;
    cursor: pointer;
  }

  .nav-item:hover {
    background: var(--surface-hover);
  }

  .nav-item[aria-current="true"] {
    background: var(--surface);
    color: var(--accent);
  }

  .nav-footer {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 4px;
  }

  .nav-footer span {
    font-size: 11px;
    font-weight: 400;
    line-height: 1;
    color: var(--text-subtle);
  }

  .panel {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    overflow-y: auto;
  }

  .panel-header {
    position: sticky;
    top: 0;
    padding: 40px 40px 32px;
    background: var(--v2-background-bg-base);
  }

  .panel-title {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    line-height: 1;
    color: var(--text);
  }

  .panel-body {
    display: flex;
    flex-direction: column;
    gap: 36px;
    padding: 0 40px 40px;
  }

  .list {
    border-radius: 8px;
    background: var(--v2-background-bg-layer-01);
    padding-inline: 20px;
    box-shadow: inset 0 0 0 0.5px var(--v2-border-border-muted);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    padding-block: 20px;
    border-bottom: 0.5px solid var(--v2-border-border-base);
  }

  .row:last-child {
    border-bottom: 0;
  }

  .row-copy {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .row-title {
    font-size: 13px;
    font-weight: 500;
    line-height: 1;
    letter-spacing: -0.04px;
    color: var(--text);
  }

  .row-description {
    font-size: 13px;
    font-weight: 400;
    line-height: 20px;
    color: var(--text-muted);
  }

  .row-control {
    display: flex;
    justify-content: flex-end;
  }

  .row-value {
    font-size: 13px;
    font-weight: 500;
    line-height: 16px;
    color: var(--text);
    white-space: nowrap;
  }

  .readonly-note {
    margin: 0;
    font-size: 13px;
    line-height: 20px;
    color: var(--text-subtle);
  }

  .nav-item:focus-visible {
    outline: var(--focus-width) solid var(--focus);
    outline-offset: var(--focus-offset);
  }

  @media (max-width: 639px) {
    .nav {
      width: 144px;
      min-width: 144px;
    }

    .panel-header {
      padding: 24px 20px 20px;
    }

    .panel-body {
      padding: 0 20px 24px;
    }
  }
</style>
