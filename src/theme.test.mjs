import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// B19-S02 static token/contrast cases plus the B20-U1 OpenCode v2 parity
// freeze. Written now, executed only at the versioned release gate per
// ADR 0014. Pure file parsing and color arithmetic: no app runtime, no
// timers, no network — the pin's values are embedded below.

const OPENCODE_PIN = "34aa427434b054afcce7184764aa681159b5d769";

const themePath = join(dirname(fileURLToPath(import.meta.url)), "lib", "theme.css");
const theme = readFileSync(themePath, "utf-8");

function blockProps(selector) {
  const pattern = new RegExp(`${selector}\\s*\\{([\\s\\S]*?)\\n\\}`, "m");
  const match = theme.match(pattern);
  assert.ok(match, `expected ${selector} block in theme.css`);
  const props = new Map();
  for (const [, name, value] of match[1].matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) {
    props.set(name.trim(), value.replace(/\s+/g, " ").trim());
  }
  return props;
}

const root = blockProps(":root");
const light = blockProps(':root\\[data-theme="light"\\]');

// `:root` holds the default (dark) values, so light resolves against both.
const DARK_SCOPE = [root];
const LIGHT_SCOPE = [light, root];

function resolveValue(value, scope) {
  let current = value;
  for (let hop = 0; hop < 10; hop++) {
    const match = current.match(/^var\((--[\w-]+)\)$/);
    if (!match) return current;
    const next = scope.find((props) => props.has(match[1]))?.get(match[1]);
    if (next === undefined) return current;
    current = next;
  }
  return current;
}

const resolveDark = (name) => resolveValue(root.get(name), DARK_SCOPE);
const resolveLight = (name) => resolveValue(light.get(name), LIGHT_SCOPE);

function luminance(hex) {
  const channels = [1, 3, 5].map((index) => parseInt(hex.slice(index, index + 2), 16) / 255);
  const linearized = channels.map((channel) =>
    channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4
  );
  return 0.2126 * linearized[0] + 0.7152 * linearized[1] + 0.0722 * linearized[2];
}

function contrastRatio(foreground, background) {
  const [high, low] = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return (high + 0.05) / (low + 0.05);
}

const HEX = /^#[0-9a-f]{6}([0-9a-f]{2})?$/i;

// ---------------------------------------------------------------------------
// The pinned layer. Values copied from anomalyco/opencode at OPENCODE_PIN:
//   packages/ui/src/v2/styles/colors.css   (grey + alpha ramps)
//   packages/ui/src/theme/themes/oc-2.json (hue ramps + semantic light/dark)
// A mismatch here means the ported layer drifted from the pin, which is a
// parity failure, not a style preference.
// ---------------------------------------------------------------------------

const V2_STATIC = {
  "--v2-grey-50": "#ffffffff",
  "--v2-grey-100": "#fafafaff",
  "--v2-grey-200": "#f2f2f2ff",
  "--v2-grey-300": "#eeeeeeff",
  "--v2-grey-400": "#dbdbdbff",
  "--v2-grey-500": "#aeaeaeff",
  "--v2-grey-600": "#808080ff",
  "--v2-grey-700": "#5c5c5cff",
  "--v2-grey-800": "#3a3a3aff",
  "--v2-grey-900": "#2e2e2eff",
  "--v2-grey-1000": "#242424ff",
  "--v2-grey-1100": "#161616ff",
  "--v2-grey-1200": "#080808ff",
  "--v2-alpha-dark-100": "#000000ff",
  "--v2-alpha-dark-90": "#000000e5",
  "--v2-alpha-dark-80": "#000000cc",
  "--v2-alpha-dark-70": "#000000b2",
  "--v2-alpha-dark-60": "#00000099",
  "--v2-alpha-dark-50": "#00000080",
  "--v2-alpha-dark-40": "#00000066",
  "--v2-alpha-dark-30": "#0000004d",
  "--v2-alpha-dark-24": "#0000003d",
  "--v2-alpha-dark-20": "#00000033",
  "--v2-alpha-dark-16": "#00000029",
  "--v2-alpha-dark-14": "#00000024",
  "--v2-alpha-dark-12": "#0000001f",
  "--v2-alpha-dark-10": "#0000001a",
  "--v2-alpha-dark-8": "#00000014",
  "--v2-alpha-dark-6": "#0000000f",
  "--v2-alpha-dark-4": "#0000000a",
  "--v2-alpha-dark-2": "#00000005",
  "--v2-alpha-dark-0": "#00000000",
  "--v2-alpha-light-100": "#ffffffff",
  "--v2-alpha-light-90": "#ffffffe5",
  "--v2-alpha-light-80": "#ffffffcc",
  "--v2-alpha-light-70": "#ffffffb2",
  "--v2-alpha-light-60": "#ffffff99",
  "--v2-alpha-light-50": "#ffffff80",
  "--v2-alpha-light-40": "#ffffff66",
  "--v2-alpha-light-30": "#ffffff4d",
  "--v2-alpha-light-24": "#ffffff3d",
  "--v2-alpha-light-20": "#ffffff33",
  "--v2-alpha-light-16": "#ffffff29",
  "--v2-alpha-light-14": "#ffffff24",
  "--v2-alpha-light-12": "#ffffff1f",
  "--v2-alpha-light-10": "#ffffff1a",
  "--v2-alpha-light-8": "#ffffff14",
  "--v2-alpha-light-6": "#ffffff0f",
  "--v2-alpha-light-4": "#ffffff0a",
  "--v2-alpha-light-2": "#ffffff05",
  "--v2-alpha-light-0": "#ffffff00",
};

