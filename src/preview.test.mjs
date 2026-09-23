import assert from "node:assert/strict";
import test from "node:test";

import {
  MIN_PREVIEW_SIDE,
  isPreviewStatus,
  isPreviewViewStatus,
  normalizeCanvasRect,
  parseCommandLine,
  previewReasonMessage
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
