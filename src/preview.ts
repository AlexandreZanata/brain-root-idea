import { invoke } from "@tauri-apps/api/core";

export const PREVIEW_STATUS_EVENT = "preview-status";
export const MIN_PREVIEW_SIDE = 40;

export type PreviewPhase =
  | "idle"
  | "starting"
  | "ready"
  | "stopping"
  | "stopped"
  | "failed";

export type PreviewStatus = {
  phase: PreviewPhase;
  port: number | null;
  reason: string | null;
  exit_code: number | null;
};

export type PreviewViewStatus = {
  visible: boolean;
  port: number | null;
  bounds: [number, number, number, number] | null;
};

export type CanvasRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type ViewportPresetId = "desktop" | "tablet" | "phone" | "custom";

export type ViewportSize = {
  width: number;
  height: number;
  exact: boolean;
};

export const CUSTOM_LIMITS = {
  minWidth: 240,
  maxWidth: 1920,
  minHeight: 240,
  maxHeight: 1200
};

export const VIEWPORT_PRESETS: Record<
  Exclude<ViewportPresetId, "custom">,
  { width: number; height: number; label: string }
> = {
  desktop: { width: 1280, height: 800, label: "Desktop" },
  tablet: { width: 834, height: 1112, label: "Tablet" },
  phone: { width: 390, height: 844, label: "Phone" }
};

export const DEFAULT_PREVIEW_PRESET: ViewportPresetId = "phone";

export function presetLabel(preset: ViewportPresetId): string {
  return preset === "custom" ? "Custom" : VIEWPORT_PRESETS[preset].label;
}

function clamp(value: number, minimum: number, maximum: number): number {
  if (!Number.isFinite(value)) {
    return minimum;
  }
  return Math.min(maximum, Math.max(minimum, Math.round(value)));
}

export function customViewportSize(custom?: { width: number; height: number }): {
  width: number;
  height: number;
} {
  return {
    width: clamp(custom?.width ?? 390, CUSTOM_LIMITS.minWidth, CUSTOM_LIMITS.maxWidth),
    height: clamp(custom?.height ?? 844, CUSTOM_LIMITS.minHeight, CUSTOM_LIMITS.maxHeight)
  };
}

export function viewportSize(
  preset: ViewportPresetId,
  available: { width: number; height: number },
  custom?: { width: number; height: number }
): ViewportSize {
  const nominal =
    preset === "custom" ? customViewportSize(custom) : VIEWPORT_PRESETS[preset];
  const width = Math.max(1, Math.min(nominal.width, Math.floor(available.width)));
  const height = Math.max(1, Math.min(nominal.height, Math.floor(available.height)));
  return {
    width,
    height,
    exact: width === nominal.width && height === nominal.height
  };
}

export function isPreviewStatus(value: unknown): value is PreviewStatus {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  const phases: PreviewPhase[] = [
    "idle",
    "starting",
    "ready",
    "stopping",
    "stopped",
    "failed"
  ];
  return (
    typeof candidate.phase === "string" &&
    phases.includes(candidate.phase as PreviewPhase) &&
    (candidate.port === null || typeof candidate.port === "number") &&
    (candidate.reason === null || typeof candidate.reason === "string") &&
    (candidate.exit_code === null || typeof candidate.exit_code === "number")
  );
}

export function isPreviewViewStatus(value: unknown): value is PreviewViewStatus {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return (
    typeof candidate.visible === "boolean" &&
    (candidate.port === null || typeof candidate.port === "number") &&
    (candidate.bounds === null ||
      (Array.isArray(candidate.bounds) && candidate.bounds.length === 4))
  );
}

export function previewReasonMessage(reason: string | null | undefined): string {
  switch (reason) {
    case "preview_command_missing":
      return "Enter the command that starts your app.";
    case "preview_cwd_invalid":
      return "Enter the full path of your project folder.";
    case "preview_already_active":
      return "A preview is already running. Stop it before starting another.";
    case "preview_spawn_failed":
      return "The command could not be started. Check the command and the folder.";
    case "preview_not_ready":
      return "The app did not open a local address in time. Check its output and try again.";
    case "preview_exited_early":
      return "The app stopped before the preview was ready. Check the command and try again.";
    case "preview_port_unavailable":
      return "BrainRoot could not reserve a local address. Try again.";
    case "preview_view_failed":
      return "The preview could not be shown. Try again.";
    case "preview_view_not_visible":
      return "The preview is not visible right now.";
    case "preview_stop_failed":
      return "The preview did not stop cleanly. Try again.";
    case "preview_profile_unavailable":
      return "BrainRoot could not prepare the preview storage. Try again.";
    default:
      return "The preview could not start. Try again.";
  }
}