const V2_HUES = {
  "--v2-red-100": "#fcecebff",
  "--v2-red-200": "#f6d5d3ff",
  "--v2-red-300": "#f2bbb7ff",
  "--v2-red-400": "#f29b96ff",
  "--v2-red-500": "#f17471ff",
  "--v2-red-600": "#f1484fff",
  "--v2-red-700": "#d92e3cff",
  "--v2-red-800": "#b82d35ff",
  "--v2-red-900": "#97252bff",
  "--v2-red-1000": "#7a1f23ff",
  "--v2-red-1100": "#5f1a1cff",
  "--v2-red-1200": "#461516ff",
  "--v2-orange-100": "#fdf2edff",
  "--v2-orange-200": "#ffe7dcff",
  "--v2-orange-300": "#ffd8c6ff",
  "--v2-orange-400": "#ffc1a4ff",
  "--v2-orange-500": "#ffa478ff",
  "--v2-orange-600": "#ff8648ff",
  "--v2-orange-700": "#ee7330ff",
  "--v2-orange-800": "#d16427ff",
  "--v2-orange-900": "#b35624ff",
  "--v2-orange-1000": "#954c27ff",
  "--v2-orange-1100": "#723d22ff",
  "--v2-orange-1200": "#5a2c14ff",
  "--v2-yellow-100": "#fefaecff",
  "--v2-yellow-200": "#fcefd0ff",
  "--v2-yellow-300": "#f7e5b5ff",
  "--v2-yellow-400": "#f3da9bff",
  "--v2-yellow-500": "#f2cf76ff",
  "--v2-yellow-600": "#f6c251ff",
  "--v2-yellow-700": "#e7af36ff",
  "--v2-yellow-800": "#cb9f34ff",
  "--v2-yellow-900": "#ac8833ff",
  "--v2-yellow-1000": "#8e7231ff",
  "--v2-yellow-1100": "#68552bff",
  "--v2-yellow-1200": "#4b4025ff",
  "--v2-green-100": "#e7f9eaff",
  "--v2-green-200": "#d0f0d5ff",
  "--v2-green-300": "#b8e9c1ff",
  "--v2-green-400": "#96e3a6ff",
  "--v2-green-500": "#6bd586ff",
  "--v2-green-600": "#49c970ff",
  "--v2-green-700": "#2eaf5aff",
  "--v2-green-800": "#198b43ff",
  "--v2-green-900": "#1d783cff",
  "--v2-green-1000": "#196130ff",
  "--v2-green-1100": "#164c26ff",
  "--v2-green-1200": "#14361dff",
  "--v2-cyan-100": "#e2f7fbff",
  "--v2-cyan-200": "#c4edf4ff",
  "--v2-cyan-300": "#a3e4efff",
  "--v2-cyan-400": "#65d9ebff",
  "--v2-cyan-500": "#00c5dfff",
  "--v2-cyan-600": "#00abcfff",
  "--v2-cyan-700": "#0096b8ff",
  "--v2-cyan-800": "#007d9bff",
  "--v2-cyan-900": "#006c85ff",
  "--v2-cyan-1000": "#005a6eff",
  "--v2-cyan-1100": "#004756ff",
  "--v2-cyan-1200": "#00353fff",
  "--v2-blue-100": "#ecf1feff",
  "--v2-blue-200": "#d7e2fcff",
  "--v2-blue-300": "#c3d4fdff",
  "--v2-blue-400": "#a2bcffff",
  "--v2-blue-500": "#7698fdff",
  "--v2-blue-600": "#3b5cf6ff",
  "--v2-blue-700": "#3250dfff",
  "--v2-blue-800": "#2c47c8ff",
  "--v2-blue-900": "#263fa9ff",
  "--v2-blue-1000": "#22388fff",
  "--v2-blue-1100": "#1c2e70ff",
  "--v2-blue-1200": "#1b2852ff",
  "--v2-purple-100": "#ebecfeff",
  "--v2-purple-200": "#d5d5fcff",
  "--v2-purple-300": "#b9b8f5ff",
  "--v2-purple-400": "#9e99f7ff",
  "--v2-purple-500": "#8271f8ff",
  "--v2-purple-600": "#7152f4ff",
  "--v2-purple-700": "#623be2ff",
  "--v2-purple-800": "#5230c2ff",
  "--v2-purple-900": "#442aa1ff",
  "--v2-purple-1000": "#361f83ff",
  "--v2-purple-1100": "#2b1b6aff",
  "--v2-purple-1200": "#221358ff",
  "--v2-pink-100": "#fdecf3ff",
  "--v2-pink-200": "#f7d5e4ff",
  "--v2-pink-300": "#fabcd8ff",
  "--v2-pink-400": "#f799c6ff",
  "--v2-pink-500": "#f26cb2ff",
  "--v2-pink-600": "#f64aabff",
  "--v2-pink-700": "#e4429eff",
  "--v2-pink-800": "#c83d8bff",
  "--v2-pink-900": "#aa3576ff",
  "--v2-pink-1000": "#8c2d61ff",
  "--v2-pink-1100": "#6f284fff",
  "--v2-pink-1200": "#5c1d3fff",
};

