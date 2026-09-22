import { invoke } from "@tauri-apps/api/core";

export const HEALTH_CONTRACT_VERSION = 1;

export type HealthReport = {
  contract_version: number;
  status: "ready";
};

export class HealthContractError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "HealthContractError";
  }
}

export function isHealthReport(value: unknown): value is HealthReport {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const candidate = value as Record<string, unknown>;

  return (
    candidate.contract_version === HEALTH_CONTRACT_VERSION &&
    candidate.status === "ready"
  );
}

export async function requestHealth(): Promise<HealthReport> {
  const result: unknown = await invoke("health", {
    request: { contract_version: HEALTH_CONTRACT_VERSION }
  });

  if (!isHealthReport(result)) {
    throw new HealthContractError("The core returned an unexpected health response");
  }

  return result;
}
