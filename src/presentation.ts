import type { ConversationEvent, ConversationState, ConversationTurn } from "./conversation";

export type TaskPhase =
  | "idle"
  | "starting"
  | "building"
  | "cancelling"
  | "done"
  | "needs_attention";

export type TaskStatus = {
  phase: TaskPhase;
  label: string;
};

export function taskStatus(
  state: ConversationState,
  setupMessage: string | null = null
): TaskStatus {
  switch (state) {
    case "sending":
      return { phase: "starting", label: "Starting…" };
    case "streaming":
      return { phase: "building", label: "Building…" };
    case "cancelling":
      return { phase: "cancelling", label: "Cancelling…" };
    case "succeeded":
      return { phase: "done", label: "Done" };
    case "failed":
      return { phase: "needs_attention", label: "Needs attention" };
    default:
      return setupMessage
        ? { phase: "idle", label: "Needs setup" }
        : { phase: "idle", label: "Ready for a request" };
  }
}

export type TurnStatus = "working" | "done" | "cancelled" | "failed";

export type TurnPresentation = {
  status: TurnStatus;
  label: string;
  showTechnicalDetail: boolean;
};

export function presentTurn(turn: ConversationTurn): TurnPresentation {
  switch (turn.status) {
    case "active":
      return { status: "working", label: "Working…", showTechnicalDetail: false };
    case "succeeded":
      return { status: "done", label: "Done", showTechnicalDetail: false };
    case "cancelled":
      return { status: "cancelled", label: "Cancelled", showTechnicalDetail: false };
    case "failed":
      return {
        status: "failed",
        label: "Needs attention",
        showTechnicalDetail: turn.errorCode.length > 0
      };
  }
}

export function acceptsEvent(
  state: ConversationState,
  type: ConversationEvent["type"]
): boolean {
  switch (type) {
    case "started":
      return state === "sending";
    case "text_chunk":
    case "completed":
    case "failed":
      return state === "sending" || state === "streaming";
    case "cancelled":
      return state === "sending" || state === "streaming" || state === "cancelling";
  }
}

export const RESPONSE_PREVIEW_CHARS = 600;

export function shouldCollapseTurn(turn: ConversationTurn, isLatest: boolean): boolean {
  return (
    !isLatest && turn.status === "succeeded" && turn.response.length > RESPONSE_PREVIEW_CHARS
  );
}

export function responsePreview(text: string, limit = RESPONSE_PREVIEW_CHARS): string {
  if (limit <= 0) {
    return "";
  }
  if (text.length <= limit) {
    return text;
  }
  const clipped = text.slice(0, limit);
  const boundary = clipped.search(/\s+\S*$/);
  const preview = boundary > limit / 2 ? clipped.slice(0, boundary) : clipped;
  return `${preview.trimEnd()}…`;
}
