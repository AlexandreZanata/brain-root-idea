import assert from "node:assert/strict";
import test from "node:test";

import {
  chooseSendPath,
  formatContext,
  formatPrice,
  hostLabel,
  isAgentEventEnvelope,
  isAgentHostStatus,
  isAgentMode,
  isAgentModelList,
  isCatalogResult,
  isCostProfile,
  modelDetail,
  modelKey,
  pickProfileModel
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
  for (const phase of ["checking", "stopped", "starting", "running", "failed"]) {
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

test("catalog results carry price tags honestly", () => {
  const entry = {
    provider_id: "acme",
    provider_name: "Acme",
    model_id: "fast",
    model_name: "Fast",
    context_length: 128000,
    prompt_usd_per_m: 3,
    completion_usd_per_m: 15
  };
  assert.equal(isCatalogResult({ models: [entry], selected: null, stale: false }), true);
  assert.equal(isCatalogResult({ models: [], selected: null, stale: true }), true);
  assert.equal(isCatalogResult({ models: [], stale: false }), false);
  assert.equal(formatContext(128000), "128k ctx");
  assert.equal(formatContext(1048576), "1.0M ctx");
  assert.equal(formatContext(null), "? ctx");
  assert.equal(formatPrice(3), "$3.00/M");
  assert.equal(formatPrice(null), "?/M");
  assert.equal(modelDetail(entry), "Fast · Acme · 128k ctx · $3.00/M in");
});

test("agent modes are plan or build only", () => {
  assert.equal(isAgentMode("plan"), true);
  assert.equal(isAgentMode("build"), true);
  assert.equal(isAgentMode("turbo"), false);
  assert.equal(isAgentMode(null), false);
});

test("profiles route over live prices", () => {
  const models = [
    { provider_id: "a", provider_name: "A", model_id: "cheap", model_name: "Cheap", context_length: 32000, prompt_usd_per_m: 0.5, completion_usd_per_m: 1 },
    { provider_id: "b", provider_name: "B", model_id: "roomy", model_name: "Roomy", context_length: 200000, prompt_usd_per_m: 3, completion_usd_per_m: 6 },
    { provider_id: "c", provider_name: "C", model_id: "pricey", model_name: "Pricey", context_length: 64000, prompt_usd_per_m: 30, completion_usd_per_m: 60 }
  ];
  assert.equal(isCostProfile("fast"), true);
  assert.equal(isCostProfile("nope"), false);
  assert.deepEqual(pickProfileModel(models, "fast"), { providerId: "a", modelId: "cheap", agent: "plan" });
  assert.deepEqual(pickProfileModel(models, "balanced"), { providerId: "b", modelId: "roomy", agent: "build" });
  assert.deepEqual(pickProfileModel(models, "max"), { providerId: "c", modelId: "pricey", agent: "build" });
  assert.equal(pickProfileModel([], "fast"), null);
  const unpriced = [
    { provider_id: "a", provider_name: "A", model_id: "x", model_name: "X", context_length: null, prompt_usd_per_m: null, completion_usd_per_m: null }
  ];
  assert.deepEqual(pickProfileModel(unpriced, "balanced"), { providerId: "a", modelId: "x", agent: "build" });
});
