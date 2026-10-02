/**
 * B20-U5 — the model picker's pure model: the search matcher, the grouping and
 * ordering rules, the active-row arithmetic, and the key intent.
 *
 * The matcher is ported character for character from the pin
 * (`packages/app/src/components/dialog-select-model-search.ts` @ 34aa427) and
 * the ordering and navigation come from `ModelSelectorPopoverV2View` in
 * `components/dialog-select-model.tsx`. It lives in a plain `.ts` module so
 * `src/modelPicker.test.mjs` can freeze the behavior without a DOM. Formatting
 * stays in `agentHost.ts` with the other presentation helpers.
 */
import { modelKey, modelRowMeta, modelTooltipDetail, type CatalogModel } from "./agentHost.ts";

/** The pin's normalizer: lowercase, drop punctuation, collapse space. */
export function normalizeModelSearch(value: string): string {
  return value
    .toLowerCase()
    .replace(/[^\p{Letter}\p{Number}]+/gu, " ")
    .trim()
    .replace(/\s+/g, " ");
}

export function compactModelSearch(value: string): string {
  return normalizeModelSearch(value).replaceAll(" ", "");
}

/**
 * A value matches when either its normalized or its compact form contains the
 * query's corresponding form, which is what makes `claude3.5` and `claude 3.5`
 * both match. An empty query matches everything.
 */
export function matchesModelSearch(query: string, values: string[]): boolean {
  const search = normalizeModelSearch(query);
  if (!search) {
    return true;
  }
  const compact = compactModelSearch(query);
  return values.some(
    (value) =>
      normalizeModelSearch(value).includes(search) ||
      compactModelSearch(value).includes(compact)
  );
}

export type ModelOption = {
  /** `provider/model`, the same key `agentHost.modelKey` already produced. */
  key: string;
  /** Carried separately because a model id can itself contain a slash. */
  providerId: string;
  modelId: string;
  label: string;
  /** The muted part of the row: `128k ctx · $3.00/M in`, unknowns preserved. */
  meta: string;
  /** The active row's description: the full detail line. */
  detail: string;
};

export type ModelGroup = {
  providerId: string;
  providerName: string;
  options: ModelOption[];
};

function toOption(entry: CatalogModel): ModelOption {
  return {
    key: modelKey(entry),
    providerId: entry.provider_id,
    modelId: entry.model_id,
    label: entry.model_name,
    meta: modelRowMeta(entry),
    detail: modelTooltipDetail(entry)
  };
}

/** The pin searches name, id, and provider name, then sorts by model name. */
export function filterModels(models: CatalogModel[], query: string): CatalogModel[] {
  const matched = query.trim()
    ? models.filter((entry) =>
        matchesModelSearch(query, [entry.model_name, entry.model_id, entry.provider_name])
      )
    : models;
  return [...matched].sort((a, b) => a.model_name.localeCompare(b.model_name));
}

/**
 * The pin groups the name-sorted list by provider and sorts the groups by
 * provider name. Its `sortModelGroups` ranks a hardcoded popular-provider list
 * first; BrainRoot has no such list, so alphabetical order is the whole rule
 * here rather than a silent half of one.
 */
export function groupModels(models: CatalogModel[], query = ""): ModelGroup[] {
  const byProvider = new Map<string, ModelGroup>();
  for (const entry of filterModels(models, query)) {
    const group = byProvider.get(entry.provider_id) ?? {
      providerId: entry.provider_id,
      providerName: entry.provider_name,
      options: []
    };
    group.options.push(toOption(entry));
    byProvider.set(entry.provider_id, group);
  }
  return [...byProvider.values()].sort((a, b) => a.providerName.localeCompare(b.providerName));
}

export function optionKeys(groups: ModelGroup[]): string[] {
  return groups.flatMap((group) => group.options.map((option) => option.key));
}

export function findOption(groups: ModelGroup[], key: string | null): ModelOption | null {
  if (!key) {
    return null;
  }
  for (const group of groups) {
    const found = group.options.find((option) => option.key === key);
    if (found) {
      return found;
    }
  }
  return null;
}

/** The pin opens with the current model active when it survives, else the first row. */
export function initialActive(keys: string[], currentKey: string | null): string | null {
  if (currentKey && keys.includes(currentKey)) {
    return currentKey;
  }
  return keys[0] ?? null;
}

/**
 * The pin's `moveActive`, the same arithmetic as the palette's: naive modulo
 * on the active index, where a missing row counts as index -1 — so the move
 * wraps at both ends and an unknown active row lands on the first option
 * instead of skipping it.
 */
export function moveActive(keys: string[], active: string | null, delta: 1 | -1): string | null {
  if (keys.length === 0) {
    return null;
  }
  const index = active ? keys.indexOf(active) : -1;
  return keys[(index + delta + keys.length) % keys.length] ?? null;
}

export type PickerKeyIntent =
  | { kind: "none" }
  | { kind: "close" }
  | { kind: "select" }
  | { kind: "move"; delta: 1 | -1 }
  | { kind: "edge"; edge: "first" | "last" };

export type PickerKey = {
  key: string;
  altKey?: boolean;
  metaKey?: boolean;
  ctrlKey?: boolean;
  isComposing?: boolean;
};

/**
 * The pin's search-field handler: `Tab` is left to the browser, `Escape` closes,
 * the arrows move, `Enter` selects, and alt/meta modified keys are ignored.
 * Home/End are this stage's addition — the issue's accessibility contract names
 * them and they are standard listbox behavior.
 */
export function pickerKeyIntent(event: PickerKey): PickerKeyIntent {
  if (event.key === "Tab") {
    return { kind: "none" };
  }
  if (event.key === "Escape") {
    return { kind: "close" };
  }
  if (event.altKey || event.metaKey) {
    return { kind: "none" };
  }
  if (event.key === "ArrowDown") {
    return { kind: "move", delta: 1 };
  }
  if (event.key === "ArrowUp") {
    return { kind: "move", delta: -1 };
  }
  if (event.key === "Home") {
    return { kind: "edge", edge: "first" };
  }
  if (event.key === "End") {
    return { kind: "edge", edge: "last" };
  }
  if (event.key === "Enter" && !event.isComposing) {
    return { kind: "select" };
  }
  return { kind: "none" };
}

export const NO_MATCH_LABEL = "No models match.";
export const EMPTY_CATALOG_LABEL = "No models are available. Start the sidecar to list models.";
export const PICKER_EMPTY_CHIP = "no models";
export const PICKER_STALE_CHIP = "stale";

/**
 * The count is announced instead of the rows, so a 460-model catalog does not
 * flood a screen reader with every option as the filter changes.
 */
export function countLabel(matched: number, total: number, filtering: boolean): string {
  const models = total === 1 ? "model" : "models";
  if (filtering) {
    return `${matched} of ${total} ${models} match`;
  }
  return `${total} ${models}`;
}