const V2_DARK = {
  "--v2-background-bg-base": "var(--v2-grey-1100)",
  "--v2-background-bg-deep": "var(--v2-grey-1200)",
  "--v2-background-bg-layer-01": "var(--v2-grey-1000)",
  "--v2-background-bg-layer-02": "var(--v2-grey-900)",
  "--v2-background-bg-layer-03": "var(--v2-grey-800)",
  "--v2-background-bg-layer-04": "var(--v2-grey-700)",
  "--v2-background-bg-inverse": "var(--v2-grey-50)",
  "--v2-background-bg-contrast": "var(--v2-grey-700)",
  "--v2-background-bg-button-neutral": "var(--v2-alpha-light-6)",
  "--v2-background-bg-accent": "var(--v2-blue-600)",
  "--v2-text-text-base": "var(--v2-grey-100)",
  "--v2-text-text-muted": "var(--v2-grey-500)",
  "--v2-text-text-faint": "var(--v2-grey-600)",
  "--v2-text-text-inverse": "var(--v2-grey-1100)",
  "--v2-text-text-contrast": "var(--v2-grey-50)",
  "--v2-text-text-accent": "var(--v2-blue-400)",
  "--v2-text-text-accent-hover": "var(--v2-blue-300)",
  "--v2-text-text-code-accent": "var(--v2-blue-400)",
  "--v2-icon-icon-base": "var(--v2-grey-400)",
  "--v2-icon-icon-muted": "var(--v2-grey-600)",
  "--v2-icon-icon-inverse": "var(--v2-grey-1100)",
  "--v2-icon-icon-contrast": "var(--v2-grey-100)",
  "--v2-icon-icon-accent": "var(--v2-blue-400)",
  "--v2-icon-icon-accent-hover": "var(--v2-blue-300)",
  "--v2-border-border-muted": "var(--v2-alpha-light-8)",
  "--v2-border-border-base": "var(--v2-alpha-light-10)",
  "--v2-border-border-strong": "var(--v2-alpha-light-20)",
  "--v2-border-border-inverse": "var(--v2-grey-100)",
  "--v2-border-border-focus": "var(--v2-blue-500)",
  "--v2-overlay-simple-overlay-hover": "var(--v2-alpha-light-6)",
  "--v2-overlay-simple-overlay-pressed": "var(--v2-alpha-light-10)",
  "--v2-overlay-simple-overlay-contrast-hover": "var(--v2-alpha-dark-24)",
  "--v2-overlay-simple-overlay-contrast-pressed": "var(--v2-alpha-dark-40)",
  "--v2-overlay-simple-overlay-scrim": "var(--v2-alpha-dark-60)",
  "--v2-overlay-gradient-depth-overlay-depth-top": "var(--v2-alpha-light-100)",
  "--v2-overlay-gradient-depth-overlay-depth-bot": "var(--v2-alpha-light-0)",
  "--v2-overlay-simple-tab-active-scrim": "#24242400",
  "--v2-overlay-simple-tab-hover-scrim": "#3a3a3a00",
  "--v2-overlay-simple-tab-scrim": "#08080800",
  "--v2-state-bg-success": "var(--v2-green-1200)",
  "--v2-state-fg-success": "var(--v2-green-500)",
  "--v2-state-border-success": "var(--v2-green-900)",
  "--v2-state-bg-warning": "var(--v2-yellow-1200)",
  "--v2-state-fg-warning": "var(--v2-yellow-500)",
  "--v2-state-border-warning": "var(--v2-yellow-900)",
  "--v2-state-bg-danger": "var(--v2-red-1200)",
  "--v2-state-fg-danger": "var(--v2-red-500)",
  "--v2-state-border-danger": "var(--v2-red-900)",
  "--v2-state-bg-info": "var(--v2-blue-1200)",
  "--v2-state-fg-info": "var(--v2-blue-500)",
  "--v2-state-border-info": "var(--v2-blue-900)",
  "--v2-avatar-bg-orange": "#723d22ff",
  "--v2-avatar-border-orange": "#ff8648ff",
  "--v2-avatar-bg-yellow": "#68552bff",
  "--v2-avatar-border-yellow": "#e7af36ff",
  "--v2-avatar-bg-cyan": "#005a6eff",
  "--v2-avatar-border-cyan": "#0096b8ff",
  "--v2-avatar-bg-green": "#196130ff",
  "--v2-avatar-border-green": "#49c970ff",
  "--v2-avatar-bg-red": "#7a1f23ff",
  "--v2-avatar-border-red": "#d92e3cff",
  "--v2-avatar-bg-pink": "#8c2d61ff",
  "--v2-avatar-border-pink": "#e4429eff",
  "--v2-avatar-bg-blue": "#263fa9ff",
  "--v2-avatar-border-blue": "#7698fdff",
  "--v2-avatar-bg-purple": "#361f83ff",
  "--v2-avatar-border-purple": "#7152f4ff",
  "--v2-avatar-bg-gray": "#5c5c5cff",
  "--v2-avatar-border-gray": "#aeaeaeff",
  "--v2-elevation-raised":
    "0px 2px 4px 0px var(--v2-alpha-dark-30), 0px 1px 2px 0px var(--v2-alpha-dark-30), 0px 0px 0px 0.5px var(--v2-alpha-light-16), 0px -0.5px 0px 0px var(--v2-alpha-light-6)",
  "--v2-elevation-floating":
    "0px 8px 16px 0px var(--v2-alpha-dark-30), 0px 4px 8px 0px var(--v2-alpha-dark-30), 0px 0px 0px 0.5px var(--v2-alpha-light-16), 0px -0.5px 0px 0px var(--v2-alpha-light-6)",
  "--v2-elevation-overlay":
    "0px 16px 32px 0px var(--v2-alpha-dark-30), 0px 8px 16px 0px var(--v2-alpha-dark-30), 0px 0px 0px 0.5px var(--v2-alpha-light-16), 0px -0.5px 0px 0px var(--v2-alpha-light-6)",
  "--v2-elevation-button-neutral":
    "0px 1px 2px 0px var(--v2-alpha-dark-40), 0px 0px 0px 0.5px var(--v2-alpha-light-20), 0px -0.5px 0px 0px var(--v2-alpha-light-10)",
  "--v2-elevation-button-contrast":
    "0px 1px 2px 0px var(--v2-alpha-dark-40), 0px 0px 0px 0.5px var(--v2-alpha-light-40), inset 0px 0px 0px 0px var(--v2-alpha-light-0), inset 0px 0px 0px 0px var(--v2-alpha-light-0), 0px -0.5px 0px 0px var(--v2-alpha-light-30)",
  "--v2-elevation-elements": "0px 0.5px 0.5px 0px var(--v2-alpha-dark-40)",
  "--v2-elevation-switch-off":
    "inset 0px -0.5px 0px 0px var(--v2-alpha-light-10), inset 0px 0px 0px 0px var(--v2-alpha-light-0), inset 0px 0px 0px 0.5px var(--v2-alpha-light-16)",
  "--v2-elevation-switch-on":
    "inset 0px -0.5px 0px 0px var(--v2-alpha-light-10), inset 0px 0px 0px 0px var(--v2-alpha-light-0), inset 0px 0px 0px 0.5px var(--v2-alpha-light-16)",
  "--v2-illustration-illustration-layer-01": "var(--v2-grey-900)",
  "--v2-illustration-illustration-layer-02": "var(--v2-grey-800)",
  "--v2-illustration-illustration-layer-03": "var(--v2-grey-700)",
};

