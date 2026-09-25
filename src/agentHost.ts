import { invoke } from "@tauri-apps/api/core";

export type AgentHostStatus = {
  running: boolean;
  port: number;
  version: string | null;
  pid: number | null;
};

export type HostPhase = "checking" | "stopped" | "starting" | "running" | "failed";

export const AGENT_EVENT_NAME = "agent_event";
export const AGENT_CONTRACT_VERSION = 1;

export type AgentStreamEvent =
  | { type: "started"; session: string }
  | { type: "text_chunk"; session: string; text: string }
  | { type: "completed"; session: string }
  | { type: "cancelled"; session: string }
  | { type: "failed"; session: string; error: { code: string; message: string } };

export type AgentEventEnvelope = {
  contractVersion: number;
  event: AgentStreamEvent;
};

export type AgentSendAccepted = {
  contract_version: number;
  session: string;
};

export type SendPath = "agent" | "legacy";

export function chooseSendPath(hostRunning: boolean): SendPath {
  return hostRunning ? "agent" : "legacy";
}

export function isAgentEventEnvelope(value: unknown): value is AgentEventEnvelope {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  if (candidate.contractVersion !== AGENT_CONTRACT_VERSION) {
    return false;
  }
  const event = candidate.event as Record<string, unknown> | null;
  if (typeof event !== "object" || event === null || typeof event.session !== "string") {
    return false;
  }
  switch (event.type) {
    case "started":
    case "completed":
    case "cancelled":
      return true;
    case "text_chunk":
      return typeof event.text === "string";
    case "failed":
      return (
        typeof event.error === "object" &&
        event.error !== null &&
        typeof (event.error as Record<string, unknown>).code === "string" &&
        typeof (event.error as Record<string, unknown>).message === "string"
      );
    default:
      return false;
  }
}

export type AgentModelEntry = {
  provider_id: string;
  provider_name: string;
  model_id: string;
  model_name: string;
};

export type AgentModelSelection = {
  provider_id: string;
  model_id: string;
};

export type AgentModelList = {
  models: AgentModelEntry[];
  selected: AgentModelSelection | null;
};

export function isAgentHostStatus(value: unknown): value is AgentHostStatus {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return (
    typeof candidate.running === "boolean" &&
    typeof candidate.port === "number" &&
    (candidate.version === null || typeof candidate.version === "string") &&
    (candidate.pid === null || typeof candidate.pid === "number")
  );
}

export function hostLabel(phase: HostPhase, status: AgentHostStatus | null): string {
  switch (phase) {
    case "checking":
      return "Checking sidecar…";
    case "starting":
      return "Starting sidecar…";
    case "stopped":
      return "Sidecar stopped";
    case "failed":
      return "Sidecar failed to start";
    case "running":
      if (status?.version) {
        return `Sidecar ${status.version}`;
      }
      return "Sidecar running";
  }
}

function checkedStatus(command: string, result: unknown): AgentHostStatus {
  if (!isAgentHostStatus(result)) {
    throw new Error(`The core returned an unexpected ${command} response`);
  }
  return result;
}

export async function requestHostStatus(): Promise<AgentHostStatus> {
  const result: unknown = await invoke("agent_host_status");
  return checkedStatus("agent_host_status", result);
}

export async function requestHostStart(port?: number): Promise<AgentHostStatus> {
  const result: unknown = await invoke("agent_host_start", { port: port ?? null });
  return checkedStatus("agent_host_start", result);
}

export async function requestHostStop(): Promise<AgentHostStatus> {
  const result: unknown = await invoke("agent_host_stop");
  return checkedStatus("agent_host_stop", result);
}

export function modelKey(entry: { provider_id: string; model_id: string }): string {
  return `${entry.provider_id}/${entry.model_id}`;
}

export function isAgentModelList(value: unknown): value is AgentModelList {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  if (!Array.isArray(candidate.models)) {
    return false;
  }
  const selected = candidate.selected;
  return (
    selected === null ||
    (typeof selected === "object" &&
      selected !== null &&
      typeof (selected as Record<string, unknown>).provider_id === "string" &&
      typeof (selected as Record<string, unknown>).model_id === "string")
  );
}

