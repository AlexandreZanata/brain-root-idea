import assert from "node:assert/strict";
import test from "node:test";

import {
  RESPONSE_PREVIEW_CHARS,
  acceptsEvent,
  canvasTransition,
  composerKeyAction,
  isPinnedToBottom,
  presentTurn,
  responsePreview,
  shouldCollapseTurn,
  taskStatus
} from "./presentation.ts";

const states = [
  "empty",
  "ready",
  "sending",
  "streaming",
  "cancelling",
  "succeeded",
  "failed"
];

const eventTypes = ["started", "text_chunk", "completed", "cancelled", "failed"];

test("taskStatus labels every state and honors the setup message", () => {
  const labels = Object.fromEntries(states.map((state) => [state, taskStatus(state, null).label]));

  assert.deepEqual(labels, {
    empty: "Ready for a request",
    ready: "Ready for a request",
    sending: "Starting…",
    streaming: "Building…",
    cancelling: "Cancelling…",
    succeeded: "Done",
    failed: "Needs attention"
  });
  assert.equal(taskStatus("empty", "No credential").label, "Needs setup");
  assert.equal(taskStatus("ready", "No credential").label, "Needs setup");
  assert.equal(taskStatus("streaming", "No credential").label, "Building…");
});

test("presentTurn maps each turn status with technical detail only on coded failures", () => {
  const base = {
    id: 1,
    prompt: "Build it",
    response: "",
    error: "",
    errorCode: "",
    status: "active"
  };

  assert.deepEqual(presentTurn(base), {
    status: "working",
    label: "Working…",
    showTechnicalDetail: false
  });
  assert.deepEqual(presentTurn({ ...base, status: "succeeded" }), {
    status: "done",
    label: "Done",
    showTechnicalDetail: false
  });
  assert.deepEqual(presentTurn({ ...base, status: "cancelled" }), {
    status: "cancelled",
    label: "Cancelled",
    showTechnicalDetail: false
  });
  assert.deepEqual(
    presentTurn({
      ...base,
      status: "failed",
      error: "The provider is unavailable.",
      errorCode: "provider_unavailable"
    }),
    { status: "failed", label: "Needs attention", showTechnicalDetail: true }
  );
  assert.deepEqual(
    presentTurn({
      ...base,
      status: "failed",
      error: "The provider is unavailable.",
      errorCode: ""
    }),
    { status: "failed", label: "Needs attention", showTechnicalDetail: false }
  );
});

test("acceptsEvent mirrors the Rust transition table for every state", () => {
  const legal = {
    started: ["sending"],
    text_chunk: ["sending", "streaming"],
    completed: ["sending", "streaming"],
    failed: ["sending", "streaming"],
    cancelled: ["sending", "streaming", "cancelling"]
  };

  for (const type of eventTypes) {
    for (const state of states) {
      assert.equal(
        acceptsEvent(state, type),
        legal[type].includes(state),
        `${type} in ${state}`
      );
    }
  }
});

test("reordered, late, and duplicate events are ignored", () => {
  assert.equal(acceptsEvent("cancelling", "started"), false);
  assert.equal(acceptsEvent("cancelling", "text_chunk"), false);
  assert.equal(acceptsEvent("cancelling", "completed"), false);
  assert.equal(acceptsEvent("cancelling", "failed"), false);
  assert.equal(acceptsEvent("succeeded", "completed"), false);
  assert.equal(acceptsEvent("failed", "failed"), false);
  assert.equal(acceptsEvent("ready", "cancelled"), false);
  assert.equal(acceptsEvent("streaming", "started"), false);
  assert.equal(acceptsEvent("empty", "text_chunk"), false);
});

test("shouldCollapseTurn only collapses old long succeeded answers", () => {
  const long = "x".repeat(RESPONSE_PREVIEW_CHARS + 1);
  const base = {
    id: 1,
    prompt: "Build it",
    response: long,
    error: "",
    errorCode: "",
    status: "succeeded"
  };

  assert.equal(shouldCollapseTurn(base, true), false);
  assert.equal(shouldCollapseTurn(base, false), true);
  assert.equal(shouldCollapseTurn({ ...base, status: "failed" }, false), false);
  assert.equal(shouldCollapseTurn({ ...base, status: "cancelled" }, false), false);
  assert.equal(shouldCollapseTurn({ ...base, status: "active" }, false), false);
  assert.equal(shouldCollapseTurn({ ...base, response: "short" }, false), false);
});

test("responsePreview keeps short text and cuts long text at a word boundary", () => {
  assert.equal(responsePreview("short answer"), "short answer");
  assert.equal(responsePreview("abcdefghij", 10), "abcdefghij");
  assert.equal(responsePreview("abcdefghij", 0), "");

  const words = Array.from({ length: 200 }, (_, index) => `word${index}`).join(" ");
  const preview = responsePreview(words, 60);

  assert.ok(preview.length <= 61);
  assert.equal(preview.endsWith("…"), true);
  assert.equal(preview.slice(0, -1).endsWith(" "), false);
  assert.equal(words.startsWith(preview.slice(0, -1).trimEnd()), true);
});

test("composerKeyAction maps Enter, Shift+Enter, Escape, and IME composition", () => {
  assert.equal(composerKeyAction({ key: "Enter", shiftKey: false }, false), "submit");
  assert.equal(composerKeyAction({ key: "Enter", shiftKey: true }, false), "newline");
  assert.equal(composerKeyAction({ key: "Escape", shiftKey: false }, false), "cancel");
  assert.equal(composerKeyAction({ key: "a", shiftKey: false }, false), "none");
  assert.equal(composerKeyAction({ key: "Enter", shiftKey: false }, true), "none");
  assert.equal(composerKeyAction({ key: "Escape", shiftKey: false }, true), "none");
});

test("isPinnedToBottom keeps the reader's position unless near the bottom", () => {
  assert.equal(isPinnedToBottom(900, 100, 1000), true);
  assert.equal(isPinnedToBottom(880, 100, 1000), true);
  assert.equal(isPinnedToBottom(800, 100, 1000), false);
  assert.equal(isPinnedToBottom(0, 500, 300), true);
  assert.equal(isPinnedToBottom(700, 100, 1000, 0), false);
});

test("canvasTransition states the honest reload rule for each destination", () => {
  const preview = canvasTransition("preview");

  assert.equal(preview.destination, "preview");
  assert.equal(preview.label, "Opening Preview");
  assert.match(preview.note, /reloads when you return/);
  assert.match(preview.note, /dev server keeps running/);

  const browser = canvasTransition("browser");

  assert.equal(browser.destination, "browser");
  assert.equal(browser.label, "Opening Browser");
  assert.match(browser.note, /reloads when you return/);
  assert.match(browser.note, /in-memory state may be lost/);
});