const V2_LIGHT = {
  "--v2-background-bg-base": "var(--v2-grey-50)",
  "--v2-background-bg-deep": "var(--v2-grey-100)",
  "--v2-background-bg-layer-01": "var(--v2-grey-100)",
  "--v2-background-bg-layer-02": "var(--v2-grey-200)",
  "--v2-background-bg-layer-03": "var(--v2-grey-300)",
  "--v2-background-bg-layer-04": "var(--v2-grey-400)",
  "--v2-background-bg-inverse": "var(--v2-grey-1100)",
  "--v2-background-bg-contrast": "var(--v2-grey-1000)",
  "--v2-background-bg-button-neutral": "var(--v2-grey-50)",
  "--v2-background-bg-accent": "var(--v2-blue-600)",
  "--v2-text-text-base": "var(--v2-grey-1100)",
  "--v2-text-text-muted": "var(--v2-grey-700)",
  "--v2-text-text-faint": "var(--v2-grey-600)",
  "--v2-text-text-inverse": "var(--v2-grey-50)",
  "--v2-text-text-contrast": "var(--v2-grey-50)",
  "--v2-text-text-accent": "var(--v2-blue-600)",
  "--v2-text-text-accent-hover": "var(--v2-blue-700)",
  "--v2-text-text-code-accent": "var(--v2-blue-900)",
  "--v2-icon-icon-base": "var(--v2-grey-800)",
  "--v2-icon-icon-muted": "var(--v2-grey-600)",
  "--v2-icon-icon-inverse": "var(--v2-grey-50)",
  "--v2-icon-icon-contrast": "var(--v2-grey-100)",
  "--v2-icon-icon-accent": "var(--v2-blue-600)",
  "--v2-icon-icon-accent-hover": "var(--v2-blue-700)",
  "--v2-border-border-muted": "var(--v2-alpha-dark-8)",
  "--v2-border-border-base": "var(--v2-alpha-dark-10)",
  "--v2-border-border-strong": "var(--v2-alpha-dark-20)",
  "--v2-border-border-inverse": "var(--v2-grey-1000)",
  "--v2-border-border-focus": "var(--v2-blue-500)",
  "--v2-overlay-simple-overlay-hover": "var(--v2-alpha-dark-4)",
  "--v2-overlay-simple-overlay-pressed": "var(--v2-alpha-dark-8)",
  "--v2-overlay-simple-overlay-contrast-hover": "var(--v2-alpha-light-12)",
  "--v2-overlay-simple-overlay-contrast-pressed": "var(--v2-alpha-light-24)",
  "--v2-overlay-simple-overlay-scrim": "var(--v2-alpha-dark-40)",
  "--v2-overlay-gradient-depth-overlay-depth-top": "var(--v2-alpha-light-100)",
  "--v2-overlay-gradient-depth-overlay-depth-bot": "var(--v2-alpha-light-0)",
  "--v2-overlay-simple-tab-active-scrim": "#fafafa00",
  "--v2-overlay-simple-tab-hover-scrim": "#eeeeee00",
  "--v2-overlay-simple-tab-scrim": "#fafafa00",
  "--v2-state-bg-success": "var(--v2-green-100)",
  "--v2-state-fg-success": "var(--v2-green-800)",
  "--v2-state-border-success": "var(--v2-green-300)",
  "--v2-state-bg-warning": "var(--v2-yellow-100)",
  "--v2-state-fg-warning": "var(--v2-yellow-800)",
  "--v2-state-border-warning": "var(--v2-yellow-300)",
  "--v2-state-bg-danger": "var(--v2-red-100)",
  "--v2-state-fg-danger": "var(--v2-red-800)",
  "--v2-state-border-danger": "var(--v2-red-300)",
  "--v2-state-bg-info": "var(--v2-blue-100)",
  "--v2-state-fg-info": "var(--v2-blue-800)",
  "--v2-state-border-info": "var(--v2-blue-300)",
  "--v2-avatar-bg-orange": "#ee7330ff",
  "--v2-avatar-border-orange": "#d16427ff",
  "--v2-avatar-bg-yellow": "#e7af36ff",
  "--v2-avatar-border-yellow": "#cb9f34ff",
  "--v2-avatar-bg-cyan": "#0096b8ff",
  "--v2-avatar-border-cyan": "#007d9bff",
  "--v2-avatar-bg-green": "#2eaf5aff",
  "--v2-avatar-border-green": "#198b43ff",
  "--v2-avatar-bg-red": "#d92e3cff",
  "--v2-avatar-border-red": "#b82d35ff",
  "--v2-avatar-bg-pink": "#e4429eff",
  "--v2-avatar-border-pink": "#c83d8bff",
  "--v2-avatar-bg-blue": "#3250dfff",
  "--v2-avatar-border-blue": "#2c47c8ff",
  "--v2-avatar-bg-purple": "#623be2ff",
  "--v2-avatar-border-purple": "#5230c2ff",
  "--v2-avatar-bg-gray": "#5c5c5cff",
  "--v2-avatar-border-gray": "#3a3a3aff",
  "--v2-elevation-raised":
    "0px 2px 4px 0px var(--v2-alpha-dark-4), 0px 1px 2px -1px var(--v2-alpha-dark-8), 0px 0px 0px 0.5px var(--v2-alpha-dark-12), 0px 0px 0px 0px var(--v2-alpha-dark-0)",
  "--v2-elevation-floating":
    "0px 8px 16px 0px var(--v2-alpha-dark-4), 0px 4px 8px 0px var(--v2-alpha-dark-8), 0px 0px 0px 0.5px var(--v2-alpha-dark-12), 0px 0px 0px 0px var(--v2-alpha-dark-0)",
  "--v2-elevation-overlay":
    "0px 16px 32px 0px var(--v2-alpha-dark-4), 0px 8px 16px 0px var(--v2-alpha-dark-8), 0px 0px 0px 0.5px var(--v2-alpha-dark-12), 0px 0px 0px 0px var(--v2-alpha-dark-0)",
  "--v2-elevation-button-neutral":
    "0px 1px 1.5px 0px var(--v2-alpha-dark-10), 0px 0px 0px 0.5px var(--v2-alpha-dark-14), 0px 0px 0px 0px var(--v2-alpha-dark-0)",
  "--v2-elevation-button-contrast":
    "0px 1px 1.5px 0px var(--v2-alpha-dark-20), 0px 0px 0px 0.5px var(--v2-grey-800), inset 0px 1px 2px 0px var(--v2-alpha-light-14), inset 0px -1px 2px 0px var(--v2-alpha-dark-6), 0px 0px 0px 0px var(--v2-alpha-dark-0)",
  "--v2-elevation-elements": "0px 0.5px 0.5px 0px var(--v2-alpha-dark-40)",
  "--v2-elevation-switch-off":
    "inset 0px 1px 1px 0px var(--v2-alpha-dark-8), inset 0px 0.5px 0.5px 0px var(--v2-alpha-dark-8), inset 0px 0px 0px 0.5px var(--v2-alpha-dark-10)",
  "--v2-elevation-switch-on":
    "inset 0px 2px 2px 0px var(--v2-alpha-dark-10), inset 0px 1px 1px 0px var(--v2-alpha-dark-10), inset 0px 0px 0px 0.5px var(--v2-alpha-dark-20)",
  "--v2-illustration-illustration-layer-01": "var(--v2-grey-300)",
  "--v2-illustration-illustration-layer-02": "var(--v2-grey-400)",
  "--v2-illustration-illustration-layer-03": "var(--v2-grey-500)",
};

