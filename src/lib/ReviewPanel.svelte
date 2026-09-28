<script lang="ts">
  /**
   * B20-U7 — the review tab, on the pinned `review-tab.tsx` shape (@ 34aa427):
   * a title header, a diff-style selector, and the content region on the pin's
   * `pr-3 / px-3 / pl-3` rhythm. The pin renders `SessionReview` diffs and
   * line comments; there is no diff source in BrainRoot yet (no workspace, no
   * VCS bridge), so the content region is the honest empty state and no diff
   * is fabricated to fill it. The style selector only renders when there is
   * something to restyle — a control with nothing to act on is a fake control.
   */
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";
  import EmptyState from "./EmptyState.svelte";

  let {
    diffs = [],
    diffStyle = "unified",
    ondiffstyle,
    children
  }: {
    /** Real diffs only; empty today, which is the point. */
    diffs: { path: string }[];
    diffStyle?: "unified" | "split";
    ondiffstyle?: (style: "unified" | "split") => void;
    children?: Snippet;
  } = $props();
</script>

<div class="review">
  <div class="review__header">
    <h3 class="review__title">Review</h3>
    {#if diffs.length > 0 && ondiffstyle}
      <div class="review__styles" role="group" aria-label="Diff style">
        <Button
          variant="secondary"
          current={diffStyle === "unified"}
          onclick={() => ondiffstyle("unified")}>Unified</Button
        >
        <Button
          variant="secondary"
          current={diffStyle === "split"}
          onclick={() => ondiffstyle("split")}>Split</Button
        >
      </div>
    {/if}
  </div>

  <div class="review__body">
    {#if diffs.length === 0}
      <EmptyState
        icon="repo"
        title="Nothing to review yet"
        description="The review tab collects the changes a turn makes to your project. It fills in when a workspace exists to change."
      />
    {:else}
      {@render children?.()}
    {/if}
  </div>
</div>

<style>
  .review {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    padding-inline-end: 12px;
  }

  .review__header {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
    padding-inline: 12px;
  }

  .review__title {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    line-height: 20px;
    letter-spacing: -0.04px;
    color: var(--text);
  }

  .review__styles {
    display: flex;
    gap: 4px;
    margin-inline-start: auto;
  }

  .review__body {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding-inline-start: 12px;
  }
</style>