export async function requestHostModels(): Promise<AgentModelList> {
  const result: unknown = await invoke("agent_host_models");
  if (!isAgentModelList(result)) {
    throw new Error("The core returned an unexpected agent_host_models response");
  }
  return result;
}

export async function requestHostSelectModel(
  providerId: string,
  modelId: string
): Promise<AgentModelSelection> {  const result: unknown = await invoke("agent_host_select_model", {
    provider_id: providerId,
    model_id: modelId
  });
  if (
    typeof result !== "object" ||
    result === null ||
    typeof (result as Record<string, unknown>).provider_id !== "string" ||
    typeof (result as Record<string, unknown>).model_id !== "string"
  ) {
    throw new Error("The core returned an unexpected agent_host_select_model response");
  }
  return result as AgentModelSelection;
}

export async function requestHostSend(prompt: string): Promise<AgentSendAccepted> {
  const result: unknown = await invoke("agent_host_send", { prompt });
  if (
    typeof result !== "object" ||
    result === null ||
    typeof (result as Record<string, unknown>).session !== "string"
  ) {
    throw new Error("The core returned an unexpected agent_host_send response");
  }
  return result as AgentSendAccepted;
}

export async function requestHostCancelSend(): Promise<AgentHostStatus> {
  const result: unknown = await invoke("agent_host_cancel_send");
  return checkedStatus("agent_host_cancel_send", result);
}

export type CatalogModel = {
  provider_id: string;
  provider_name: string;
  model_id: string;
  model_name: string;
  context_length: number | null;
  prompt_usd_per_m: number | null;
  completion_usd_per_m: number | null;
};

export type CatalogResult = {
  models: CatalogModel[];
  selected: AgentModelSelection | null;
  stale: boolean;
};

export function isCatalogResult(value: unknown): value is CatalogResult {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  const selected = candidate.selected;
  return (
    Array.isArray(candidate.models) &&
    typeof candidate.stale === "boolean" &&
    (selected === null ||
      (typeof selected === "object" &&
        selected !== null &&
        typeof (selected as Record<string, unknown>).provider_id === "string" &&
        typeof (selected as Record<string, unknown>).model_id === "string"))
  );
}

export async function requestHostCatalog(refresh = false): Promise<CatalogResult> {
  const result: unknown = await invoke("agent_host_catalog", { refresh });
  if (!isCatalogResult(result)) {
    throw new Error("The core returned an unexpected agent_host_catalog response");
  }
  return result;
}

export type AgentMode = "plan" | "build";

export function isAgentMode(value: unknown): value is AgentMode {
  return value === "plan" || value === "build";
}

export async function requestHostSetAgent(mode: AgentMode): Promise<AgentMode> {
  const result: unknown = await invoke("agent_host_set_agent", { agent: mode });
  const agent = (result as Record<string, unknown> | null)?.agent;
  if (!isAgentMode(agent)) {
    throw new Error("The core returned an unexpected agent_host_set_agent response");
  }
  return agent;
}

export function formatContext(length: number | null): string {
  if (length === null || !Number.isFinite(length) || length < 0) {
    return "? ctx";
  }
  if (length >= 1_000_000) {
    return `${(length / 1_000_000).toFixed(1)}M ctx`;
  }
  if (length >= 1_000) {
    return `${Math.round(length / 1_000)}k ctx`;
  }
  return `${length} ctx`;
}

export function formatPrice(perMillion: number | null): string {
  if (perMillion === null || !Number.isFinite(perMillion) || perMillion < 0) {
    return "?/M";
  }
  return `$${perMillion.toFixed(2)}/M`;
}

export function modelDetail(entry: CatalogModel): string {
  return `${entry.model_name} · ${entry.provider_name} · ${formatContext(entry.context_length)} · ${formatPrice(entry.prompt_usd_per_m)} in`;
}
