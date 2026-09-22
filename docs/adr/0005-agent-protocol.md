# ADR 0005: Agent Adapter boundary with ACP preference

**Status:** Accepted  
**Date:** 2026-09-22

## Context

BrainRoot must support one agent well without making its UI, permissions, tools, or sessions belong to one vendor. Agent Client Protocol (ACP) is an open interoperability standard with active editor and agent implementations, but provider capabilities vary and the protocol does not define BrainRoot's product model.

## Decision

Define a small internal Agent Adapter contract and prefer an ACP connection/adapter when it supplies the required capability reliably. Implement only one agent integration in the MVP. Capability negotiation is explicit; unsupported resume, tools, images, or progress features degrade honestly. BrainRoot remains the permission and tool authority.

## Alternatives considered

- **Direct provider SDK:** fastest single integration, rejected as the product boundary because provider semantics would leak into UI/core.
- **ACP types throughout the app:** rejected because protocol evolution and editor-oriented concepts should not own BrainRoot state.
- **Build a proprietary general protocol:** rejected while an open ecosystem protocol exists.
- **Many adapters immediately:** rejected by MVP scope.

## Consequences

One normalization layer must map protocol events to product events. Some native provider capabilities may be unavailable until the adapter expands. ACP version and registry behavior require ongoing compatibility tests.

## Performance implications

Agent/adapter processes start only for sessions and close afterward. Measure startup, stream latency, cancellation, memory, and orphan cleanup. Avoid an always-on broker.

## Security implications

ACP/provider messages remain untrusted. Tool requests pass through BrainRoot permission policy; agent-native auth is stored according to its supported secure flow and never grants human browser access.

## Reversibility

High for individual providers because adapters are replaceable. Moderate for the internal contract; keep it minimal and versioned.

## References

[ACP overview](https://zed.dev/acp), [ACP Python SDK quick start](https://agentclientprotocol.github.io/python-sdk/quickstart/), and [Zed external-agent model](https://github.com/zed-industries/zed/blob/main/docs/src/ai/external-agents.md).

