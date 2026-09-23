export const CONVERSATION_CONTRACT_VERSION = 1;
export const MAX_RENDERED_TURNS = 20;
export const MAX_RENDERED_HISTORY_BYTES = 200 * 1024;

export type ConversationState =
  | "empty"
  | "ready"
  | "sending"
  | "streaming"
  | "cancelling"
  | "succeeded"
  | "failed";

export type ConversationEvent =
  | { type: "started" }
  | { type: "text_chunk"; text: string }
  | {
      type: "completed";
      completion: { conversation: string; stop_reason: "end_turn" };
    }
  | {
      type: "cancelled";
      cancellation: {
        conversation: string;
        reason: "user_requested" | "timeout";
      };
    }
  | {
      type: "failed";
      error: { code: string; message: string };
    };

export type ConversationEnvelope = {
  contractVersion: number;
  conversation: string;
  event: ConversationEvent;
};

export type ConversationTurn = {
  id: number;
  prompt: string;
  response: string;
  error: string;
  errorCode: string;
  status: "active" | "succeeded" | "failed" | "cancelled";
};

export type CredentialStatus = "configured" | "not_configured" | "unavailable";

export function credentialSetupMessage(status: CredentialStatus): string | null {
  switch (status) {
    case "configured":
      return null;
    case "not_configured":
      return "No OpenCode Go credential is configured. Add the key to the system credential store, then try again.";
    case "unavailable":
      return "The system credential store is not available, so prompts cannot be sent on this system.";
  }
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function isConversationEnvelope(value: unknown): value is ConversationEnvelope {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  if (
    candidate.contractVersion !== CONVERSATION_CONTRACT_VERSION ||
    typeof candidate.conversation !== "string" ||
    candidate.conversation.length === 0 ||
    typeof candidate.event !== "object" ||
    candidate.event === null
  ) {
    return false;
  }
  const event = candidate.event as Record<string, unknown>;
  switch (event.type) {
    case "started":
      return true;
    case "text_chunk":
      return typeof event.text === "string";
    case "completed":
      return hasCompletion(event.completion);
    case "cancelled":
      return hasCancellation(event.cancellation);
    case "failed":
      return hasError(event.error);
    default:
      return false;
  }
}

function hasCompletion(value: unknown): boolean {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const completion = value as Record<string, unknown>;
  return (
    typeof completion.conversation === "string" && completion.stop_reason === "end_turn"
  );
}

function hasCancellation(value: unknown): boolean {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const cancellation = value as Record<string, unknown>;
  return (
    typeof cancellation.conversation === "string" &&
    (cancellation.reason === "user_requested" || cancellation.reason === "timeout")
  );
}

function hasError(value: unknown): boolean {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const error = value as Record<string, unknown>;
  return typeof error.code === "string" && typeof error.message === "string";
}

export function beginTurn(
  turns: readonly ConversationTurn[],
  id: number,
  prompt: string
): ConversationTurn[] {
  return boundConversation([
    ...turns,
    { id, prompt, response: "", error: "", errorCode: "", status: "active" }
  ]);
}

export function appendChunk(
  turns: readonly ConversationTurn[],
  id: number,
  text: string
): ConversationTurn[] {
  return boundConversation(
    turns.map((turn) =>
      turn.id === id ? { ...turn, response: turn.response + text } : turn
    )
  );
}

export function settleTurn(
  turns: readonly ConversationTurn[],
  id: number,
  status: "succeeded" | "failed",
  error = "",
  errorCode = ""
): ConversationTurn[] {
  return boundConversation(
    turns.map((turn) => (turn.id === id ? { ...turn, status, error, errorCode } : turn))
  );
}

export function cancelTurn(
  turns: readonly ConversationTurn[],
  id: number
): ConversationTurn[] {
  return boundConversation(
    turns.map((turn) =>
      turn.id === id ? { ...turn, status: "cancelled", error: "", errorCode: "" } : turn
    )
  );
}

export function renderedHistoryBytes(turns: readonly ConversationTurn[]): number {
  return turns.reduce(
    (total, turn) =>
      total + bytes(turn.prompt) + bytes(turn.response) + bytes(turn.error),
    0
  );
}

export function boundConversation(
  source: readonly ConversationTurn[]
): ConversationTurn[] {
  const turns = source.slice(-MAX_RENDERED_TURNS).map((turn) => ({ ...turn }));
  while (
    turns.length > 1 &&
    renderedHistoryBytes(turns) > MAX_RENDERED_HISTORY_BYTES
  ) {
    turns.shift();
  }
  if (turns.length === 0 || renderedHistoryBytes(turns) <= MAX_RENDERED_HISTORY_BYTES) {
    return turns;
  }

  const newest = turns[0];
  newest.error = takeNewestUtf8(newest.error, MAX_RENDERED_HISTORY_BYTES);
  let remaining = MAX_RENDERED_HISTORY_BYTES - bytes(newest.error);
  newest.response = takeNewestUtf8(newest.response, remaining);
  remaining -= bytes(newest.response);
  newest.prompt = takeNewestUtf8(newest.prompt, remaining);
  return turns;
}

export function clampPanelWidth(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) {
    return min;
  }
  if (value < min) {
    return min;
  }
  if (value > max) {
    return max;
  }
  return value;
}

function bytes(value: string): number {
  return encoder.encode(value).byteLength;
}

function takeNewestUtf8(value: string, limit: number): string {
  if (limit <= 0) {
    return "";
  }
  const encoded = encoder.encode(value);
  if (encoded.byteLength <= limit) {
    return value;
  }
  let start = encoded.byteLength - limit;
  while (start < encoded.byteLength && (encoded[start] & 0xc0) === 0x80) {
    start += 1;
  }
  return decoder.decode(encoded.subarray(start));
}
