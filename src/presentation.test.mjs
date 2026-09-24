import assert from "node:assert/strict";
import test from "node:test";

import { acceptsEvent, presentTurn, taskStatus } from "./presentation.ts";

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
