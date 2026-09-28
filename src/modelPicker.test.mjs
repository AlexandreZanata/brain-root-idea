import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { modelRowMeta, modelTooltipDetail } from "./agentHost.ts";
import {
  EMPTY_CATALOG_LABEL,
  NO_MATCH_LABEL,
  compactModelSearch,
  countLabel,
  filterModels,
  findOption,
  groupModels,
  initialActive,
  matchesModelSearch,
  moveActive,
  normalizeModelSearch,
  optionKeys,
  pickerKeyIntent
} from "./modelPicker.ts";

// B20-U5. The matcher, the orders, the active-row arithmetic, and the key intent
// are imported and exercised directly; the picker's markup and its
// no-fabrication rules are asserted against the component source. The pinned
// geometry (284 px layer, 28 px search row, 220 px scroll) is layout, so it is
// read from the markup.
//
// Deferred to the versioned release gate (ADR 0014): case IDs B20-U5-T01…T06
// live on issue #134.

const here = dirname(fileURLToPath(import.meta.url));
const pickerSource = readFileSync(join(here, "lib", "ModelPicker.svelte"), "utf8");

const entry = (provider_id, provider_name, model_id, model_name, over = {}) => ({
  provider_id,
  provider_name,
  model_id,
  model_name,
  context_length: 128000,
  prompt_usd_per_m: 3,
  completion_usd_per_m: 15,
  ...over
});

const CATALOG = [
  entry("openai", "OpenAI", "gpt-4.1", "GPT-4.1"),
  entry("anthropic", "Anthropic", "claude-sonnet-4-20250514", "Claude Sonnet 4"),
  entry("anthropic", "Anthropic", "claude-3-5-haiku", "Claude 3.5 Haiku"),
  entry("zai", "Z.ai", "glm-4.6", "GLM-4.6"),
  entry("openrouter", "OpenRouter", "meta-llama/Llama-3-70B", "Llama 3 70B")
];

test("the normalizer drops punctuation and collapses space", () => {
  assert.equal(normalizeModelSearch("  Claude-3.5!!  Sonnet "), "claude 3 5 sonnet");
  assert.equal(compactModelSearch("Claude-3.5 Sonnet"), "claude35sonnet");
  assert.equal(normalizeModelSearch(""), "");
  assert.equal(compactModelSearch("!!!"), "");
});

test("the matcher accepts a compact or a spaced query and ignores case", () => {
  assert.equal(matchesModelSearch("claude3.5", ["Claude 3.5 Haiku"]), true);
  assert.equal(matchesModelSearch("Claude 3.5", ["claude35-haiku"]), true);
  assert.equal(matchesModelSearch("HAIKU", ["Claude 3.5 Haiku"]), true);
  assert.equal(matchesModelSearch("gemini", ["Claude 3.5 Haiku"]), false);
  assert.equal(matchesModelSearch("", ["anything"]), true);
  assert.equal(matchesModelSearch("   ", ["anything"]), true);
  // Any one of the searched values may match: here the provider does.
  assert.equal(matchesModelSearch("openrouter", ["Llama 3 70B", "openrouter/meta-llama"]), true);
});

test("filtering searches name, id, and provider, and sorts by model name", () => {
  assert.deepEqual(
    filterModels(CATALOG, "").map((item) => item.model_name),
    ["Claude 3.5 Haiku", "Claude Sonnet 4", "GLM-4.6", "GPT-4.1", "Llama 3 70B"]
  );
  assert.deepEqual(
    filterModels(CATALOG, "anthropic").map((item) => item.model_id),
    ["claude-3-5-haiku", "claude-sonnet-4-20250514"]
  );
  assert.deepEqual(
    filterModels(CATALOG, "llama").map((item) => item.model_id),
    ["meta-llama/Llama-3-70B"]
  );
  assert.deepEqual(
    filterModels(CATALOG, "gpt4").map((item) => item.model_id),
    ["gpt-4.1"]
  );
  assert.deepEqual(filterModels(CATALOG, "nothing-here"), []);
});

test("groups are ordered by provider name and rows by model name", () => {
  const groups = groupModels(CATALOG, "");
  assert.deepEqual(
    groups.map((group) => group.providerName),
    ["Anthropic", "OpenAI", "OpenRouter", "Z.ai"]
  );
  assert.deepEqual(
    groups[0].options.map((option) => option.label),
    ["Claude 3.5 Haiku", "Claude Sonnet 4"]
  );
  const keys = optionKeys(groups);
  assert.equal(keys.length, CATALOG.length);
  assert.equal(keys[0], "anthropic/claude-3-5-haiku");
  assert.equal(findOption(groups, "zai/glm-4.6")?.label, "GLM-4.6");
  assert.equal(findOption(groups, "missing/key"), null);
  assert.equal(findOption(groups, null), null);
});

test("a model id that contains a slash stays one option", () => {
  const groups = groupModels(CATALOG, "llama");
  const option = groups[0].options[0];
  assert.equal(option.providerId, "openrouter");
  assert.equal(option.modelId, "meta-llama/Llama-3-70B");
  assert.equal(option.key, "openrouter/meta-llama/Llama-3-70B");
});

test("the current model is active when it survives, otherwise the first row", () => {
  const keys = ["a", "b", "c"];
  assert.equal(initialActive(keys, "b"), "b");
  assert.equal(initialActive(keys, "gone"), "a");
  assert.equal(initialActive(keys, null), "a");
  assert.equal(initialActive([], "b"), null);
});

