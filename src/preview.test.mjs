import assert from "node:assert/strict";
import test from "node:test";

import {
  CUSTOM_LIMITS,
  DEFAULT_PREVIEW_PRESET,
  MIN_PREVIEW_SIDE,
  VIEWPORT_PRESETS,
  createBoundsSync,
  customViewportSize,
  isPreviewStatus,
  isPreviewViewStatus,
  normalizeCanvasRect,
  parseCommandLine,
  presetLabel,
  previewReasonMessage,
  sameCanvasBounds,
  viewportSize
} from "./preview.ts";

test("maps known preview reasons to plain language and falls back", () => {
  assert.equal(
    previewReasonMessage("preview_command_missing"),
    "Enter the command that starts your app."
  );
  assert.equal(
    previewReasonMessage("preview_not_ready"),
    "The app did not open a local address in time. Check its output and try again."
  );
  assert.equal(
    previewReasonMessage("preview_exited_early"),
    "The app stopped before the preview was ready. Check the command and try again."
  );
  assert.equal(
    previewReasonMessage("preview_stop_failed"),
    "The preview did not stop cleanly. Try again."
  );
  assert.equal(
    previewReasonMessage("something_unknown"),
    "The preview could not start. Try again."
  );
  assert.equal(previewReasonMessage(null), "The preview could not start. Try again.");
});

test("parses commands with quotes, spacing, and empty input", () => {
  assert.deepEqual(parseCommandLine("npm run dev"), {
    command: "npm",
    args: ["run", "dev"]
  });
  assert.deepEqual(parseCommandLine('  pnpm   exec  "vite --host" '), {
    command: "pnpm",
    args: ["exec", "vite --host"]
  });
  assert.deepEqual(parseCommandLine(""), { command: "", args: [] });
  assert.deepEqual(parseCommandLine("   "), { command: "", args: [] });
});

test("normalizes canvas rectangles and rejects unusable ones", () => {
  assert.deepEqual(
    normalizeCanvasRect(
      { x: 12.4, y: 20.6, width: 300.2, height: 200.7 },
      { width: 800, height: 600 }
    ),
    [12, 21, 300, 201]
  );
  assert.equal(
    normalizeCanvasRect(
      { x: 780, y: 580, width: 400, height: 400 },
      { width: 800, height: 600 }
    ),
    null
  );
  assert.equal(
    normalizeCanvasRect(
      { x: -10, y: -10, width: 20, height: 20 },
      { width: 800, height: 600 }
    ),
    null
  );
  assert.equal(
    normalizeCanvasRect(
      { x: 0, y: 0, width: MIN_PREVIEW_SIDE - 1, height: 200 },
      { width: 800, height: 600 }
    ),
    null
  );
});

test("guards preview status and view status payloads", () => {
  assert.equal(
    isPreviewStatus({ phase: "ready", port: 3000, reason: null, exit_code: null }),
    true
  );
  assert.equal(isPreviewStatus({ phase: "nonsense", port: 3000 }), false);
  assert.equal(isPreviewStatus({ phase: "ready", port: "3000" }), false);
  assert.equal(isPreviewStatus(null), false);
  assert.equal(
    isPreviewViewStatus({ visible: true, port: 3000, bounds: [0, 0, 300, 200] }),
    true
  );
  assert.equal(
    isPreviewViewStatus({ visible: false, port: null, bounds: null }),
    true
  );
  assert.equal(isPreviewViewStatus({ visible: "yes", port: null, bounds: null }), false);
  assert.equal(isPreviewViewStatus({ visible: true, port: null, bounds: [0, 0] }), false);
});

test("the phone preset is the B18 default and clamps to a tight area", () => {
  assert.equal(DEFAULT_PREVIEW_PRESET, "phone");
  assert.deepEqual(viewportSize(DEFAULT_PREVIEW_PRESET, { width: 320, height: 480 }), {
    width: 320,
    height: 480,
    exact: false
  });
});

test("resolves viewport presets against the available Canvas area", () => {
  const roomy = { width: 1600, height: 1200 };
  assert.deepEqual(viewportSize("desktop", roomy), {
    width: 1280,
    height: 800,
    exact: true
  });
  assert.deepEqual(viewportSize("tablet", roomy), {
    width: 834,
    height: 1112,
    exact: true
  });
  assert.deepEqual(viewportSize("phone", roomy), {
    width: 390,
    height: 844,
    exact: true
  });
  const tight = { width: 700, height: 500 };
  assert.deepEqual(viewportSize("desktop", tight), {
    width: 700,
    height: 500,
    exact: false
  });
});