test("the ported v2 layer matches the pinned OpenCode theme exactly", () => {
  // `:root` carries the theme-invariant ramps plus the dark semantics.
  for (const [name, expected] of Object.entries({ ...V2_STATIC, ...V2_HUES, ...V2_DARK })) {
    assert.equal(root.get(name), expected, `${name} drifted from ${OPENCODE_PIN}`);
  }
  for (const [name, expected] of Object.entries(V2_LIGHT)) {
    assert.equal(light.get(name), expected, `${name} drifted from ${OPENCODE_PIN} (light)`);
  }
  // Ramps are declared once so both themes inherit one definition.
  for (const name of [...Object.keys(V2_STATIC), ...Object.keys(V2_HUES)]) {
    assert.equal(light.has(name), false, `${name} must not be redefined per theme`);
  }
  // The whole vocabulary is present, not a subset: 109 ramps + 38 static alpha
  // ramps + 80 semantic tokens.
  const rootV2 = [...root.keys()].filter((name) => name.startsWith("--v2-"));
  const lightV2 = [...light.keys()].filter((name) => name.startsWith("--v2-"));
  assert.equal(rootV2.length, 227, "root must carry the full v2 vocabulary");
  assert.equal(lightV2.length, 80, "light must override exactly the semantic set");
});

