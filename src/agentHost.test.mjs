import assert from "node:assert/strict";
import test from "node:test";

import {
  chooseSendPath,
  hostLabel,
  isAgentEventEnvelope,
  isAgentHostStatus,
  isAgentModelList,
  modelKey
} from "./agentHost.ts";

test("rejects non-status payloads", () => {
  assert.equal(isAgentHostStatus(null), false);
  assert.equal(isAgentHostStatus({}), false);
  assert.equal(
    isAgentHostStatus({ running: true, port: 4099, version: "1", pid: 1, password: "x" }),
    true
  );
  assert.equal(isAgentHostStatus({ running: "yes", port: 4099, version: null, pid: null }), false);
  assert.equal(isAgentHostStatus({ running: true, port: "4099", version: null, pid: null }), false);
});

test("accepts the redacted status shape", () => {
  assert.equal(
    isAgentHostStatus({ running: false, port: 4099, version: null, pid: null }),
    true
  );
  assert.equal(
    isAgentHostStatus({ running: true, port: 4099, version: "1.18.31", pid: 42 }),
    true
  );
});

test("labels never include secrets", () => {
  const status = { running: true, port: 4099, version: "1.18.31", pid: 42 };
  for (const phase of ["checking", "stopped", "starting", "running", "failed"] as const) {
    const label = hostLabel(phase, status);
    assert.ok(label.length > 0);
    assert.ok(!label.includes("sk-"));
    assert.ok(!label.includes("password"));
  }
  assert.equal(hostLabel("running", status), "Sidecar 1.18.31");
  assert.equal(hostLabel("stopped", status), "Sidecar stopped");
});

test("model keys are stable and lists are shape-checked", () => {
  const entry = { provider_id: "acme", provider_name: "Acme", model_id: "fast", model_name: "Fast" };
  assert.equal(modelKey(entry), "acme/fast");
  assert.equal(isAgentModelList({ models: [entry], selected: null }), true);
  assert.equal(
    isAgentModelList({ models: [entry], selected: { provider_id: "acme", model_id: "fast" } }),
    true
  );
  assert.equal(isAgentModelList({ models: [], selected: { provider_id: "acme" } }), false);
  assert.equal(isAgentModelList({ models: "x", selected: null }), false);
});

test("send path follows the sidecar", () => {
  assert.equal(chooseSendPath(true), "agent");
  assert.equal(chooseSendPath(false), "legacy");
});

test("agent envelopes are shape-checked", () => {
  assert.equal(isAgentEventEnvelope(null), false);
  assert.equal(isAgentEventEnvelope({ contractVersion: 2, event: { type: "started", session: "ses_1" } }), false);
  assert.equal(
    isAgentEventEnvelope({ contractVersion: 1, event: { type: "started", session: "ses_1" } }),
    true
  );
  assert.equal(
    isAgentEventEnvelope({ contractVersion: 1, event: { type: "text_chunk", session: "ses_1", text: "hi" } }),
    true
  );
  assert.equal(
    isAgentEventEnvelope({ contractVersion: 1, event: { type: "text_chunk", session: "ses_1" } }),
    false
  );
  assert.equal(
    isAgentEventEnvelope({
      contractVersion: 1,
      event: { type: "failed", session: "ses_1", error: { code: "x", message: "y" } }
    }),
    true
  );
  assert.equal(
    isAgentEventEnvelope({ contractVersion: 1, event: { type: "bogus", session: "ses_1" } }),
    false
  );
});