test("clamps custom viewport sizes and labels presets", () => {
  assert.deepEqual(customViewportSize({ width: 100, height: 9000 }), {
    width: CUSTOM_LIMITS.minWidth,
    height: CUSTOM_LIMITS.maxHeight
  });
  assert.deepEqual(
    viewportSize(
      "custom",
      { width: 1600, height: 1200 },
      { width: 500, height: 700 }
    ),
    { width: 500, height: 700, exact: true }
  );
  assert.equal(presetLabel("custom"), "Custom");
  assert.equal(presetLabel("phone"), VIEWPORT_PRESETS.phone.label);
});

// B18-S06 deferred release cases (written now, executed only at the S07
// versioned gate per ADR 0014; promises and microtasks only, no timers).

test("compares canvas bounds rectangles", () => {
  assert.equal(sameCanvasBounds(null, null), true);
  assert.equal(sameCanvasBounds([0, 0, 100, 100], null), false);
  assert.equal(sameCanvasBounds(null, [0, 0, 100, 100]), false);
  assert.equal(sameCanvasBounds([1, 2, 3, 4], [1, 2, 3, 4]), true);
  assert.equal(sameCanvasBounds([1, 2, 3, 4], [1, 2, 3, 5]), false);
});

test("bounds sync applies only the final rectangle of a rapid burst", async () => {
  const sent = [];
  const applied = [];
  const sync = createBoundsSync({
    send: (bounds) => {
      sent.push([...bounds]);
      return Promise.resolve(bounds);
    },
    apply: (result) => {
      applied.push([...result]);
    },
    onError: () => {
      throw new Error("unexpected bounds error");
    }
  });
  sync.schedule([0, 0, 100, 100]);
  sync.schedule([0, 0, 120, 100]);
  sync.schedule([0, 0, 140, 100]);
  sync.schedule([0, 0, 140, 100]);
  for (let i = 0; i < 20 && applied.length < 2; i++) {
    await Promise.resolve();
  }
  assert.deepEqual(sent, [
    [0, 0, 100, 100],
    [0, 0, 140, 100]
  ]);
  assert.deepEqual(applied, [
    [0, 0, 100, 100],
    [0, 0, 140, 100]
  ]);
  assert.equal(sync.sentCount(), 2);
  assert.equal(sync.suppressedCount(), 2);
  sync.dispose();
});

test("bounds sync suppresses duplicate rectangles", async () => {
  const applied = [];
  const sync = createBoundsSync({
    send: (bounds) => Promise.resolve(bounds),
    apply: (result) => {
      applied.push(result);
    },
    onError: () => {
      throw new Error("unexpected bounds error");
    }
  });
  sync.schedule([2, 2, 20, 20]);
  for (let i = 0; i < 20 && applied.length < 1; i++) {
    await Promise.resolve();
  }
  sync.schedule([2, 2, 20, 20]);
  for (let i = 0; i < 10; i++) {
    await Promise.resolve();
  }
  assert.equal(sync.sentCount(), 1);
  assert.equal(sync.suppressedCount(), 1);
  sync.dispose();
});

test("bounds sync re-sends an identical rectangle after a failure", async () => {
  let calls = 0;
  let errors = 0;
  const applied = [];
  const sync = createBoundsSync({
    send: (bounds) => {
      calls += 1;
      return calls === 1
        ? Promise.reject(new Error("boom"))
        : Promise.resolve(bounds);
    },
    apply: (result) => {
      applied.push(result);
    },
    onError: () => {
      errors += 1;
    }
  });
  sync.schedule([5, 5, 50, 50]);
  for (let i = 0; i < 20 && errors < 1; i++) {
    await Promise.resolve();
  }
  assert.equal(errors, 1);
  sync.schedule([5, 5, 50, 50]);
  for (let i = 0; i < 20 && applied.length < 1; i++) {
    await Promise.resolve();
  }
  assert.equal(calls, 2);
  assert.deepEqual(applied, [[5, 5, 50, 50]]);
  sync.dispose();
});

test("bounds sync never applies after dispose", async () => {
  let resolveSend = () => undefined;
  const applied = [];
  const sync = createBoundsSync({
    send: () =>
      new Promise((resolve) => {
        resolveSend = resolve;
      }),
    apply: (result) => {
      applied.push(result);
    },
    onError: () => {}
  });
  sync.schedule([9, 9, 90, 90]);
  sync.dispose();
  resolveSend("late");
  for (let i = 0; i < 10; i++) {
    await Promise.resolve();
  }
  assert.deepEqual(applied, []);
  assert.equal(sync.sentCount(), 1);
});