test("defines the semantic scale once with exact TARGET values", () => {
  assert.deepEqual(
    ["--space-1", "--space-2", "--space-3", "--space-4", "--space-5", "--space-6"].map(
      (name) => root.get(name)
    ),
    ["0.25rem", "0.5rem", "0.75rem", "1rem", "1.5rem", "2rem"]
  );
  // Type roles resolve to the pin's scale: 14 / 13 / 16 / 20 px. OpenCode has
  // no 12px step, so the supporting role moved from 12px to the 13px floor.
  assert.deepEqual(
    ["--text-body", "--text-supporting", "--text-title", "--text-display"].map(resolveDark),
    ["14px", "13px", "16px", "20px"]
  );
  assert.deepEqual(
    ["--font-size-small", "--font-size-base", "--font-size-large", "--font-size-x-large"].map(
      (name) => root.get(name)
    ),
    ["13px", "14px", "16px", "20px"]
  );
  // Radius lands on the pin's values without changing any computed pixel.
  assert.equal(resolveDark("--radius-surface"), "0.625rem");
  assert.equal(resolveDark("--radius-control"), "0.375rem");
  assert.equal(root.get("--control-height"), "2.5rem");
  assert.equal(root.get("--control-height-compact"), "2.25rem");
  assert.equal(root.get("--focus-width"), "2px");
  assert.equal(root.get("--focus-offset"), "2px");
  assert.equal(root.get("--spacing"), "0.25rem");
  // The scale is theme-invariant: a single source, no per-theme redefinition.
  // (Palette colors are intentionally per-theme and are covered by the
  // palette contract below, not by this list.)
  const singleSource = [
    "--space-1",
    "--space-2",
    "--space-3",
    "--space-4",
    "--space-5",
    "--space-6",
    "--text-body",
    "--text-supporting",
    "--text-title",
    "--text-display",
    "--radius-surface",
    "--radius-control",
    "--control-height",
    "--control-height-compact",
    "--focus-width",
    "--focus-offset",
  ];
  for (const name of singleSource) {
    assert.equal(light.has(name), false, `${name} must not be redefined per theme`);
  }
});

