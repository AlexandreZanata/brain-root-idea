import { invoke } from "@tauri-apps/api/core";

export const HUMAN_STATUS_EVENT = "human-browser-status";

const DEFAULT_SCHEME = "https";

export type HumanStatus = {
  role: "human";
  visible: boolean;
  url: string | null;
  title: string | null;
  can_go_back: boolean;
  can_go_forward: boolean;
  last_denial: string | null;
};

export function isHumanStatus(value: unknown): value is HumanStatus {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return (
    candidate.role === "human" &&
    typeof candidate.visible === "boolean" &&
    (candidate.url === null || typeof candidate.url === "string") &&
    (candidate.title === null || typeof candidate.title === "string") &&
    typeof candidate.can_go_back === "boolean" &&
    typeof candidate.can_go_forward === "boolean" &&
    (candidate.last_denial === null || typeof candidate.last_denial === "string")
  );
}

export function normalizeAddress(input: string): string {
  const trimmed = input.trim();
  if (trimmed.length === 0) {
    return "";
  }
  if (/^[a-z][a-z0-9+.-]*:/i.test(trimmed)) {
    return trimmed;
  }
  return `${DEFAULT_SCHEME}://${trimmed}`;
}

export function humanErrorMessage(code: string | null | undefined): string {
  if (typeof code === "string" && code.startsWith("human_permission_denied")) {
    return "This page asked for a permission. BrainRoot blocks camera, microphone, location, notifications, and clipboard in this browser.";
  }
  switch (code) {
    case "human_scheme_denied":
      return "This link type is not opened in the browser.";
    case "human_userinfo_denied":
      return "Links with embedded credentials are not opened.";
    case "human_url_invalid":
      return "That address is not valid.";
    case "human_external_open":
      return "This link opens outside BrainRoot; it is not loaded here.";
    case "human_popup_denied":
      return "A popup was blocked.";
    case "human_download_denied":
      return "Downloads are blocked in this version.";
    case "human_view_failed":
      return "The browser view could not be shown.";
    case "human_view_not_visible":
      return "The browser is not visible.";
    case "human_profile_unavailable":
      return "BrainRoot could not prepare the browser profile.";
    case "human_bounds_invalid":
      return "The browser area is not valid.";
    default:
      return "The browser request could not be completed.";
  }
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const result: unknown = await invoke(command, args);
  if (!isHumanStatus(result)) {
    throw new Error("The core returned an unexpected browser status");
  }
  return result as T;
}

export function humanShow(
  url: string,
  bounds: [number, number, number, number]
): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_show", { url, bounds });
}

export function humanNavigate(url: string): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_navigate", { url });
}

export function humanBack(): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_back");
}

export function humanForward(): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_forward");
}

export function humanReload(): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_reload");
}

export function humanSetBounds(
  bounds: [number, number, number, number]
): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_set_bounds", { bounds });
}

export function humanHide(): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_hide");
}

export function humanStatus(): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_status");
}

export function humanClearData(): Promise<HumanStatus> {
  return call<HumanStatus>("human_browser_clear_data");
}