test("arrow navigation wraps at both ends", () => {
  const keys = ["a", "b", "c"];
  assert.equal(moveActive(keys, "a", 1), "b");
  assert.equal(moveActive(keys, "c", 1), "a");
  assert.equal(moveActive(keys, "a", -1), "c");
  assert.equal(moveActive(keys, "b", -1), "a");
  // An unknown active row starts at the first option.
  assert.equal(moveActive(keys, "gone", 1), "a");
  assert.equal(moveActive(keys, null, 1), "a");
  assert.equal(moveActive(["only"], "only", 1), "only");
  assert.equal(moveActive([], null, 1), null);
});

test("the key intent matches the pinned handler", () => {
  assert.deepEqual(pickerKeyIntent({ key: "Tab" }), { kind: "none" });
  assert.deepEqual(pickerKeyIntent({ key: "Escape" }), { kind: "close" });
  assert.deepEqual(pickerKeyIntent({ key: "ArrowDown" }), { kind: "move", delta: 1 });
  assert.deepEqual(pickerKeyIntent({ key: "ArrowUp" }), { kind: "move", delta: -1 });
  assert.deepEqual(pickerKeyIntent({ key: "Enter" }), { kind: "select" });
  assert.deepEqual(pickerKeyIntent({ key: "Enter", isComposing: true }), { kind: "none" });
  assert.deepEqual(pickerKeyIntent({ key: "Enter", altKey: true }), { kind: "none" });
  assert.deepEqual(pickerKeyIntent({ key: "ArrowDown", metaKey: true }), { kind: "none" });
  assert.deepEqual(pickerKeyIntent({ key: "a" }), { kind: "none" });
  // Home/End are this stage's addition to the pinned set.
  assert.deepEqual(pickerKeyIntent({ key: "Home" }), { kind: "edge", edge: "first" });
  assert.deepEqual(pickerKeyIntent({ key: "End" }), { kind: "edge", edge: "last" });
});

test("the announced count names models, never the rows", () => {
  assert.equal(countLabel(1, 1, false), "1 model");
  assert.equal(countLabel(460, 460, false), "460 models");
  assert.equal(countLabel(0, 460, true), "0 of 460 models match");
  assert.equal(countLabel(3, 460, true), "3 of 460 models match");
  assert.equal(countLabel(1, 1, true), "1 of 1 model match");
});

test("unknown context and price stay question marks, never a number", () => {
  const unknown = entry("p", "Provider", "m", "Model", {
    context_length: null,
    prompt_usd_per_m: null,
    completion_usd_per_m: null
  });
  assert.equal(modelRowMeta(unknown), "? ctx · ?/M in");
  assert.equal(modelTooltipDetail(unknown), "Provider · ? ctx · ?/M in · ?/M out");
  const known = entry("p", "Provider", "m", "Model");
  assert.equal(modelRowMeta(known), "128k ctx · $3.00/M in");
  assert.equal(modelTooltipDetail(known), "Provider · 128k ctx · $3.00/M in · $15.00/M out");
});

test("every option carries the data the row and the tooltip need", () => {
  const option = groupModels(CATALOG, "").flatMap((group) => group.options)[0];
  assert.equal(option.label, "Claude 3.5 Haiku");
  assert.equal(option.meta, "128k ctx · $3.00/M in");
  assert.equal(option.detail, "Anthropic · 128k ctx · $3.00/M in · $15.00/M out");
  // The empty state must not pretend a catalog exists.
  assert.equal(EMPTY_CATALOG_LABEL.includes("Start the sidecar"), true);
  assert.equal(NO_MATCH_LABEL, "No models match.");
});

test("the picker keeps the repository's accessibility rules", () => {
  assert.equal(/(?<!aria-)\bdisabled(?:=|\s|>)/.test(pickerSource), false);
  assert.equal(pickerSource.includes("user-select"), false);
  assert.equal(/outline\s*:\s*none/.test(pickerSource), false);
  assert.equal(pickerSource.includes(":focus-visible"), true);
  assert.equal(pickerSource.includes("prefers-reduced-motion"), true);
  assert.equal(pickerSource.includes('role="listbox"'), true);
  assert.equal(pickerSource.includes('role="option"'), true);
  assert.equal(pickerSource.includes('role="group"'), true);
  assert.equal(pickerSource.includes("aria-activedescendant"), true);
  assert.equal(pickerSource.includes('aria-live="polite"'), true);
  assert.equal(pickerSource.includes('aria-haspopup="listbox"'), true);
  assert.equal(pickerSource.includes("aria-selected"), true);
  // The native control this stage replaced is gone, and with it the undefined
  // `--radius` that U2 and U4 both flagged.
  assert.equal(pickerSource.includes("<select"), false);
  assert.equal(/var\(--radius\)/.test(pickerSource), false);
});

test("the picker keeps the pinned geometry and invents no capability", () => {
  assert.equal(/width:\s*284px/.test(pickerSource), true);
  assert.equal(/height:\s*28px/.test(pickerSource), true);
  assert.equal(/max-height:\s*220px/.test(pickerSource), true);
  assert.equal(pickerSource.includes("--v2-elevation-floating"), true);
  assert.equal(pickerSource.includes("--v2-background-bg-layer-01"), true);
  for (const absent of ["Manage models", "<img", "provider-icons", ">Free<", ">Latest<"]) {
    assert.equal(pickerSource.includes(absent), false, `${absent} must not be built`);
  }
});