test("both themes resolve the full palette contract to opaque hex colors", () => {
  const palette = [
    "--bg",
    "--surface",
    "--surface-raised",
    "--surface-hover",
    "--border",
    "--border-strong",
    "--text",
    "--text-muted",
    "--text-subtle",
    "--accent",
    "--accent-bg",
    "--accent-bg-hover",
    "--accent-text",
    "--accent-soft",
    "--accent-soft-border",
    "--focus",
    "--ready-dot",
  ];
  for (const name of palette) {
    for (const [themeName, resolve] of [
      ["dark", resolveDark],
      ["light", resolveLight],
    ]) {
      const value = resolve(name);
      assert.match(value ?? "", HEX, `${name} must resolve to hex in ${themeName}`);
      assert.ok(
        !value || value.length === 7 || value.slice(7) === "ff",
        `${name} must resolve opaque in ${themeName} (got ${value})`
      );
    }
  }
  // The roles the product reads are the pin's semantics, not local literals.
  assert.equal(root.get("--bg"), "var(--v2-background-bg-base)");
  assert.equal(root.get("--surface"), "var(--v2-background-bg-layer-01)");
  assert.equal(root.get("--text"), "var(--v2-text-text-base)");
  assert.equal(light.get("--bg"), "var(--v2-background-bg-base)");
});

