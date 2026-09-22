import assert from "node:assert/strict";
import test from "node:test";

import {
  MAX_RENDERED_HISTORY_BYTES,
  MAX_RENDERED_TURNS,
  appendChunk,
  beginTurn,
  boundConversation,
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
