import assert from "node:assert/strict";
import test from "node:test";

import { createStreamBuffer } from "./streamBuffer.ts";

function manualScheduler() {
  let nextId = 1;
  const callbacks = new Map();

  return {
    scheduler: {
      schedule(callback) {
        const id = nextId;
        nextId += 1;
        callbacks.set(id, callback);
        return id;
      },
      cancel(id) {
        callbacks.delete(id);
      }
    },
    run() {
      const pending = [...callbacks.values()];
      callbacks.clear();
      for (const callback of pending) {
        callback();
      }
    },
    pending() {
      return callbacks.size;
    }
  };
}

test("chunks coalesce into one flush per scheduled frame", () => {
  const flushes = [];
  const clock = manualScheduler();
  const buffer = createStreamBuffer((text) => flushes.push(text), clock.scheduler);

  buffer.push("Hel");
  buffer.push("lo ");
  buffer.push("world");
  assert.deepEqual(flushes, []);
  assert.equal(clock.pending(), 1);

  clock.run();
  assert.deepEqual(flushes, ["Hello world"]);
  assert.equal(clock.pending(), 0);

  buffer.push("!");
  clock.run();
  assert.deepEqual(flushes, ["Hello world", "!"]);
});

test("flush is synchronous, ordered, and idempotent", () => {
  const flushes = [];
  const clock = manualScheduler();
  const buffer = createStreamBuffer((text) => flushes.push(text), clock.scheduler);

  buffer.push("a");
  buffer.push("b");
  buffer.flush();
  assert.deepEqual(flushes, ["ab"]);
  assert.equal(clock.pending(), 0);

  buffer.flush();
  assert.deepEqual(flushes, ["ab"]);
});

test("the byte cap forces an immediate bounded flush", () => {
  const flushes = [];
  const clock = manualScheduler();
  const buffer = createStreamBuffer((text) => flushes.push(text), clock.scheduler, 8);

  buffer.push("1234");
  buffer.push("5678");
  assert.deepEqual(flushes, ["12345678"]);
  assert.equal(buffer.pendingBytes(), 0);
  assert.equal(clock.pending(), 0);
});

test("the cap counts UTF-8 bytes, not code units", () => {
  const flushes = [];
  const clock = manualScheduler();
  const buffer = createStreamBuffer((text) => flushes.push(text), clock.scheduler, 8);

  buffer.push("🙂");
  assert.equal(buffer.pendingBytes(), 4);
  buffer.push("🙂");
  assert.deepEqual(flushes, ["🙂🙂"]);
});

test("dispose cancels the scheduled flush and ignores later pushes", () => {
  const flushes = [];
  const clock = manualScheduler();
  const buffer = createStreamBuffer((text) => flushes.push(text), clock.scheduler);

  buffer.push("pending");
  assert.equal(clock.pending(), 1);
  buffer.dispose();
  assert.equal(clock.pending(), 0);
  clock.run();
  assert.deepEqual(flushes, []);

  buffer.push("after");
  clock.run();
  assert.deepEqual(flushes, []);
  assert.equal(buffer.pendingBytes(), 0);
});

test("an empty push does not schedule a frame", () => {
  const flushes = [];
  const clock = manualScheduler();
  const buffer = createStreamBuffer((text) => flushes.push(text), clock.scheduler);

  buffer.push("");
  assert.equal(clock.pending(), 0);
  assert.deepEqual(flushes, []);
});