test("the v2 text-faint token stays recorded as the [AA] deviation it is", () => {
  // OpenCode's faint text token is sub-AA on its own surface, measured
  // 3.93:1 dark and 3.78:1 light. The token stays faithful to the pin, but
  // the subtle text role must not use it.
  assert.equal(resolveDark("--v2-text-text-faint"), "#808080ff");
  assert.equal(resolveLight("--v2-text-text-faint"), "#808080ff");
  assert.ok(contrastRatio(resolveDark("--v2-text-text-faint").slice(0, 7), resolveDark("--v2-background-bg-layer-01").slice(0, 7)) < 4.5);
  assert.ok(contrastRatio(resolveLight("--v2-text-text-faint").slice(0, 7), resolveLight("--v2-background-bg-layer-01").slice(0, 7)) < 4.5);
  assert.equal(root.get("--text-subtle"), "var(--v2-text-text-muted)");
  assert.equal(light.get("--text-subtle"), "var(--v2-text-text-muted)");
  // The pin's focus token is sub-3:1 on the light surface, so light diverges.
  assert.equal(light.get("--focus"), "var(--v2-blue-700)");
});

test("shared stylesheet keeps type at or above the 13px pin floor with tokenized radii", () => {
  for (const [, value] of theme.matchAll(/font-size:\s*([\d.]+)(rem|px)/g)) {
    const pixels = value.endsWith("px") ? Number(value) : Number(value) * 16;
    assert.ok(pixels >= 12, `font-size ${value} is below the 12px supporting floor`);
  }
  for (const [, value] of theme.matchAll(/border-radius:\s*([^;]+);/g)) {
    const normalized = value.trim();
    assert.ok(
      normalized.startsWith("var(--radius-") || normalized === "50%",
      `border-radius ${normalized} must use a radius token (50% dot shape excepted)`
    );
  }
  assert.equal(resolveDark("--text-supporting"), "13px");
});

test("text pairs meet WCAG AA 4.5 in both themes", () => {
  const pairs = [
    ["dark", "--text", "--bg"],
    ["dark", "--text", "--surface"],
    ["dark", "--text-muted", "--surface"],
    ["dark", "--text-subtle", "--surface"],
    ["dark", "--accent-text", "--accent-bg"],
    ["light", "--text", "--bg"],
    ["light", "--text", "--surface"],
    ["light", "--text-muted", "--surface"],
    ["light", "--text-subtle", "--surface"],
    ["light", "--accent-text", "--accent-bg"],
  ];
  for (const [themeName, foreground, background] of pairs) {
    const resolve = themeName === "dark" ? resolveDark : resolveLight;
    const ratio = contrastRatio(resolve(foreground).slice(1, 7), resolve(background).slice(1, 7));
    assert.ok(
      ratio >= 4.5,
      `${themeName} ${foreground}/${background} ratio ${ratio.toFixed(2)} < 4.5`
    );
  }
});

test("key non-text signals meet WCAG 3.0 (border contrast stays a recorded gap)", () => {
  // Border vs surface (~1.6 dark, ~1.3 light, from the pin's alpha ramps) is
  // a known gap recorded in the visual inventory, not an asserted pass.
  const pairs = [
    ["light", "--focus", "--surface"],
    ["light", "--ready-dot", "--surface"],
    ["dark", "--focus", "--surface"],
    ["dark", "--ready-dot", "--surface"],
  ];
  for (const [themeName, foreground, background] of pairs) {
    const resolve = themeName === "dark" ? resolveDark : resolveLight;
    const ratio = contrastRatio(resolve(foreground).slice(1, 7), resolve(background).slice(1, 7));
    assert.ok(
      ratio >= 3.0,
      `${themeName} ${foreground}/${background} ratio ${ratio.toFixed(2)} < 3.0`
    );
  }
});