export function parseCommandLine(input: string): { command: string; args: string[] } {
  const tokens: string[] = [];
  let current = "";
  let quote: '"' | "'" | null = null;
  for (const character of input.trim()) {
    if (quote) {
      if (character === quote) {
        quote = null;
      } else {
        current += character;
      }
      continue;
    }
    if (character === '"' || character === "'") {
      quote = character;
      continue;
    }
    if (/\s/.test(character)) {
      if (current.length > 0) {
        tokens.push(current);
        current = "";
      }
      continue;
    }
    current += character;
  }
  if (current.length > 0) {
    tokens.push(current);
  }
  const [command = "", ...args] = tokens;
  return { command, args };
}

export function normalizeCanvasRect(
  rect: CanvasRect,
  viewport: { width: number; height: number }
): [number, number, number, number] | null {
  const x = Math.max(0, Math.round(rect.x));
  const y = Math.max(0, Math.round(rect.y));
  const width = Math.min(
    Math.round(rect.width),
    Math.max(0, Math.round(viewport.width) - x)
  );
  const height = Math.min(
    Math.round(rect.height),
    Math.max(0, Math.round(viewport.height) - y)
  );
  if (width < MIN_PREVIEW_SIDE || height < MIN_PREVIEW_SIDE) {
    return null;
  }
  return [x, y, width, height];
}

export type CanvasBounds = [number, number, number, number];

export function sameCanvasBounds(
  a: CanvasBounds | null,
  b: CanvasBounds | null
): boolean {
  if (a === b) {
    return true;
  }
  if (a === null || b === null) {
    return false;
  }
  return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3];
}

export type BoundsSync = {
  schedule: (bounds: CanvasBounds | null) => void;
  dispose: () => void;
  sentCount: () => number;
  suppressedCount: () => number;
};

// Serializes native bounds traffic: at most one send in flight plus one
// latest pending rectangle. Duplicate rectangles are suppressed, the final
// rectangle of a burst is always applied, and a disposed sync never applies.
// Sends are strictly sequential, so results settle in initiation order; the
// caller still guards domain staleness (for example navigation vs bounds) in
// `apply`. Counters are local-only release-probe instrumentation.
export function createBoundsSync<T>(options: {
  send: (bounds: CanvasBounds) => Promise<T>;
  apply: (result: T) => void;
  onError: () => void;
}): BoundsSync {
  const { send, apply, onError } = options;
  let disposed = false;
  let inFlight = false;
  let pending: CanvasBounds | null = null;
  let lastSent: CanvasBounds | null = null;
  let sent = 0;
  let suppressed = 0;

  function pump() {
    if (disposed || inFlight || pending === null) {
      return;
    }
    const next = pending;
    pending = null;
    if (sameCanvasBounds(next, lastSent)) {
      suppressed += 1;
      return;
    }
    inFlight = true;
    lastSent = next;
    sent += 1;
    void send(next).then(
      (result) => {
        inFlight = false;
        if (!disposed) {
          apply(result);
        }
        pump();
      },
      () => {
        inFlight = false;
        if (disposed) {
          return;
        }
        // Allow an identical retry to re-send instead of being deduped.
        lastSent = null;
        onError();
        pump();
      }
    );
  }

  return {
    schedule(bounds: CanvasBounds | null) {
      if (disposed || bounds === null) {
        return;
      }
      if (sameCanvasBounds(bounds, pending)) {
        suppressed += 1;
        return;
      }
      if (sameCanvasBounds(bounds, lastSent)) {
        suppressed += 1;
        return;
      }
      if (pending !== null) {
        // A newer rectangle replaces the waiting one; it never sends.
        suppressed += 1;
      }
      pending = bounds;
      pump();
    },
    dispose() {
      disposed = true;
      pending = null;
      inFlight = false;
    },
    sentCount: () => sent,
    suppressedCount: () => suppressed
  };
}

export async function startPreview(request: {
  command: string;
  args: string[];
  cwd: string;
  readiness_timeout_ms?: number;
  stop_grace_ms?: number;
}): Promise<number> {
  const response = await invoke<{ port: number }>("preview_start", { request });
  return response.port;
}

export async function stopPreview(): Promise<void> {
  await invoke("preview_stop", { graceMs: null });
}

export async function showPreview(
  port: number,
  bounds: [number, number, number, number]
): Promise<void> {
  await invoke("preview_show", { port, bounds });
}

export async function setPreviewBounds(
  bounds: [number, number, number, number]
): Promise<void> {
  await invoke("preview_set_bounds", { bounds });
}

export async function hidePreview(): Promise<void> {
  await invoke("preview_hide");
}

export async function previewStatus(): Promise<PreviewStatus> {
  const status: unknown = await invoke("preview_status");
  if (!isPreviewStatus(status)) {
    throw new Error("The core returned an unexpected preview status");
  }
  return status;
}

export async function previewViewStatus(): Promise<PreviewViewStatus> {
  const status: unknown = await invoke("preview_view_status");
  if (!isPreviewViewStatus(status)) {
    throw new Error("The core returned an unexpected preview view status");
  }
  return status;
}
