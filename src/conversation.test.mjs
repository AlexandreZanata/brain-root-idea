import assert from "node:assert/strict";
import test from "node:test";

import {
  MAX_RENDERED_HISTORY_BYTES,
  MAX_RENDERED_TURNS,
  appendChunk,
  beginTurn,
  boundConversation,
  cancelTurn,
  clampPanelWidth,
  credentialSetupMessage,
  isConversationEnvelope,
  renderedHistoryBytes,
  settleTurn
} from "./conversation.ts";

test("accepts the versioned neutral envelope and rejects provider-shaped input", () => {
  assert.equal(
    isConversationEnvelope({
      contractVersion: 1,
      conversation: "conversation-1",
      event: { type: "text_chunk", text: "hello" }
    }),
    true
  );
  assert.equal(
    isConversationEnvelope({
      contractVersion: 1,
      conversation: "conversation-1",
      event: { choices: [{ delta: { content: "raw" } }] }
    }),
    false
  );
  assert.equal(
    isConversationEnvelope({
      contractVersion: 2,
      conversation: "conversation-1",
      event: { type: "started" }
    }),
    false
  );
});

test("turn helpers append streamed text without mutating prior snapshots", () => {
  const empty = [];
  const active = beginTurn(empty, 1, "Build it");
  const streamed = appendChunk(active, 1, "Hello");
  const settled = settleTurn(streamed, 1, "succeeded");

  assert.equal(empty.length, 0);
  assert.equal(active[0].response, "");
  assert.equal(streamed[0].response, "Hello");
  assert.equal(settled[0].status, "succeeded");
});

test("cancelTurn marks only the cancelled turn and never carries an error", () => {
  const first = settleTurn(beginTurn([], 1, "First"), 1, "succeeded");
  const active = beginTurn(first, 2, "Second");
  const cancelled = cancelTurn(active, 2);

  assert.equal(cancelled[0].status, "succeeded");
  assert.equal(cancelled[1].status, "cancelled");
  assert.equal(cancelled[1].error, "");
  assert.equal(cancelled[1].errorCode, "");
});

test("credentialSetupMessage explains the next action for each status", () => {
  assert.equal(credentialSetupMessage("configured"), null);
  assert.match(
    credentialSetupMessage("not_configured") ?? "",
    /Add the key to the system credential store/
  );
  assert.match(
    credentialSetupMessage("unavailable") ?? "",
    /credential store is not available/
  );
});

test("settleTurn carries the sanitized failure code for technical details", () => {
  const active = beginTurn([], 1, "Build it");
  const failed = settleTurn(
    active,
    1,
    "failed",
    "The provider could not be reached. Check your connection and try again.",
    "provider_unavailable"
  );

  assert.equal(failed[0].status, "failed");
  assert.equal(failed[0].errorCode, "provider_unavailable");
  assert.equal(failed[0].error, "The provider could not be reached. Check your connection and try again.");
});

test("clampPanelWidth keeps the agent width inside the resizable range", () => {
  assert.equal(clampPanelWidth(100, 280, 560), 280);
  assert.equal(clampPanelWidth(900, 280, 560), 560);
  assert.equal(clampPanelWidth(368, 280, 560), 368);
  assert.equal(clampPanelWidth(Number.NaN, 280, 560), 280);
  assert.equal(clampPanelWidth(Number.POSITIVE_INFINITY, 280, 560), 280);
});

test("long output stays inside both rendered limits and preserves newest UTF-8", () => {
  const oversized = `${"🙂".repeat(MAX_RENDERED_HISTORY_BYTES)}newest-marker`;
  const turns = Array.from({ length: MAX_RENDERED_TURNS + 8 }, (_, index) => ({
    id: index,
    prompt: `prompt-${index}`,
    response: index === MAX_RENDERED_TURNS + 7 ? oversized : "old response",
    error: "",
    status: "succeeded"
  }));

  const bounded = boundConversation(turns);

  assert.ok(bounded.length <= MAX_RENDERED_TURNS);
  assert.ok(renderedHistoryBytes(bounded) <= MAX_RENDERED_HISTORY_BYTES);
  assert.equal(bounded.at(-1)?.response.endsWith("newest-marker"), true);
  assert.equal(bounded.at(-1)?.response.includes("�"), false);
});
