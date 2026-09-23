import assert from "node:assert/strict";
import test from "node:test";

import corpus from "../src-tauri/tests/human-navigation-corpus.json" with { type: "json" };
import {
  humanErrorMessage,
  isHumanStatus,
  normalizeAddress
} from "./humanBrowser.ts";

test("normalizes addresses with a default scheme and preserves explicit schemes", () => {
  const normalized = normalizeAddress("example.com/path");
  assert.ok(normalized.startsWith("https" + "://"), normalized);
  assert.ok(normalized.endsWith("example.com/path"), normalized);
  assert.equal(normalizeAddress("  example.com  "), normalizeAddress("example.com"));
  for (const entry of [
    ...corpus.allowed,
    ...corpus.denied_scheme,
    ...corpus.open_external
  ]) {
    assert.equal(normalizeAddress(entry), entry, entry);
  }
  assert.equal(normalizeAddress(""), "");
  assert.equal(normalizeAddress("   "), "");
});

test("maps engine denial codes to plain language", () => {
  assert.equal(
    humanErrorMessage("human_scheme_denied"),
    "This link type is not opened in the browser."
  );
  assert.equal(
    humanErrorMessage("human_userinfo_denied"),
    "Links with embedded credentials are not opened."
  );
  assert.equal(
    humanErrorMessage("human_download_denied"),
    "Downloads are blocked in this version."
  );
  assert.equal(
    humanErrorMessage("something_unknown"),
    "The browser request could not be completed."
  );
  assert.equal(humanErrorMessage(null), "The browser request could not be completed.");
});

test("guards browser status payloads", () => {
  assert.equal(
    isHumanStatus({
      role: "human",
      visible: true,
      url: corpus.allowed[0],
      title: "Example",
      can_go_back: false,
      can_go_forward: true,
      last_denial: null
    }),
    true
  );
  assert.equal(
    isHumanStatus({
      role: "human",
      visible: false,
      url: null,
      title: null,
      can_go_back: false,
      can_go_forward: false,
      last_denial: "human_popup_denied"
    }),
    true
  );
  assert.equal(isHumanStatus({ role: "preview", visible: true }), false);
  assert.equal(isHumanStatus({ role: "human", visible: "yes" }), false);
  assert.equal(isHumanStatus(null), false);
});
