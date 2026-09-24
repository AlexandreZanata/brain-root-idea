import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// B19-S02 static token/contrast cases (written now, executed only at the B19
// release gate per ADR 0014). Pure file parsing and color arithmetic: no app
// runtime, no timers, no network.

const themePath = join(dirname(fileURLToPath(import.meta.url)), "lib", "theme.css");
const theme = readFileSync(themePath, "utf-8");

function blockProps(selector) {
  const pattern = new RegExp(`${selector}\\s*\\{([\\s\\S]*?)\\}`, "m");
  const match = theme.match(pattern);
  assert.ok(match, `expected ${selector} block in theme.css`);
  const props = new Map();
  for (const [, name, value] of match[1].matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) {
    props.set(name.trim(), value.trim());
  }
  return props;
}

const root = blockProps(":root");
const light = blockProps(':root\\[data-theme="light"\\]');

function luminance(hex) {
  const channels = [0, 2, 4].map((index) => parseInt(hex.slice(index, index + 2), 16) / 255);
  const linearized = channels.map((channel) =>
    channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4
  );
  return 0.2126 * linearized[0] + 0.7152 * linearized[1] + 0.0722 * linearized[2];
}

function contrastRatio(foreground, background) {
  const [high, low] = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return (high + 0.05) / (low + 0.05);
}

test("defines the semantic scale once with exact TARGET values", () => {
  assert.deepEqual(
    ["--space-1", "--space-2", "--space-3", "--space-4", "--space-5", "--space-6"].map(
      (name) => root.get(name)
    ),
    ["0.25rem", "0.5rem", "0.75rem", "1rem", "1.5rem", "2rem"]
  );
  assert.deepEqual(
    ["--text-body", "--text-supporting", "--text-title", "--text-display"].map(
      (name) => root.get(name)
    ),
    ["0.875rem", "0.75rem", "1rem", "1.25rem"]
  );
  assert.equal(root.get("--radius-surface"), "10px");
  assert.equal(root.get("--radius-control"), "6px");
  assert.equal(root.get("--control-height"), "2.5rem");
  assert.equal(root.get("--control-height-compact"), "2.25rem");
  assert.equal(root.get("--focus-width"), "2px");
  assert.equal(root.get("--focus-offset"), "2px");
  // The scale is theme-invariant: a single source, no per-theme redefinition.
  // (Palette colors such as --text-muted are intentionally per-theme and are
  // covered by the palette contract below, not by this list.)
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
    "--focus-offset"
  ];
  for (const name of singleSource) {
    assert.equal(light.has(name), false, `${name} must not be redefined per theme`);
  }
});

test("both themes define the full palette contract as hex colors", () => {
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
    "--ready-dot"
  ];
  for (const name of palette) {
    for (const [themeName, props] of [
      ["dark", root],
      ["light", light]
    ]) {
      const value = props.get(name);
      assert.match(value ?? "", /^#[0-9a-f]{6}$/i, `${name} must be hex in ${themeName}`);
    }
  }
});

test("shared stylesheet keeps type at or above 12px with tokenized radii", () => {
  for (const [, value] of theme.matchAll(/font-size:\s*([\d.]+)rem/g)) {
    assert.ok(
      Number(value) >= 0.75,
      `font-size ${value}rem is below the 12px supporting floor`
    );
  }
  for (const [, value] of theme.matchAll(/border-radius:\s*([^;]+);/g)) {
    const normalized = value.trim();
    assert.ok(
      normalized.startsWith("var(--radius-") || normalized === "50%",
      `border-radius ${normalized} must use a radius token (50% dot shape excepted)`
    );
  }
});

test("text pairs meet WCAG AA 4.5 in both themes", () => {
  const pairs = [
    [root, "--text", "--bg"],
    [root, "--text", "--surface"],
    [root, "--text-muted", "--surface"],
    [root, "--text-subtle", "--surface"],
    [root, "--accent-text", "--accent-bg"],
    [light, "--text", "--bg"],
    [light, "--text", "--surface"],
    [light, "--text-muted", "--surface"],
    [light, "--text-subtle", "--surface"],
    [light, "--accent-text", "--accent-bg"]
  ];
  for (const [props, foreground, background] of pairs) {
    const ratio = contrastRatio(
      props.get(foreground).slice(1),
      props.get(background).slice(1)
    );
    assert.ok(ratio >= 4.5, `${foreground}/${background} ratio ${ratio.toFixed(2)} < 4.5`);
  }
});

test("key non-text signals meet WCAG 3.0 (border contrast stays a recorded gap)", () => {
  // Border-strong vs surface (~1.7 both themes) is a known gap recorded in
  // the visual inventory, not an asserted pass.
  const pairs = [
    [light, "--focus", "--surface"],
    [light, "--ready-dot", "--surface"],
    [root, "--ready-dot", "--surface"]
  ];
  for (const [props, foreground, background] of pairs) {
    const ratio = contrastRatio(
      props.get(foreground).slice(1),
      props.get(background).slice(1)
    );
    assert.ok(ratio >= 3.0, `${foreground}/${background} ratio ${ratio.toFixed(2)} < 3.0`);
  }
});
