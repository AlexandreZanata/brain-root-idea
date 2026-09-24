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
